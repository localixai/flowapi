//! Reading one operation's parts, Reference Objects followed: a reference that names nothing is
//! an error that names it, never a part dropped in silence.

use super::{Body, Location, METHODS, Media, Operation, Param, Payload};
use crate::diag::Diagnostics;
use crate::id::{DocId, NodeId};
use crate::json::Tree;
use crate::schema::Resource;
use crate::source::{Dialect, Documents, Version, resolve_pointer};
use crate::uri;
use std::sync::Arc;

/// Header parameters the spec says SHALL be ignored, compared without case (in Swagger 2.0 too,
/// where `consumes`, `produces` and security say what they would).
const IGNORED_HEADERS: [&str; 3] = ["accept", "content-type", "authorization"];

/// RFC 9110's `token` (5.6.2): what a header's name is, and RFC 6265's cookie name.
fn token(s: &str) -> bool {
    let tchar = |b: &u8| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(b);
    !s.is_empty() && s.bytes().all(|b| tchar(&b))
}

/// A Reference Object followed to what it names, at most 32 times; any other node as it is.
/// `base` is the first document's URI (its `$self`, [`crate::spec::self_uri`]); `None` when a
/// reference names nothing.
#[must_use]
pub fn follow(
    docs: &Documents<'_>,
    base: &str,
    doc: DocId,
    node: NodeId,
) -> Option<(DocId, NodeId)> {
    let (mut doc, mut node) = (doc, node);
    for _ in 0..32 {
        let d = docs.get(doc)?;
        let Some(r) = d.tree.get(node, "$ref").and_then(|n| d.tree.as_str(n)) else {
            return Some((doc, node));
        };
        let absolute = uri::resolve(if doc == DocId::FIRST { base } else { &d.uri }, &r);
        let (resource, fragment) = uri::split_fragment(&absolute);
        let target = if resource == base || resource.is_empty() {
            DocId::FIRST
        } else {
            docs.find(resource)?
        };
        let t = &docs.get(target)?.tree;
        node = resolve_pointer(t, t.root(), fragment)?;
        doc = target;
    }
    None
}

pub(super) struct Reader<'d, 't> {
    pub(super) docs: &'d Documents<'t>,
    pub(super) base: &'d str,
    pub(super) shared: Arc<str>,
    pub(super) dialect: Dialect,
    pub(super) version: Version,
}

impl<'t> Reader<'_, 't> {
    pub(super) fn tree(&self, doc: DocId) -> Option<&Tree<'t>> {
        self.docs.get(doc).map(|d| &d.tree)
    }

    /// [`follow`] in this description.
    pub(super) fn follow(&self, doc: DocId, node: NodeId) -> Option<(DocId, NodeId)> {
        follow(self.docs, self.base, doc, node)
    }

    /// [`Self::follow`], reporting a reference that names nothing.
    pub(super) fn follow_or_report(
        &self,
        doc: DocId,
        node: NodeId,
        what: &str,
        diags: &mut Diagnostics,
    ) -> Option<(DocId, NodeId)> {
        let found = self.follow(doc, node);
        if found.is_none() {
            let r = self
                .tree(doc)
                .and_then(|t| t.get(node, "$ref"))
                .and_then(|n| self.tree(doc)?.as_str(n));
            let r = r.as_deref().unwrap_or("?");
            diags.error(
                self.reference_rule(),
                self.docs.loc(doc, node),
                format!("{what} is a reference that names nothing: {r}"),
            );
        }
        found
    }

    /// The requirement a dangling Reference Object breaks, in the document's version.
    fn reference_rule(&self) -> &'static str {
        match self.version {
            // Swagger 2.0 is not in the requirements ledger: our own code.
            Version::V20 => "swagger.ref",
            Version::V30 => "oas-3.0.reference-object.$ref",
            Version::V31 => "oas-3.1.reference-object.$ref",
            Version::V32 => "oas-3.2.reference-object.$ref",
        }
    }

    /// The operations of a Path Item, the first item's before those its `$ref` names: the fixed
    /// methods in the spec's order, then 3.2's `additionalOperations` as written.
    pub(super) fn methods(&self, items: &[(DocId, NodeId)]) -> Vec<(String, (DocId, NodeId))> {
        let mut out: Vec<(String, (DocId, NodeId))> = Vec::new();
        // `query` and `additionalOperations` are 3.2's: in 3.0 and 3.1 they are not fields;
        // `trace` is not one in Swagger 2.0.
        let v32 = self.version == Version::V32;
        let v20 = self.version == Version::V20;
        let fields = METHODS
            .iter()
            .filter(|m| (v32 || **m != "query") && !(v20 && **m == "trace"));
        for method in fields {
            if let Some(found) = items
                .iter()
                .find_map(|&(doc, item)| Some((doc, self.tree(doc)?.get(item, method)?)))
            {
                out.push(((*method).to_owned(), found));
            }
        }
        for &(doc, item) in items.iter().filter(|_| v32) {
            let Some(t) = self.tree(doc) else { continue };
            for m in t
                .get(item, "additionalOperations")
                .map_or(&[][..], |n| t.members(n))
            {
                let Some(name) = t.string(m.key) else {
                    continue;
                };
                if out.iter().all(|(n, _)| *n != name) {
                    out.push((name.into_owned(), (doc, m.value)));
                }
            }
        }
        out
    }

    pub(super) fn schema(&self, doc: DocId, node: NodeId) -> Resource {
        let base = match self.docs.get(doc) {
            Some(d) if doc != DocId::FIRST => Arc::from(d.uri.as_str()),
            Some(_) | None => Arc::clone(&self.shared),
        };
        Resource {
            doc,
            node,
            base,
            dialect: self.dialect,
        }
    }

    pub(super) fn flag(&self, doc: DocId, node: NodeId, key: &str) -> bool {
        self.tree(doc)
            .and_then(|t| t.get(node, key).and_then(|n| t.as_bool(n)))
            == Some(true)
    }

    /// A parameter's or a header's schema: `schema`, or that of its one media type. In Swagger
    /// 2.0 a parameter other than `body`, or a header, is its own schema: its `type`, `format`,
    /// `items`, `enum` and bounds are the Schema Object's keywords, read in the same dialect
    /// (`collectionFormat` says how a value is written, not what it is).
    pub(super) fn value_schema(
        &self,
        doc: DocId,
        node: NodeId,
        diags: &mut Diagnostics,
    ) -> Option<Resource> {
        if self.version == Version::V20 {
            return Some(self.schema(doc, node));
        }
        let t = self.tree(doc)?;
        // `content` of a parameter or a header holds one media type (the spec's MUST).
        if let Some(c) = t.get(node, "content").filter(|&c| t.members(c).len() > 1) {
            diags.warn(
                "oas-3.1.parameter-object.content.must-map-only-contain-one-entry",
                self.docs.loc(doc, c),
                "`content` names more than one media type; the first one's schema is read",
            );
        }
        let s = t.get(node, "schema").or_else(|| {
            t.get(node, "content")
                .and_then(|c| t.members(c).first().and_then(|m| t.get(m.value, "schema")))
        })?;
        Some(self.schema(doc, s))
    }

    /// The `parameters` of a Path Item or an operation; `None` when it has none.
    pub(super) fn params(
        &self,
        doc: DocId,
        owner: NodeId,
        diags: &mut Diagnostics,
    ) -> Option<Vec<Param>> {
        let t = self.tree(doc)?;
        let list = t.get(owner, "parameters")?;
        let mut out = Vec::new();
        for &n in t.items(list) {
            let Some((pd, p)) = self.follow_or_report(doc, n, "a parameter", diags) else {
                continue;
            };
            let Some(pt) = self.tree(pd) else { continue };
            let location = match pt.get(p, "in").and_then(|n| pt.as_str(n)).as_deref() {
                Some("path") => Location::Path,
                Some("query") => Location::Query,
                Some("querystring") => Location::QueryString,
                Some("header") => Location::Header,
                Some("cookie") => Location::Cookie,
                Some(_) | None => continue,
            };
            let Some(name) = pt.get(p, "name").and_then(|n| pt.as_str(n)) else {
                continue;
            };
            if location == Location::Header
                && IGNORED_HEADERS.contains(&name.to_ascii_lowercase().as_str())
            {
                continue;
            }
            // A header's or a cookie's name goes into the request as written: one that is not a
            // token could end the header it is written in.
            if matches!(location, Location::Header | Location::Cookie) && !token(&name) {
                let message = format!(
                    "{} name {name:?} is not a token (RFC 9110, 5.6.2): left out",
                    location.name()
                );
                diags.warn("parameter.token", self.docs.loc(pd, p), message);
                continue;
            }
            let required = location == Location::Path || self.flag(pd, p, "required");
            let deprecated = self.flag(pd, p, "deprecated");
            out.push(Param {
                name: name.into_owned(),
                location,
                required,
                deprecated,
                schema: self.value_schema(pd, p, diags),
                serialization: self.serialization(pd, p, location, diags),
            });
        }
        Some(out)
    }

    pub(super) fn content(&self, doc: DocId, owner: NodeId, diags: &mut Diagnostics) -> Vec<Media> {
        let Some(t) = self.tree(doc) else {
            return Vec::new();
        };
        let Some(content) = t.get(owner, "content") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for m in t.members(content) {
            let Some(media_type) = t.string(m.key) else {
                continue;
            };
            // 3.2: a Media Type Object may be a reference (`components/mediaTypes`).
            let Some((md, media)) = self.follow_or_report(doc, m.value, "a media type", diags)
            else {
                continue;
            };
            let schema = self
                .tree(md)
                .and_then(|mt| mt.get(media, "schema"))
                .map(|s| self.schema(md, s));
            out.push(Media {
                media_type: media_type.into_owned(),
                payload: Payload::Schema(schema),
            });
        }
        out
    }

    fn body(&self, doc: DocId, node: NodeId, diags: &mut Diagnostics) -> Option<Body> {
        let (bd, b) = self.follow_or_report(doc, node, "a request body", diags)?;
        Some(Body {
            required: self.flag(bd, b, "required"),
            content: self.content(bd, b, diags),
        })
    }

    /// One operation, its parameters already merged with its Path Item's.
    pub(super) fn operation(
        &self,
        path: &str,
        method: &str,
        doc: DocId,
        op: NodeId,
        params: Vec<Param>,
        diags: &mut Diagnostics,
    ) -> Operation {
        let t = self.tree(doc);
        let name = t
            .and_then(|t| t.get(op, "operationId").and_then(|n| t.as_str(n)))
            .map_or_else(|| format!("{method} {path}"), std::borrow::Cow::into_owned);
        // 3.0: a body on GET, HEAD or DELETE SHALL be ignored.
        let no_body = self.version == Version::V30 && matches!(method, "get" | "head" | "delete");
        let body = t
            .and_then(|t| t.get(op, "requestBody"))
            .filter(|_| !no_body)
            .and_then(|b| self.body(doc, b, diags));
        // Swagger 2.0: a response's schema is its content under each media type it produces.
        let produces = match self.version {
            Version::V20 => self.media_types(doc, op, "produces"),
            Version::V30 | Version::V31 | Version::V32 => Vec::new(),
        };
        let responses = t
            .and_then(|t| t.get(op, "responses"))
            .map(|n| self.responses(doc, n, &produces, diags))
            .unwrap_or_default();
        Operation {
            name,
            path: path.to_owned(),
            method: method.to_owned(),
            deprecated: self.flag(doc, op, "deprecated"),
            params,
            body,
            responses,
        }
    }
}
