//! What a call gave against what it must: the Style Examples' cell, or a case's expectation as
//! the TypeScript oracle reads it. The Rust SDK holds a body's content type beside its bytes
//! (`Request::body`), not among the headers, so a case's `content-type` header is checked there;
//! a content type's parameters (`; charset=utf-8`) are the caller's to add.

use crate::describe::{Call, Check, bare};
use serde_json::{Value, json};

pub(crate) fn check(call: &Call, got: &Value, failures: &mut Vec<String>) {
    let what = &call.what;
    match &call.check {
        Check::Table {
            id,
            location,
            serialized,
        } => {
            let header = |name: &str| pair(got, name).unwrap_or_default();
            let cell = match location.as_str() {
                "path" => str_of(got, "path")
                    .get(id.len() + 3..)
                    .unwrap_or("")
                    .to_owned(),
                "query" => str_of(got, "query").to_owned(),
                "header" => header("color"),
                _ => header("cookie"),
            };
            if &cell != serialized {
                failures.push(format!(
                    "{what}: got {cell:?}, the spec shows {serialized:?}"
                ));
            }
        }
        Check::Request {
            id,
            want,
            content_type,
        } => request(id, want, content_type.as_deref(), got, what, failures),
        Check::Response { want, status } => response(want, *status, got, what, failures),
    }
}

fn str_of<'v>(v: &'v Value, k: &str) -> &'v str {
    v[k].as_str().unwrap_or("")
}

/// A header's value, its name as given.
fn pair(got: &Value, name: &str) -> Option<String> {
    got["headers"]
        .as_array()?
        .iter()
        .find(|h| h[0] == name)
        .and_then(|h| h[1].as_str())
        .map(str::to_owned)
}

fn request(
    id: &str,
    want: &Value,
    given: Option<&str>,
    got: &Value,
    what: &str,
    failures: &mut Vec<String>,
) {
    let mut fail = |m: String| failures.push(format!("{what}: {m}"));
    if got.get("input").is_some() {
        fail(format!("the input does not deserialize: {}", got["input"]));
        return;
    }
    match (want.get("refused"), got.get("refused")) {
        (Some(w), g) => {
            if g != Some(w) {
                fail(format!("refused {}, want {w}", got));
            }
            return;
        }
        (None, Some(g)) => {
            fail(format!("refused: {g}"));
            return;
        }
        (None, None) => {}
    }
    if let Some(p) = want["path"].as_str() {
        let w = format!("/{id}{p}");
        if str_of(got, "path") != w {
            fail(format!("path {:?}, want {w:?}", str_of(got, "path")));
        }
    }
    for k in ["method", "query"] {
        if want.get(k).is_some() && want[k] != got[k] {
            fail(format!("{k} {}, want {}", got[k], want[k]));
        }
    }
    if let Some(b) = want.get("body")
        && got["body"][1] != *b
    {
        fail(format!("body {}, want {b}", got["body"][1]));
    }
    if let Some(h) = want["headers"].as_object() {
        let wanted: Vec<Value> = h
            .iter()
            .filter(|(k, _)| k.as_str() != "content-type")
            .map(|(k, v)| json!([k, v]))
            .collect();
        if got["headers"].as_array().map(Vec::as_slice) != Some(wanted.as_slice()) {
            fail(format!(
                "headers {}, want {}",
                got["headers"],
                Value::Array(wanted)
            ));
        }
        let sent = got["body"][0].as_str();
        match (h.get("content-type").and_then(Value::as_str), sent) {
            (None, None) => {}
            (Some(w), Some(s))
                if w == s || (Some(w) == given && w.contains(';') && bare(w) == bare(s)) => {}
            (w, s) => fail(format!("content type {s:?}, want {w:?}")),
        }
    }
    if let Some(p) = want.get("pairs") {
        let parsed = pairs(str_of(got, "query"));
        if parsed != *p {
            fail(format!("the query parses as {parsed}, want {p}"));
        }
    }
    if let Some(p) = want.get("bodyPairs") {
        let parsed = pairs(got["body"][1].as_str().unwrap_or(""));
        if parsed != *p {
            fail(format!("the body parses as {parsed}, want {p}"));
        }
    }
    if let Some(p) = want["decodedPath"].as_str() {
        let decoded = String::from_utf8_lossy(&decode(str_of(got, "path"), false)).into_owned();
        if decoded != format!("/{id}{p}") {
            fail(format!("the path decodes as {decoded:?}"));
        }
    }
}

fn response(want: &Value, status: u64, got: &Value, what: &str, failures: &mut Vec<String>) {
    let body = body(got);
    let read = json!({ "response": got["response"], "status": status, "mediaType": got["mediaType"], "body": body });
    for (k, v) in want.as_object().into_iter().flatten() {
        if read[k] != *v {
            failures.push(format!(
                "{what}: response {status} {k}: got {}, want {v}",
                read[k]
            ));
        }
    }
}

/// The body a response was read as: the payload of its variant (after the status a range or
/// `default` keeps, the bytes of a media range), bytes as text; an error's body as text.
fn body(got: &Value) -> Value {
    if got["error"].is_string() {
        return got["payload"].clone();
    }
    let key = got["response"].as_str().unwrap_or("");
    let media = got["mediaType"].as_str().unwrap_or("");
    let mut v = got["payload"].clone();
    let exact = key.len() == 3 && key.bytes().all(|b| b.is_ascii_digit());
    if !exact && let Value::Array(a) = &v {
        let rest: Vec<Value> = a.iter().skip(1).cloned().collect();
        v = if rest.len() == 1 {
            rest.into_iter().next().unwrap_or(Value::Null)
        } else {
            Value::Array(rest)
        };
    }
    if media.contains('*') {
        v = v[1].clone();
    }
    let json = bare(media) == "application/json" || bare(media).ends_with("+json");
    match v {
        Value::Null => json!(""),
        Value::Array(bytes) if !json => {
            let b: Vec<u8> = bytes
                .iter()
                .filter_map(|x| x.as_u64().and_then(|n| u8::try_from(n).ok()))
                .collect();
            json!(String::from_utf8_lossy(&b))
        }
        other => other,
    }
}

/// WHATWG's `application/x-www-form-urlencoded` parser: the pairs as `[name, value]`.
fn pairs(text: &str) -> Value {
    let mut out = Vec::new();
    for part in text.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = part.split_once('=').unwrap_or((part, ""));
        let k = String::from_utf8_lossy(&decode(k, true)).into_owned();
        let v = String::from_utf8_lossy(&decode(v, true)).into_owned();
        out.push(json!([k, v]));
    }
    Value::Array(out)
}

/// Percent-decoding (and `+` as a space in a form); a `%` not before two hex digits stays.
fn decode(text: &str, plus: bool) -> Vec<u8> {
    let b = text.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |j: usize| b.get(j).and_then(|c| char::from(*c).to_digit(16));
        match (b[i], hex(i + 1), hex(i + 2)) {
            (b'%', Some(h), Some(l)) => {
                out.push(u8::try_from(h * 16 + l).unwrap_or(0));
                i += 3;
            }
            (b'+', _, _) if plus => {
                out.push(b' ');
                i += 1;
            }
            (c, _, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}
