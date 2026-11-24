//! The keywords that bound one kind's values (`minimum`, `maximum`, their exclusive forms and
//! `multipleOf` on numbers; `minLength`, `maxLength` and `pattern` on strings) put to the values
//! a type lists: an `enum` or `const` value they rule out is dropped. Each value is dropped only
//! when the bound rules it out for certain (a float comparison is exact only one way; a
//! `multipleOf` is decided on integers; a `pattern` only when it is a literal text, `affix.rs`).
//! `maxLength: 0` is the empty string. A keyword the type cannot state for every value its kind
//! allows is returned, to be recorded as lossy; one whose kind it lists exactly, or allows none
//! of, is stated.

use super::own::Own;
use super::{Affix, Branches, Lit};
use crate::id::NodeId;
use crate::json::{Tree, Value};
use crate::schema::Keyword;
use crate::source::Dialect;
use std::borrow::Cow;

/// What one bound says of one value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Says {
    Keep,
    Drop,
    /// Not decided: the value is kept and the keyword recorded as lossy.
    Unsure,
}

const ON_NUMBERS: [Keyword; 5] = [
    Keyword::Minimum,
    Keyword::Maximum,
    Keyword::ExclusiveMinimum,
    Keyword::ExclusiveMaximum,
    Keyword::MultipleOf,
];
const ON_STRINGS: [Keyword; 3] = [Keyword::MinLength, Keyword::MaxLength, Keyword::Pattern];

/// Whether the keyword bounds numbers.
pub(super) fn on_numbers(k: Keyword) -> bool {
    ON_NUMBERS.contains(&k)
}

/// Whether the keyword bounds numbers or strings.
pub(super) fn bounds(k: Keyword) -> bool {
    on_numbers(k) || ON_STRINGS.contains(&k)
}

/// Narrows the branches by the bounds; returns the keywords the type does not state.
pub(super) fn apply<'t>(
    own: &Own,
    b: &mut Branches<'t>,
    tree: &Tree<'t>,
    dialect: Dialect,
) -> Vec<Keyword> {
    let mut lossy = Vec::new();
    numbers(own, b, tree, dialect, &mut lossy);
    strings(own, b, tree, &mut lossy);
    lossy
}

fn numbers<'t>(
    own: &Own,
    b: &mut Branches<'t>,
    tree: &Tree<'t>,
    dialect: Dialect,
    lossy: &mut Vec<Keyword>,
) {
    let Some(n) = b.number.as_mut() else { return };
    let present = ON_NUMBERS.into_iter().filter(|&k| own.get(k).is_some());
    let Some(lits) = n.lits.as_mut() else {
        return lossy.extend(present);
    };
    // Draft 4, OpenAPI 3.0 and Swagger 2.0: `exclusiveMinimum: true` makes `minimum` exclusive;
    // an exclusive keyword of the other form is not read (lossy).
    let flags = matches!(
        dialect,
        Dialect::Draft4 | Dialect::Oas30 | Dialect::Swagger20
    );
    let read = |k: Keyword| own.get(k).and_then(|n| number(tree, n));
    let flag = |k: Keyword| own.get(k).and_then(|n| tree.as_bool(n));
    let (lo, hi) = (Keyword::ExclusiveMinimum, Keyword::ExclusiveMaximum);
    let mut unsure: Vec<Keyword> = [lo, hi]
        .into_iter()
        .filter(|&k| own.get(k).is_some() && flag(k).is_some() != flags)
        .collect();
    lits.retain(|x| {
        let v = (x.raw.parse::<f64>().unwrap_or(f64::NAN), x.raw);
        let says = [
            (
                Keyword::Minimum,
                read(Keyword::Minimum).map(|m| below(m, v, flags && flag(lo) == Some(true))),
            ),
            (
                Keyword::Maximum,
                read(Keyword::Maximum).map(|m| below(v, m, flags && flag(hi) == Some(true))),
            ),
            (lo, read(lo).filter(|_| !flags).map(|m| below(m, v, true))),
            (hi, read(hi).filter(|_| !flags).map(|m| below(v, m, true))),
            (
                Keyword::MultipleOf,
                read(Keyword::MultipleOf).map(|m| multiple_of(x.raw, m.1)),
            ),
        ];
        let mut keep = true;
        for (k, s) in says.into_iter().filter_map(|(k, s)| Some((k, s?))) {
            keep &= s != Says::Drop;
            if s == Says::Unsure && !unsure.contains(&k) {
                unsure.push(k);
            }
        }
        keep
    });
    lossy.extend(unsure);
    if lits.is_empty() {
        b.number = None;
    }
}

fn strings<'t>(own: &Own, b: &mut Branches<'t>, tree: &Tree<'t>, lossy: &mut Vec<Keyword>) {
    let Some(s) = b.string.as_mut() else { return };
    let present = ON_STRINGS.into_iter().filter(|&k| own.get(k).is_some());
    let read = |k: Keyword| own.get(k).and_then(|n| number(tree, n)).map(|x| x.0);
    let (min, max) = (read(Keyword::MinLength), read(Keyword::MaxLength));
    if s.lits.is_none() {
        if !max.is_some_and(|m| m < 1.0) {
            return lossy.extend(present);
        }
        // At most no character: the empty string.
        let empty = Lit {
            raw: "\"\"",
            norm: Cow::Borrowed(""),
        };
        s.lits = Some(vec![empty]);
    }
    let Some(lits) = s.lits.as_mut() else { return };
    let pattern = own.get(Keyword::Pattern).and_then(|n| tree.as_str(n));
    let affix = pattern.as_deref().and_then(Affix::of);
    let mut undecoded = false;
    lits.retain(|x| {
        // A string not decoded (`string_lit` fell back to the token): nothing is decided.
        undecoded |= x.norm == x.raw;
        let len = x.norm.chars().count() as f64;
        let fits = min.is_none_or(|m| len >= m) && max.is_none_or(|m| len <= m);
        x.norm == x.raw || fits && affix.as_ref().is_none_or(|a| a.matches(&x.norm))
    });
    if undecoded {
        lossy.extend(present);
    } else if pattern.is_some() && affix.is_none() {
        lossy.push(Keyword::Pattern);
    }
    if lits.is_empty() {
        b.string = None;
    }
}

/// Whether `a` (a lower bound or a value) is at most `b` (a value or an upper bound), strictly
/// when `open`: `Drop` when it is certainly not. `f64` rounding keeps order, so a strict
/// comparison is exact; two equal floats are one value only when their tokens say so.
fn below(a: (f64, &str), b: (f64, &str), open: bool) -> Says {
    if a.0 < b.0 {
        Says::Keep
    } else if a.0 > b.0 {
        Says::Drop
    } else if !(a.1 == b.1 || exact_integer(a.1).is_some() && exact_integer(b.1).is_some()) {
        Says::Unsure
    } else if open {
        Says::Drop
    } else {
        Says::Keep
    }
}

/// A token that is an integer a float holds exactly, as that integer.
fn exact_integer(raw: &str) -> Option<i64> {
    let digits = raw.strip_prefix('-').unwrap_or(raw);
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    raw.parse::<i64>()
        .ok()
        .filter(|v| v.unsigned_abs() < 1 << 53)
}

fn multiple_of(value: &str, by: &str) -> Says {
    match (exact_integer(value), exact_integer(by)) {
        (Some(0), _) => Says::Keep,
        (Some(v), Some(m)) if m != 0 && v % m == 0 => Says::Keep,
        (Some(_), Some(m)) if m != 0 => Says::Drop,
        (Some(_) | None, Some(_) | None) => Says::Unsure,
    }
}

/// A JSON number and its token.
pub(super) fn number<'t>(tree: &Tree<'t>, node: NodeId) -> Option<(f64, &'t str)> {
    match tree.value(node) {
        Value::Number(span) => {
            let raw = tree.raw(span);
            raw.parse::<f64>().ok().map(|v| (v, raw))
        }
        Value::Null | Value::Bool(_) | Value::String(_) | Value::Array(_) | Value::Object(_) => {
            None
        }
    }
}
