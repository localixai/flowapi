//! A parameter's value as the README shows it, for a person to read: what the description writes
//! for the parameter (its Parameter Object's `example`, else the value of the first of its
//! `examples`, else Swagger 2.0's `x-example`), else what its schema writes (`example::own`: its
//! `example`, `default`, `const`, the first of `examples` or `enum`), else a placeholder of its
//! name and type: the name for a string with no `format` or `pattern` (`"pet_id"`, fitted to its
//! lengths), `true` for a boolean, the made value for the rest (a number within its bounds, a
//! value of its `format`, which a name would break: pitfall 20 of the study). The generated tests
//! and the mock keep `example::of`'s values, which the mock is held to.

use super::example::{self, Value};
use crate::doc::{At, Doc};
use flowapi::spec::{Location, Operation, Param};

#[cfg(test)]
#[path = "sample_tests.rs"]
mod tests;

/// The value the README gives `p`, a parameter of `op`.
pub(crate) fn param(doc: &Doc<'_, '_>, op: &Operation, p: &Param) -> Value {
    if let Some(v) = object(doc, op, p).and_then(|at| written(doc, at)) {
        return v;
    }
    let schema = p.schema.as_ref().and_then(|r| doc.resolve((r.doc, r.node)));
    if let Some(v) = schema.and_then(|at| example::own(doc, at)) {
        return v;
    }
    match (example::of(doc, schema, false), schema) {
        (Value::Str(_), Some(at))
            if doc.get(at, "format").is_none() && doc.get(at, "pattern").is_none() =>
        {
            Value::Str(example::fitted(doc, at, &p.name))
        }
        (Value::Bool(_), _) => Value::Bool(true),
        (v, _) => v,
    }
}

/// The parameter's own example: `example`, the first of `examples` (an Example Object's 3.2
/// `dataValue`, else its `value`), 2.0's `x-example`.
fn written(doc: &Doc<'_, '_>, at: At) -> Option<Value> {
    if let Some(v) = doc.get(at, "example").or_else(|| doc.get(at, "x-example")) {
        return Some(example::written(doc, v, 0));
    }
    let (_, first) = doc.members(doc.get(at, "examples")?).into_iter().next()?;
    let first = doc.resolve(first)?;
    let v = doc
        .get(first, "dataValue")
        .or_else(|| doc.get(first, "value"))?;
    Some(example::written(doc, v, 0))
}

/// The Parameter Object of `p`: the operation's own first, then its Path Item's, references
/// followed; a header's name in any case.
fn object(doc: &Doc<'_, '_>, op: &Operation, p: &Param) -> Option<At> {
    let items = doc.path_items(op);
    let own = doc.operation(&items, op);
    own.into_iter()
        .chain(items.iter().copied())
        .filter_map(|at| doc.get(at, "parameters"))
        .flat_map(|list| doc.items(list))
        .filter_map(|at| doc.resolve(at))
        .find(|&at| {
            let name = doc.str(at, "name").unwrap_or_default();
            let named = if p.location == Location::Header {
                name.eq_ignore_ascii_case(&p.name)
            } else {
                name == p.name
            };
            named && doc.str(at, "in").as_deref() == Some(p.location.name())
        })
}
