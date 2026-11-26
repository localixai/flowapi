//! An `enum` or a `const` narrowing a schema's own branches: each kind kept only if a value is of
//! it, each scalar kind to its values (equal ones once: `1` and `1.0` are one value).

use super::{Branches, Lit, Numbers, Strings};
use crate::id::NodeId;
use crate::json::{Span, Tree, Value};
use std::borrow::Cow;

/// Narrows the branches to the kinds and scalars of an `enum` or `const`; arrays and objects
/// among its values are narrowed by their exact types (`literal.rs`).
pub(super) fn restrict<'t>(b: &mut Branches<'t>, tree: &Tree<'t>, values: &[NodeId]) {
    let (mut null, mut yes, mut no, mut array, mut object) = (false, false, false, false, false);
    let (mut numbers, mut strings) = (Vec::new(), Vec::new());
    for &n in values {
        match tree.value(n) {
            Value::Null => null = true,
            Value::Bool(v) => {
                if v {
                    yes = true
                } else {
                    no = true
                }
            }
            Value::Number(span) => numbers.push(number_lit(tree.raw(span))),
            Value::String(span) => strings.push(string_lit(tree, span)),
            Value::Array(_) => array = true,
            Value::Object(_) => object = true,
        }
    }
    b.null &= null;
    b.boolean.allow_true &= yes;
    b.boolean.allow_false &= no;
    b.number = b.number.take().and_then(|n| narrow_numbers(&n, numbers));
    b.string = b.string.take().and_then(|s| narrow_strings(s, strings));
    if !array {
        b.array = None;
    }
    if !object {
        b.object = None;
    }
}

fn narrow_numbers<'t>(n: &Numbers<'t>, values: Vec<Lit<'t>>) -> Option<Numbers<'t>> {
    let values = values
        .into_iter()
        .filter(|l| !n.integer || l.is_integer())
        .collect();
    let mut lits = Lit::common(Some(values), n.lits.as_deref()).unwrap_or_default();
    dedupe(&mut lits);
    (!lits.is_empty()).then_some(Numbers {
        integer: n.integer,
        lits: Some(lits),
    })
}

fn narrow_strings<'t>(s: Strings<'t>, values: Vec<Lit<'t>>) -> Option<Strings<'t>> {
    let mut lits = Lit::common(Some(values), s.lits.as_deref()).unwrap_or_default();
    dedupe(&mut lits);
    (!lits.is_empty()).then_some(Strings {
        format: s.format,
        lits: Some(lits),
    })
}

fn dedupe(lits: &mut Vec<Lit<'_>>) {
    let mut seen: Vec<Cow<'_, str>> = Vec::new();
    lits.retain(|l| {
        let fresh = !seen.contains(&l.norm);
        if fresh {
            seen.push(l.norm.clone());
        }
        fresh
    });
}

pub(super) fn number_lit(raw: &str) -> Lit<'_> {
    let norm = raw.parse::<f64>().map_or_else(
        |_| raw.to_owned(),
        |v| {
            if v == 0.0 {
                String::from("0")
            } else {
                v.to_string()
            }
        },
    );
    Lit {
        raw,
        norm: Cow::Owned(norm),
    }
}

pub(super) fn string_lit<'t>(tree: &Tree<'t>, span: Span) -> Lit<'t> {
    let raw = tree.raw(span);
    Lit {
        raw,
        norm: tree.string(span).unwrap_or(Cow::Borrowed(raw)),
    }
}
