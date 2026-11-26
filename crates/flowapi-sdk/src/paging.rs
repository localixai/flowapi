//! Pagination schemes, as a config writes them and as this layer suggests them: which request
//! parameter continues a list and where the response says how (`cursor`, `last_item_id`,
//! `offset`, `page`, `link`, `single`). `fit.rs` holds a scheme to an operation's parameters and
//! response schema, `suggest.rs` finds the schemes an operation looks like it has, and both read
//! a schema through `schema.rs`.

pub(crate) mod fit;
mod schema;
pub(crate) mod suggest;

use crate::config::shape::{Field, SCHEMES, Shape};
use crate::out;
use std::fmt::Write as _;

/// A scheme's members: its `type`, then each of `request` and `response` as role and value (a
/// parameter's name, a path in the body), and a page scheme's `start`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Paging {
    pub(crate) kind: &'static str,
    pub(crate) request: Vec<(&'static str, String)>,
    pub(crate) response: Vec<(&'static str, String)>,
    pub(crate) start: Option<i64>,
}

/// The members of `type`'s `request` or `response`, as the format names them.
pub(crate) fn roles(kind: &str, part: &str) -> &'static [Field] {
    SCHEMES
        .iter()
        .find(|(name, _)| *name == kind)
        .and_then(|(_, fields)| fields.iter().find(|f| f.name == part))
        .and_then(|f| match f.shape {
            Shape::Obj(fields) => Some(*fields),
            Shape::Str(_)
            | Shape::Int(..)
            | Shape::Seconds
            | Shape::Bool
            | Shape::False
            | Shape::One
            | Shape::Map { .. }
            | Shape::List(_)
            | Shape::Either(_)
            | Shape::Tagged(_)
            | Shape::Named(..) => None,
        })
        .unwrap_or(&[])
}

impl Paging {
    /// A scheme of `kind` with these members; a role the format does not name for it, or an empty
    /// value, is left out.
    pub(crate) fn new(
        kind: &'static str,
        request: &[(&str, &str)],
        response: &[(&str, &str)],
    ) -> Self {
        let pick = |part: &str, given: &[(&str, &str)]| {
            roles(kind, part)
                .iter()
                .filter_map(|f| {
                    given
                        .iter()
                        .find(|(r, v)| *r == f.name && !v.is_empty())
                        .map(|(_, v)| (f.name, (*v).to_owned()))
                })
                .collect()
        };
        Self {
            kind,
            request: pick("request", request),
            response: pick("response", response),
            start: None,
        }
    }

    /// A role's value: the request's first, then the response's.
    pub(crate) fn get(&self, role: &str) -> Option<&str> {
        self.request
            .iter()
            .chain(&self.response)
            .find(|(r, _)| *r == role)
            .map(|(_, v)| v.as_str())
    }

    /// The scheme as a JSON object, the config's members.
    pub(crate) fn json(&self) -> String {
        let part = |members: &[(&str, String)]| {
            let m: Vec<String> = members
                .iter()
                .map(|(r, v)| format!("{}:{}", out::json_str(r), out::json_str(v)))
                .collect();
            format!("{{{}}}", m.join(","))
        };
        let mut out = format!("{{\"type\":{}", out::json_str(self.kind));
        if !self.request.is_empty() {
            let _ = write!(out, ",\"request\":{}", part(&self.request));
        }
        if !self.response.is_empty() || self.kind != "single" {
            let _ = write!(out, ",\"response\":{}", part(&self.response));
        }
        if let Some(s) = self.start {
            let _ = write!(out, ",\"start\":{s}");
        }
        out.push('}');
        out
    }

    /// The scheme as YAML lines, indented by `indent` spaces.
    pub(crate) fn yaml(&self, indent: usize, out: &mut String) {
        let pad = " ".repeat(indent);
        let _ = writeln!(out, "{pad}type: {}", self.kind);
        for (name, members) in [("request", &self.request), ("response", &self.response)] {
            if members.is_empty() && (name == "request" || self.kind == "single") {
                continue;
            }
            if members.is_empty() {
                let _ = writeln!(out, "{pad}{name}: {{}}");
                continue;
            }
            let _ = writeln!(out, "{pad}{name}:");
            for (r, v) in members {
                let _ = writeln!(out, "{pad}  {r}: {}", out::yaml_str(v));
            }
        }
        if let Some(s) = self.start {
            let _ = writeln!(out, "{pad}start: {s}");
        }
    }
}
