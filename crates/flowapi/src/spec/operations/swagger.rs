//! Swagger 2.0's operations lifted into the 3.x model: the `body` parameter is the request body
//! under each media type the operation `consumes`, the `formData` parameters a form under the
//! form media types, and a response's `schema` its content under each type it `produces`. The
//! operation's own lists win over the description's; with neither, `application/json`.

use super::reader::Reader;
use super::{Body, Field, Location, Media, Payload};
use crate::diag::Diagnostics;
use crate::id::{DocId, NodeId};

/// The media types a form is sent as (2.0, "Parameter Object", `formData`): with no file, then
/// with one.
pub(super) const FORM_TYPES: [&str; 2] =
    ["application/x-www-form-urlencoded", "multipart/form-data"];

/// A `body` or `formData` parameter: whether it is the body, its name, and where it is.
type Part = (bool, String, (DocId, NodeId));

impl Reader<'_, '_> {
    /// The media types of an operation's `consumes` or `produces`: its own list (an empty one
    /// clears the description's), else the description's, else `application/json`.
    pub(super) fn media_types(&self, doc: DocId, op: NodeId, key: &str) -> Vec<String> {
        let own = self.tree(doc).and_then(|t| Some((t, t.get(op, key)?)));
        let global = self
            .tree(DocId::FIRST)
            .and_then(|t| Some((t, t.get(t.root(), key)?)));
        let mut out: Vec<String> = Vec::new();
        if let Some((t, list)) = own.or(global) {
            for name in t.items(list).iter().filter_map(|&n| t.as_str(n)) {
                if !out.iter().any(|m| *m == name) {
                    out.push(name.into_owned());
                }
            }
        }
        if out.is_empty() {
            out.push(String::from("application/json"));
        }
        out
    }

    /// A Swagger 2.0 response's content: its `schema` under each media type given; none when it
    /// has no `schema` (no body).
    pub(super) fn produced(&self, doc: DocId, response: NodeId, produces: &[String]) -> Vec<Media> {
        let Some(schema) = self.tree(doc).and_then(|t| t.get(response, "schema")) else {
            return Vec::new();
        };
        produces
            .iter()
            .map(|media_type| Media {
                media_type: media_type.clone(),
                payload: Payload::Schema(Some(self.schema(doc, schema))),
                encoding: Vec::new(),
            })
            .collect()
    }

    /// The `body` and `formData` parameters of a Path Item (the first of `items` that has
    /// `parameters`) and its operation, the operation's winning by `in` and name. A reference
    /// that names nothing was reported when the parameters were read.
    fn parts(&self, items: &[(DocId, NodeId)], doc: DocId, op: NodeId) -> Vec<Part> {
        let shared = items.iter().find(|&&(d, item)| {
            self.tree(d)
                .is_some_and(|t| t.get(item, "parameters").is_some())
        });
        let mut out: Vec<Part> = Vec::new();
        for &(d, owner) in shared.into_iter().chain([&(doc, op)]) {
            let Some(t) = self.tree(d) else { continue };
            for &n in t.get(owner, "parameters").map_or(&[][..], |l| t.items(l)) {
                let Some((pd, p)) = self.follow(d, n) else {
                    continue;
                };
                let Some(pt) = self.tree(pd) else { continue };
                let is_body = match pt.get(p, "in").and_then(|n| pt.as_str(n)).as_deref() {
                    Some("body") => true,
                    Some("formData") => false,
                    Some(_) | None => continue,
                };
                let Some(name) = pt.get(p, "name").and_then(|n| pt.as_str(n)) else {
                    continue;
                };
                out.retain(|(b, other, _)| !(*b == is_body && *other == name));
                out.push((is_body, name.into_owned(), (pd, p)));
            }
        }
        out
    }

    /// A Swagger 2.0 operation's request body: its `body` parameter, else its form; `None` when
    /// it has neither.
    pub(super) fn swagger_body(
        &self,
        items: &[(DocId, NodeId)],
        doc: DocId,
        op: NodeId,
        diags: &mut Diagnostics,
    ) -> Option<Body> {
        let parts = self.parts(items, doc, op);
        let has_form = parts.iter().any(|(is_body, _, _)| !is_body);
        if let Some((_, _, (bd, b))) = parts.iter().rev().find(|(is_body, _, _)| *is_body) {
            if has_form {
                diags.warn(
                    "swagger.body",
                    self.docs.loc(*bd, *b),
                    "an operation has both a `body` and `formData` parameters, which 2.0 \
                     forbids: the body is read, the form left out",
                );
            }
            let schema = self.tree(*bd).and_then(|t| t.get(*b, "schema"));
            let content = self
                .media_types(doc, op, "consumes")
                .into_iter()
                .map(|media_type| Media {
                    media_type,
                    payload: Payload::Schema(schema.map(|s| self.schema(*bd, s))),
                    encoding: Vec::new(),
                })
                .collect();
            return Some(Body {
                required: self.flag(*bd, *b, "required"),
                content,
            });
        }
        if !has_form {
            return None;
        }
        let fields: Vec<Field> = parts
            .iter()
            .map(|(_, name, (fd, f))| Field {
                name: name.clone(),
                required: self.flag(*fd, *f, "required"),
                schema: Some(self.schema(*fd, *f)),
            })
            .collect();
        // An array field is written by its `collectionFormat`, as a query parameter would be.
        let mut encoding = Vec::new();
        for (_, name, at) in parts.iter().filter(|(_, _, at)| self.typed(*at, "array")) {
            let s = self.collection_format(*at, Location::Query, diags);
            encoding.push((name.clone(), s));
        }
        let content = self
            .form_types(doc, op, &parts)
            .into_iter()
            .map(|media_type| Media {
                media_type,
                payload: Payload::Form(fields.clone()),
                encoding: encoding.clone(),
            })
            .collect();
        Some(Body {
            required: fields.iter().any(|f| f.required),
            content,
        })
    }

    /// The media types a form is sent as: those of `consumes` that are form types, else
    /// `multipart/form-data` when a field is a file and `application/x-www-form-urlencoded`
    /// when none is.
    fn form_types(&self, doc: DocId, op: NodeId, parts: &[Part]) -> Vec<String> {
        let form = |m: &String| {
            let bare = m.split(';').next().unwrap_or("").trim();
            FORM_TYPES.iter().any(|f| bare.eq_ignore_ascii_case(f))
        };
        let mut types: Vec<String> = self
            .media_types(doc, op, "consumes")
            .into_iter()
            .filter(form)
            .collect();
        if types.is_empty() {
            let file = parts.iter().any(|(_, _, at)| self.typed(*at, "file"));
            types.push(String::from(FORM_TYPES[usize::from(file)]));
        }
        types
    }

    /// Whether the parameter at `(doc, node)` has `type` `ty`.
    fn typed(&self, (doc, node): (DocId, NodeId), ty: &str) -> bool {
        self.tree(doc)
            .and_then(|t| t.get(node, "type").and_then(|n| t.as_str(n)))
            .is_some_and(|t| t == ty)
    }
}
