//! The graph made ready for a target: unions and intersections flattened, intersections of
//! object and array shapes merged into one shape where their parts are the schema's own, kinds
//! another part rules out dropped, and cycles a type alias cannot state broken. Every rewrite
//! keeps the set of values (or widens it, never narrows it: G3).

mod canon;
mod cycles;
mod kinds;
mod merge;

use crate::graph::Graph;

/// What a target's types can state, which decides how far the graph is rewritten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    /// The target can say "both A and B" (`A & B`). Without it every intersection is merged
    /// into one shape, or widened.
    pub intersections: bool,
    /// The target can say "A or B" of any two types (`A | B`). Without it a union is a tagged
    /// choice the target builds itself.
    pub unions: bool,
}

/// A graph lowered for one target's capabilities.
#[derive(Clone, Debug, PartialEq)]
pub struct Lowered<'t> {
    graph: Graph<'t>,
    capabilities: Capabilities,
}

impl<'t> Lowered<'t> {
    /// The graph.
    #[must_use]
    pub fn graph(&self) -> &Graph<'t> {
        &self.graph
    }

    /// The capabilities it was lowered for.
    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// The graph, taken.
    #[must_use]
    pub fn into_graph(self) -> Graph<'t> {
        self.graph
    }
}

/// Lowers a graph for a target.
#[must_use]
pub fn lower(graph: Graph<'_>, capabilities: Capabilities) -> Lowered<'_> {
    let mut graph = graph;
    canon::edges(&mut graph);
    let mut kinds = kinds::of(&graph);
    merge::simplify(&mut graph, &mut kinds);
    cycles::break_direct(&mut graph);
    Lowered {
        graph,
        capabilities,
    }
}
