//! Which types are declared, in the output's order: the description's own, then each type that
//! needs a name, in the order a walk from them reaches it. The walks keep their own stacks: no
//! recursion, whatever the graph (G1).

use super::MAX_INLINE;
use crate::graph::{Graph, Kind};
use crate::id::TypeId;

/// A type to declare, and the name the description gives it, if any.
pub(super) struct Wanted {
    pub(super) ty: TypeId,
    pub(super) name: Option<String>,
}

/// The types each other type refers to.
fn children(graph: &Graph<'_>, id: TypeId) -> Vec<TypeId> {
    match &graph.get(id).kind {
        Kind::Any | Kind::Never => Vec::new(),
        Kind::Union(ms) | Kind::Intersection(ms) => ms.clone(),
        Kind::Alias(t) => vec![*t],
        Kind::Value(b) => {
            let mut out = Vec::new();
            if let Some(a) = &b.array {
                out.extend(a.prefix.iter().copied());
                out.push(a.rest);
            }
            if let Some(o) = &b.object {
                out.extend(o.props.iter().map(|p| p.ty));
                out.extend(o.patterns.iter().map(|p| p.ty));
                out.push(o.rest);
                out.push(o.extra);
            }
            out
        }
    }
}

/// Whether printing a type twice would print a shape twice.
fn is_shape(graph: &Graph<'_>, id: TypeId) -> bool {
    match &graph.get(id).kind {
        Kind::Union(_) | Kind::Intersection(_) => true,
        Kind::Value(b) => {
            b.object.as_ref().is_some_and(|o| !o.props.is_empty())
                || b.array.as_ref().is_some_and(|a| !a.prefix.is_empty())
        }
        Kind::Any | Kind::Never | Kind::Alias(_) => false,
    }
}

pub(super) fn declared(graph: &Graph<'_>) -> Vec<Wanted> {
    let len = graph.len();
    let mut named = vec![false; len];
    let mut out: Vec<Wanted> = Vec::new();
    for root in graph.roots() {
        if named.get(root.ty.index()) == Some(&false) {
            mark(&mut named, root.ty);
            out.push(Wanted {
                ty: root.ty,
                name: Some(root.name.clone()),
            });
        }
    }
    let through = |named: &[bool], mut t: TypeId| {
        for _ in 0..=len {
            match &graph.get(t).kind {
                Kind::Alias(next) if !named.get(t.index()).copied().unwrap_or(false) => t = *next,
                Kind::Alias(_)
                | Kind::Any
                | Kind::Never
                | Kind::Value(_)
                | Kind::Union(_)
                | Kind::Intersection(_) => break,
            }
        }
        t
    };
    let mut uses = vec![0u32; len];
    let mut seen = vec![false; len];
    let mut stack: Vec<TypeId> = out.iter().map(|w| w.ty).collect();
    while let Some(t) = stack.pop() {
        if std::mem::replace(seen.get_mut(t.index()).unwrap_or(&mut true), true) {
            continue;
        }
        for c in children(graph, t) {
            let c = through(&named, c);
            if let Some(u) = uses.get_mut(c.index()) {
                *u = u.saturating_add(1);
            }
            stack.push(c);
        }
    }
    // One set of path marks for every walk: each walk clears its own marks as it unwinds.
    let mut on_path = vec![false; len];
    let mut i = 0;
    while let Some(decl) = out.get(i).map(|w| w.ty) {
        walk(
            graph,
            decl,
            &mut named,
            &mut on_path,
            &uses,
            &mut out,
            &through,
        );
        i += 1;
    }
    out
}

/// Walks a declaration's body as it will be printed inline, declaring what must not be: a
/// type met again on its own path, a shared shape, a shape past `MAX_INLINE` levels.
fn walk(
    graph: &Graph<'_>,
    decl: TypeId,
    named: &mut [bool],
    on_path: &mut [bool],
    uses: &[u32],
    out: &mut Vec<Wanted>,
    through: &dyn Fn(&[bool], TypeId) -> TypeId,
) {
    let mut stack: Vec<(TypeId, Vec<TypeId>, usize)> = vec![(decl, children(graph, decl), 0)];
    mark(on_path, decl);
    while let Some((t, kids, next)) = stack.last_mut() {
        let Some(&c) = kids.get(*next) else {
            if let Some(slot) = on_path.get_mut(t.index()) {
                *slot = false;
            }
            stack.pop();
            continue;
        };
        *next += 1;
        let c = through(named, c);
        if named.get(c.index()).copied().unwrap_or(true) {
            continue;
        }
        let again = on_path.get(c.index()).copied().unwrap_or(false);
        let shared = uses.get(c.index()).copied().unwrap_or(0) > 1 && is_shape(graph, c);
        if again || shared || stack.len() > MAX_INLINE {
            mark(named, c);
            out.push(Wanted { ty: c, name: None });
            continue;
        }
        mark(on_path, c);
        stack.push((c, children(graph, c), 0));
    }
}

fn mark(flags: &mut [bool], id: TypeId) {
    if let Some(slot) = flags.get_mut(id.index()) {
        *slot = true;
    }
}
