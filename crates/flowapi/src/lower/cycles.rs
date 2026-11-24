//! A type that is its own member, through aliases, unions and intersections only (`A = A | null`,
//! `{"$ref": "#"}`), names no shape to stop at: no target can state it, and a validator never
//! ends on it. The edge back is replaced by `Any`, which widens the type (sound).

use super::kinds::members;
use crate::graph::{Graph, Kind};
use crate::id::TypeId;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    New,
    Open,
    Done,
}

/// Breaks every direct cycle; the edge followed last in a depth-first walk is the one replaced.
pub(super) fn break_direct(graph: &mut Graph<'_>) {
    let mut marks = vec![Mark::New; graph.len()];
    let any = TypeId::FIRST;
    for start in 0..graph.len() {
        if marks.get(start) != Some(&Mark::New) {
            continue;
        }
        let Some(start) = TypeId::new(start) else {
            continue;
        };
        // The walk is a stack of (type, next member to visit): no recursion (G1).
        let mut stack = vec![(start, 0usize)];
        set(&mut marks, start, Mark::Open);
        while let Some(&mut (id, ref mut next)) = stack.last_mut() {
            let list = members(&graph.get(id).kind);
            let Some(&m) = list.get(*next) else {
                set(&mut marks, id, Mark::Done);
                stack.pop();
                continue;
            };
            let position = *next;
            *next += 1;
            match marks.get(m.index()).copied().unwrap_or(Mark::Done) {
                Mark::New => {
                    set(&mut marks, m, Mark::Open);
                    stack.push((m, 0));
                }
                Mark::Open => replace(graph, id, position, any),
                Mark::Done => {}
            }
        }
    }
}

fn set(marks: &mut [Mark], id: TypeId, mark: Mark) {
    if let Some(slot) = marks.get_mut(id.index()) {
        *slot = mark;
    }
}

fn replace(graph: &mut Graph<'_>, id: TypeId, position: usize, with: TypeId) {
    let Some(ty) = graph.types.get_mut(id.index()) else {
        return;
    };
    match &mut ty.kind {
        Kind::Union(ms) | Kind::Intersection(ms) => {
            if let Some(slot) = ms.get_mut(position) {
                *slot = with;
            }
        }
        Kind::Alias(t) => *t = with,
        Kind::Any | Kind::Never | Kind::Value(_) => {}
    }
}
