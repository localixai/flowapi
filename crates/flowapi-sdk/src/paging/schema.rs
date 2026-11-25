//! A response's or a parameter's schema read as far as pagination needs: its types, a property
//! by name or by a dotted path, an array's items, its property names. References are followed
//! (`Doc::resolve`) and `allOf`, `anyOf` and `oneOf` are looked through, each walk with
//! its own bounded list (G1). Where a schema says nothing (no `type`, no `properties`), the
//! answer is "unknown", never "missing": a check errs only on what a schema does say.

use crate::doc::{At, Doc};

/// The most nodes one question visits.
const MAX: usize = 64;

/// What a schema says its types are.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Types {
    /// It says nothing.
    Unknown,
    /// These, as JSON Schema names them.
    Known(Vec<String>),
}

impl Types {
    /// Whether a value of one of `wanted` may be there.
    pub(crate) fn allows(&self, wanted: &[&str]) -> bool {
        match self {
            Self::Unknown => true,
            Self::Known(all) => all.iter().any(|t| wanted.contains(&t.as_str())),
        }
    }

    /// The types for an issue: `string`, `integer or null`.
    pub(crate) fn show(&self) -> String {
        match self {
            Self::Unknown => String::from("anything"),
            Self::Known(all) => all.join(" or "),
        }
    }
}

/// A property looked for.
pub(crate) enum Prop {
    /// Found: its schema.
    Found(At),
    /// The schema names its properties and this is not one: these are.
    Missing(Vec<String>),
    /// The schema names no properties: it may hold any.
    Open,
}

/// The node itself and every schema its `allOf`, `anyOf` and `oneOf` reach, references
/// followed, each once, at most [`MAX`].
fn reach(doc: &Doc<'_, '_>, at: At) -> Vec<At> {
    let mut out: Vec<At> = Vec::new();
    let mut queue = vec![at];
    while let Some(next) = queue.pop() {
        let Some(node) = doc.resolve(next) else {
            continue;
        };
        if out.contains(&node) || out.len() >= MAX {
            continue;
        }
        out.push(node);
        for key in ["oneOf", "anyOf", "allOf"] {
            if let Some(list) = doc.get(node, key) {
                queue.extend(doc.items(list).into_iter().rev());
            }
        }
    }
    out
}

/// What the schema says its types are: `type` (a name or a list), else `object` where it has
/// `properties` and `array` where it has `items`.
pub(crate) fn types(doc: &Doc<'_, '_>, at: At) -> Types {
    let mut all: Vec<String> = Vec::new();
    for node in reach(doc, at) {
        let mut found: Vec<String> = Vec::new();
        if let Some(t) = doc.get(node, "type") {
            if let Some(name) = doc.tree(t.0).and_then(|tree| tree.as_str(t.1)) {
                found.push(name.into_owned());
            }
            for item in doc.items(t) {
                if let Some(name) = doc.tree(item.0).and_then(|tree| tree.as_str(item.1)) {
                    found.push(name.into_owned());
                }
            }
        } else if doc.get(node, "properties").is_some() {
            found.push(String::from("object"));
        } else if doc.get(node, "items").is_some() {
            found.push(String::from("array"));
        }
        for t in found {
            if !all.contains(&t) {
                all.push(t);
            }
        }
    }
    if all.is_empty() {
        Types::Unknown
    } else {
        Types::Known(all)
    }
}

/// The property `name` of an object schema.
pub(crate) fn property(doc: &Doc<'_, '_>, at: At, name: &str) -> Prop {
    let mut named: Vec<String> = Vec::new();
    let mut closed = false;
    for node in reach(doc, at) {
        if let Some(props) = doc.get(node, "properties") {
            closed = true;
            for (key, value) in doc.members(props) {
                if key == name {
                    return Prop::Found(value);
                }
                if !named.contains(&key) {
                    named.push(key);
                }
            }
        }
    }
    if closed {
        Prop::Missing(named)
    } else {
        Prop::Open
    }
}

/// The schema at a dotted path (`result_info.cursor`); `Ok(None)` where a schema on the way names
/// no properties; `Err` with the segment missing and the names there.
pub(crate) fn path(
    doc: &Doc<'_, '_>,
    at: At,
    dotted: &str,
) -> Result<Option<At>, (String, Vec<String>)> {
    let mut here = at;
    for segment in dotted.split('.') {
        match property(doc, here, segment) {
            Prop::Found(next) => here = next,
            Prop::Open => return Ok(None),
            Prop::Missing(named) => return Err((segment.to_owned(), named)),
        }
    }
    Ok(Some(here))
}

/// An array schema's items.
pub(crate) fn items(doc: &Doc<'_, '_>, at: At) -> Option<At> {
    reach(doc, at)
        .into_iter()
        .find_map(|node| doc.get(node, "items").filter(|i| doc.is(*i, false)))
}

/// An object schema's properties, each with its schema, in the order written.
pub(crate) fn properties(doc: &Doc<'_, '_>, at: At) -> Vec<(String, At)> {
    let mut out: Vec<(String, At)> = Vec::new();
    for node in reach(doc, at) {
        if let Some(props) = doc.get(node, "properties") {
            for (key, value) in doc.members(props) {
                if !out.iter().any(|(k, _)| *k == key) {
                    out.push((key, value));
                }
            }
        }
    }
    out
}
