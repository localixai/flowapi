//! `dependentRequired` and `dependentSchemas` (and `dependencies` before 2019-09) as unions an
//! object picks a side of: the key is there and what it brings holds, or the key is not there
//! (`foo?: never`). Values that are not objects pass either side, as the keywords leave them.

use super::build::Builder;
use super::{Branches, Kind, Kinds, Object, Prop};
use crate::id::{NodeId, TypeId};
use crate::json::{Tree, Value};
use crate::schema::Resource;
use std::borrow::Cow;

/// At most this many keys are made unions; past it the keyword stays unchecked (sound).
const MAX_KEYS: usize = 3;

impl<'t> Builder<'_, 't> {
    /// The unions of a `dependentRequired`, `dependentSchemas` or `dependencies` value; `None`
    /// when it has more keys than [`MAX_KEYS`] or is not an object.
    pub(super) fn dependent(&mut self, at: &Resource, value: NodeId) -> Option<Vec<TypeId>> {
        let tree = &self.docs.get(at.doc)?.tree;
        let members = tree.members(value);
        if !matches!(tree.value(value), Value::Object(_)) || members.len() > MAX_KEYS {
            return None;
        }
        let mut unions = Vec::with_capacity(members.len());
        for m in members {
            let raw = tree.raw(m.key);
            let name = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
            let mut needed = vec![(Cow::Borrowed(raw), name.clone())];
            let schema = match tree.value(m.value) {
                Value::Array(_) => {
                    needed.extend(names(tree, m.value)?);
                    None
                }
                Value::Object(_) | Value::Bool(_) => Some(self.sub(at, m.value)),
                Value::Null | Value::Number(_) | Value::String(_) => return None,
            };
            let present = self.object_only(needed, true);
            let present = match schema {
                Some(s) => self.make(Kind::Intersection(vec![present, s])),
                None => present,
            };
            let absent = self.object_only(vec![(Cow::Borrowed(raw), name)], false);
            unions.push(self.make(Kind::Union(vec![present, absent])));
        }
        Some(unions)
    }

    /// Objects with the keys given all there (`present`), or the one key given not there, every
    /// other value besides when not `present`.
    fn object_only(&mut self, keys: Vec<(Cow<'t, str>, Cow<'t, str>)>, present: bool) -> TypeId {
        let never = TypeId::new(1).unwrap_or(self.any);
        let mut b = Branches::of(if present { Kinds::OBJECT } else { Kinds::ALL }, self.any);
        let props = keys
            .into_iter()
            .map(|(raw, name)| Prop {
                raw,
                name,
                ty: if present { self.any } else { never },
                required: present,
            })
            .collect();
        b.object = Some(Object {
            props,
            patterns: Vec::new(),
            rest: self.any,
            extra: self.any,
        });
        self.make(Kind::Value(b))
    }
}

/// The keys of a `dependentRequired` list.
fn names<'t>(tree: &Tree<'t>, list: NodeId) -> Option<Vec<(Cow<'t, str>, Cow<'t, str>)>> {
    tree.items(list)
        .iter()
        .map(|&n| match tree.value(n) {
            Value::String(span) => {
                let raw = tree.raw(span);
                Some((
                    Cow::Borrowed(raw),
                    tree.string(span).unwrap_or(Cow::Borrowed(raw)),
                ))
            }
            Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::Array(_)
            | Value::Object(_) => None,
        })
        .collect()
}
