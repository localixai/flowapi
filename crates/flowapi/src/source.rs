//! The input: documents read as JSON, each with its URI and what it is (an OpenAPI description
//! of a given version, or a JSON Schema of a given draft). The host reads the bytes; nothing
//! here touches a file or the network (G5).

mod dialect;
mod pointer;

pub use dialect::{Dialect, Version};
pub use pointer::{node_pointer, resolve_pointer};

use crate::diag::{Diagnostic, Loc, Severity};
use crate::id::{DocId, NodeId};
use crate::json::{self, Tree};

/// What a document is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    /// An OpenAPI description, by the version its `openapi` (or `swagger`) field names.
    OpenApi(Version),
    /// A JSON Schema, of the dialect its `$schema` names (2020-12 when it names none).
    Schema(Dialect),
}

/// One document.
#[derive(Debug)]
pub struct Document<'t> {
    /// Its URI: a retrieval URI, a path, or any name the host gives it.
    pub uri: String,
    /// Its JSON.
    pub tree: Tree<'t>,
    /// What it is.
    pub kind: DocKind,
}

/// The documents of one run. The first is the one generated from; the others are what its
/// `$ref`s reach.
#[derive(Debug, Default)]
pub struct Documents<'t> {
    docs: Vec<Document<'t>>,
}

impl<'t> Documents<'t> {
    /// An empty set.
    #[must_use]
    pub fn new() -> Self {
        Self { docs: Vec::new() }
    }

    /// Reads a document and adds it under `uri`.
    ///
    /// # Errors
    ///
    /// When the text is not JSON, or names an OpenAPI version this build does not read.
    pub fn add(&mut self, uri: impl Into<String>, text: &'t str) -> Result<DocId, Diagnostic> {
        let doc = DocId::new(self.docs.len()).unwrap_or(DocId::FIRST);
        let tree = json::parse(text).map_err(|e| Diagnostic {
            severity: Severity::Error,
            code: "json.syntax",
            message: format!("not JSON: expected {}", e.message),
            at: Loc {
                doc,
                offset: e.offset,
            },
        })?;
        let kind = dialect::detect(&tree).map_err(|message| Diagnostic {
            severity: Severity::Error,
            code: "oas.version",
            message,
            at: Loc { doc, offset: 0 },
        })?;
        self.docs.push(Document {
            uri: uri.into(),
            tree,
            kind,
        });
        Ok(doc)
    }

    /// A document.
    #[must_use]
    pub fn get(&self, doc: DocId) -> Option<&Document<'t>> {
        self.docs.get(doc.index())
    }

    /// Every document with its id, in the order added.
    pub fn iter(&self) -> impl Iterator<Item = (DocId, &Document<'t>)> {
        self.docs
            .iter()
            .enumerate()
            .filter_map(|(i, d)| Some((DocId::new(i)?, d)))
    }

    /// The document added under `uri`, compared without its fragment.
    #[must_use]
    pub fn find(&self, uri: &str) -> Option<DocId> {
        let (resource, _) = crate::uri::split_fragment(uri);
        self.iter()
            .find(|(_, d)| crate::uri::split_fragment(&d.uri).0 == resource)
            .map(|(id, _)| id)
    }

    /// Where a node is, for a diagnostic.
    #[must_use]
    pub fn loc(&self, doc: DocId, node: NodeId) -> Loc {
        Loc {
            doc,
            offset: self.get(doc).map_or(0, |d| d.tree.offset(node)),
        }
    }

    /// A place as `uri:line:column` and its JSON Pointer, for a person to read.
    #[must_use]
    pub fn describe(&self, at: Loc) -> String {
        let Some(d) = self.get(at.doc) else {
            return String::from("?");
        };
        let before = d.tree.text().get(..at.offset as usize).unwrap_or("");
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        format!("{}:{line}:{column}", d.uri)
    }
}
