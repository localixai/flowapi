//! JSON Pointers (RFC 6901) as URI fragments: resolved in a tree, and written for a node.

use crate::id::NodeId;
use crate::json::{Tree, Value};
use std::fmt::Write;

/// The node a fragment's JSON Pointer names, from `from`. The fragment is percent-decoded
/// first, as a URI fragment is (RFC 6901 §6). `None` when it names nothing.
#[must_use]
pub fn resolve_pointer(tree: &Tree<'_>, from: NodeId, fragment: &str) -> Option<NodeId> {
    let decoded = percent_decode(fragment)?;
    if decoded.is_empty() {
        return Some(from);
    }
    let rest = decoded.strip_prefix('/')?;
    let mut node = from;
    for token in rest.split('/') {
        let token = token.replace("~1", "/").replace("~0", "~");
        node = match tree.value(node) {
            Value::Object(_) => tree.get(node, &token)?,
            Value::Array(_) => {
                if token.len() > 1 && token.starts_with('0') {
                    return None;
                }
                *tree.items(node).get(token.parse::<usize>().ok()?)?
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => return None,
        };
    }
    Some(node)
}

/// The JSON Pointer from the root to a node, e.g. `/components/schemas/Pet`.
#[must_use]
pub fn node_pointer(tree: &Tree<'_>, node: NodeId) -> String {
    let mut tokens = Vec::new();
    let mut at = node;
    while let Some(parent) = tree.parent(at) {
        let token = match tree.value(parent) {
            Value::Array(_) => tree
                .items(parent)
                .iter()
                .position(|&n| n == at)
                .map(|i| i.to_string()),
            Value::Object(_) => tree
                .members(parent)
                .iter()
                .find(|m| m.value == at)
                .and_then(|m| tree.string(m.key))
                .map(|k| k.replace('~', "~0").replace('/', "~1")),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
        };
        tokens.push(token.unwrap_or_default());
        at = parent;
    }
    tokens.iter().rev().fold(String::new(), |mut out, t| {
        let _ = write!(out, "/{t}");
        out
    })
}

fn percent_decode(s: &str) -> Option<String> {
    if !s.contains('%') {
        return Some(s.to_owned());
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        if b == b'%' {
            let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
