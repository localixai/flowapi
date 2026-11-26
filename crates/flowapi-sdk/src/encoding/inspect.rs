//! What the schemas say of a part where its Encoding Object does not: 3.2, "Non-JSON Data". A
//! value is serialized by inspecting the data where it can be (the runtime does: bytes are a
//! file, an object JSON); a string cannot tell whether its text is encoded bytes, so its schema
//! is inspected: the schema and every schema its `$ref` and `allOf` reach, which apply to any
//! instance. `oneOf`, `anyOf` and the rest are not looked into, so no ambiguity is resolved. A
//! string with `contentEncoding` (3.0: `format` `binary` or `byte`) defaults to
//! `application/octet-stream`, not `text/plain`; in a `multipart` type other than
//! `multipart/form-data` its part also says `Content-Transfer-Encoding`, as 3.2 makes
//! `contentEncoding` equivalent to (RFC 7578, 4.7 deprecates it for `multipart/form-data`).

use super::FORM_DATA;
use crate::doc::{At, Doc};
use crate::model::{Part, Parts};
use flowapi::Version;

/// The most schemas one inspection reads, so a cycle of references or a huge `allOf` ends.
const REACH: usize = 64;

/// A string's bytes encoded: whether a schema says so, and by what (`contentEncoding`).
#[derive(Default)]
struct Facts {
    binary: bool,
    encoding: Option<String>,
}

impl Doc<'_, '_> {
    /// The schemas that apply to every instance of `schema`: it, and those its `$ref` and `allOf`
    /// reach (a 3.0 `$ref`'s siblings are not read, as 3.0 says).
    fn reached(&self, schema: At) -> Vec<At> {
        let mut out: Vec<At> = Vec::new();
        let mut stack = vec![schema];
        while let Some(s) = stack.pop() {
            if out.len() >= REACH || out.contains(&s) {
                continue;
            }
            let referred = self.get(s, "$ref").and_then(|_| self.resolve(s));
            if let Some(target) = referred {
                stack.push(target);
                if self.version == Version::V30 {
                    continue;
                }
            }
            out.push(s);
            if let Some(all) = self.get(s, "allOf") {
                stack.extend(self.items(all));
            }
        }
        out
    }

    /// Whether some schema reached from `schema` has `type` this one name.
    fn typed(&self, reached: &[At], name: &str) -> bool {
        reached
            .iter()
            .any(|&s| self.str(s, "type").as_deref() == Some(name))
    }

    /// What `schema` says of a value's bytes; an array's, of its `items`.
    fn facts(&self, schema: At) -> Facts {
        let mut reached = self.reached(schema);
        if self.typed(&reached, "array")
            && let Some(items) = reached.iter().find_map(|&s| self.get(s, "items"))
        {
            reached = self.reached(items);
        }
        let mut facts = Facts::default();
        for &s in &reached {
            if self.version == Version::V30 {
                let format = self.str(s, "format");
                facts.binary |= matches!(format.as_deref(), Some("binary" | "byte"));
            } else if let Some(e) = self.str(s, "contentEncoding") {
                facts.encoding = Some(e);
            }
        }
        facts.binary = (facts.binary && self.typed(&reached, "string")) || facts.encoding.is_some();
        facts
    }

    /// `parts` given what the media type's `schema` says where its Encoding Objects do not:
    /// `multipart/form-data` by its properties' names, another `multipart` type (3.2) by its
    /// items' positions (`prefixItems`, then `items`).
    pub(super) fn inferred(&self, parts: &mut Parts, kind: &str, schema: At) {
        let reached = self.reached(schema);
        if kind == FORM_DATA {
            for &s in &reached {
                let Some(props) = self.get(s, "properties") else {
                    continue;
                };
                for (name, prop) in self.members(props) {
                    let facts = self.facts(prop);
                    if !facts.binary {
                        continue;
                    }
                    let at = parts.by_name.iter().position(|(n, _)| *n == name);
                    let at = at.unwrap_or_else(|| {
                        parts.by_name.push((name, Part::default()));
                        parts.by_name.len() - 1
                    });
                    if let Some((_, part)) = parts.by_name.get_mut(at) {
                        binary(part, None);
                    }
                }
            }
        } else if kind.starts_with("multipart/") && self.version == Version::V32 {
            let prefix = reached.iter().find_map(|&s| self.get(s, "prefixItems"));
            for (i, item) in prefix
                .map(|p| self.items(p))
                .unwrap_or_default()
                .into_iter()
                .enumerate()
            {
                let facts = self.facts(item);
                if facts.binary {
                    while parts.prefix.len() <= i {
                        parts.prefix.push(Part::default());
                    }
                    if let Some(part) = parts.prefix.get_mut(i) {
                        binary(part, facts.encoding);
                    }
                }
            }
            if let Some(items) = reached.iter().find_map(|&s| self.get(s, "items")) {
                let facts = self.facts(items);
                if facts.binary {
                    binary(parts.item.get_or_insert_with(Box::default), facts.encoding);
                }
            }
        }
    }
}

/// A part whose value is encoded bytes: `application/octet-stream` unless its Encoding Object
/// gives a type or a style, and the encoding as `Content-Transfer-Encoding` where given and no
/// header of that name is.
fn binary(part: &mut Part, encoding: Option<String>) {
    let styled = part.style.is_some() || part.explode.is_some() || part.allow_reserved.is_some();
    if part.content_type.is_none() && !styled {
        part.content_type = Some(String::from("application/octet-stream"));
    }
    let named = |(n, _): &(String, String)| n.eq_ignore_ascii_case("content-transfer-encoding");
    if let Some(e) = encoding
        && !part.headers.iter().any(named)
    {
        part.headers
            .push((String::from("Content-Transfer-Encoding"), e));
    }
}
