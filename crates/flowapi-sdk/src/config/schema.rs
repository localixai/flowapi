//! The format (`shape.rs`) printed as a JSON Schema (2020-12), for an agent that writes a config
//! by it: the members of each object and none other (`additionalProperties: false`), the names'
//! and strings' patterns (`strings.rs`), defaults, a pagination scheme's members chosen by its
//! `type`. Printed from the definitions the checker reads by, so it says what the checker does.

use super::shape::{CONFIG, Field, Shape, Str};
use super::strings;
use crate::out::json_str;
use std::fmt::Write as _;

/// A JSON value as printed: an object's members in order.
enum J {
    Obj(Vec<(&'static str, J)>),
    List(Vec<J>),
    Str(String),
    Raw(String),
}

fn s(text: &str) -> J {
    J::Str(text.to_owned())
}

fn raw(text: impl Into<String>) -> J {
    J::Raw(text.into())
}

/// The JSON Schema of `sdk-anything.yml`.
pub(crate) fn schema() -> String {
    let mut defs: Vec<(&'static str, J)> = Vec::new();
    let J::Obj(root) = shape(&CONFIG, &mut defs) else {
        return String::new();
    };
    let mut top = vec![
        ("$schema", s("https://json-schema.org/draft/2020-12/schema")),
        (
            "$id",
            s("https://flowapi.localix.ai/schemas/sdk-anything-1.json"),
        ),
        ("title", s("sdk-anything.yml")),
        (
            "description",
            s(
                "The config flowapi generates a TypeScript SDK by: its resources and methods, each naming an operation of the description, its pagination, environments and authentication. Version 1.",
            ),
        ),
    ];
    top.extend(root);
    defs.sort_by(|a, b| a.0.cmp(b.0));
    top.push(("$defs", J::Obj(defs)));
    let mut out = String::new();
    print(&J::Obj(top), 0, &mut out);
    out.push('\n');
    out
}

fn shape(sh: &'static Shape, defs: &mut Vec<(&'static str, J)>) -> J {
    match sh {
        Shape::Named(name, inner) => {
            if !defs.iter().any(|(n, _)| n == name) {
                defs.push((name, raw("null")));
                let body = shape(inner, defs);
                if let Some(slot) = defs.iter_mut().find(|(n, _)| n == name) {
                    slot.1 = body;
                }
            }
            J::Obj(vec![("$ref", J::Str(format!("#/$defs/{name}")))])
        }
        Shape::Str(kind) => {
            let mut m = vec![("type", s("string"))];
            if let Some(p) = strings::pattern(*kind) {
                m.push(("pattern", s(p)));
            }
            if *kind == Str::Url {
                m.push(("format", s("uri")));
            }
            J::Obj(m)
        }
        Shape::Int(min, max) => J::Obj(vec![
            ("type", s("integer")),
            ("minimum", raw(min.to_string())),
            ("maximum", raw(max.to_string())),
        ]),
        Shape::Seconds => J::Obj(vec![("type", s("number")), ("exclusiveMinimum", raw("0"))]),
        Shape::Bool => J::Obj(vec![("type", s("boolean"))]),
        Shape::False => J::Obj(vec![("const", raw("false"))]),
        Shape::One => J::Obj(vec![("const", raw("1"))]),
        Shape::Obj(fields) => object(fields, None, defs),
        Shape::Map { key, value, fixed } => {
            let mut m = match object(fixed, Some(shape(value, defs)), defs) {
                J::Obj(m) => m,
                J::List(_) | J::Str(_) | J::Raw(_) => Vec::new(),
            };
            if let Some(p) = strings::pattern(*key) {
                m.push(("propertyNames", J::Obj(vec![("pattern", s(p))])));
            }
            J::Obj(m)
        }
        Shape::List(item) => J::Obj(vec![("type", s("array")), ("items", shape(item, defs))]),
        Shape::Either(all) => J::Obj(vec![(
            "anyOf",
            J::List(all.iter().map(|a| shape(a, defs)).collect()),
        )]),
        Shape::Tagged(variants) => {
            let names: Vec<J> = variants.iter().map(|(n, _)| s(n)).collect();
            let all = variants
                .iter()
                .map(|(tag, fields)| tagged(tag, fields, defs))
                .collect();
            J::Obj(vec![
                ("type", s("object")),
                ("required", J::List(vec![s("type")])),
                (
                    "properties",
                    J::Obj(vec![("type", J::Obj(vec![("enum", J::List(names))]))]),
                ),
                ("oneOf", J::List(all)),
            ])
        }
    }
}

/// One pagination scheme: its members, `type` its tag.
fn tagged(tag: &str, fields: &'static [Field], defs: &mut Vec<(&'static str, J)>) -> J {
    let J::Obj(mut m) = object(fields, None, defs) else {
        return raw("{}");
    };
    for (key, value) in &mut m {
        if *key == "properties"
            && let J::Obj(props) = value
            && let Some(t) = props.iter_mut().find(|(k, _)| *k == "type")
        {
            t.1 = J::Obj(vec![("const", s(tag))]);
        }
    }
    J::Obj(m)
}

fn object(fields: &'static [Field], extra: Option<J>, defs: &mut Vec<(&'static str, J)>) -> J {
    let props: Vec<(&'static str, J)> = fields
        .iter()
        .map(|f| {
            let J::Obj(mut m) = shape(f.shape, defs) else {
                return (f.name, raw("{}"));
            };
            m.push(("description", s(f.about)));
            if let Some(d) = f.default {
                m.push(("default", raw(d)));
            }
            (f.name, J::Obj(m))
        })
        .collect();
    let required: Vec<J> = fields
        .iter()
        .filter(|f| f.required)
        .map(|f| s(f.name))
        .collect();
    let mut m = vec![("type", s("object"))];
    if !props.is_empty() {
        m.push(("properties", J::Obj(props)));
    }
    if !required.is_empty() {
        m.push(("required", J::List(required)));
    }
    m.push((
        "additionalProperties",
        extra.unwrap_or_else(|| raw("false")),
    ));
    J::Obj(m)
}

/// The value as indented JSON. The schema nests a few levels: the walk is bounded by the format,
/// which no input changes.
fn print(j: &J, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent + 1);
    match j {
        J::Str(t) => out.push_str(&json_str(t)),
        J::Raw(t) => out.push_str(t),
        J::List(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                out.push_str(&pad);
                print(item, indent + 1, out);
            }
            let _ = write!(out, "\n{}]", "  ".repeat(indent));
        }
        J::Obj(members) => {
            out.push('{');
            for (i, (k, v)) in members.iter().enumerate() {
                out.push_str(if i == 0 { "\n" } else { ",\n" });
                let _ = write!(out, "{pad}{}: ", json_str(k));
                print(v, indent + 1, out);
            }
            let _ = write!(out, "\n{}}}", "  ".repeat(indent));
        }
    }
}
