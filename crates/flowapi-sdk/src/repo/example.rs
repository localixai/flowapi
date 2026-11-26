//! An example value of a schema, the same on every run (G4): the schema's own `example` (or the
//! first of `examples`), else its `default`, `const` or first `enum` value, else one made from its
//! type and `format` (pitfall 20 of the study: a value made from a field's name breaks a `uuid`
//! or a `date`). The mock server answers with it, the generated tests send it, the README shows
//! it in a body (a parameter's value there is `sample.rs`'s). Bounded: past `MAX_DEPTH` schemas
//! deep, or past `MAX_JSON` levels of a written value, it is `null`, so no description exhausts
//! the stack (G1).

use crate::doc::{At, Doc};
use crate::out::json_str;
use flowapi::json::Value as Json;

/// How many schemas deep a value is made.
const MAX_DEPTH: usize = 8;
/// How many levels of a value written in the description are copied.
const MAX_JSON: usize = 32;

/// A value, printed as JSON or as a JavaScript expression.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    /// A number, as its digits.
    Num(String),
    Str(String),
    /// Bytes (`format: binary`, 2.0's `file`): a `Uint8Array` in JavaScript, `null` in JSON.
    Bytes,
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    /// As JSON, on one line.
    pub(crate) fn json(&self) -> String {
        let mut out = String::new();
        self.print(&mut out, false);
        out
    }

    /// As a JavaScript expression, on one line; a key that is an identifier is written bare.
    pub(crate) fn js(&self) -> String {
        let mut out = String::new();
        self.print(&mut out, true);
        out
    }

    fn print(&self, out: &mut String, js: bool) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::Num(n) => out.push_str(n),
            Self::Str(s) => out.push_str(&json_str(s)),
            Self::Bytes if js => out.push_str("new Uint8Array([1, 2, 3])"),
            Self::Bytes => out.push_str("null"),
            Self::Arr(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    out.push_str(if i == 0 { "" } else { ", " });
                    v.print(out, js);
                }
                out.push(']');
            }
            Self::Obj(members) if members.is_empty() => out.push_str("{}"),
            Self::Obj(members) => {
                out.push_str(if js { "{ " } else { "{" });
                for (i, (k, v)) in members.iter().enumerate() {
                    out.push_str(if i == 0 { "" } else { ", " });
                    if js && is_identifier(k) {
                        out.push_str(k);
                    } else {
                        out.push_str(&json_str(k));
                    }
                    out.push_str(": ");
                    v.print(out, js);
                }
                out.push_str(if js { " }" } else { "}" });
            }
        }
    }
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// The example of a schema; `all` makes every property, else only the required ones (what a
/// request needs, so that it sends no more than it must).
pub(crate) fn of(doc: &Doc<'_, '_>, schema: Option<At>, all: bool) -> Value {
    schema.map_or(Value::Null, |at| walk(doc, at, all, 0))
}

fn walk(doc: &Doc<'_, '_>, at: At, all: bool, depth: usize) -> Value {
    let Some(at) = doc.resolve(at) else {
        return Value::Null;
    };
    if depth > MAX_DEPTH || !doc.is(at, false) {
        return Value::Null;
    }
    if let Some(v) = own(doc, at) {
        return v;
    }
    if let Some(parts) = doc.get(at, "allOf") {
        let mut members = Vec::new();
        for part in doc.items(parts) {
            // An object's members are merged; a part of another type is the value, when no
            // part before it was an object.
            let value = walk(doc, part, all, depth + 1);
            let Value::Obj(more) = value else {
                if members.is_empty() {
                    return value;
                }
                continue;
            };
            for (k, v) in more {
                if !members.iter().any(|(m, _)| *m == k) {
                    members.push((k, v));
                }
            }
        }
        // The schema's own properties beside its `allOf`.
        if let Value::Obj(own) = made(doc, at, all, depth) {
            for (k, v) in own {
                if !members.iter().any(|(m, _)| *m == k) {
                    members.push((k, v));
                }
            }
        }
        return Value::Obj(members);
    }
    for key in ["oneOf", "anyOf"] {
        let choices = doc.get(at, key).map(|c| doc.items(c)).unwrap_or_default();
        let first = choices
            .into_iter()
            .find(|&c| kind(doc, c).as_deref() != Some("null"));
        if let Some(c) = first {
            return walk(doc, c, all, depth + 1);
        }
    }
    made(doc, at, all, depth)
}

/// The value a schema (its references followed) writes itself: its `example`, `default` or
/// `const`, else the first of its `examples` or `enum`.
pub(crate) fn own(doc: &Doc<'_, '_>, at: At) -> Option<Value> {
    for key in ["example", "default", "const"] {
        if let Some(v) = doc.get(at, key) {
            return Some(written(doc, v, 0));
        }
    }
    ["examples", "enum"].into_iter().find_map(|key| {
        let first = doc
            .get(at, key)
            .and_then(|v| doc.items(v).first().copied())?;
        Some(written(doc, first, 0))
    })
}

/// The schema's type: the first of `type` that is not `null`, else what its members imply.
fn kind(doc: &Doc<'_, '_>, at: At) -> Option<String> {
    let at = doc.resolve(at)?;
    if let Some(t) = doc.str(at, "type") {
        return Some(t);
    }
    if let Some(list) = doc.get(at, "type") {
        let tree = doc.tree(list.0)?;
        let types: Vec<String> = doc
            .items(list)
            .iter()
            .filter_map(|t| tree.as_str(t.1).map(|s| s.into_owned()))
            .collect();
        return types
            .iter()
            .find(|t| *t != "null")
            .or_else(|| types.first())
            .cloned();
    }
    if doc.get(at, "properties").is_some() {
        return Some(String::from("object"));
    }
    doc.get(at, "items").map(|_| String::from("array"))
}

/// A value made from the type, `format`, bounds and properties.
fn made(doc: &Doc<'_, '_>, at: At, all: bool, depth: usize) -> Value {
    match kind(doc, at).as_deref() {
        Some("string") => string(doc, at),
        Some("integer" | "number") => Value::Num(number(doc, at)),
        Some("boolean") => Value::Bool(false),
        Some("file") => Value::Bytes,
        Some("array") => {
            let item = doc.get(at, "items");
            let count = int(doc, at, "minItems").unwrap_or(1).clamp(1, 3);
            if int(doc, at, "maxItems") == Some(0) {
                return Value::Arr(Vec::new());
            }
            let one = item.map_or(Value::Null, |i| walk(doc, i, all, depth + 1));
            Value::Arr(vec![one; usize::try_from(count).unwrap_or(1)])
        }
        Some("object") => {
            let required: Vec<String> = doc
                .get(at, "required")
                .map(|r| doc.items(r))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|r| doc.tree(r.0)?.as_str(r.1).map(|s| s.into_owned()))
                .collect();
            let props = doc
                .get(at, "properties")
                .map(|p| doc.members(p))
                .unwrap_or_default();
            let members = props
                .into_iter()
                .filter(|(name, _)| all || required.contains(name))
                .map(|(name, s)| (name, walk(doc, s, all, depth + 1)))
                .collect();
            Value::Obj(members)
        }
        Some(_) | None => Value::Null,
    }
}

fn string(doc: &Doc<'_, '_>, at: At) -> Value {
    let text = match doc.str(at, "format").as_deref() {
        Some("binary") => return Value::Bytes,
        Some("date-time") => "2024-01-01T00:00:00Z",
        Some("date") => "2024-01-01",
        Some("time") => "00:00:00Z",
        Some("uuid") => "00000000-0000-4000-8000-000000000000",
        Some("email" | "idn-email") => "user@example.com",
        Some("uri" | "url" | "uri-reference" | "iri" | "iri-reference") => "https://example.com",
        Some("hostname" | "idn-hostname") => "example.com",
        Some("ipv4") => "192.0.2.1",
        Some("ipv6") => "2001:db8::1",
        Some("byte") => "AAAA",
        Some("duration") => "P1D",
        Some(_) | None => "string",
    };
    Value::Str(fitted(doc, at, text))
}

/// A text made as long as the schema's `minLength` and `maxLength` let it be.
pub(crate) fn fitted(doc: &Doc<'_, '_>, at: At, text: &str) -> String {
    let mut s = text.to_owned();
    let min = int(doc, at, "minLength").map_or(0, |n| usize::try_from(n).unwrap_or(0));
    while s.chars().count() < min.min(256) {
        s.push('x');
    }
    if let Some(max) = int(doc, at, "maxLength").and_then(|n| usize::try_from(n).ok()) {
        s = s.chars().take(max).collect();
    }
    s
}

/// A number within the schema's bounds: 1 unless they leave it out.
fn number(doc: &Doc<'_, '_>, at: At) -> String {
    let mut n = int(doc, at, "minimum").unwrap_or(1);
    if int(doc, at, "exclusiveMinimum").is_some_and(|m| m >= n) {
        n = int(doc, at, "exclusiveMinimum")
            .unwrap_or(n)
            .saturating_add(1);
    }
    if let Some(max) = int(doc, at, "maximum").filter(|&m| m < n) {
        n = max;
    }
    n.to_string()
}

/// An integer member (a number written with a fraction is read up to its point).
fn int(doc: &Doc<'_, '_>, at: At, key: &str) -> Option<i64> {
    let (d, node) = doc.get(at, key)?;
    let tree = doc.tree(d)?;
    match tree.value(node) {
        Json::Number(span) => tree.raw(span).split(['.', 'e', 'E']).next()?.parse().ok(),
        Json::Null | Json::Bool(_) | Json::String(_) | Json::Array(_) | Json::Object(_) => None,
    }
}

/// A value written in the description, copied.
pub(crate) fn written(doc: &Doc<'_, '_>, at: At, depth: usize) -> Value {
    let Some(tree) = doc.tree(at.0) else {
        return Value::Null;
    };
    if depth > MAX_JSON {
        return Value::Null;
    }
    match tree.value(at.1) {
        Json::Null => Value::Null,
        Json::Bool(b) => Value::Bool(b),
        Json::Number(span) => Value::Num(tree.raw(span).to_owned()),
        Json::String(_) => Value::Str(
            tree.as_str(at.1)
                .map(|s| s.into_owned())
                .unwrap_or_default(),
        ),
        Json::Array(_) => Value::Arr(
            doc.items(at)
                .into_iter()
                .map(|i| written(doc, i, depth + 1))
                .collect(),
        ),
        Json::Object(_) => Value::Obj(
            doc.members(at)
                .into_iter()
                .map(|(k, v)| (k, written(doc, v, depth + 1)))
                .collect(),
        ),
    }
}
