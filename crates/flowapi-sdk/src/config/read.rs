//! A config's JSON held to the format (`shape.rs`): every member it does not name, every value of
//! the wrong type, every required member missing, each an issue, all of them in one pass. The
//! walk keeps its own stack (G1). It returns the value of every member the format names, so that
//! `build.rs` can say which of them nothing read.

use super::issue::{At, Issues, closest};
use super::shape::{Field, Shape, Str};
use super::strings;
use crate::words;
use flowapi::id::NodeId;
use flowapi::json::{Tree, Value};

/// The JSON types a shape takes.
fn takes(shape: &Shape, value: Value) -> bool {
    match (shape, value) {
        (Shape::Named(_, inner), v) => takes(inner, v),
        (Shape::Str(_), Value::String(_))
        | (Shape::Int(..) | Shape::Seconds | Shape::One, Value::Number(_))
        | (Shape::Bool | Shape::False, Value::Bool(_))
        | (Shape::Obj(_) | Shape::Map { .. } | Shape::Tagged(_), Value::Object(_))
        | (Shape::List(_), Value::Array(_)) => true,
        (Shape::Either(all), v) => all.iter().any(|s| takes(s, v)),
        (
            Shape::Str(_)
            | Shape::Int(..)
            | Shape::Seconds
            | Shape::One
            | Shape::Bool
            | Shape::False
            | Shape::Obj(_)
            | Shape::Map { .. }
            | Shape::Tagged(_)
            | Shape::List(_),
            _,
        ) => false,
    }
}

/// What a shape is, in the words of an issue.
pub(crate) fn describe(shape: &Shape) -> String {
    match shape {
        Shape::Named(_, inner) => describe(inner),
        Shape::Str(Str::Name) => String::from("a name"),
        Shape::Str(_) => String::from("a string"),
        Shape::Int(min, max) => format!("an integer from {min} to {max}"),
        Shape::Seconds => String::from("a number of seconds above 0"),
        Shape::Bool => String::from("`true` or `false`"),
        Shape::False => String::from("`false`"),
        Shape::One => String::from("1"),
        Shape::Obj(_) | Shape::Map { .. } | Shape::Tagged(_) => String::from("an object"),
        Shape::List(_) => String::from("an array"),
        Shape::Either(all) => all
            .iter()
            .map(|s| describe(s))
            .collect::<Vec<_>>()
            .join(" or "),
    }
}

fn kind(value: Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// Holds the tree to the root shape; the values of the members the format names, in the order
/// met.
pub(crate) fn validate(
    tree: &Tree<'_>,
    root: &'static Shape,
    issues: &mut Issues,
) -> Vec<(NodeId, At)> {
    let mut declared = Vec::new();
    let mut stack: Vec<(NodeId, &'static Shape, At)> = vec![(tree.root(), root, At::default())];
    while let Some((node, shape, at)) = stack.pop() {
        let value = tree.value(node);
        if !takes(shape, value) {
            let message = format!(
                "`{}` is {}; it is {}",
                show(&at),
                kind(value),
                describe(shape)
            );
            issues.error("config.schema.type", &at, message);
            continue;
        }
        match shape {
            Shape::Named(_, inner) => stack.push((node, inner, at)),
            Shape::Either(all) => {
                if let Some(s) = all.iter().find(|s| takes(s, value)) {
                    stack.push((node, s, at));
                }
            }
            Shape::Str(s) => text(tree, node, *s, &at, issues),
            Shape::Int(..) | Shape::Seconds | Shape::One => number(tree, node, shape, &at, issues),
            Shape::Bool | Shape::False => {
                if matches!(shape, Shape::False) && tree.as_bool(node) == Some(true) {
                    let message = format!("`{}` is `true`; it is a name or `false`", show(&at));
                    issues.error("config.schema.type", &at, message);
                }
            }
            Shape::Obj(fields) => members(
                tree,
                node,
                fields,
                None,
                &at,
                issues,
                &mut declared,
                &mut stack,
            ),
            Shape::Map { key, value, fixed } => {
                let entries = Some((*key, *value));
                members(
                    tree,
                    node,
                    fixed,
                    entries,
                    &at,
                    issues,
                    &mut declared,
                    &mut stack,
                );
            }
            Shape::Tagged(variants) => {
                if let Some(fields) = variant(tree, node, variants, &at, issues) {
                    members(
                        tree,
                        node,
                        fields,
                        None,
                        &at,
                        issues,
                        &mut declared,
                        &mut stack,
                    );
                }
            }
            Shape::List(item) => {
                for (i, &n) in tree.items(node).iter().enumerate() {
                    stack.push((n, item, at.child(&i.to_string(), tree.offset(n))));
                }
            }
        }
    }
    declared
}

/// A pointer as an issue shows it: `/` for the root.
pub(crate) fn show(at: &At) -> String {
    if at.pointer.is_empty() {
        String::from("/")
    } else {
        at.pointer.clone()
    }
}

/// The members of an object: those `fields` names, and, for a map, the entries; any other is
/// an issue, and so is a required one missing or a key written twice.
#[expect(
    clippy::too_many_arguments,
    reason = "one walk's state, passed down rather than shared"
)]
fn members(
    tree: &Tree<'_>,
    node: NodeId,
    fields: &'static [Field],
    entries: Option<(Str, &'static Shape)>,
    at: &At,
    issues: &mut Issues,
    declared: &mut Vec<(NodeId, At)>,
    stack: &mut Vec<(NodeId, &'static Shape, At)>,
) {
    let mut seen: Vec<String> = Vec::new();
    for m in tree.members(node) {
        let key = tree
            .string(m.key)
            .map(|k| k.into_owned())
            .unwrap_or_default();
        let here = at.child(&key, m.key.start);
        if seen.contains(&key) {
            issues.error(
                "config.schema.duplicate",
                &here,
                format!("`{key}` is written twice"),
            );
            continue;
        }
        seen.push(key.clone());
        if let Some(field) = fields.iter().find(|f| f.name == key) {
            declared.push((m.value, here.clone()));
            stack.push((m.value, field.shape, here));
        } else if let Some((kind, shape)) = entries {
            if kind == Str::Name && !words::is_name(&key) {
                issues.error("config.name.invalid", &here, not_a_name(&key));
            }
            declared.push((m.value, here.clone()));
            stack.push((m.value, shape, here));
        } else {
            let hint = closest(&key, fields.iter().map(|f| f.name))
                .map(|k| format!("did you mean `{k}`?"));
            let names: Vec<&str> = fields.iter().map(|f| f.name).collect();
            let message = format!(
                "`{key}` is not a member of `{}` (it has {})",
                show(at),
                names.join(", ")
            );
            issues.error_hint("config.schema.unknown", &here, message, hint);
        }
    }
    for f in fields
        .iter()
        .filter(|f| f.required && !seen.iter().any(|k| k == f.name))
    {
        let message = format!("`{}` needs `{}`: {}", show(at), f.name, f.about);
        issues.error("config.schema.required", at, message);
    }
}

pub(crate) fn not_a_name(key: &str) -> String {
    format!("`{key}` is not a name: ASCII letters, digits, `_` and `-`, opening with a letter")
}

/// The members of a pagination scheme, by its `type`.
fn variant(
    tree: &Tree<'_>,
    node: NodeId,
    variants: &'static [(&'static str, &'static [Field])],
    at: &At,
    issues: &mut Issues,
) -> Option<&'static [Field]> {
    let Some(t) = tree.get(node, "type") else {
        let names: Vec<&str> = variants.iter().map(|v| v.0).collect();
        let message = format!("`{}` needs `type`: one of {}", show(at), names.join(", "));
        issues.error("config.schema.required", at, message);
        return None;
    };
    let name = tree.as_str(t).map(|s| s.into_owned()).unwrap_or_default();
    if let Some((_, fields)) = variants.iter().find(|v| v.0 == name) {
        return Some(fields);
    }
    let here = at.child("type", tree.offset(t));
    let names: Vec<&str> = variants.iter().map(|v| v.0).collect();
    let hint = closest(&name, names.iter().copied()).map(|k| format!("did you mean `{k}`?"));
    let message = format!(
        "`{name}` is not a pagination scheme's type: one of {}",
        names.join(", ")
    );
    issues.error_hint("config.pagination.type", &here, message, hint);
    None
}

fn number(tree: &Tree<'_>, node: NodeId, shape: &Shape, at: &At, issues: &mut Issues) {
    let Value::Number(span) = tree.value(node) else {
        return;
    };
    let raw = tree.raw(span);
    let fits = match shape {
        Shape::Int(min, max) => raw.parse::<i64>().is_ok_and(|n| (*min..=*max).contains(&n)),
        Shape::One => raw == "1",
        Shape::Seconds => raw.parse::<f64>().is_ok_and(|n| n > 0.0 && n.is_finite()),
        Shape::Str(_)
        | Shape::Bool
        | Shape::False
        | Shape::Obj(_)
        | Shape::Map { .. }
        | Shape::List(_)
        | Shape::Either(_)
        | Shape::Tagged(_)
        | Shape::Named(..) => true,
    };
    if !fits {
        let code = if matches!(shape, Shape::One) {
            "config.version"
        } else {
            "config.schema.range"
        };
        issues.error(
            code,
            at,
            format!("`{}` is {raw}; it is {}", show(at), describe(shape)),
        );
    }
}

fn text(tree: &Tree<'_>, node: NodeId, kind: Str, at: &At, issues: &mut Issues) {
    let s = tree
        .as_str(node)
        .map(|s| s.into_owned())
        .unwrap_or_default();
    if let Some((code, message)) = strings::problem(kind, &s) {
        issues.error(code, at, format!("`{}`: {message}", show(at)));
    }
}
