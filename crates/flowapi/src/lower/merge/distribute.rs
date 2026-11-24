//! An intersection of a shape with a union of shapes, distributed: `S & (A | B)` is
//! `(S ∧ A) | (S ∧ B)`, each merged into one shape. The values are the same; a closed shape
//! stays closed in each branch, which an intersection with an open one would hide.

use super::Merger;
use crate::graph::{Branches, Kind, Kinds, Type};
use crate::id::TypeId;

/// The most branches a union may have to be distributed over.
const MAX_BRANCHES: usize = 16;

impl<'t> Merger<'_, 't> {
    /// Distributes `b` over the first union among `others` whose members are all unnamed shapes;
    /// `true` when it did (the union is replaced and `b` is in each of its branches).
    pub(super) fn distribute(
        &mut self,
        b: &Branches<'t>,
        others: &mut [TypeId],
        allowed: Kinds,
    ) -> bool {
        let Some((at, members)) = others.iter().enumerate().find_map(|(i, &m)| {
            let shapes = self.shapes(m)?;
            Some((i, shapes))
        }) else {
            return false;
        };
        let mut branches = Vec::with_capacity(members.len());
        for member in members {
            let mut merged = self.meet(b.clone(), &member);
            merged.keep(allowed);
            if merged.kinds().is_empty() {
                continue;
            }
            let k = merged.kinds();
            branches.push(self.graph.push(Type::of(Kind::Value(merged))));
            self.kinds.push(k);
        }
        let kind = match branches.as_slice() {
            [] => Kind::Never,
            [one] => Kind::Alias(*one),
            _ => Kind::Union(branches),
        };
        let union = self.graph.push(Type::of(kind));
        let k = super::super::kinds::step(self.graph, self.kinds, union);
        self.kinds.push(k);
        if let Some(slot) = others.get_mut(at) {
            *slot = union;
        }
        true
    }

    /// The shapes of an unnamed union of unnamed shapes, at most [`MAX_BRANCHES`] of them.
    fn shapes(&self, id: TypeId) -> Option<Vec<Branches<'t>>> {
        let Kind::Union(members) = &self.graph.get(id).kind else {
            return None;
        };
        if self.is_named(id) || members.len() > MAX_BRANCHES || self.graph.len() >= self.budget {
            return None;
        }
        members
            .iter()
            .map(|&m| {
                let m = self.through(m)?;
                match &self.graph.get(m).kind {
                    Kind::Value(b) => Some(b.clone()),
                    Kind::Any
                    | Kind::Never
                    | Kind::Union(_)
                    | Kind::Intersection(_)
                    | Kind::Alias(_) => None,
                }
            })
            .collect()
    }

    /// An unnamed alias followed to what it names; `None` at a named type.
    fn through(&self, id: TypeId) -> Option<TypeId> {
        let mut at = id;
        for _ in 0..=self.graph.len() {
            if self.is_named(at) {
                return None;
            }
            match &self.graph.get(at).kind {
                Kind::Alias(next) => at = *next,
                Kind::Any
                | Kind::Never
                | Kind::Value(_)
                | Kind::Union(_)
                | Kind::Intersection(_) => return Some(at),
            }
        }
        None
    }
}
