//! `unevaluatedProperties: false` and `unevaluatedItems: false` as a closed object or tuple. A
//! property is evaluated by the schema or any schema applied in place (`allOf`, `anyOf`,
//! `oneOf`, `if`, `then`, `else`, `dependentSchemas`, `$ref`); taking every name any of them
//! could evaluate is wider than the schema, never narrower (sound). Where one evaluates names no
//! list can hold (`patternProperties`, `additionalProperties`, `items`, `contains`, a dynamic
//! reference), the type stays open.

use super::build::Builder;
use crate::json::Value;
use crate::schema::{Resource, Target, scoped};
use std::borrow::Cow;

/// How many schemas a walk reads before it gives up (the type stays open).
const MAX_WALK: usize = 256;

/// What a walk found: the property names, and the most positions, any schema could evaluate.
#[derive(Default)]
pub(super) struct Evaluated<'t> {
    pub(super) names: Vec<(Cow<'t, str>, Cow<'t, str>)>,
    pub(super) items: usize,
}

impl<'t> Builder<'_, 't> {
    /// What the schema at `at` and those applied in place could evaluate; `None` when one of them
    /// evaluates what no list holds (of properties when `props`, of items otherwise).
    pub(super) fn evaluated(&self, at: &Resource, props: bool) -> Option<Evaluated<'t>> {
        let docs = self.docs;
        let mut out = Evaluated::default();
        let mut seen: Vec<(crate::id::DocId, crate::id::NodeId)> = Vec::new();
        let mut stack = vec![(at.clone(), true)];
        while let Some((r, root)) = stack.pop() {
            if seen.contains(&(r.doc, r.node)) {
                continue;
            }
            if seen.len() >= MAX_WALK {
                return None;
            }
            seen.push((r.doc, r.node));
            let tree = &docs.get(r.doc)?.tree;
            match tree.value(r.node) {
                Value::Bool(_) => continue,
                Value::Object(_) => {}
                Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => return None,
            }
            let r = scoped(tree, &r);
            let open = if props {
                [
                    "patternProperties",
                    "additionalProperties",
                    "$dynamicRef",
                    "$recursiveRef",
                ]
                .as_slice()
            } else {
                [
                    "items",
                    "additionalItems",
                    "contains",
                    "$dynamicRef",
                    "$recursiveRef",
                ]
                .as_slice()
            };
            if open.iter().any(|k| tree.get(r.node, k).is_some()) {
                return None;
            }
            let own_key = if props {
                "unevaluatedProperties"
            } else {
                "unevaluatedItems"
            };
            if !root
                && tree
                    .get(r.node, own_key)
                    .is_some_and(|n| tree.as_bool(n) != Some(false))
            {
                return None;
            }
            if props {
                for m in tree
                    .get(r.node, "properties")
                    .map_or(&[][..], |n| tree.members(n))
                {
                    let raw = tree.raw(m.key);
                    let name = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
                    if out.names.iter().all(|(_, n)| *n != name) {
                        out.names.push((Cow::Borrowed(raw), name));
                    }
                }
            } else if let Some(p) = tree.get(r.node, "prefixItems") {
                out.items = out.items.max(tree.items(p).len());
            }
            applied(tree, &r, &mut stack);
            if let Some(reference) = tree.get(r.node, "$ref").and_then(|n| tree.as_str(n)) {
                match self.index.resolve(docs, &r.base, &reference, r.dialect) {
                    Target::Found(t) => stack.push((t, false)),
                    Target::Missing(_) => return None,
                }
            }
        }
        Some(out)
    }
}

/// The schemas applied in place under `r`, pushed to be walked.
fn applied(tree: &crate::json::Tree<'_>, r: &Resource, stack: &mut Vec<(Resource, bool)>) {
    for key in ["allOf", "anyOf", "oneOf"] {
        for &n in tree.get(r.node, key).map_or(&[][..], |a| tree.items(a)) {
            stack.push((
                Resource {
                    node: n,
                    ..r.clone()
                },
                false,
            ));
        }
    }
    for key in ["if", "then", "else"] {
        if let Some(n) = tree.get(r.node, key) {
            stack.push((
                Resource {
                    node: n,
                    ..r.clone()
                },
                false,
            ));
        }
    }
    for m in tree
        .get(r.node, "dependentSchemas")
        .map_or(&[][..], |n| tree.members(n))
    {
        stack.push((
            Resource {
                node: m.value,
                ..r.clone()
            },
            false,
        ));
    }
}
