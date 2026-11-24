//! Which JSON kinds each type allows, as the greatest fixpoint over the graph: every type starts
//! with every kind and loses those its shape rules out, until nothing changes. A cycle keeps
//! the kinds no step rules out, which is sound.

use crate::graph::{Graph, Kind, Kinds};
use crate::id::TypeId;

/// The kinds of every type, by its index.
pub(super) fn of(graph: &Graph<'_>) -> Vec<Kinds> {
    let mut kinds = vec![Kinds::ALL; graph.len()];
    let mut dependents: Vec<Vec<TypeId>> = vec![Vec::new(); graph.len()];
    for (id, ty) in graph.iter() {
        for &m in members(&ty.kind) {
            if let Some(list) = dependents.get_mut(m.index()) {
                list.push(id);
            }
        }
    }
    let mut work: Vec<TypeId> = (0..graph.len()).rev().filter_map(TypeId::new).collect();
    while let Some(id) = work.pop() {
        let next = step(graph, &kinds, id);
        if kinds.get(id.index()) != Some(&next) {
            if let Some(slot) = kinds.get_mut(id.index()) {
                *slot = next;
            }
            work.extend(dependents.get(id.index()).into_iter().flatten());
        }
    }
    kinds
}

/// A type's kinds from its members' current kinds.
pub(super) fn step(graph: &Graph<'_>, kinds: &[Kinds], id: TypeId) -> Kinds {
    let get = |t: &TypeId| kinds.get(t.index()).copied().unwrap_or(Kinds::ALL);
    match &graph.get(id).kind {
        Kind::Any => Kinds::ALL,
        Kind::Never => Kinds::NONE,
        Kind::Value(b) => b.kinds(),
        Kind::Union(ms) => ms.iter().map(get).fold(Kinds::NONE, Kinds::union),
        Kind::Intersection(ms) => ms.iter().map(get).fold(Kinds::ALL, Kinds::meet),
        Kind::Alias(t) => get(t),
    }
}

/// The types a union, an intersection or an alias is made of.
pub(super) fn members<'k>(kind: &'k Kind<'_>) -> &'k [TypeId] {
    match kind {
        Kind::Union(ms) | Kind::Intersection(ms) => ms,
        Kind::Alias(t) => std::slice::from_ref(t),
        Kind::Any | Kind::Never | Kind::Value(_) => &[],
    }
}
