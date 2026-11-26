//! The first document as this layer reads it, through the core's public modules: its version,
//! the URI its references resolve against, Reference Objects followed (one that names nothing is
//! a warning), strings read, and each endpoint's Operation Object found again by its path and
//! method as the core found it.

use flowapi::id::{DocId, NodeId};
use flowapi::json::Tree;
use flowapi::spec::Operation;
use flowapi::{Diagnostics, DocKind, Documents, Version};
use std::collections::BTreeMap;

/// A requirement id in the description's version (`oas-3.x.<section>`), or our own code for
/// Swagger 2.0, which the requirements ledger does not hold.
macro_rules! oas {
    ($v:expr, $swagger:literal, $rest:literal) => {
        match $v {
            flowapi::Version::V20 => $swagger,
            flowapi::Version::V30 => concat!("oas-3.0.", $rest),
            flowapi::Version::V31 => concat!("oas-3.1.", $rest),
            flowapi::Version::V32 => concat!("oas-3.2.", $rest),
        }
    };
}
pub(crate) use oas;

/// The fields a Path Item names an operation by (the core's order); 3.2's other methods are under
/// `additionalOperations`.
const FIELDS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

/// A node of one of the documents.
pub(crate) type At = (DocId, NodeId);

/// The first document's root.
pub(crate) const ROOT: At = (DocId::FIRST, NodeId::FIRST);

pub(crate) struct Doc<'d, 't> {
    pub(crate) docs: &'d Documents<'t>,
    pub(crate) base: String,
    pub(crate) version: Version,
    /// The first document's named schemas by the fragment a local reference names them with
    /// (`#/components/schemas/Pet`, 2.0's `#/definitions/Pet`): a reference into thousands of
    /// schemas found without walking them each time.
    named: BTreeMap<String, NodeId>,
}

impl<'d, 't> Doc<'d, 't> {
    /// The first document, when it is an OpenAPI or Swagger description.
    pub(crate) fn of(docs: &'d Documents<'t>) -> Option<Self> {
        let d = docs.get(DocId::FIRST)?;
        let DocKind::OpenApi(version) = d.kind else {
            return None;
        };
        let base = flowapi::spec::self_uri(docs, DocId::FIRST).unwrap_or_else(|| d.uri.clone());
        let mut named = BTreeMap::new();
        let tree = &d.tree;
        let holders = [
            (
                "#/components/schemas/",
                ["components", "schemas"].as_slice(),
            ),
            ("#/definitions/", ["definitions"].as_slice()),
        ];
        for (prefix, keys) in holders {
            let holder = keys.iter().try_fold(tree.root(), |n, k| tree.get(n, k));
            for m in holder.map(|h| tree.members(h)).unwrap_or_default() {
                if let Some(key) = tree.string(m.key) {
                    let escaped = key.replace('~', "~0").replace('/', "~1");
                    named.entry(format!("{prefix}{escaped}")).or_insert(m.value);
                }
            }
        }
        Some(Self {
            docs,
            base,
            version,
            named,
        })
    }

    /// A schema with its `$ref`s followed: a local one to a named schema of the first document
    /// by the index, any other as the core follows it. `None` for one that names nothing.
    pub(crate) fn resolve(&self, at: At) -> Option<At> {
        let mut here = at;
        for _ in 0..32 {
            let Some(r) = self.str(here, "$ref") else {
                return Some(here);
            };
            match self.named.get(&r) {
                Some(&node) if here.0 == DocId::FIRST => here = (DocId::FIRST, node),
                Some(_) | None => {
                    return flowapi::spec::follow(self.docs, &self.base, here.0, here.1);
                }
            }
        }
        None
    }

    pub(crate) fn tree(&self, doc: DocId) -> Option<&'d Tree<'t>> {
        self.docs.get(doc).map(|d| &d.tree)
    }

    /// A member of an object.
    pub(crate) fn get(&self, (doc, node): (DocId, NodeId), key: &str) -> Option<(DocId, NodeId)> {
        Some((doc, self.tree(doc)?.get(node, key)?))
    }

    /// A string member of an object.
    pub(crate) fn str(&self, at: (DocId, NodeId), key: &str) -> Option<String> {
        let (doc, node) = self.get(at, key)?;
        Some(self.tree(doc)?.as_str(node)?.into_owned())
    }

    /// An array's items.
    pub(crate) fn items(&self, (doc, node): (DocId, NodeId)) -> Vec<(DocId, NodeId)> {
        self.tree(doc)
            .map(|t| t.items(node).iter().map(|&n| (doc, n)).collect())
            .unwrap_or_default()
    }

    /// An object's members, keys read; one whose key cannot be read is skipped.
    pub(crate) fn members(&self, (doc, node): (DocId, NodeId)) -> Vec<(String, (DocId, NodeId))> {
        let Some(t) = self.tree(doc) else {
            return Vec::new();
        };
        t.members(node)
            .iter()
            .filter_map(|m| Some((t.string(m.key)?.into_owned(), (doc, m.value))))
            .collect()
    }

    /// Whether a node is an array, or an object.
    pub(crate) fn is(&self, (doc, node): (DocId, NodeId), array: bool) -> bool {
        use flowapi::json::Value;
        let value = self.tree(doc).map_or(Value::Null, |t| t.value(node));
        matches!(
            (value, array),
            (Value::Array(_), true) | (Value::Object(_), false)
        )
    }

    pub(crate) fn warn(
        &self,
        diags: &mut Diagnostics,
        code: &'static str,
        at: (DocId, NodeId),
        message: String,
    ) {
        diags.warn(code, self.docs.loc(at.0, at.1), message);
    }

    /// A Reference Object followed (the core's [`flowapi::spec::follow`]); one that names nothing
    /// is a warning, and `None`.
    pub(crate) fn follow(
        &self,
        at: (DocId, NodeId),
        what: &str,
        diags: &mut Diagnostics,
    ) -> Option<(DocId, NodeId)> {
        let found = flowapi::spec::follow(self.docs, &self.base, at.0, at.1);
        if found.is_none() {
            let r = self.str(at, "$ref").unwrap_or_default();
            let code = oas!(self.version, "swagger.ref", "reference-object.$ref");
            self.warn(
                diags,
                code,
                at,
                format!("{what} is a reference that names nothing: {r}"),
            );
        }
        found
    }

    /// The Path Item of an endpoint: its own object first, then what its `$ref` names (the core
    /// reported a reference that names nothing already).
    pub(crate) fn path_items(&self, op: &Operation) -> Vec<(DocId, NodeId)> {
        let Some(item) = self
            .get(ROOT, "paths")
            .and_then(|paths| self.get(paths, &op.path))
        else {
            return Vec::new();
        };
        let mut items = vec![item];
        if let Some(target) = flowapi::spec::follow(self.docs, &self.base, item.0, item.1)
            && target != item
        {
            items.push(target);
        }
        items
    }

    /// An endpoint's Operation Object, found as the core found it: a field of the first Path Item
    /// that has it, else an entry of `additionalOperations` (3.2).
    pub(crate) fn operation(
        &self,
        items: &[(DocId, NodeId)],
        op: &Operation,
    ) -> Option<(DocId, NodeId)> {
        let method = op.method.as_str();
        if FIELDS.contains(&method)
            && let Some(found) = items.iter().find_map(|&item| self.get(item, method))
        {
            return Some(found);
        }
        items.iter().find_map(|&item| {
            self.get(item, "additionalOperations")
                .and_then(|extra| self.get(extra, method))
        })
    }
}
