//! `<name>Response(status, headers, body)`: an answer read back into the response of the
//! description it is. The status picks the response (its code, then its range, then `default`),
//! the `Content-Type` its media type; the result says which (`response`, `mediaType`), so a
//! `switch` on either narrows `body` to that media type's type. The body is given as text or as
//! bytes and read by the type sent, as the Rust SDK reads it: JSON parsed, `text/*` a `string`,
//! any other a `Uint8Array`; a media range is any of those its range allows. What the
//! description does not name is `response: null` with the body as given: a type is never claimed
//! for a body nothing said what it is.

use super::{bare, is_json, lit};
use flowapi::graph::Endpoint;
use std::fmt::Write;

pub(super) fn write(out: &mut String, e: &Endpoint, decl: &str, base: &str) {
    let op = &e.operation;
    // The table the runtime reads, built once: per response its media types as written and bare.
    let _ = write!(out, "const ${base}Response: $Table = [");
    for (i, r) in op.responses.iter().enumerate() {
        let written: Vec<String> = r.content.iter().map(|m| lit(&m.media_type)).collect();
        let bares: Vec<String> = r
            .content
            .iter()
            .map(|m| lit(&bare(&m.media_type)))
            .collect();
        let _ = write!(
            out,
            "{}[{}, [{}], [{}]]",
            if i == 0 { "" } else { ", " },
            lit(&r.status),
            written.join(", "),
            bares.join(", ")
        );
    }
    out.push_str("];\n");
    let _ = write!(
        out,
        "export function {base}Response(\n  status: number,\n  headers: Record<string, \
         string>,\n  body: string | Uint8Array,\n):\n"
    );
    for r in &op.responses {
        let status = if r.status.len() == 3 && r.status.bytes().all(|b| b.is_ascii_digit()) {
            r.status.clone()
        } else {
            String::from("number")
        };
        let key = lit(&r.status);
        if r.content.is_empty() {
            let _ = writeln!(
                out,
                "  | {{ response: {key}; status: {status}; mediaType: null; body: string | Uint8Array }}"
            );
        }
        for m in &r.content {
            let t = &m.media_type;
            let ty = format!("{decl}[\"responses\"][{key}][\"content\"][{}]", lit(t));
            let body = body_type(t, ty);
            let _ = writeln!(
                out,
                "  | {{ response: {key}; status: {status}; mediaType: {}; body: {body} }}",
                lit(t)
            );
        }
    }
    out.push_str(
        "  | { response: null; status: number; mediaType: string | null; body: string | Uint8Array } \
         {\n",
    );
    let _ = writeln!(
        out,
        "  return $read(${base}Response, status, headers, body) as ReturnType<typeof \
         {base}Response>;\n}}"
    );
}

/// What a body of media type `t` is read as (`ty` its type in the declaration): JSON its type,
/// `text/*` a `string`, any other type the bytes. A range is read by the type the answer sends,
/// so it is each of those its range allows: `text/*` JSON (a `+json` type) or text, `*/*` any,
/// another range JSON or bytes.
fn body_type(t: &str, ty: String) -> String {
    let b = bare(t);
    if b.contains('*') {
        if b == "*/*" {
            format!("{ty} | string | Uint8Array")
        } else if b.starts_with("text/") {
            format!("{ty} | string")
        } else {
            format!("{ty} | Uint8Array")
        }
    } else if is_json(&b) {
        ty
    } else if b.starts_with("text/") {
        String::from("string")
    } else {
        String::from("Uint8Array")
    }
}
