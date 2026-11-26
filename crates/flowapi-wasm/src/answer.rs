//! The answer written as JSON: the files, or an SDK command's text, then the diagnostics with
//! their places.

use flowapi::{File, Severity};
use std::fmt::Write;

/// A diagnostic as the answer carries it.
pub(crate) struct Found<'d> {
    pub(crate) severity: Severity,
    pub(crate) code: &'d str,
    pub(crate) message: &'d str,
    pub(crate) uri: &'d str,
    pub(crate) offset: u32,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

/// The error of a request that could not be read: no place.
pub(crate) fn refusal(message: &str) -> Found<'_> {
    Found {
        severity: Severity::Error,
        code: "flowapi.request",
        message,
        uri: "",
        offset: 0,
        line: 1,
        column: 1,
    }
}

/// The answer to a request that could not be read: no file, one error.
pub(crate) fn refused(message: &str) -> String {
    write(&[], &[refusal(message)])
}

/// The answer: `{"files":[…],"diagnostics":[…]}`.
pub(crate) fn write(files: &[File], diags: &[Found<'_>]) -> String {
    let size: usize = files.iter().map(|f| f.text.len() + f.path.len()).sum();
    let mut out = String::with_capacity(size + size / 8 + 64);
    out.push_str("{\"files\":[");
    for (i, f) in files.iter().enumerate() {
        out.push_str(if i == 0 { "{\"path\":" } else { ",{\"path\":" });
        string(&mut out, &f.path);
        out.push_str(",\"text\":");
        string(&mut out, &f.text);
        out.push('}');
    }
    out.push(']');
    diagnostics(&mut out, diags);
    out
}

/// The answer of an SDK command: `{"text":…,"json":…,"diagnostics":[…]}`, `null` for an output
/// it does not give.
#[cfg(feature = "sdk")]
pub(crate) fn sdk(text: Option<&str>, json: Option<&str>, diags: &[Found<'_>]) -> String {
    let size = text.map_or(0, str::len) + json.map_or(0, str::len);
    let mut out = String::with_capacity(size + size / 8 + 64);
    for (key, value) in [("{\"text\":", text), (",\"json\":", json)] {
        out.push_str(key);
        match value {
            Some(v) => string(&mut out, v),
            None => out.push_str("null"),
        }
    }
    diagnostics(&mut out, diags);
    out
}

/// `,"diagnostics":[…]}`, the end of an answer.
pub(crate) fn diagnostics(out: &mut String, diags: &[Found<'_>]) {
    out.push_str(",\"diagnostics\":[");
    for (i, d) in diags.iter().enumerate() {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        out.push_str(if i == 0 {
            "{\"severity\":\""
        } else {
            ",{\"severity\":\""
        });
        out.push_str(severity);
        out.push_str("\",\"code\":");
        string(out, d.code);
        out.push_str(",\"message\":");
        string(out, d.message);
        out.push_str(",\"uri\":");
        string(out, d.uri);
        let _ = write!(
            out,
            ",\"offset\":{},\"line\":{},\"column\":{}}}",
            d.offset, d.line, d.column
        );
    }
    out.push_str("]}");
}

/// A JSON array of strings.
pub(crate) fn strings(list: &[String]) -> String {
    let mut out = String::from("[");
    for (i, s) in list.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(&mut out, s);
    }
    out.push(']');
    out
}

/// A JSON string: quotes, backslashes and control characters escaped, the rest as it is.
pub(crate) fn string(out: &mut String, s: &str) {
    out.push('"');
    let mut plain = 0;
    for (i, c) in s.char_indices() {
        let escape = match c {
            '"' => "\\\"",
            '\\' => "\\\\",
            '\n' => "\\n",
            '\r' => "\\r",
            '\t' => "\\t",
            c if c < ' ' => "",
            _ => continue,
        };
        out.push_str(s.get(plain..i).unwrap_or(""));
        if escape.is_empty() {
            let _ = write!(out, "\\u{:04x}", u32::from(c));
        } else {
            out.push_str(escape);
        }
        plain = i + c.len_utf8();
    }
    out.push_str(s.get(plain..).unwrap_or(""));
    out.push('"');
}
