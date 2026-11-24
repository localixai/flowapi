//! Every edge to a type that allows every value, or none, made an edge to the graph's own `Any`
//! (the first type) or `Never` (the second), so the passes after it compare ids.

use crate::graph::{Graph, Kind};
use crate::id::TypeId;

pub(super) fn edges(graph: &mut Graph<'_>) {
    let any = TypeId::FIRST;
    let never = TypeId::new(1).unwrap_or(TypeId::FIRST);
    let target: Vec<Option<TypeId>> = graph
        .types
        .iter()
        .map(|t| match t.kind {
            Kind::Any => Some(any),
            Kind::Never => Some(never),
            Kind::Value(_) | Kind::Union(_) | Kind::Intersection(_) | Kind::Alias(_) => None,
        })
        .collect();
    let map = |t: &mut TypeId| {
        if let Some(Some(c)) = target.get(t.index()) {
            *t = *c;
        }
    };
    for ty in &mut graph.types {
        match &mut ty.kind {
            Kind::Any | Kind::Never => {}
            Kind::Union(ms) | Kind::Intersection(ms) => ms.iter_mut().for_each(map),
            Kind::Alias(t) => map(t),
            Kind::Value(b) => {
                if let Some(a) = &mut b.array {
                    a.prefix.iter_mut().for_each(map);
                    map(&mut a.rest);
                }
                if let Some(o) = &mut b.object {
                    o.props.iter_mut().for_each(|p| map(&mut p.ty));
                    o.patterns.iter_mut().for_each(|p| map(&mut p.ty));
                    map(&mut o.rest);
                    map(&mut o.extra);
                }
            }
        }
    }
}
