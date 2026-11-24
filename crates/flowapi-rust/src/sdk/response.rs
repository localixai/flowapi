//! `<name>_response(status, headers, body)`: an answer read back into an enum of the responses
//! the description names. The status picks the response by a `match` decided here (its code,
//! then its range `NXX`, then `default`), the `Content-Type` its media type (exact, then
//! `type/*`, then `*/*`). A JSON body is read by serde_json into its type, a `text/*` body is a
//! `String`, any other the bytes; a media range keeps the content type sent beside the bytes. A
//! range or `default` keeps the status. What the description does not name, or a body that is
//! not its type, is an `sdk::Error` holding the answer: a type is never claimed for a body
//! nothing said what it is, and an empty JSON body is an error, never a default value.

use super::{Op, bare, field, fields, is_json, type_of};
use crate::emit::Emitter;
use crate::ident::{Taken, literal, pascal};
use std::fmt::Write;

/// Which statuses a response is for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Code(u16),
    /// `1XX` to `5XX` (any case): the first digit.
    Range(u16),
    Default,
}

fn status(s: &str) -> Option<Status> {
    let b = s.as_bytes();
    if s == "default" {
        return Some(Status::Default);
    }
    if b.len() != 3 {
        return None;
    }
    let first = b.first().filter(|c| (b'1'..=b'9').contains(c))?;
    let digit = u16::from(first - b'0');
    if b.get(1..).is_some_and(|x| x.eq_ignore_ascii_case(b"XX")) {
        return Some(Status::Range(digit));
    }
    s.parse().ok().map(Status::Code)
}

/// One variant: its name, its payload's types, and what it says of itself.
struct Variant {
    name: String,
    payload: String,
    response: String,
    media: Option<String>,
}

pub(super) fn write(e: &mut Emitter<'_, '_>, out: &mut String, op: &Op<'_>) {
    let o = &op.endpoint.operation;
    let decl = e.name(op.item).to_owned();
    let name = e.fresh(&format!("{decl}Response"));
    let responses = field(&op.fields, "responses").and_then(|f| fields(e.p, f.ty));
    let mut taken = Taken::default();
    let mut seen: Vec<Status> = Vec::new();
    // Per response kept, its status and its arm's text; exact codes first, then ranges, then
    // `default`, so the `match` tries them in that order.
    let mut arms: Vec<(Status, String)> = Vec::new();
    let mut variants: Vec<Variant> = Vec::new();
    for r in &o.responses {
        let Some(st) = status(&r.status).filter(|s| !seen.contains(s)) else {
            continue;
        };
        seen.push(st);
        let keep = !matches!(st, Status::Code(_));
        let base = match st {
            Status::Code(c) => format!("Status{c}"),
            Status::Range(d) => format!("Status{d}XX"),
            Status::Default => String::from("Default"),
        };
        let media = responses
            .as_ref()
            .and_then(|rs| field(rs, &r.status))
            .and_then(|f| fields(e.p, f.ty))
            .and_then(|fs| field(&fs, "content").and_then(|c| fields(e.p, c.ty)))
            .unwrap_or_default();
        let arm = if r.content.is_empty() {
            let v = taken.take(&base);
            let (payload, read) = if keep {
                (
                    String::from("u16, Vec<u8>"),
                    format!("Ok({name}::{v}(status, body.to_vec()))"),
                )
            } else {
                (String::new(), format!("Ok({name}::{v})"))
            };
            variants.push(Variant {
                name: v,
                payload,
                response: r.status.clone(),
                media: None,
            });
            read
        } else {
            let mut to = Variants {
                name: &name,
                base: &base,
                keep,
                taken: &mut taken,
                variants: &mut variants,
            };
            bodies(e, op.item, r, &media, &mut to)
        };
        arms.push((st, arm));
    }
    arms.sort_by_key(|(s, _)| match s {
        Status::Code(_) => 0,
        Status::Range(_) => 1,
        Status::Default => 2,
    });
    print_enum(out, op, &name, &variants);
    let has_default = arms.iter().any(|(s, _)| *s == Status::Default);
    // The content type is read where a response has a body or an error may be made.
    let reads = !has_default || o.responses.iter().any(|r| !r.content.is_empty());
    let _ = write!(
        out,
        "\n/// Reads an answer to `{}_request` as the response its status and content type name.\n\
         pub fn {}_response(status: u16, {}: &[(String, String)], body: &[u8]) -> \
         Result<{name}, sdk::Error> {{\n{}    match status {{\n",
        op.base,
        op.base,
        if reads { "headers" } else { "_headers" },
        if reads {
            "    let ct = sdk::content_type(headers);\n"
        } else {
            ""
        },
    );
    for (st, arm) in &arms {
        let pattern = match st {
            Status::Code(c) => c.to_string(),
            Status::Range(d) => format!("{}..={}", d * 100, d * 100 + 99),
            Status::Default => String::from("_"),
        };
        let _ = writeln!(out, "        {pattern} => {arm},");
    }
    if !has_default {
        out.push_str(
            "        _ => Err(sdk::Error::new(status, ct, body, sdk::ErrorKind::Status)),\n",
        );
    }
    out.push_str("    }\n}\n");
}

/// What reads a JSON or text body into variant `v`.
fn reader(f: &str, name: &str, v: &str, keep: bool) -> String {
    if keep {
        format!("sdk::{f}(status, ct, body).map(|v| {name}::{v}(status, v))")
    } else {
        format!("sdk::{f}(status, ct, body).map({name}::{v})")
    }
}

/// The enum and what each variant says of itself (`sdk::Response`).
fn print_enum(out: &mut String, op: &Op<'_>, name: &str, variants: &[Variant]) {
    let _ = writeln!(
        out,
        "\n/// The responses of `{}_request`, by status and media type. It serializes as its \
         body: a range or `default` as the status and the body.\n#[derive(Serialize, Debug, \
         Clone, PartialEq)]\n#[serde(untagged)]\npub enum {name} {{",
        op.base
    );
    for v in variants {
        if v.payload.is_empty() {
            let _ = writeln!(out, "    {},", v.name);
        } else {
            let _ = writeln!(out, "    {}({}),", v.name, v.payload);
        }
    }
    out.push_str("}\n");
    let _ = write!(
        out,
        "\nimpl sdk::Response for {name} {{\n    fn response(&self) -> &'static str {{\n        \
         match *self {{\n"
    );
    for v in variants {
        let _ = writeln!(
            out,
            "            {} => {},",
            pattern(name, v),
            literal(&v.response)
        );
    }
    out.push_str(
        "        }\n    }\n\n    fn media_type(&self) -> Option<&'static str> {\n        match \
         *self {\n",
    );
    for v in variants {
        let media = v
            .media
            .as_ref()
            .map_or_else(|| String::from("None"), |m| format!("Some({})", literal(m)));
        let _ = writeln!(out, "            {} => {media},", pattern(name, v));
    }
    out.push_str("        }\n    }\n}\n");
}

fn pattern(name: &str, v: &Variant) -> String {
    if v.payload.is_empty() {
        format!("{name}::{}", v.name)
    } else {
        format!("{name}::{}(..)", v.name)
    }
}

/// Where a response's variants go, and how they are named and read.
struct Variants<'a> {
    name: &'a str,
    base: &'a str,
    /// A range or `default`: the status is kept.
    keep: bool,
    taken: &'a mut Taken,
    variants: &'a mut Vec<Variant>,
}

/// A response with a body: its arm, a `match` on the media type, and a variant per media type.
fn bodies(
    e: &mut Emitter<'_, '_>,
    item: usize,
    r: &flowapi::spec::Response,
    media: &[super::Field<'_>],
    to: &mut Variants<'_>,
) -> String {
    let (name, base, keep) = (to.name, to.base, to.keep);
    let lead = if keep { "status, " } else { "" };
    let lead_ty = if keep { "u16, " } else { "" };
    let mut arm = String::new();
    let table: Vec<String> = r
        .content
        .iter()
        .map(|m| literal(&bare(&m.media_type)))
        .collect();
    let _ = writeln!(arm, "match sdk::media(ct, &[{}]) {{", table.join(", "));
    for (i, m) in r.content.iter().enumerate() {
        let t = &m.media_type;
        let v = if r.content.len() == 1 {
            to.taken.take(base)
        } else {
            to.taken.take(&format!("{base}{}", pascal(t, "Any", "V")))
        };
        let b = bare(t);
        let (payload, read) = if b.contains('*') {
            (
                format!("{lead_ty}String, Vec<u8>"),
                format!(
                    "{{\n                let (t, b) = sdk::ranged(ct, body);\n                Ok({name}::{v}({lead}t, b))\n            }}"
                ),
            )
        } else if is_json(&b) {
            let ty = field(media, t).map_or_else(
                || String::from("serde_json::Value"),
                |f| type_of(e, item, f.ty),
            );
            (format!("{lead_ty}{ty}"), reader("json", name, &v, keep))
        } else if b.starts_with("text/") {
            (format!("{lead_ty}String"), reader("text", name, &v, keep))
        } else {
            (
                format!("{lead_ty}Vec<u8>"),
                format!("Ok({name}::{v}({lead}body.to_vec()))"),
            )
        };
        let _ = writeln!(arm, "            Some({i}) => {read},");
        to.variants.push(Variant {
            name: v,
            payload,
            response: r.status.clone(),
            media: Some(t.clone()),
        });
    }
    arm.push_str(
        "            _ => Err(sdk::Error::new(status, ct, body, sdk::ErrorKind::MediaType)),\n        }",
    );
    arm
}
