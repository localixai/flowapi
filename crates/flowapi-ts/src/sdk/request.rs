//! `<name>Request(input)`: the request an operation's input makes. The input is the operation's
//! `parameters` and, when it has a body, `body: { contentType, value }` of one of its media
//! types. Each parameter is one call of the runtime with its style already chosen: the RFC 6570
//! operator's prefix and separators (3.2, Appendix C, "Equivalences Between Fields and RFC6570
//! Operators"), the encoder (none for headers and `style: cookie`, reserved expansion for
//! `allowReserved`), and its name percent-encoded where it goes into a URI.

use super::{ReadAs, bare, lit, percent};
use flowapi::graph::Endpoint;
use flowapi::spec::{Location, Operation, Param, Style};
use std::fmt::Write;

/// The methods a Path Item names by a field, written in lower case; 3.2's
/// `additionalOperations` are sent as written.
const FIELDS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

/// The method a request is sent with: a Path Item's field in upper case, 3.2's
/// `additionalOperations` as written.
#[must_use]
pub fn method(op: &Operation) -> String {
    if FIELDS.contains(&op.method.as_str()) {
        op.method.to_ascii_uppercase()
    } else {
        op.method.clone()
    }
}

pub(super) fn write(out: &mut String, e: &Endpoint, decl: &str, base: &str) {
    let op = &e.operation;
    let body = op.body.as_ref().filter(|b| !b.content.is_empty());
    let mut input: Vec<String> = Vec::new();
    if !op.params.is_empty() {
        input.push(format!("NonNullable<{decl}[\"parameters\"]>"));
    }
    if let Some(b) = body {
        let members: Vec<String> = b
            .content
            .iter()
            .map(|m| {
                let t = &m.media_type;
                let content_type = if t.contains('*') {
                    String::from("string")
                } else {
                    format!(
                        "{} | `{};${{string}}`",
                        lit(t),
                        t.replace(['`', '$', '\\'], "")
                    )
                };
                let typed = format!("NonNullable<{decl}[\"requestBody\"]>[{}]", lit(t));
                let kind = bare(t);
                // `multipart/form-data` is written from its parts, by the Encoding Object's
                // defaults and the `encoding` given; another multipart type is the caller's to
                // write (its boundary goes in the content type); a type neither JSON, text nor a
                // form, or a range, may be given as bytes.
                let (value, more) = if kind == "multipart/form-data" {
                    (format!("$Form<{typed}>"), "; encoding?: FlowapiEncoding")
                } else if kind.starts_with("multipart/") {
                    (String::from("string | Uint8Array"), "")
                } else if ReadAs::of(t) == ReadAs::Json
                    || ReadAs::of(t) == ReadAs::Text
                    || kind == "application/x-www-form-urlencoded"
                {
                    (typed, "")
                } else {
                    (format!("{typed} | Uint8Array"), "")
                };
                format!("{{ contentType: {content_type}; value: {value}{more} }}")
            })
            .collect();
        let optional = if b.required { "" } else { "?" };
        input.push(format!("{{ body{optional}: {} }}", members.join(" | ")));
    }
    let _ = write!(out, "export function {base}Request(");
    if !input.is_empty() {
        let _ = write!(out, "input: {}", input.join(" & "));
    }
    out.push_str("): $Request {\n  const headers: Record<string, string> = {};\n");
    let has = |l: Location| op.params.iter().any(|p| p.location == l);
    let query = has(Location::Query) || has(Location::QueryString);
    if query {
        out.push_str("  let query = \"\";\n");
    }
    if has(Location::Cookie) {
        out.push_str("  let cookie = \"\";\n");
    }
    for p in &op.params {
        param(out, p);
    }
    if has(Location::Cookie) {
        out.push_str("  if (cookie !== \"\") headers[\"cookie\"] = cookie;\n");
    }
    let method = method(op);
    let _ = writeln!(
        out,
        "  const request: $Request = {{ method: {}, path: {}, query{}, headers }};",
        lit(&method),
        path(&op.path, &op.params),
        if query { "" } else { ": \"\"" }
    );
    if body.is_some() {
        out.push_str("  if (input.body !== undefined) $put(request, input.body);\n");
    }
    out.push_str("  return request;\n}\n");
}

/// The value of a parameter in the input.
fn access(p: &Param) -> String {
    format!("input.{}?.[{}]", p.location.name(), lit(&p.name))
}

/// The value, refused when it holds NaN or an infinity (`$x` refuses them itself).
fn finite(p: &Param) -> String {
    format!(
        "$fin({}, {}, {})",
        access(p),
        lit(p.location.name()),
        lit(&p.name)
    )
}

/// `s` checked for a CR, LF or NUL where a cookie is written as it is (`style: cookie`); a
/// header's value is checked by `$set`.
fn guarded(p: &Param, s: String) -> String {
    if p.serialization.style == Style::Cookie && p.serialization.media_type.is_none() {
        format!("$ctl({s}, {}, {})", lit(p.location.name()), lit(&p.name))
    } else {
        s
    }
}

/// The call that writes one parameter's value, `undefined` when it writes nothing.
fn expand(p: &Param) -> String {
    let s = &p.serialization;
    let mut value = access(p);
    // A parameter with `content` is its media type's text, then written as a string.
    if let Some(t) = &s.media_type {
        value = format!("$content({}, {})", finite(p), lit(t));
    }
    let style = if s.media_type.is_some() {
        match p.location {
            Location::Query | Location::Cookie => Style::Form,
            Location::Path | Location::Header | Location::QueryString => Style::Simple,
        }
    } else {
        s.style
    };
    let raw = p.location == Location::Header || style == Style::Cookie;
    let enc = if raw {
        "$raw"
    } else if s.allow_reserved {
        "$res"
    } else {
        "$enc"
    };
    let name = if raw {
        lit(&p.name)
    } else {
        lit(&percent(&p.name))
    };
    let (first, sep, named, ifemp, join) = match style {
        Style::Simple => ("", ",", false, "", ","),
        Style::Label => (".", ".", false, "", ","),
        Style::Matrix => (";", ";", true, "", ","),
        Style::Form => ("", "&", true, "=", ","),
        Style::SpaceDelimited => ("", "&", true, "=", "%20"),
        Style::PipeDelimited => ("", "&", true, "=", "%7C"),
        Style::Cookie => ("", "; ", true, "=", ","),
        Style::DeepObject => return format!("$deep({}, {name}, {enc})", finite(p)),
    };
    format!(
        "$x({value}, {name}, {}, {}, {named}, {}, {}, {}, {enc}, {}, {})",
        lit(first),
        lit(sep),
        lit(ifemp),
        s.explode,
        lit(join),
        lit(p.location.name()),
        lit(&p.name)
    )
}

/// One parameter's statement: added to the query, the cookie header or the headers. A path
/// parameter goes into the path, written by [`path`].
fn param(out: &mut String, p: &Param) {
    let _ = match p.location {
        Location::Path => Ok(()),
        Location::Query => writeln!(out, "  query = $add(query, {}, \"&\");", expand(p)),
        Location::QueryString => {
            let t = p
                .serialization
                .media_type
                .as_deref()
                .unwrap_or("text/plain");
            writeln!(
                out,
                "  query = $add(query, $qs({}, {}), \"&\");",
                finite(p),
                lit(t)
            )
        }
        Location::Header => writeln!(out, "  $set(headers, {}, {});", lit(&p.name), expand(p)),
        Location::Cookie => writeln!(
            out,
            "  cookie = $add(cookie, {}, \"; \");",
            guarded(p, expand(p))
        ),
    };
}

/// The path template filled: its text as written, each `{name}` the path parameter's value. An
/// expression no parameter names is left as written. A segment a parameter writes into is
/// `$seg(...)`: refused when it comes out empty, `.` or `..`.
fn path(template: &str, params: &[Param]) -> String {
    let mut parts: Vec<String> = Vec::new();
    // The open segment: its first parameter's name, and its pieces so far.
    let mut seg: Option<(&str, Vec<String>, bool)> = None;
    let mut text = String::new();
    let mut rest = template;
    loop {
        let (before, expr) = match rest
            .split_once('{')
            .and_then(|(b, a)| Some((b, a.split_once('}')?)))
        {
            Some((before, (name, tail))) => {
                rest = tail;
                (before, Some(name))
            }
            None => (std::mem::take(&mut rest), None),
        };
        match (seg.take(), before.split_once('/')) {
            (Some((name, mut pieces, reserved)), Some((head, tail))) => {
                text.push_str(head);
                if !text.is_empty() {
                    pieces.push(lit(&text));
                }
                parts.push(segment(&pieces, name, reserved));
                text = format!("/{tail}");
            }
            (open, _) => {
                seg = open;
                text.push_str(before);
            }
        }
        let Some(name) = expr else { break };
        let found = params
            .iter()
            .find(|p| p.location == Location::Path && p.name == name);
        let Some(p) = found else {
            text.push('{');
            text.push_str(name);
            text.push('}');
            continue;
        };
        let value = format!("({} ?? \"\")", expand(p));
        let keeps = p.serialization.allow_reserved;
        match &mut seg {
            Some((_, pieces, reserved)) => {
                if !text.is_empty() {
                    pieces.push(lit(&text));
                    text.clear();
                }
                pieces.push(value);
                *reserved |= keeps;
            }
            None => {
                // The segment starts after the last `/` of the text before it.
                let (lead, start) = match text.rsplit_once('/') {
                    Some((l, s)) => (format!("{l}/"), s.to_owned()),
                    None => (String::new(), std::mem::take(&mut text)),
                };
                if !lead.is_empty() {
                    parts.push(lit(&lead));
                }
                let mut pieces = Vec::new();
                if !start.is_empty() {
                    pieces.push(lit(&start));
                }
                pieces.push(value);
                text.clear();
                seg = Some((&p.name, pieces, keeps));
            }
        }
    }
    if let Some((name, mut pieces, reserved)) = seg {
        if !text.is_empty() {
            pieces.push(lit(&text));
        }
        parts.push(segment(&pieces, name, reserved));
    } else if !text.is_empty() || parts.is_empty() {
        parts.push(lit(&text));
    }
    parts.join(" + ")
}

/// A segment's pieces joined, held by `$seg`.
fn segment(pieces: &[String], name: &str, reserved: bool) -> String {
    format!("$seg({}, {}, {reserved})", pieces.join(" + "), lit(name))
}
