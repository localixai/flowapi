//! `if`, `then` and `else`: (if ∧ then) ∨ (¬if ∧ else), a missing side any value. Where `if`
//! is read on its own (`local.rs`), both sides of the test are types: what it may accept (its
//! kinds and values, objects holding its `required` names), and what it may refuse (the kinds it
//! does not accept every value of, and objects missing one of the names). Stated in full when it
//! tests kinds and `required` alone; any other `if` passes as any value either way, so the type
//! is then ∨ else, wider than the schema (sound).

use super::build::Builder;
use super::{Branches, Kind, Kinds};
use crate::id::{NodeId, TypeId};
use crate::schema::{Keyword, Resource};

impl<'t> Builder<'_, 't> {
    /// The type of an `if` at `test` with its `then` and `else`; `None` when neither is there.
    pub(super) fn conditional(
        &mut self,
        at: &Resource,
        test: NodeId,
        then: Option<NodeId>,
        otherwise: Option<NodeId>,
        lossy: &mut Vec<Keyword>,
    ) -> Option<TypeId> {
        if then.is_none() && otherwise.is_none() {
            return None;
        }
        let local = self.local(at, test);
        if !local
            .as_ref()
            .is_some_and(|l| l.exact && l.values.kinds() == l.whole)
        {
            lossy.push(Keyword::If);
        }
        let (pass, fail) = match local {
            None => (self.any, self.any),
            Some(l) => {
                let mut members = Vec::new();
                let others = Kinds::ALL.without(l.whole);
                if !others.is_empty() {
                    members.push(self.make(Kind::Value(Branches::of(others, self.any))));
                }
                if l.whole.has(Kinds::OBJECT) {
                    for key in &l.required {
                        members.push(self.object_only(vec![key.clone()], false, Kinds::OBJECT));
                    }
                }
                let fail = self.make(Kind::Union(members));
                let values = self.make(Kind::Value(l.values));
                let holding = self.object_only(l.required, true, Kinds::ALL);
                (self.make(Kind::Intersection(vec![values, holding])), fail)
            }
        };
        let mut side = |cond: TypeId, branch: Option<NodeId>| match branch {
            None => cond,
            Some(n) => {
                let t = self.sub(at, n);
                self.make(Kind::Intersection(vec![cond, t]))
            }
        };
        let members = vec![side(pass, then), side(fail, otherwise)];
        Some(self.make(Kind::Union(members)))
    }
}
