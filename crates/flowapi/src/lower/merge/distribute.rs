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

    /// The shapes of an unnamed union of unnamed shapes, nested unnamed unions flattened and
    /// `Never` dropped, at most [`MAX_BRANCHES`] of them.
    fn shapes(&self, id: TypeId) -> Option<Vec<Branches<'t>>> {
        let id = self.through(id)?;
        // A union of one member is an alias by now: its one shape is distributed over as well.
        if !matches!(self.graph.get(id).kind, Kind::Union(_) | Kind::Value(_)) {
            return None;
        }
        if self.is_named(id) || self.graph.len() >= self.budget {
            return None;
        }
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(at) = stack.pop() {
            let at = self.through(at)?;
            match &self.graph.get(at).kind {
                Kind::Value(b) => out.push(b.clone()),
                Kind::Never => {}
                Kind::Union(members) if !self.is_named(at) => stack.extend(members.iter().rev()),
                Kind::Any | Kind::Union(_) | Kind::Intersection(_) | Kind::Alias(_) => return None,
            }
            if out.len() > MAX_BRANCHES {
                return None;
            }
        }
        Some(out)
    }

    /// `(A | B) & R` as `(A & R) | (B & R)` when no part is a shape of the schema's own, so each
    /// branch is one shape the parts after it meet; the parts are unnamed (a named one keeps
    /// its name in the output). `None` when no part is such a union.
    pub(super) fn split(&mut self, others: &[TypeId]) -> Option<Kind<'t>> {
        if others.len() < 2 || others.iter().any(|&m| self.is_named(m)) {
            return None;
        }
        let (at, shapes) = others.iter().enumerate().find_map(|(i, &m)| {
            // Only a union is split: a lone shape is merged, not split.
            let union = matches!(self.graph.get(self.through(m)?).kind, Kind::Union(_));
            Some((i, self.shapes(m).filter(|_| union)?))
        })?;
        let rest: Vec<TypeId> = others
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != at)
            .map(|(_, &m)| m)
            .collect();
        let mut branches = Vec::with_capacity(shapes.len());
        for b in shapes {
            let k = b.kinds();
            let value = self.graph.push(Type::of(Kind::Value(b)));
            self.kinds.push(k);
            let mut parts = vec![value];
            parts.extend(rest.iter().copied());
            let both = self.graph.push(Type::of(Kind::Intersection(parts)));
            let k = super::super::kinds::step(self.graph, self.kinds, both);
            self.kinds.push(k);
            branches.push(both);
        }
        Some(match branches.as_slice() {
            [] => Kind::Never,
            [one] => Kind::Alias(*one),
            _ => Kind::Union(branches),
        })
    }

    /// The unnamed shape an unnamed alias stands for, if that is what it is.
    pub(super) fn unnamed_shape(&self, id: TypeId) -> Option<TypeId> {
        let at = self.through(id)?;
        matches!(self.graph.get(at).kind, Kind::Value(_)).then_some(at)
    }

    /// An unnamed alias followed to what it names; `None` at a named type.
    pub(super) fn through(&self, id: TypeId) -> Option<TypeId> {
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
