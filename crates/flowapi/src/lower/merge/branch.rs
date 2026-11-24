//! Two shapes merged into the one that allows the values both allow. Where two property types
//! meet, their intersection is made (once per pair); the merge never narrows past what both
//! say, and a kind only one side allows is gone.

use super::Merger;
use crate::graph::{Array, Bools, Branches, Kinds, Lit, Numbers, Object, Prop, Strings};
use crate::id::TypeId;

impl<'t> Merger<'_, 't> {
    /// The values both shapes allow.
    pub(super) fn meet(&mut self, a: Branches<'t>, b: &Branches<'t>) -> Branches<'t> {
        let number = match (a.number, &b.number) {
            (Some(x), Some(y)) => numbers(x, y),
            (Some(_) | None, None) | (None, Some(_)) => None,
        };
        let string = match (a.string, &b.string) {
            (Some(x), Some(y)) => strings(x, y),
            (Some(_) | None, None) | (None, Some(_)) => None,
        };
        let array = match (a.array, &b.array) {
            (Some(x), Some(y)) => Some(self.arrays(&x, y)),
            (Some(_) | None, None) | (None, Some(_)) => None,
        };
        let object = match (a.object, &b.object) {
            (Some(x), Some(y)) => self.objects(x, y),
            (Some(_) | None, None) | (None, Some(_)) => None,
        };
        Branches {
            null: a.null && b.null,
            boolean: Bools {
                allow_true: a.boolean.allow_true && b.boolean.allow_true,
                allow_false: a.boolean.allow_false && b.boolean.allow_false,
            },
            number,
            string,
            array,
            object,
        }
    }

    fn arrays(&mut self, a: &Array, b: &Array) -> Array {
        let len = a.prefix.len().max(b.prefix.len());
        let at = |arr: &Array, i: usize| arr.prefix.get(i).copied().unwrap_or(arr.rest);
        let prefix = (0..len).map(|i| self.both(at(a, i), at(b, i))).collect();
        Array {
            prefix,
            rest: self.both(a.rest, b.rest),
            min: a.min.max(b.min),
        }
    }

    /// `None` when no object passes both: a property one side requires and the other forbids.
    fn objects(&mut self, a: Object<'t>, b: &Object<'t>) -> Option<Object<'t>> {
        let never = TypeId::new(1).unwrap_or(TypeId::FIRST);
        let mut props: Vec<Prop<'t>> = Vec::with_capacity(a.props.len() + b.props.len());
        for p in a.props {
            let ty = match b.props.iter().find(|q| q.name == p.name) {
                Some(q) => self.both(p.ty, q.ty),
                None => self.both(p.ty, b.extra),
            };
            let required = p.required || b.props.iter().any(|q| q.name == p.name && q.required);
            props.push(Prop { ty, required, ..p });
        }
        for q in &b.props {
            if props.iter().all(|p| p.name != q.name) {
                let ty = self.both(q.ty, a.extra);
                props.push(Prop { ty, ..q.clone() });
            }
        }
        if props
            .iter()
            .any(|p| p.required && (p.ty == never || self.kind_of(p.ty).is_empty()))
        {
            return None;
        }
        let extra = self.both(a.extra, b.extra);
        // Two objects' patterns are not merged: the merge keeps `extra`, which allows them.
        Some(Object {
            props,
            patterns: Vec::new(),
            rest: extra,
            extra,
        })
    }
}

/// Whether a shape says more than "any value of these kinds".
pub(super) fn narrows(b: &Branches<'_>, allowed: Kinds) -> bool {
    let any = TypeId::FIRST;
    let plain = Branches::of(allowed, any);
    let wide_number = |n: &Option<Numbers<'_>>| n.as_ref().map(|n| !n.integer && n.lits.is_none());
    let wide_string = |s: &Option<Strings<'_>>| s.as_ref().map(|s| s.lits.is_none());
    let wide_array = |a: &Option<Array>| a.as_ref().map(|a| a.prefix.is_empty() && a.rest == any);
    let wide_object =
        |o: &Option<Object<'_>>| o.as_ref().map(|o| o.props.is_empty() && o.extra == any);
    !(b.null == plain.null
        && b.boolean == plain.boolean
        && wide_number(&b.number) == plain.number.as_ref().map(|_| true)
        && wide_string(&b.string) == plain.string.as_ref().map(|_| true)
        && wide_array(&b.array) == plain.array.as_ref().map(|_| true)
        && wide_object(&b.object) == plain.object.as_ref().map(|_| true))
}

fn numbers<'t>(a: Numbers<'t>, b: &Numbers<'t>) -> Option<Numbers<'t>> {
    let integer = a.integer || b.integer;
    let lits = match (a.lits, &b.lits) {
        (Some(x), Some(y)) => Some(common(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y.clone()),
        (None, None) => None,
    };
    let lits = lits.map(|l| {
        if integer {
            l.into_iter().filter(|v| is_integer(v.raw)).collect()
        } else {
            l
        }
    });
    match &lits {
        Some(l) if l.is_empty() => None,
        Some(_) | None => Some(Numbers { integer, lits }),
    }
}

fn strings<'t>(a: Strings<'t>, b: &Strings<'t>) -> Option<Strings<'t>> {
    let lits = match (a.lits, &b.lits) {
        (Some(x), Some(y)) => Some(common(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y.clone()),
        (None, None) => None,
    };
    match &lits {
        Some(l) if l.is_empty() => None,
        Some(_) | None => Some(Strings {
            format: a.format.or_else(|| b.format.clone()),
            lits,
        }),
    }
}

fn common<'t>(a: Vec<Lit<'t>>, b: &[Lit<'t>]) -> Vec<Lit<'t>> {
    a.into_iter()
        .filter(|x| b.iter().any(|y| y.norm == x.norm))
        .collect()
}

fn is_integer(raw: &str) -> bool {
    raw.parse::<f64>()
        .is_ok_and(|v| v.is_finite() && v.fract() == 0.0)
}
