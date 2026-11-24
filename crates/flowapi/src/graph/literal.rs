//! `enum` and `const` values that are arrays or objects, as exact types: an array is a tuple of
//! its items' exact types, an object is a closed object of its members', each required. With
//! them, the values are a union of exact types, which the schema's other keywords meet.

use super::build::Builder;
use super::narrow::{number_lit, string_lit};
use super::{Array, Bools, Branches, Kind, Kinds, Numbers, Object, Prop, Strings};
use crate::id::{NodeId, TypeId};
use crate::json::{Tree, Value};
use std::borrow::Cow;

impl<'t> Builder<'_, 't> {
    /// The union of the exact types of `values`, when one of them is an array or an object;
    /// `None` otherwise (scalars are narrowed in place).
    pub(super) fn literals(&mut self, tree: &Tree<'t>, values: &[NodeId]) -> Option<TypeId> {
        let compound = values
            .iter()
            .any(|&n| matches!(tree.value(n), Value::Array(_) | Value::Object(_)));
        if !compound {
            return None;
        }
        let members: Vec<TypeId> = values.iter().map(|&n| self.exact(tree, n)).collect();
        Some(self.make(Kind::Union(members)))
    }

    /// The type of exactly one JSON value. JSON nests at most 512 levels (the reader's bound),
    /// which bounds this recursion.
    fn exact(&mut self, tree: &Tree<'t>, node: NodeId) -> TypeId {
        let never = TypeId::new(1).unwrap_or(self.any);
        let mut b = Branches::of(Kinds::NONE, self.any);
        match tree.value(node) {
            Value::Null => b.null = true,
            Value::Bool(v) => {
                b.boolean = Bools {
                    allow_true: v,
                    allow_false: !v,
                }
            }
            Value::Number(span) => {
                b.number = Some(Numbers {
                    integer: false,
                    lits: Some(vec![number_lit(tree.raw(span))]),
                });
            }
            Value::String(span) => {
                b.string = Some(Strings {
                    format: None,
                    lits: Some(vec![string_lit(tree, span)]),
                });
            }
            Value::Array(_) => {
                let prefix: Vec<TypeId> = tree
                    .items(node)
                    .to_vec()
                    .into_iter()
                    .map(|n| self.exact(tree, n))
                    .collect();
                let min = prefix.len();
                b.array = Some(Array {
                    prefix,
                    rest: never,
                    min,
                });
            }
            Value::Object(_) => {
                let mut props: Vec<Prop<'t>> = Vec::new();
                for m in tree.members(node).to_vec() {
                    let raw = tree.raw(m.key);
                    let name = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
                    let ty = self.exact(tree, m.value);
                    props.retain(|p| p.name != name);
                    props.push(Prop {
                        raw: Cow::Borrowed(raw),
                        name,
                        ty,
                        required: true,
                    });
                }
                b.object = Some(Object {
                    props,
                    patterns: Vec::new(),
                    rest: never,
                    extra: never,
                });
            }
        }
        self.make(Kind::Value(b))
    }
}
