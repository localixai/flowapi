//! How a request body of a form or `multipart` media type is written, as the description says:
//! the Media Type Object's `encoding` (by name, for `multipart/form-data` and
//! `application/x-www-form-urlencoded`), `prefixEncoding` and `itemEncoding` (by position, for any
//! `multipart` type, 3.2), each Encoding Object read for the fields its media type takes
//! (`headers` for `multipart`; `style`, `explode` and `allowReserved` for a form, and in 3.1 and
//! 3.2 for `multipart/form-data`; 3.2's nested `encoding`, `prefixEncoding`, `itemEncoding` for a
//! part that is itself `multipart`). Request bodies only: the Encoding Object applies to no
//! response (3.0, 3.1), and to no other media type. Where the description gives a part no type,
//! its schema may (`inspect.rs`); Swagger 2.0's forms by their fields' `collectionFormat`
//! (`swagger.rs`). `<op>Request` takes the result in its body, and the client passes the
//! description's when the caller gives none.

mod inspect;
mod print;
mod swagger;

pub(crate) use print::literal;

use crate::doc::{At, Doc, oas};
use crate::model::{Encoding, Part, Parts};
use flowapi::spec::Operation;
use flowapi::{Diagnostics, Version};

/// `application/x-www-form-urlencoded`.
const FORM: &str = "application/x-www-form-urlencoded";
/// `multipart/form-data`.
const FORM_DATA: &str = "multipart/form-data";
/// How deep nested Encoding Objects are read: 3.2 asks for one level and allows more; a deeper
/// one is a warning and left out, so no description makes reading it recurse without end (G1).
const NESTING: usize = 4;

/// A media type without its parameters, in lower case.
fn bare(media_type: &str) -> String {
    media_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

/// The requirement a nesting too deep to read is reported under.
const NESTED: &str = "oas-3.2.encoding-object.nested-encoding.must-implementations-support-one-level-nesting-support";

/// The styles a form member takes (those of a `query` parameter).
const STYLES: [&str; 4] = ["form", "spaceDelimited", "pipeDelimited", "deepObject"];

impl Doc<'_, '_> {
    /// The encodings of an operation's request body, one per form or `multipart` media type that
    /// says anything of its values, in the order written.
    pub(crate) fn encodings(
        &self,
        op: Option<At>,
        operation: &Operation,
        diags: &mut Diagnostics,
    ) -> Vec<Encoding> {
        if self.version == Version::V20 {
            return self.swagger_encodings(operation, diags);
        }
        let Some(content) = op
            .and_then(|op| self.get(op, "requestBody"))
            .and_then(|b| self.resolve(b))
            .and_then(|b| self.get(b, "content"))
        else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (media_type, media) in self.members(content) {
            let kind = bare(&media_type);
            if kind != FORM && !kind.starts_with("multipart/") {
                continue;
            }
            let mut parts = self.parts_of(media, &kind, 0, diags);
            if let Some(schema) = self.get(media, "schema") {
                self.inferred(&mut parts, &kind, schema);
            }
            if !parts.is_empty() {
                out.push(Encoding { media_type, parts });
            }
        }
        out
    }

    /// The Encoding Objects of `holder` (a Media Type Object, or an Encoding Object for its
    /// nested ones) as they apply to a value of `kind`: by name for a form and
    /// `multipart/form-data`, by position (3.2) for any `multipart` type.
    fn parts_of(&self, holder: At, kind: &str, depth: usize, diags: &mut Diagnostics) -> Parts {
        let mut parts = Parts::default();
        if (kind == FORM || kind == FORM_DATA)
            && let Some(map) = self.get(holder, "encoding")
        {
            for (name, object) in self.members(map) {
                let part = self.part(object, kind, depth, diags);
                parts.by_name.push((name, part));
            }
        }
        if kind.starts_with("multipart/") && self.version == Version::V32 {
            if let Some(list) = self.get(holder, "prefixEncoding") {
                for object in self.items(list) {
                    parts.prefix.push(self.part(object, kind, depth, diags));
                }
            }
            if let Some(object) = self.get(holder, "itemEncoding") {
                parts.item = Some(Box::new(self.part(object, kind, depth, diags)));
            }
        }
        parts
    }

    /// One Encoding Object as it applies to a value written in `kind`.
    fn part(&self, object: At, kind: &str, depth: usize, diags: &mut Diagnostics) -> Part {
        let mut part = Part {
            content_type: self
                .str(object, "contentType")
                .map(|t| t.trim().to_owned())
                .filter(|t| !t.is_empty()),
            ..Part::default()
        };
        let styled = kind == FORM
            || (kind == FORM_DATA && matches!(self.version, Version::V31 | Version::V32));
        if styled {
            part.style = self.style(object, diags);
            part.explode = self.boolean(object, "explode");
            part.allow_reserved = self.boolean(object, "allowReserved");
        }
        let inner = part.content_type.as_deref().map(bare).unwrap_or_default();
        let nests = inner.starts_with("multipart/") || inner == FORM;
        if self.version == Version::V32 && nests {
            if depth + 1 < NESTING {
                part.parts = self.parts_of(object, &inner, depth + 1, diags);
            } else {
                let message = format!(
                    "Encoding Objects nested deeper than {NESTING} levels are not read: the \
                     part is written by its value's defaults"
                );
                self.warn(diags, NESTED, object, message);
            }
        }
        part
    }

    /// An Encoding Object's `style`, when it is one a form member takes; another is a warning,
    /// and read as not written.
    fn style(&self, object: At, diags: &mut Diagnostics) -> Option<String> {
        let at = self.get(object, "style")?;
        let style = self.str(object, "style");
        if let Some(s) = style.filter(|s| STYLES.contains(&s.as_str())) {
            return Some(s);
        }
        let code = oas!(self.version, "swagger.encoding", "encoding-object.style");
        let message = String::from(
            "`style` is not one a form member takes (form, spaceDelimited, pipeDelimited, \
             deepObject): read as not written",
        );
        self.warn(diags, code, at, message);
        None
    }

    /// A boolean member, where written.
    fn boolean(&self, object: At, key: &str) -> Option<bool> {
        let (doc, node) = self.get(object, key)?;
        self.tree(doc)?.as_bool(node)
    }
}
