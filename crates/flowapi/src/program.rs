//! What a target is given and what it gives back. A target sees a [`Program`] only: the lowered
//! graph with every name already chosen, read-only. It cannot name a type itself, nor read a
//! document (G9).

use crate::diag::Diagnostics;
use crate::graph::{Endpoint, Graph, Kind, Type};
use crate::id::TypeId;
use crate::lower::Capabilities;

/// The rules a target's identifiers follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentifierRules {
    /// Words no type may be named, compared exactly.
    pub reserved: &'static [&'static str],
}

/// A type the output declares under a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decl {
    /// The identifier.
    pub name: String,
    /// The type.
    pub ty: TypeId,
    /// Whether the description names it (a component, a document's root), or the names pass
    /// gave it a name to state a recursion or a type used in several places.
    pub named_by_description: bool,
}

/// The lowered graph with its names.
#[derive(Clone, Debug, PartialEq)]
pub struct Program<'t> {
    pub(crate) graph: Graph<'t>,
    pub(crate) decls: Vec<Decl>,
    pub(crate) names: Vec<Option<usize>>,
    pub(crate) capabilities: Capabilities,
}

impl<'t> Program<'t> {
    /// Every declaration, in the output's order: the description's, then the others as reached.
    #[must_use]
    pub fn decls(&self) -> &[Decl] {
        &self.decls
    }

    /// A type.
    #[must_use]
    pub fn get(&self, id: TypeId) -> &Type<'t> {
        self.graph.get(id)
    }

    /// The name a type is declared under, if it is.
    #[must_use]
    pub fn name_of(&self, id: TypeId) -> Option<&str> {
        let decl = self.names.get(id.index()).copied().flatten()?;
        self.decls.get(decl).map(|d| d.name.as_str())
    }

    /// What a reference to a type prints as: the type itself when it is named or a shape, the
    /// type an unnamed alias stands for otherwise. The walk is bounded by the graph's size
    /// (cycles of aliases were broken by the lowering).
    #[must_use]
    pub fn resolve(&self, id: TypeId) -> TypeId {
        let mut at = id;
        for _ in 0..=self.graph.len() {
            if self.name_of(at).is_some() {
                return at;
            }
            match &self.get(at).kind {
                Kind::Alias(next) => at = *next,
                Kind::Any
                | Kind::Never
                | Kind::Value(_)
                | Kind::Union(_)
                | Kind::Intersection(_) => return at,
            }
        }
        at
    }

    /// Whether a reference to a type allows every value.
    #[must_use]
    pub fn is_any(&self, id: TypeId) -> bool {
        matches!(self.get(self.resolve(id)).kind, Kind::Any)
    }

    /// Whether a reference to a type allows no value.
    #[must_use]
    pub fn is_never(&self, id: TypeId) -> bool {
        matches!(self.get(self.resolve(id)).kind, Kind::Never)
    }

    /// The operations, each with its type (a declaration), in the description's order.
    #[must_use]
    pub fn endpoints(&self) -> &[Endpoint] {
        self.graph.endpoints()
    }

    /// The capabilities the graph was lowered for.
    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }
}

/// A file the output holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    /// Its path, relative to where the host writes the output.
    pub path: String,
    /// Its text.
    pub text: String,
}

/// What a target gives back.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Output {
    /// The files, in a fixed order.
    pub files: Vec<File>,
    /// What the target found.
    pub diagnostics: Diagnostics,
}

/// A target language: what its types can state, how its identifiers are made, how a program is
/// printed. The core knows no target (G9); a target is a crate of its own.
pub trait Target {
    /// What its types can state.
    fn capabilities(&self) -> Capabilities;
    /// The rules its identifiers follow.
    fn identifiers(&self) -> IdentifierRules;
    /// The program printed.
    fn emit(&self, program: &Program<'_>) -> Output;
}
