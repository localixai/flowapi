//! Schemas into types. Every schema reached gets its type id before its body is read, and the
//! body is read from a queue, not by recursion: a reference is an id, a cycle costs nothing, and
//! no chain of references can exhaust the stack (G1).

use super::own::Own;
use super::vocab::Meaning;
use super::{Branches, Graph, Kind, Kinds, Origin, Root, Type};
use crate::diag::Diagnostics;
use crate::id::{DocId, NodeId, TypeId};
use crate::json::Value;
use crate::schema::{Index, Keyword, Resource, Target, scoped};
use crate::source::{Dialect, Documents};
use crate::spec::Operation;
use std::collections::HashMap;

/// The graph of the schemas the roots name; each root keeps its name.
#[must_use]
pub fn build<'t>(
    docs: &Documents<'t>,
    index: &Index,
    roots: &[(String, Resource)],
) -> (Graph<'t>, Diagnostics) {
    build_with(docs, index, roots, &[])
}

/// The graph of the schemas the roots name and of the operations given, each a root of its own
/// after the schemas, then `Paths`.
#[must_use]
pub fn build_with<'t>(
    docs: &Documents<'t>,
    index: &Index,
    roots: &[(String, Resource)],
    operations: &[Operation],
) -> (Graph<'t>, Diagnostics) {
    let mut b = Builder::new(docs, index);
    let mut named: Vec<Root> = roots
        .iter()
        .map(|(name, r)| Root {
            name: name.clone(),
            ty: b.type_of(r),
        })
        .collect();
    named.extend(b.operations(operations));
    while let Some((id, at)) = b.queue.pop() {
        let ty = b.schema(&at);
        if let Some(slot) = b.graph.types.get_mut(id.index()) {
            *slot = ty;
        }
    }
    b.graph.roots = named;
    (b.graph, b.diags)
}

pub(super) struct Builder<'d, 't> {
    pub(super) docs: &'d Documents<'t>,
    pub(super) index: &'d Index,
    pub(super) graph: Graph<'t>,
    memo: HashMap<(DocId, NodeId), TypeId>,
    /// Each absolute reference resolved once.
    refs: HashMap<String, TypeId>,
    queue: Vec<(TypeId, Resource)>,
    pub(super) diags: Diagnostics,
    pub(super) any: TypeId,
}

/// What a Schema Object's keywords say, gathered in one pass over its members.
#[derive(Default)]
pub(super) struct Read {
    pub(super) reference: Option<TypeId>,
    pub(super) parts: Vec<TypeId>,
    pub(super) then_else: (Option<NodeId>, Option<NodeId>),
    /// The schema's `if`: without one, `then` and `else` do nothing (2020-12 §10.2.2).
    pub(super) test: Option<NodeId>,
    pub(super) lossy: Vec<Keyword>,
    /// The kinds a `not` rules out.
    pub(super) exclude: Kinds,
}

impl<'d, 't> Builder<'d, 't> {
    fn new(docs: &'d Documents<'t>, index: &'d Index) -> Self {
        let mut graph = Graph::default();
        let any = graph.push(Type::of(Kind::Any));
        // The second type is always `Never`: the lowering relies on both places.
        graph.push(Type::of(Kind::Never));
        Self {
            docs,
            index,
            graph,
            memo: HashMap::new(),
            refs: HashMap::new(),
            queue: Vec::new(),
            diags: Diagnostics::default(),
            any,
        }
    }

    /// The type of a schema: its id at once, its body read later from the queue.
    pub(super) fn type_of(&mut self, at: &Resource) -> TypeId {
        if let Some(&id) = self.memo.get(&(at.doc, at.node)) {
            return id;
        }
        let id = self.graph.push(Type::of(Kind::Any));
        self.memo.insert((at.doc, at.node), id);
        self.queue.push((id, at.clone()));
        id
    }

    /// The type of a schema under `parent`, read with the same base and dialect.
    pub(super) fn sub(&mut self, parent: &Resource, node: NodeId) -> TypeId {
        self.type_of(&Resource {
            doc: parent.doc,
            node,
            base: std::sync::Arc::clone(&parent.base),
            dialect: parent.dialect,
        })
    }

    /// A type the graph makes, with no schema of its own.
    pub(super) fn make(&mut self, kind: Kind<'t>) -> TypeId {
        self.graph.push(Type::of(kind))
    }

    fn schema(&mut self, at: &Resource) -> Type<'t> {
        let docs = self.docs;
        let Some(doc) = docs.get(at.doc) else {
            return Type::of(Kind::Any);
        };
        let tree = &doc.tree;
        let loc = docs.loc(at.doc, at.node);
        let mut ty = Type::of(Kind::Any);
        ty.origin = Some(Origin {
            doc: at.doc,
            node: at.node,
        });
        match tree.value(at.node) {
            Value::Bool(true) => return ty,
            Value::Bool(false) => {
                ty.kind = Kind::Never;
                return ty;
            }
            Value::Object(_) => {}
            Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => {
                self.diags.error(
                    "schema.shape",
                    loc,
                    "a schema must be an object or a boolean",
                );
                return ty;
            }
        }
        let at = self.scoped(at);
        let mut at = at;
        match self.vocabularies(&at) {
            Meaning::All => {}
            Meaning::NoValidation => at.dialect = Dialect::Draft2020Annotations,
            Meaning::Nothing => {
                // Its keywords are annotations here: any value passes.
                ty.lossy.push(Keyword::Vocabulary);
                return ty;
            }
        }
        let str_of = |key: &str| tree.get(at.node, key).and_then(|n| tree.as_str(n));
        let bool_of =
            |key: &str| tree.get(at.node, key).and_then(|n| tree.as_bool(n)) == Some(true);
        ty.title = str_of("title");
        ty.description = str_of("description");
        ty.deprecated = bool_of("deprecated");
        ty.read_only = bool_of("readOnly");
        ty.write_only = bool_of("writeOnly");
        let (mut read, own) = self.read(&at);
        let mut lossy = std::mem::take(&mut read.lossy);
        ty.kind = self.combine(&at, read, &own, &mut lossy);
        ty.lossy = lossy;
        ty
    }

    fn scoped(&self, at: &Resource) -> Resource {
        self.docs
            .get(at.doc)
            .map_or_else(|| at.clone(), |d| scoped(&d.tree, at))
    }

    pub(super) fn read(&mut self, at: &Resource) -> (Read, Own) {
        let docs = self.docs;
        let tree = docs.get(at.doc).map(|d| &d.tree);
        let mut read = Read::default();
        let mut own = Own::default();
        let members = tree.map_or(&[][..], |t| t.members(at.node));
        for member in members {
            let Some(key) = tree.and_then(|t| t.string(member.key)) else {
                continue;
            };
            let keyword = Keyword::of(&key, at.dialect);
            self.keyword(at, keyword, member.value, &mut read, &mut own);
        }
        (read, own)
    }

    pub(super) fn reference(&mut self, at: &Resource, value: NodeId) -> TypeId {
        let docs = self.docs;
        let loc = docs.loc(at.doc, value);
        let Some(reference) = docs.get(at.doc).and_then(|d| d.tree.as_str(value)) else {
            self.diags
                .error("schema.ref", loc, "`$ref` must be a string");
            return self.any;
        };
        let key = format!("{}\u{0}{reference}", at.base);
        if let Some(&t) = self.refs.get(&key) {
            return t;
        }
        match self.index.resolve(docs, &at.base, &reference, at.dialect) {
            Target::Found(target) => {
                let t = self.type_of(&target);
                self.refs.insert(key, t);
                t
            }
            Target::Missing(uri) => {
                self.diags.error(
                    "schema.ref",
                    loc,
                    format!("`$ref` names nothing among the documents given: {uri}"),
                );
                self.any
            }
        }
    }

    fn combine(
        &mut self,
        at: &Resource,
        read: Read,
        own: &Own,
        lossy: &mut Vec<Keyword>,
    ) -> Kind<'t> {
        if let (Some(target), true) = (read.reference, at.dialect.ref_overrides()) {
            return Kind::Alias(target);
        }
        let mut parts: Vec<TypeId> = read.reference.into_iter().collect();
        parts.extend(read.parts);
        if let Some(test) = read.test {
            let (then, otherwise) = read.then_else;
            parts.extend(self.conditional(at, test, then, otherwise, lossy));
        }
        let mut literal = Vec::new();
        let mut value = self.own(at, own, &mut literal, lossy);
        parts.extend(literal);
        if !read.exclude.is_empty() {
            let keep = Kinds::ALL.without(read.exclude);
            let b = value.get_or_insert_with(|| Branches::of(Kinds::ALL, self.any));
            b.keep(keep);
        }
        match (value, parts.len()) {
            (None, 0) => Kind::Any,
            (None, 1) => parts.first().map_or(Kind::Any, |&t| Kind::Alias(t)),
            (None, _) => Kind::Intersection(parts),
            (Some(v), 0) => Kind::Value(v),
            (Some(v), _) => {
                let mut own_type = Type::of(Kind::Value(v));
                own_type.origin = Some(Origin {
                    doc: at.doc,
                    node: at.node,
                });
                parts.insert(0, self.graph.push(own_type));
                Kind::Intersection(parts)
            }
        }
    }
}
