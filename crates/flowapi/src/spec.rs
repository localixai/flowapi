//! What a description names: for an OpenAPI description, its components' schemas (Swagger
//! 2.0's `definitions`) and its operations; for a JSON Schema document, its root.

mod operations;

pub use operations::{
    Body, Field, Location, Media, Operation, Param, Payload, Response, Serialization, Style,
};

use crate::diag::Diagnostics;
use crate::id::DocId;
use crate::schema::{Index, Resource};
use crate::source::{Dialect, DocKind, Documents, Version};
use std::sync::Arc;

/// The named schemas of the first document, each with what it is read by; and every schema root
/// of every document, for the index.
#[must_use]
pub fn roots(docs: &Documents<'_>) -> (Vec<(String, Resource)>, Vec<Resource>, Diagnostics) {
    let mut named = Vec::new();
    let mut all = Vec::new();
    let diags = Diagnostics::default();
    for (doc, d) in docs.iter() {
        let tree = &d.tree;
        let root = tree.root();
        match d.kind {
            DocKind::Schema(dialect) => {
                let r = Resource {
                    doc,
                    node: root,
                    base: Arc::from(d.uri.as_str()),
                    dialect,
                };
                if doc == DocId::FIRST {
                    named.push((String::from("Root"), r.clone()));
                }
                all.push(r);
            }
            DocKind::OpenApi(version) => {
                let base: Arc<str> =
                    Arc::from(self_uri(docs, doc).unwrap_or_else(|| d.uri.clone()));
                let dialect = default_dialect(docs, doc, version);
                // Swagger 2.0 names its schemas under `definitions`.
                let schemas = if version == Version::V20 {
                    tree.get(root, "definitions")
                } else {
                    tree.get(root, "components")
                        .and_then(|c| tree.get(c, "schemas"))
                };
                for m in schemas.map_or(&[][..], |s| tree.members(s)) {
                    let r = Resource {
                        doc,
                        node: m.value,
                        base: Arc::clone(&base),
                        dialect,
                    };
                    if doc == DocId::FIRST {
                        let name = tree
                            .string(m.key)
                            .map_or_else(|| tree.raw(m.key).to_owned(), |k| k.into_owned());
                        named.push((name, r.clone()));
                    }
                    all.push(r);
                }
            }
        }
    }
    (named, all, diags)
}

/// What a description names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Description {
    /// The named schemas: each component's, or a JSON Schema document's root as `Root`.
    pub named: Vec<(String, Resource)>,
    /// Every schema root of every document, for the index.
    pub all: Vec<Resource>,
    /// The first document's operations.
    pub operations: Vec<Operation>,
}

/// Reads what the documents name.
#[must_use]
pub fn read(docs: &Documents<'_>) -> (Description, Diagnostics) {
    let (named, mut all, mut diags) = roots(docs);
    let mut operations = Vec::new();
    if let Some(d) = docs.get(DocId::FIRST)
        && let DocKind::OpenApi(version) = d.kind
    {
        let base = self_uri(docs, DocId::FIRST).unwrap_or_else(|| d.uri.clone());
        let dialect = default_dialect(docs, DocId::FIRST, version);
        operations = operations::read(docs, &base, dialect, version, &mut diags);
        for op in &operations {
            let schemas = op.params.iter().filter_map(|p| p.schema.as_ref());
            let bodies = op
                .body
                .iter()
                .flat_map(|b| b.content.iter())
                .flat_map(Media::schemas);
            let responses = op
                .responses
                .iter()
                .flat_map(|r| r.content.iter())
                .flat_map(Media::schemas);
            let headers = op
                .responses
                .iter()
                .flat_map(|r| r.headers.iter())
                .filter_map(|h| h.schema.as_ref());
            all.extend(
                schemas
                    .chain(bodies)
                    .chain(responses)
                    .chain(headers)
                    .cloned(),
            );
        }
    }
    (
        Description {
            named,
            all,
            operations,
        },
        diags,
    )
}

/// Every schema resource and anchor of the documents.
#[must_use]
pub fn index(docs: &Documents<'_>, all: &[Resource]) -> Index {
    let mut index = Index::build(docs, all);
    for (doc, d) in docs.iter() {
        if let (DocKind::OpenApi(version), Some(own)) = (d.kind, self_uri(docs, doc)) {
            let dialect = default_dialect(docs, doc, version);
            let root = Resource {
                doc,
                node: d.tree.root(),
                base: Arc::from(own.as_str()),
                dialect,
            };
            index.alias(&own, root);
        }
    }
    index
}

/// OpenAPI 3.2's `$self`, resolved against the document's own URI.
fn self_uri(docs: &Documents<'_>, doc: DocId) -> Option<String> {
    let d = docs.get(doc)?;
    let own = d
        .tree
        .get(d.tree.root(), "$self")
        .and_then(|n| d.tree.as_str(n))?;
    Some(crate::uri::resolve(&d.uri, &own))
}

/// The dialect a description's Schema Objects are read by: 2.0's or 3.0's own, or
/// `jsonSchemaDialect`, or JSON Schema 2020-12 with the OpenAPI vocabulary.
fn default_dialect(docs: &Documents<'_>, doc: DocId, version: Version) -> Dialect {
    match version {
        Version::V20 => return Dialect::Swagger20,
        Version::V30 => return Dialect::Oas30,
        Version::V31 | Version::V32 => {}
    }
    docs.get(doc)
        .and_then(|d| {
            d.tree
                .get(d.tree.root(), "jsonSchemaDialect")
                .and_then(|n| d.tree.as_str(n))
        })
        .and_then(|uri| Dialect::from_uri(&uri))
        .unwrap_or(Dialect::Draft2020)
}
