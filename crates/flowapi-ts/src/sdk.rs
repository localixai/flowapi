//! The SDK, printed after the types when asked for ([`crate::TypeScript::sdk`]): for each
//! operation, `<name>Request(input)`, which writes the request the input makes (path, query,
//! headers, body) and sends nothing, and `<name>Response(status, headers, body)`, which reads an
//! answer back into the response the description names. A runtime of a few functions is printed
//! once before them (`sdk/runtime.ts` writes requests, `sdk/multipart.ts` their
//! `multipart/form-data` bodies, `sdk/read.ts` reads responses); the output imports nothing. What a call does not depend
//! on is decided here, once: path templates are split, names percent-encoded and each
//! parameter's style chosen as the arguments of one call.

mod request;
mod response;

pub use request::method;

use flowapi::Program;
use std::collections::BTreeSet;

/// The runtime every operation's functions call, printed once: writing a request, its multipart
/// body, then reading a response.
const RUNTIME: &str = include_str!("sdk/runtime.ts");
const MULTIPART: &str = include_str!("sdk/multipart.ts");
const READ: &str = include_str!("sdk/read.ts");

pub(crate) fn sdk(out: &mut String, p: &Program<'_>) {
    let endpoints = p.endpoints();
    if endpoints.is_empty() {
        return;
    }
    out.push_str(RUNTIME);
    out.push_str(MULTIPART);
    out.push_str(READ);
    let decls: Vec<Option<&str>> = endpoints.iter().map(|e| p.name_of(e.ty)).collect();
    let bases = function_bases(&decls);
    for ((e, decl), base) in endpoints.iter().zip(&decls).zip(&bases) {
        let Some(decl) = decl else { continue };
        out.push('\n');
        request::write(out, e, decl, base);
        out.push('\n');
        response::write(out, e, decl, base);
    }
}

/// Each endpoint's function name before its `Request` or `Response`, as [`sdk`] prints them;
/// `None` for an endpoint it prints no function for (one with no declaration). A layer above the
/// target (`flowapi-sdk`) names the functions by it.
#[must_use]
pub fn bases(p: &Program<'_>) -> Vec<Option<String>> {
    let decls: Vec<Option<&str>> = p.endpoints().iter().map(|e| p.name_of(e.ty)).collect();
    let bases = function_bases(&decls);
    decls.iter().zip(bases).map(|(d, b)| d.map(|_| b)).collect()
}

/// How a response body of a media type is read: JSON parsed, `text/*` as text, any other type
/// as bytes; a media range by the type the answer sends (`flowapiReadsBytes`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadAs {
    /// Parsed as JSON.
    Json,
    /// A `string`.
    Text,
    /// A `Uint8Array`.
    Bytes,
    /// A range: by the `Content-Type` sent.
    Sent,
}

impl ReadAs {
    /// How a body of this media type, as written, is read.
    #[must_use]
    pub fn of(media_type: &str) -> Self {
        let b = bare(media_type);
        if b.contains('*') {
            Self::Sent
        } else if is_json(&b) {
            Self::Json
        } else if b.starts_with("text/") {
            Self::Text
        } else {
            Self::Bytes
        }
    }

    /// Its name: `json`, `text`, `bytes` or `sent`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Text => "text",
            Self::Bytes => "bytes",
            Self::Sent => "sent",
        }
    }
}

/// Each function's name before its `Request` or `Response`: the declaration's name in camel
/// case, or as it is where two would meet (the declarations' names are distinct already).
fn function_bases(decls: &[Option<&str>]) -> Vec<String> {
    let camel: Vec<String> = decls
        .iter()
        .map(|d| {
            let d = d.unwrap_or("");
            let mut chars = d.chars();
            chars.next().map_or_else(String::new, |c| {
                let mut s = c.to_ascii_lowercase().to_string();
                s.push_str(chars.as_str());
                s
            })
        })
        .collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let twice: BTreeSet<&str> = camel
        .iter()
        .filter(|c| !seen.insert(c.as_str()))
        .map(String::as_str)
        .collect();
    camel
        .iter()
        .zip(decls)
        .map(|(c, d)| {
            if twice.contains(c.as_str()) {
                d.unwrap_or("").to_owned()
            } else {
                c.clone()
            }
        })
        .collect()
}

/// A string as a TypeScript string literal (a JSON string is one).
#[must_use]
pub fn lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if u32::from(c) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// RFC 3986 percent-encoding of every character but the unreserved ones, as UTF-8: a parameter
/// name in a URI (3.2, Appendix C, "Non-RFC6570 Field Values and Combinations").
pub(crate) fn percent(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            out.push('%');
            out.push(char::from(HEX[usize::from(b >> 4)]));
            out.push(char::from(HEX[usize::from(b & 15)]));
        }
    }
    out
}

/// Whether a media type is JSON: `application/json` or a `+json` type (RFC 6839), its
/// parameters left out.
pub(crate) fn is_json(media_type: &str) -> bool {
    let bare = bare(media_type);
    bare == "application/json" || bare.ends_with("+json")
}

/// A media type without its parameters, in lower case.
pub(crate) fn bare(media_type: &str) -> String {
    media_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}
