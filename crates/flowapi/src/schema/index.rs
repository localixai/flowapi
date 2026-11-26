//! Where every schema resource and anchor is, so a `$ref` resolves to a node and the base URI and
//! dialect that node is read with. Built once from the roots the spec layer names; the maps are
//! only ever looked up, never walked into output (G4).

use super::sub::subschemas;
use crate::id::{DocId, NodeId};
use crate::json::Tree;
use crate::source::{Dialect, Documents, resolve_pointer};
use crate::uri;
use std::collections::HashMap;
use std::sync::Arc;

/// A schema with what it is read by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    /// The document.
    pub doc: DocId,
    /// The schema's node.
    pub node: NodeId,
    /// The base URI the schema is read in: its parent's, before the schema's own `$id`. Shared
    /// by every schema under it.
    pub base: Arc<str>,
    /// Its dialect.
    pub dialect: Dialect,
}

impl Resource {
    /// The schema at `node` of the same document, read in the same base and dialect.
    pub(crate) fn at(&self, node: NodeId) -> Self {
        Self {
            node,
            base: Arc::clone(&self.base),
            ..*self
        }
    }
}

/// What a reference resolved to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A schema.
    Found(Resource),
    /// Nothing among the documents given: the absolute URI that was looked for.
    Missing(String),
}

/// Every resource and anchor reachable from the roots.
#[derive(Debug, Default)]
pub struct Index {
    resources: HashMap<String, Resource>,
    anchors: HashMap<String, Resource>,
    seen: HashMap<(DocId, NodeId), (Arc<str>, Dialect)>,
    /// The members of every object of at least [`BIG`] members, by key: a pointer through
    /// `components/schemas` of thousands is one lookup, not a scan.
    big: HashMap<(DocId, NodeId), HashMap<String, NodeId>>,
    duplicates: Vec<(String, DocId, NodeId)>,
}

/// From how many members an object's keys are put in a map.
const BIG: usize = 32;

impl Index {
    /// Walks every schema under the roots, recording each `$id` and `$anchor` (and
    /// `$dynamicAnchor`, which is also a plain anchor).
    #[must_use]
    pub fn build(docs: &Documents<'_>, roots: &[Resource]) -> Self {
        let mut index = Self::default();
        index.index_big_objects(docs);
        let mut stack: Vec<Resource> = roots.iter().rev().cloned().collect();
        while let Some(at) = stack.pop() {
            if index.seen.contains_key(&(at.doc, at.node)) {
                continue;
            }
            let Some(d) = docs.get(at.doc) else { continue };
            let tree = &d.tree;
            let scoped = scoped(tree, &at);
            let (base, dialect) = (Arc::clone(&scoped.base), scoped.dialect);
            // What is recorded is the scope the schema is read in, before its own `$id`: the
            // graph applies that `$id` once, when it reads the schema.
            if base != at.base {
                match index.resources.get(&*base) {
                    Some(first) if (first.doc, first.node) != (at.doc, at.node) => {
                        index.duplicates.push((base.to_string(), at.doc, at.node));
                    }
                    Some(_) => {}
                    None => {
                        index.resources.insert(base.to_string(), at.clone());
                    }
                }
            }
            for key in ["$anchor", "$dynamicAnchor"] {
                if let Some(name) = tree.get(at.node, key).and_then(|n| tree.as_str(n)) {
                    index
                        .anchors
                        .entry(format!("{base}#{name}"))
                        .or_insert_with(|| at.clone());
                }
            }
            index
                .seen
                .insert((at.doc, at.node), (Arc::clone(&at.base), at.dialect));
            for (_, node) in subschemas(tree, at.node, dialect).into_iter().rev() {
                stack.push(scoped.at(node));
            }
        }
        index
    }

    fn index_big_objects(&mut self, docs: &Documents<'_>) {
        for (doc, d) in docs.iter() {
            let tree = &d.tree;
            for node in (0..tree.len()).filter_map(NodeId::new) {
                let members = tree.members(node);
                if members.len() < BIG {
                    continue;
                }
                let mut keys = HashMap::with_capacity(members.len());
                for m in members {
                    if let Some(k) = tree.string(m.key) {
                        keys.insert(k.into_owned(), m.value);
                    }
                }
                self.big.insert((doc, node), keys);
            }
        }
    }

    /// A JSON Pointer resolved, looking keys of big objects up in their maps.
    fn pointer(&self, tree: &Tree<'_>, doc: DocId, from: NodeId, fragment: &str) -> Option<NodeId> {
        if fragment.contains('%') || fragment.contains('~') {
            return resolve_pointer(tree, from, fragment);
        }
        let mut node = from;
        for token in fragment.strip_prefix('/')?.split('/') {
            node = match self.big.get(&(doc, node)) {
                Some(keys) => *keys.get(token)?,
                None => resolve_pointer(tree, node, &format!("/{token}"))?,
            };
        }
        Some(node)
    }

    /// Names a resource by one more URI: an OpenAPI 3.2 document by its `$self`, which a
    /// reference to it must use.
    pub fn alias(&mut self, uri: &str, resource: Resource) {
        self.resources.entry(uri.to_owned()).or_insert(resource);
    }

    /// Every URI two schemas both name with `$id`: the first keeps it (2020-12 §9.1.2 says a
    /// schema SHOULD NOT do this), each later one is reported.
    #[must_use]
    pub fn duplicates(&self) -> &[(String, DocId, NodeId)] {
        &self.duplicates
    }

    /// The base URI and dialect a node is read in (before its own `$id`), as found from the roots; `None` for a node no
    /// root reaches.
    #[must_use]
    pub fn scope(&self, doc: DocId, node: NodeId) -> Option<(&str, Dialect)> {
        self.seen.get(&(doc, node)).map(|(b, d)| (&**b, *d))
    }

    /// Resolves `reference` written in a schema read with base `base`.
    #[must_use]
    pub fn resolve(
        &self,
        docs: &Documents<'_>,
        base: &str,
        reference: &str,
        dialect: Dialect,
    ) -> Target {
        let absolute = uri::resolve(base, reference);
        let (resource, fragment) = uri::split_fragment(&absolute);
        let root = self.resources.get(resource).cloned().or_else(|| {
            let doc = docs.find(resource)?;
            let d = docs.get(doc)?;
            let node = d.tree.root();
            let (base, dialect) = self.seen.get(&(doc, node)).map_or_else(
                || (Arc::from(resource), dialect),
                |(b, d)| (Arc::clone(b), *d),
            );
            Some(Resource {
                doc,
                node,
                base,
                dialect,
            })
        });
        let Some(root) = root else {
            return Target::Missing(absolute);
        };
        if fragment.is_empty() {
            return Target::Found(root);
        }
        if !fragment.starts_with('/') {
            return self
                .anchors
                .get(&absolute)
                .cloned()
                .map_or(Target::Missing(absolute), Target::Found);
        }
        let Some(d) = docs.get(root.doc) else {
            return Target::Missing(absolute);
        };
        match self.pointer(&d.tree, root.doc, root.node, fragment) {
            Some(node) => {
                let (base, dialect) = self.seen.get(&(root.doc, node)).map_or_else(
                    || (Arc::clone(&root.base), root.dialect),
                    |(b, d)| (Arc::clone(b), *d),
                );
                Target::Found(Resource {
                    doc: root.doc,
                    node,
                    base,
                    dialect,
                })
            }
            None => Target::Missing(absolute),
        }
    }
}

/// A schema with the base and dialect its own `$schema` and `$id` set. A draft 4 `id` that is
/// only a fragment names an anchor, not a resource.
#[must_use]
pub fn scoped(tree: &Tree<'_>, at: &Resource) -> Resource {
    let dialect = tree
        .get(at.node, "$schema")
        .and_then(|n| tree.as_str(n))
        .and_then(|s| Dialect::from_uri(&s))
        .unwrap_or(at.dialect);
    let id = tree
        .get(at.node, dialect.id_keyword())
        .and_then(|n| tree.as_str(n));
    let base = match id {
        Some(id) if !(dialect == Dialect::Draft4 && id.starts_with('#')) => {
            Arc::from(uri::split_fragment(&uri::resolve(&at.base, &id)).0)
        }
        Some(_) | None => Arc::clone(&at.base),
    };
    Resource {
        doc: at.doc,
        node: at.node,
        base,
        dialect,
    }
}
