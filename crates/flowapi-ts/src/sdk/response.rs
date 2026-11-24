//! `<name>Response(status, headers, body)`: an answer read back into the response of the
//! description it is. The status picks the response (its code, then its range, then `default`),
//! the `Content-Type` its media type; the result says which (`response`, `mediaType`), so a
//! `switch` on either narrows `body` to that media type's type. A JSON body is parsed; any other
//! is the text. What the description does not name is `response: null` with the text: a type is
//! never claimed for a body nothing said what it is.

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
         string>,\n  body: string,\n):\n"
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
                "  | {{ response: {key}; status: {status}; mediaType: null; body: string }}"
            );
        }
        for m in &r.content {
            let t = &m.media_type;
            let ty = format!("{decl}[\"responses\"][{key}][\"content\"][{}]", lit(t));
            // A range (`application/*`) holds JSON or text, as the answer's type says.
            let body = if t.contains('*') {
                format!("{ty} | string")
            } else if is_json(t) {
                ty
            } else {
                String::from("string")
            };
            let _ = writeln!(
                out,
                "  | {{ response: {key}; status: {status}; mediaType: {}; body: {body} }}",
                lit(t)
            );
        }
    }
    out.push_str(
        "  | { response: null; status: number; mediaType: string | null; body: string } {\n",
    );
    let _ = writeln!(
        out,
        "  return $read(${base}Response, status, headers, body) as ReturnType<typeof \
         {base}Response>;\n}}"
    );
}
