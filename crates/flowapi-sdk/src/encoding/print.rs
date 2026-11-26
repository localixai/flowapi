//! An operation's encodings as the TypeScript object its table entry holds: by media type, the
//! Media Type Object's three fields (`encoding`, `prefixEncoding`, `itemEncoding`), each Encoding
//! Object with the fields read (`FlowapiPart` in the SDK's runtime).

use crate::model::{Encoding, Part, Parts};
use crate::print::object;
use flowapi_ts::string_literal as lit;

/// `{ "<media type>": { encoding: …, prefixEncoding: …, itemEncoding: … }, … }`.
pub(crate) fn literal(encodings: &[Encoding]) -> String {
    let bodies: Vec<String> = encodings
        .iter()
        .map(|e| format!("{}: {}", lit(&e.media_type), object(&members(&e.parts))))
        .collect();
    object(&bodies)
}

/// The three fields, those that say anything.
fn members(parts: &Parts) -> Vec<String> {
    let mut out = Vec::new();
    if !parts.by_name.is_empty() {
        let named: Vec<String> = parts
            .by_name
            .iter()
            .map(|(name, p)| format!("{}: {}", lit(name), part(p)))
            .collect();
        out.push(format!("encoding: {}", object(&named)));
    }
    if !parts.prefix.is_empty() {
        let items: Vec<String> = parts.prefix.iter().map(part).collect();
        out.push(format!("prefixEncoding: [{}]", items.join(", ")));
    }
    if let Some(item) = &parts.item {
        out.push(format!("itemEncoding: {}", part(item)));
    }
    out
}

/// One Encoding Object.
fn part(p: &Part) -> String {
    let mut fields = Vec::new();
    if let Some(t) = &p.content_type {
        fields.push(format!("contentType: {}", lit(t)));
    }
    if !p.headers.is_empty() {
        let headers: Vec<String> = p
            .headers
            .iter()
            .map(|(n, v)| format!("{}: {}", lit(n), lit(v)))
            .collect();
        fields.push(format!("headers: {}", object(&headers)));
    }
    if let Some(s) = &p.style {
        fields.push(format!("style: {}", lit(s)));
    }
    if let Some(e) = p.explode {
        fields.push(format!("explode: {e}"));
    }
    if let Some(r) = p.allow_reserved {
        fields.push(format!("allowReserved: {r}"));
    }
    fields.extend(members(&p.parts));
    object(&fields)
}
