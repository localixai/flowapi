//! An OpenAPI description's operations: each path's methods (and 3.2's `additionalOperations`),
//! their parameters (the path item's and the operation's own, the latter winning), request
//! bodies, and responses with their headers, Reference Objects followed and a dangling one
//! reported. What each holds is a schema the graph reads; nothing else of an operation
//! (servers, security, callbacks, links; Swagger 2.0's `host`, `basePath`, `schemes`) says what
//! a value is. Swagger 2.0's operations are lifted into the same model (`swagger.rs`).

mod reader;
mod responses;
mod style;
mod swagger;

use crate::diag::Diagnostics;
use crate::id::DocId;
use crate::schema::Resource;
use crate::source::{Dialect, Documents, Version};
use reader::Reader;
pub use reader::follow;
use std::sync::Arc;

/// Where a parameter goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Location {
    /// A path template's expression; always required.
    Path,
    /// A query parameter.
    Query,
    /// The whole query string (3.2).
    QueryString,
    /// A header.
    Header,
    /// A cookie.
    Cookie,
}

impl Location {
    /// The `in` value, as the description writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::QueryString => "querystring",
            Self::Header => "header",
            Self::Cookie => "cookie",
        }
    }
}

/// How a parameter's value is written (3.2, "Style Values"): RFC 6570's operators and the three
/// query styles it has no operator for, 3.2's `cookie`, and Swagger 2.0's `tsv`. In 2.0 the
/// delimited styles go in a path and a header too: a `collectionFormat` is allowed in each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// `;name=value` (RFC 6570 `{;x}`), in a path.
    Matrix,
    /// `.value` (RFC 6570 `{.x}`), in a path.
    Label,
    /// `value` (RFC 6570 `{x}`), in a path or a header.
    Simple,
    /// `name=value` (RFC 6570 `{?x}`), in a query or a cookie.
    Form,
    /// Values joined by a space, in a query (2.0's `ssv`).
    SpaceDelimited,
    /// Values joined by `|`, in a query (2.0's `pipes`).
    PipeDelimited,
    /// Values joined by a tab: 2.0's `tsv`, which no 3.x style is.
    TabDelimited,
    /// `name[key]=value` for each member of an object, in a query.
    DeepObject,
    /// `name=value; name=value` with no percent-encoding, in a cookie (3.2).
    Cookie,
}

impl Style {
    /// The `style` value, as the description writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Matrix => "matrix",
            Self::Label => "label",
            Self::Simple => "simple",
            Self::Form => "form",
            Self::SpaceDelimited => "spaceDelimited",
            Self::PipeDelimited => "pipeDelimited",
            Self::TabDelimited => "tabDelimited",
            Self::DeepObject => "deepObject",
            Self::Cookie => "cookie",
        }
    }
}

/// How a parameter or a header is serialized: by `style`, `explode` and `allowReserved`
/// (a parameter with `schema`), or by its one media type (a parameter with `content`, and
/// `querystring` always). Defaults are the spec's: `form` for a query or a cookie, `simple` for a
/// path or a header; `explode` true for `form` and `cookie` only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Serialization {
    /// The style.
    pub style: Style,
    /// Whether an array or an object makes a pair for each member.
    pub explode: bool,
    /// Whether RFC 3986's reserved characters and percent-encoded triples pass unencoded.
    pub allow_reserved: bool,
    /// The media type of `content`, when the value is serialized by it rather than by style.
    pub media_type: Option<String>,
    /// Swagger 2.0: how the arrays inside an array value are joined, one style per level of
    /// `items` that is an array, outermost first (an Items Object's `collectionFormat`):
    /// `Simple` for `csv` (`,`), else a delimited style. Empty in 3.x, where an array inside a
    /// value is its JSON text.
    pub nested: Vec<Style>,
}

/// A parameter, or a response's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    /// Its name.
    pub name: String,
    /// Where it goes (`Header` for a response's header).
    pub location: Location,
    /// Whether it must be given.
    pub required: bool,
    /// Whether it is deprecated.
    pub deprecated: bool,
    /// Its schema (or the schema of its one media type); `None` for any value.
    pub schema: Option<Resource>,
    /// How its value is written.
    pub serialization: Serialization,
}

/// A body's representations: a media type each, with what it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    /// The media type or range, as written (`application/json`, `text/*`).
    pub media_type: String,
    /// What it holds.
    pub payload: Payload,
    /// How a request's form members are written, by name in the order written; empty for any
    /// other media type: `application/x-www-form-urlencoded`'s Encoding Objects (3.x), or Swagger
    /// 2.0's array `formData` fields by their `collectionFormat` (under each form type). A
    /// member written by style, as a `query` parameter of that style, has no `media_type`; one
    /// written by its Encoding Object's `contentType` alone has it. A member not here is written
    /// by the form's defaults.
    pub encoding: Vec<(String, Serialization)>,
}

impl Media {
    /// The schemas it holds.
    #[must_use]
    pub fn schemas(&self) -> Vec<&Resource> {
        match &self.payload {
            Payload::Schema(schema) => schema.iter().collect(),
            Payload::Form(fields) => fields.iter().filter_map(|f| f.schema.as_ref()).collect(),
        }
    }
}

/// What a representation holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Payload {
    /// A schema's values; `None` for any value.
    Schema(Option<Resource>),
    /// Swagger 2.0's `formData` parameters: an object of these fields and no other, as the
    /// operation's parameters are.
    Form(Vec<Field>),
}

/// A field of a Swagger 2.0 form: a `formData` parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// Its name.
    pub name: String,
    /// Whether it must be given.
    pub required: bool,
    /// Its schema: the parameter itself, read as a Schema Object.
    pub schema: Option<Resource>,
}

/// A request body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    /// Whether it must be sent.
    pub required: bool,
    /// Its representations.
    pub content: Vec<Media>,
}

/// A response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    /// The status code, a range (`4XX`) or `default`.
    pub status: String,
    /// Its headers, `Content-Type` left out (the spec says it SHALL be ignored).
    pub headers: Vec<Param>,
    /// Its representations; empty for a response with no body.
    pub content: Vec<Media>,
}

/// An operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    /// Its `operationId`, or its method and path.
    pub name: String,
    /// The path template.
    pub path: String,
    /// The method, as the Path Item names it.
    pub method: String,
    /// Whether it is deprecated.
    pub deprecated: bool,
    /// Its parameters, path item's first.
    pub params: Vec<Param>,
    /// Its request body.
    pub body: Option<Body>,
    /// Its responses, in the order written.
    pub responses: Vec<Response>,
}

const METHODS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

/// Every operation of the first document, in the order written.
#[must_use]
pub(super) fn read(
    docs: &Documents<'_>,
    base: &str,
    dialect: Dialect,
    version: Version,
    diags: &mut Diagnostics,
) -> Vec<Operation> {
    let mut out = Vec::new();
    let Some(d) = docs.get(DocId::FIRST) else {
        return out;
    };
    let tree = &d.tree;
    let Some(paths) = tree.get(tree.root(), "paths") else {
        return out;
    };
    let r = Reader {
        docs,
        base,
        shared: Arc::from(base),
        dialect,
        version,
    };
    for m in tree.members(paths) {
        let Some(path) = tree.string(m.key) else {
            continue;
        };
        // The Path Item's own fields win over those of what its `$ref` names (3.1, 3.2).
        let mut items = vec![(DocId::FIRST, m.value)];
        if let Some(target) = r.follow_or_report(
            DocId::FIRST,
            m.value,
            &format!("the Path Item of `{path}`"),
            diags,
        ) && target != (DocId::FIRST, m.value)
        {
            items.push(target);
        }
        let shared = items
            .iter()
            .find_map(|&(doc, item)| r.params(doc, item, diags));
        for (method, (doc, op)) in r.methods(&items) {
            let mut params = shared.clone().unwrap_or_default();
            for p in r.params(doc, op, diags).unwrap_or_default() {
                params.retain(|q| !(q.name == p.name && q.location == p.location));
                params.push(p);
            }
            let mut operation = r.operation(&path, &method, doc, op, params, diags);
            if version == Version::V20 {
                operation.body = r.swagger_body(&items, doc, op, diags);
            }
            out.push(operation);
        }
    }
    out
}
