//! A schema's own constraints, the keywords that narrow the kinds and shapes of values: `type`,
//! `enum`, `const`, the bounds of numbers and strings (`limits.rs`), the object and array
//! keywords, `nullable` (3.0). Each narrows only its kind: `properties` says nothing of a string,
//! so without `type` a string still passes (2020-12 §7.6).

use super::build::Builder;
use super::limits;
use super::narrow::restrict;
use super::{Branches, Kinds};
use crate::id::{NodeId, TypeId};
use crate::json::{Tree, Value};
use crate::schema::{Keyword, Resource};
use std::borrow::Cow;

/// The keywords that narrow a schema's own values, each with its node (`read.rs` routes them).
#[derive(Default)]
pub(super) struct Own(Vec<(Keyword, NodeId)>);

impl Own {
    /// Sets a keyword's node; a key written twice is the last one (as the reader keeps it).
    pub(super) fn set(&mut self, keyword: Keyword, value: NodeId) {
        self.0.retain(|(k, _)| *k != keyword);
        self.0.push((keyword, value));
    }

    /// A keyword's node.
    pub(super) fn get(&self, keyword: Keyword) -> Option<NodeId> {
        self.0.iter().find(|(k, _)| *k == keyword).map(|x| x.1)
    }

    /// Whether every keyword there is one `keep` takes.
    pub(super) fn only(&self, keep: impl Fn(Keyword) -> bool) -> bool {
        self.0.iter().all(|(k, _)| keep(*k))
    }

    /// Whether a keyword narrows: `format` is an annotation, `nullable` needs a `type`.
    fn narrows(&self) -> bool {
        self.0
            .iter()
            .any(|(k, _)| !matches!(k, Keyword::Format | Keyword::Nullable))
    }
}

impl<'t> Builder<'_, 't> {
    /// The values the schema's own keywords allow; `None` when none narrows them.
    pub(super) fn own(
        &mut self,
        at: &Resource,
        own: &Own,
        parts: &mut Vec<TypeId>,
        lossy: &mut Vec<Keyword>,
    ) -> Option<Branches<'t>> {
        if !own.narrows() {
            return None;
        }
        let tree = &self.docs.get(at.doc)?.tree;
        let (bad, mut b) = self.kinds(tree, own);
        if let Some(name) = bad {
            let loc = self.docs.loc(at.doc, own.get(Keyword::Type)?);
            let text = format!("`type` names no JSON type: \"{name}\"");
            self.diags.error("schema.type", loc, text);
        }
        if b.object.is_some() {
            b.object = self.object(tree, at, own, lossy);
        }
        if b.array.is_some() {
            b.array = self.array(tree, at, own, lossy);
        }
        for values in values_of(tree, own) {
            parts.extend(self.literals(tree, &values));
        }
        lossy.extend(scalars(&mut b, tree, at, own));
        (b != Branches::of(Kinds::ALL, self.any)).then_some(b)
    }

    /// The kinds `type`, `nullable` and `format` allow, and a name `type` gives that is no JSON
    /// type (any kind for it).
    pub(super) fn kinds(&self, tree: &Tree<'t>, own: &Own) -> (Option<Cow<'t, str>>, Branches<'t>) {
        let ty = own.get(Keyword::Type);
        let nodes = ty.map_or(Vec::new(), |n| match tree.value(n) {
            Value::Array(_) => tree.items(n).to_vec(),
            Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Object(_) => {
                vec![n]
            }
        });
        let names: Vec<Cow<'t, str>> = nodes.into_iter().filter_map(|n| tree.as_str(n)).collect();
        let (mut kinds, mut bad) = (Kinds::NONE, None);
        for name in &names {
            kinds = kinds.union(match name.as_ref() {
                "null" => Kinds::NULL,
                "boolean" => Kinds::BOOLEAN,
                "number" | "integer" => Kinds::NUMBER,
                "string" => Kinds::STRING,
                "array" => Kinds::ARRAY,
                "object" => Kinds::OBJECT,
                _ => {
                    bad = bad.or_else(|| Some(name.clone()));
                    Kinds::ALL
                }
            });
        }
        let mut b = Branches::of(if names.is_empty() { Kinds::ALL } else { kinds }, self.any);
        if let (Some(n), Some(_)) = (own.get(Keyword::Nullable), ty) {
            b.null |= tree.as_bool(n) == Some(true);
        }
        if let Some(number) = b.number.as_mut() {
            number.integer =
                names.iter().any(|n| n == "integer") && !names.iter().any(|n| n == "number");
        }
        let format = own.get(Keyword::Format).and_then(|n| tree.as_str(n));
        if let (Some(s), Some(f)) = (b.string.as_mut(), format) {
            s.format = Some(f);
        }
        (bad, b)
    }
}

/// Narrows the branches to the values of `enum` and `const` and to the bounds; returns the
/// keywords they leave unstated.
pub(super) fn scalars<'t>(
    b: &mut Branches<'t>,
    tree: &Tree<'t>,
    at: &Resource,
    own: &Own,
) -> Vec<Keyword> {
    for values in values_of(tree, own) {
        restrict(b, tree, &values);
    }
    limits::apply(own, b, tree, at.dialect)
}

/// The values of the `enum` (its items) and of the `const` (itself).
fn values_of(tree: &Tree<'_>, own: &Own) -> impl Iterator<Item = Vec<NodeId>> {
    let listed = own.get(Keyword::Enum).map(|n| tree.items(n).to_vec());
    listed
        .into_iter()
        .chain(own.get(Keyword::Const).map(|n| vec![n]))
}
