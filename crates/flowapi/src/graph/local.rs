//! A schema read on its own, when it says nothing but what one value's kind and scalars are:
//! `type`, `enum`, `const`, `required` and the bounding keywords (`limits.rs`), beside
//! annotations. `not`, `oneOf`, `if`, `contains` and `propertyNames` read their schemas so, and
//! state what a type can of them; any other schema is `None` to them.

use super::build::Builder;
use super::dependent::names;
use super::limits;
use super::limits::on_numbers;
use super::own::scalars;
use super::{Branches, Kinds};
use crate::id::NodeId;
use crate::json::Value;
use crate::schema::{Keyword, Resource};
use std::borrow::Cow;

/// What such a schema accepts.
pub(super) struct Local<'t> {
    /// Every value it may accept, `required` aside.
    pub(super) values: Branches<'t>,
    /// The kinds it accepts every value of, `required` aside.
    pub(super) whole: Kinds,
    /// Whether `values` is exactly what it accepts, `required` aside.
    pub(super) exact: bool,
    /// The names an object must hold (`required`).
    pub(super) required: Vec<(Cow<'t, str>, Cow<'t, str>)>,
}

impl Local<'_> {
    /// The kinds it accepts every value of.
    pub(super) fn lower(&self) -> Kinds {
        if self.required.is_empty() {
            self.whole
        } else {
            self.whole.without(Kinds::OBJECT)
        }
    }

    /// Whether it is a test of kinds alone: it accepts every value of some kinds and none else.
    pub(super) fn kinds_only(&self) -> bool {
        self.exact && self.required.is_empty() && self.values.kinds() == self.whole
    }
}

impl<'t> Builder<'_, 't> {
    /// The schema at `node` read on its own; `None` when it holds any other keyword.
    pub(super) fn local(&mut self, at: &Resource, node: NodeId) -> Option<Local<'t>> {
        let docs = self.docs;
        let tree = &docs.get(at.doc)?.tree;
        if !matches!(tree.value(node), Value::Object(_) | Value::Bool(_)) {
            return None;
        }
        let schema = at.at(node);
        let (read, own) = self.read(&schema);
        let plain = read.reference.is_none()
            && read.parts.is_empty()
            && read.test.is_none()
            && read.lossy.is_empty()
            && read.exclude.is_empty();
        let scalar = |k: Keyword| {
            limits::bounds(k)
                || matches!(
                    k,
                    Keyword::Type
                        | Keyword::Enum
                        | Keyword::Const
                        | Keyword::Required
                        | Keyword::Nullable
                        | Keyword::Format
                )
        };
        if !plain || !own.only(scalar) {
            return None;
        }
        let (bad, mut b) = self.kinds(tree, &own);
        if bad.is_some() {
            return None;
        }
        if tree.as_bool(node) == Some(false) {
            b = Branches::of(Kinds::NONE, self.any);
        }
        let lossy = scalars(&mut b, tree, at, &own);
        // An array or object among `enum` values keeps its kind, but not every value of it.
        let listed = own.get(Keyword::Enum).is_some() || own.get(Keyword::Const).is_some();
        let compound = listed && (b.array.is_some() || b.object.is_some());
        let mut whole = whole(&b);
        if compound {
            whole = whole.without(Kinds::ARRAY.union(Kinds::OBJECT));
        }
        for &k in &lossy {
            whole = whole.without(if on_numbers(k) {
                Kinds::NUMBER
            } else {
                Kinds::STRING
            });
        }
        let required = match own.get(Keyword::Required) {
            Some(n) => names(tree, n)?,
            None => Vec::new(),
        };
        Some(Local {
            values: b,
            whole,
            exact: lossy.is_empty() && !compound,
            required,
        })
    }
}

/// The kinds a shape allows every value of.
fn whole(b: &Branches<'_>) -> Kinds {
    let number = b
        .number
        .as_ref()
        .is_some_and(|n| !n.integer && n.lits.is_none());
    let string = b.string.as_ref().is_some_and(|s| s.lits.is_none());
    [
        (b.null, Kinds::NULL),
        (
            b.boolean.allow_true && b.boolean.allow_false,
            Kinds::BOOLEAN,
        ),
        (number, Kinds::NUMBER),
        (string, Kinds::STRING),
        (b.array.is_some(), Kinds::ARRAY),
        (b.object.is_some(), Kinds::OBJECT),
    ]
    .into_iter()
    .filter(|x| x.0)
    .fold(Kinds::NONE, |k, x| k.union(x.1))
}
