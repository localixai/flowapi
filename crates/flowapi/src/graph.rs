//! The type graph: what each schema accepts, as a type no language knows yet. A type is the
//! values of each JSON kind it allows ([`Branches`]), or a union, an intersection or an alias of
//! other types. Every type is sound: it accepts every value its schema accepts (G3); what it
//! cannot narrow, it records in [`Type::lossy`].

mod affix;
mod arrays;
mod branches;
mod build;
mod conditional;
mod dependent;
mod exclusive;
mod keys;
mod limits;
mod literal;
mod local;
mod narrow;
mod objects;
mod ops;
mod own;
mod read;
mod unevaluated;
mod vocab;

pub use affix::Affix;
pub use branches::{Array, Bools, Branches, Kinds, Lit, Numbers, Object, Pattern, Prop, Strings};
pub use build::{build, build_with};

use crate::id::{DocId, NodeId, TypeId};
use crate::schema::Keyword;
use crate::spec::Operation;
use std::borrow::Cow;

/// A type's shape.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind<'t> {
    /// Every value.
    Any,
    /// No value.
    Never,
    /// The values of each JSON kind it allows.
    Value(Branches<'t>),
    /// The values of any member.
    Union(Vec<TypeId>),
    /// The values of every member.
    Intersection(Vec<TypeId>),
    /// The same values as another type: a `$ref`, kept so the target's name is kept.
    Alias(TypeId),
}

/// The schema a type was read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    /// Its document.
    pub doc: DocId,
    /// Its node.
    pub node: NodeId,
}

/// A type and what its schema said about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Type<'t> {
    /// Its shape.
    pub kind: Kind<'t>,
    /// The schema it was read from; `None` for a type the graph made (a merge).
    pub origin: Option<Origin>,
    /// The schema's `title`.
    pub title: Option<Cow<'t, str>>,
    /// The schema's `description`.
    pub description: Option<Cow<'t, str>>,
    /// The keywords of its schema that narrow it in a way no type here expresses: the type is
    /// wider than the schema there, never narrower.
    pub lossy: Vec<Keyword>,
    /// `deprecated: true`.
    pub deprecated: bool,
    /// `readOnly: true`.
    pub read_only: bool,
    /// `writeOnly: true`.
    pub write_only: bool,
}

impl Type<'_> {
    /// A type with only a shape.
    #[must_use]
    pub fn of(kind: Kind<'_>) -> Type<'_> {
        Type {
            kind,
            origin: None,
            title: None,
            description: None,
            lossy: Vec::new(),
            deprecated: false,
            read_only: false,
            write_only: false,
        }
    }
}

/// A type a description names: a component, or the root of a JSON Schema document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    /// The name the description gives it: a component's key, or `Root`.
    pub name: String,
    /// The type.
    pub ty: TypeId,
}

/// An operation and the type the graph made of it: what a target needs, beside the types, to
/// write the operation's requests and read its responses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// The operation's type, a root.
    pub ty: TypeId,
    /// The operation, as the description gives it.
    pub operation: Operation,
}

/// The graph of one run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph<'t> {
    pub(crate) types: Vec<Type<'t>>,
    pub(crate) roots: Vec<Root>,
    pub(crate) endpoints: Vec<Endpoint>,
}

impl<'t> Graph<'t> {
    /// A type; `Any` for an id of another graph.
    #[must_use]
    pub fn get(&self, id: TypeId) -> &Type<'t> {
        self.types.get(id.index()).unwrap_or(&ANY)
    }

    /// The named types, in the order the description gives them.
    #[must_use]
    pub fn roots(&self) -> &[Root] {
        &self.roots
    }

    /// The operations, in the order the description gives them.
    #[must_use]
    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    /// How many types the graph holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Whether the graph holds no type.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Adds a type.
    pub(crate) fn push(&mut self, ty: Type<'t>) -> TypeId {
        let id = TypeId::new(self.types.len()).unwrap_or(TypeId::FIRST);
        self.types.push(ty);
        id
    }

    /// Every type with its id.
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &Type<'t>)> {
        self.types
            .iter()
            .enumerate()
            .filter_map(|(i, t)| Some((TypeId::new(i)?, t)))
    }
}

static ANY: Type<'static> = Type {
    kind: Kind::Any,
    origin: None,
    title: None,
    description: None,
    lossy: Vec::new(),
    deprecated: false,
    read_only: false,
    write_only: false,
};
