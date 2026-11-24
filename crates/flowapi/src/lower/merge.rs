//! Unions and intersections simplified. An intersection's parts that are shapes of the
//! schema's own (not a named type) merge into one shape; kinds another part rules out are
//! dropped from it; what is left adds nothing is dropped. Each rewrite keeps the values.

use super::kinds::step;
use crate::graph::{Branches, Graph, Kind, Kinds, Type};
use crate::id::TypeId;
use std::collections::HashMap;

mod branch;
mod distribute;

/// No more types than this many times the graph's own are made by merging; past it an
/// intersection stays as it is, which is sound.
const GROWTH: usize = 4;

pub(super) struct Merger<'g, 't> {
    pub(super) graph: &'g mut Graph<'t>,
    pub(super) kinds: &'g mut Vec<Kinds>,
    made: HashMap<Vec<TypeId>, TypeId>,
    named: Vec<bool>,
    pub(super) budget: usize,
}

/// Simplifies every union and intersection, those it makes among them.
pub(super) fn simplify(graph: &mut Graph<'_>, kinds: &mut Vec<Kinds>) {
    let mut named = vec![false; graph.len()];
    for root in &graph.roots {
        if let Some(slot) = named.get_mut(root.ty.index()) {
            *slot = true;
        }
    }
    let budget = graph.len().saturating_mul(GROWTH).saturating_add(10_000);
    let mut m = Merger {
        graph,
        kinds,
        made: HashMap::new(),
        named,
        budget,
    };
    let mut i = 0;
    while i < m.graph.len() {
        if let Some(id) = TypeId::new(i) {
            m.simplify(id);
        }
        i += 1;
    }
}

impl<'t> Merger<'_, 't> {
    pub(super) fn kind_of(&self, id: TypeId) -> Kinds {
        self.kinds.get(id.index()).copied().unwrap_or(Kinds::ALL)
    }

    pub(super) fn is_named(&self, id: TypeId) -> bool {
        self.named.get(id.index()).copied().unwrap_or(false)
    }

    fn set(&mut self, id: TypeId, kind: Kind<'t>) {
        if let Some(ty) = self.graph.types.get_mut(id.index()) {
            ty.kind = kind;
        }
        let k = step(self.graph, self.kinds, id);
        if let Some(slot) = self.kinds.get_mut(id.index()) {
            *slot = k;
        }
    }

    /// The intersection of two types, made once per pair.
    pub(super) fn both(&mut self, a: TypeId, b: TypeId) -> TypeId {
        let any = TypeId::FIRST;
        if a == b || b == any {
            return a;
        }
        if a == any {
            return b;
        }
        let mut key = vec![a, b];
        key.sort_unstable();
        if let Some(&t) = self.made.get(&key) {
            return t;
        }
        let t = self.graph.push(Type::of(Kind::Intersection(key.clone())));
        let k = self.kind_of(a).meet(self.kind_of(b));
        self.kinds.push(k);
        self.made.insert(key, t);
        t
    }

    fn simplify(&mut self, id: TypeId) {
        match self.graph.get(id).kind.clone() {
            Kind::Union(ms) => self.union(id, &ms),
            Kind::Intersection(ms) => self.intersection(id, &ms),
            Kind::Any | Kind::Never | Kind::Value(_) | Kind::Alias(_) => {}
        }
    }

    fn union(&mut self, id: TypeId, ms: &[TypeId]) {
        let mut out: Vec<TypeId> = Vec::new();
        for &m in ms {
            match &self.graph.get(m).kind {
                Kind::Any => return self.set(id, Kind::Any),
                Kind::Never => {}
                Kind::Union(inner) if !self.is_named(m) && m != id => {
                    out.extend(inner.iter().copied().filter(|t| *t != id));
                }
                Kind::Union(_) | Kind::Intersection(_) | Kind::Value(_) | Kind::Alias(_) => {
                    out.push(m)
                }
            }
        }
        dedupe(&mut out);
        let kind = match out.as_slice() {
            [] => Kind::Never,
            [one] => Kind::Alias(*one),
            _ => Kind::Union(out),
        };
        self.set(id, kind);
    }

    fn intersection(&mut self, id: TypeId, ms: &[TypeId]) {
        let mut parts: Vec<TypeId> = Vec::new();
        for &m in ms {
            match &self.graph.get(m).kind {
                Kind::Any => {}
                Kind::Never => return self.set(id, Kind::Never),
                Kind::Intersection(inner) if !self.is_named(m) && m != id => {
                    parts.extend(inner.iter().copied())
                }
                Kind::Intersection(_) | Kind::Union(_) | Kind::Value(_) | Kind::Alias(_) => {
                    parts.push(m)
                }
            }
        }
        dedupe(&mut parts);
        let allowed = parts
            .iter()
            .map(|&m| self.kind_of(m))
            .fold(Kinds::ALL, Kinds::meet);
        if allowed.is_empty() {
            return self.set(id, Kind::Never);
        }
        let (own, mut others): (Vec<TypeId>, Vec<TypeId>) = parts
            .into_iter()
            .partition(|&m| matches!(self.graph.get(m).kind, Kind::Value(_)) && !self.is_named(m));
        let mut merged: Option<Branches<'t>> = None;
        for m in own {
            let Kind::Value(b) = self.graph.get(m).kind.clone() else {
                continue;
            };
            merged = Some(match merged {
                None => b,
                Some(acc) if self.graph.len() < self.budget => self.meet(acc, &b),
                Some(acc) => {
                    others.push(m);
                    acc
                }
            });
        }
        if let Some(mut b) = merged {
            b.keep(allowed);
            if b.kinds().is_empty() {
                return self.set(id, Kind::Never);
            }
            if others.is_empty() {
                return self.set(id, Kind::Value(b));
            }
            if !branch::narrows(&b, allowed) {
                return self.set(id, collapse(others));
            }
            if self.distribute(&b, &mut others, allowed) {
                return self.set(id, collapse(others));
            }
            let k = b.kinds();
            others.push(self.graph.push(Type::of(Kind::Value(b))));
            self.kinds.push(k);
        }
        self.set(id, collapse(others));
    }
}

fn collapse(parts: Vec<TypeId>) -> Kind<'static> {
    match parts.as_slice() {
        [] => Kind::Any,
        [one] => Kind::Alias(*one),
        _ => Kind::Intersection(parts),
    }
}

fn dedupe(list: &mut Vec<TypeId>) {
    let mut seen = Vec::with_capacity(list.len());
    list.retain(|t| {
        let fresh = !seen.contains(t);
        if fresh {
            seen.push(*t);
        }
        fresh
    });
}
