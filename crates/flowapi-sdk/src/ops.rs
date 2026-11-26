//! The description's operations as a config names them: each by its `operationId`, or by its
//! route `"METHOD /path"`, with what an agent and a doc comment read of it (tags, summary,
//! description, `deprecated`, `externalDocs`) and the core's reading
//! of its parameters, body and responses (`flowapi::spec::read`; no type graph is built, so a
//! description of thousands of operations is read in the time its JSON takes).

use crate::doc::{Doc, ROOT};
use flowapi::spec::Operation;
use flowapi::{Diagnostics, Documents};
use std::collections::BTreeMap;

/// One operation.
pub(crate) struct Op {
    /// The core's reading: path, method, parameters, body, responses.
    pub(crate) operation: Operation,
    /// Its `operationId`, as written.
    pub(crate) operation_id: Option<String>,
    /// `"METHOD /path"`, the method in upper case.
    pub(crate) route: String,
    /// Its tags, in the order written.
    pub(crate) tags: Vec<String>,
    /// Its `summary` and `description`, as written (CommonMark).
    pub(crate) summary: Option<String>,
    pub(crate) description: Option<String>,
    /// Whether it is `deprecated: true`.
    pub(crate) deprecated: bool,
    /// Its `externalDocs`: the URL, and what it says of it.
    pub(crate) external_docs: Option<(String, Option<String>)>,
}

impl Op {
    /// The method in upper case.
    pub(crate) fn method(&self) -> String {
        self.operation.method.to_ascii_uppercase()
    }
}

/// The operations of a description, with the lookups a config's references need.
pub(crate) struct Ops<'d, 't> {
    pub(crate) doc: Doc<'d, 't>,
    /// In the order written.
    pub(crate) list: Vec<Op>,
    /// The names of the description's schemas (its components, 2.0's definitions).
    pub(crate) schema_names: Vec<String>,
    by_id: BTreeMap<String, Vec<usize>>,
    by_route: BTreeMap<String, usize>,
}

impl<'d, 't> Ops<'d, 't> {
    /// The operations of the first document, when it is an OpenAPI or Swagger description.
    pub(crate) fn read(docs: &'d Documents<'t>) -> (Option<Self>, Diagnostics) {
        let (description, diags) = flowapi::spec::read(docs);
        let Some(doc) = Doc::of(docs) else {
            return (None, diags);
        };
        let mut list = Vec::new();
        for operation in description.operations {
            let items = doc.path_items(&operation);
            let raw = doc.operation(&items, &operation);
            let tags = raw
                .and_then(|o| doc.get(o, "tags"))
                .map(|t| doc.items(t))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|t| doc.tree(t.0)?.as_str(t.1).map(|s| s.into_owned()))
                .collect();
            let external = raw.and_then(|o| doc.get(o, "externalDocs"));
            list.push(Op {
                operation_id: raw.and_then(|o| doc.str(o, "operationId")),
                summary: raw.and_then(|o| doc.str(o, "summary")),
                description: raw.and_then(|o| doc.str(o, "description")),
                deprecated: raw
                    .and_then(|o| doc.get(o, "deprecated"))
                    .and_then(|(d, n)| doc.tree(d)?.as_bool(n))
                    .unwrap_or(false),
                external_docs: external.and_then(|e| {
                    let url = doc.str(e, "url")?;
                    Some((url, doc.str(e, "description")))
                }),
                route: format!(
                    "{} {}",
                    operation.method.to_ascii_uppercase(),
                    operation.path
                ),
                tags,
                operation,
            });
        }
        let mut by_id: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut by_route = BTreeMap::new();
        for (i, op) in list.iter().enumerate() {
            if let Some(id) = &op.operation_id {
                by_id.entry(id.clone()).or_default().push(i);
            }
            by_route.insert(op.route.clone(), i);
        }
        let schema_names = description.named.into_iter().map(|(n, _)| n).collect();
        let ops = Self {
            doc,
            list,
            schema_names,
            by_id,
            by_route,
        };
        (Some(ops), diags)
    }

    /// How a config names an operation: its `operationId` when no other operation has it, else
    /// its route.
    pub(crate) fn id(&self, i: usize) -> String {
        let Some(op) = self.list.get(i) else {
            return String::new();
        };
        match &op.operation_id {
            Some(id) if self.by_id.get(id).is_some_and(|all| all.len() == 1) => id.clone(),
            Some(_) | None => op.route.clone(),
        }
    }

    /// The operation a config's reference names: an `operationId` that one operation has, else a
    /// route `"METHOD /path"` (the method in any case, the path as written).
    pub(crate) fn find(&self, reference: &str) -> Lookup {
        if let Some(all) = self.by_id.get(reference) {
            return match all.as_slice() {
                [one] => Lookup::Found(*one),
                [] => Lookup::Missing,
                [_, _, ..] => Lookup::Ambiguous,
            };
        }
        let Some((method, path)) = reference.split_once(' ') else {
            return Lookup::Missing;
        };
        let route = format!("{} {}", method.to_ascii_uppercase(), path.trim());
        self.by_route
            .get(&route)
            .map_or(Lookup::Missing, |&i| Lookup::Found(i))
    }

    /// Every name a reference may use, for the hint of one that names nothing.
    pub(crate) fn references(&self) -> Vec<&str> {
        self.by_id
            .keys()
            .map(String::as_str)
            .chain(self.by_route.keys().map(String::as_str))
            .collect()
    }

    /// The description's `info` member `key` (`title`, `version`).
    pub(crate) fn info(&self, key: &str) -> Option<String> {
        self.doc
            .get(ROOT, "info")
            .and_then(|info| self.doc.str(info, key))
    }
}

/// What a reference names.
pub(crate) enum Lookup {
    /// One operation.
    Found(usize),
    /// An `operationId` more than one operation has: a route names one of them.
    Ambiguous,
    /// Nothing.
    Missing,
}
