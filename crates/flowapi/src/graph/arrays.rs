//! A schema's own array keywords as the array branch, and `minItems` and `maxItems` as tuples
//! where they are small: at least one item is `[T, ...T[]]`, at most two is `[T?, T?]`. Past
//! [`SMALL`] items a bound narrows nothing the type shows (and the type stays wider, which is
//! sound). `contains` bounds the length: at least `minContains` items pass it, so there are at
//! least that many; when every value passes it, `maxContains` bounds the length too.

use super::build::Builder;
use super::limits::number;
use super::own::Own;
use super::{Array, Branches, Kind, Kinds};
use crate::id::TypeId;
use crate::json::{Tree, Value};
use crate::schema::{Keyword, Resource};

/// The largest bound written out as positions.
pub(super) const SMALL: usize = 8;

/// The arrays of `prefix` then `rest` with at least `min` and at most `max` items; `None` when no
/// array has both.
pub(super) fn bounded(
    prefix: Vec<TypeId>,
    rest: TypeId,
    min: Option<f64>,
    max: Option<f64>,
    never: TypeId,
) -> Option<Array> {
    let mut prefix = prefix;
    let mut rest = rest;
    let count = |bound: f64| (1..=SMALL).take_while(|&i| i as f64 <= bound).count();
    // `minItems`: the first items are there; past the prefix they are the rest's.
    let min = min.filter(|m| m.is_finite() && *m >= 0.0).map_or(0, |m| {
        let at_least = count(m.ceil());
        if rest == never && at_least > prefix.len() && m > prefix.len() as f64 {
            return usize::MAX;
        }
        while prefix.len() < at_least && rest != never {
            prefix.push(rest);
        }
        at_least.min(prefix.len())
    });
    if min == usize::MAX {
        return None;
    }
    // `maxItems`: no item past the bound.
    if let Some(m) = max.filter(|m| m.is_finite() && *m >= 0.0) {
        let at_most = count(m.floor());
        if m < SMALL as f64 + 1.0 {
            if at_most < min {
                return None;
            }
            prefix.truncate(at_most);
            while prefix.len() < at_most && rest != never {
                prefix.push(rest);
            }
            rest = never;
        }
    }
    Some(Array { prefix, rest, min })
}

impl<'t> Builder<'_, 't> {
    /// The array branch of a schema's own keywords: `prefixItems` and `items` (or `items` and
    /// `additionalItems` before 2020-12), their bounds, and what `contains` says of the length.
    pub(super) fn array(
        &mut self,
        tree: &Tree<'t>,
        at: &Resource,
        own: &Own,
        lossy: &mut Vec<Keyword>,
    ) -> Option<Array> {
        let tuple_items = at.dialect.tuple_items()
            && own
                .get(Keyword::Items)
                .is_some_and(|n| matches!(tree.value(n), Value::Array(_)));
        let (prefix_node, rest_node) = if tuple_items {
            (own.get(Keyword::Items), own.get(Keyword::AdditionalItems))
        } else {
            (own.get(Keyword::PrefixItems), own.get(Keyword::Items))
        };
        let prefix: Vec<TypeId> = prefix_node
            .map_or(&[][..], |n| tree.items(n))
            .to_vec()
            .into_iter()
            .map(|n| self.sub(at, n))
            .collect();
        let rest = rest_node.map_or(self.any, |n| self.sub(at, n));
        let number = |k: Keyword| own.get(k).and_then(|n| number(tree, n)).map(|x| x.0);
        let never = TypeId::new(1).unwrap_or(self.any);
        let (mut prefix, mut rest) = (prefix, rest);
        // `unevaluatedItems: false`: no item past those some schema applied here could evaluate.
        if own.get(Keyword::UnevaluatedItems).is_some()
            && rest_node.is_none()
            && let Some(found) = self.evaluated(at, false)
        {
            prefix.resize(prefix.len().max(found.items), self.any);
            rest = never;
        }
        let (mut min, mut max) = (number(Keyword::MinItems), number(Keyword::MaxItems));
        if let Some(c) = own.get(Keyword::Contains) {
            // At least `minContains` items (1 unless given) pass `contains`, at most `maxContains`.
            let (lo, hi) = (
                number(Keyword::MinContains).unwrap_or(1.0),
                number(Keyword::MaxContains),
            );
            let test = self.local(at, c);
            let none = test.as_ref().is_some_and(|t| t.values.kinds().is_empty());
            if hi.is_some_and(|h| lo > h) || none && lo > 0.0 {
                return None;
            }
            let at_least = |m: Option<f64>| Some(m.map_or(lo, |m| m.max(lo)));
            match test {
                // Every item passes: the bounds are the length's.
                Some(t) if t.lower() == Kinds::ALL => {
                    min = at_least(min);
                    max = max.map_or(hi, |m| Some(hi.map_or(m, |h| m.min(h))));
                }
                // No item passes, and `minContains: 0` lets any array through.
                Some(_) if none => {}
                // None may pass: every item is of the kinds it does not accept.
                Some(t) if hi == Some(0.0) && t.kinds_only() => {
                    let kinds = Kinds::ALL.without(t.whole);
                    let not = self.make(Kind::Value(Branches::of(kinds, self.any)));
                    for item in prefix.iter_mut().chain(std::iter::once(&mut rest)) {
                        *item = self.make(Kind::Intersection(vec![*item, not]));
                    }
                }
                Some(_) | None => {
                    min = at_least(min);
                    let counts = [
                        Keyword::Contains,
                        Keyword::MinContains,
                        Keyword::MaxContains,
                    ];
                    lossy.extend(counts.into_iter().filter(|&k| own.get(k).is_some()));
                }
            }
        }
        bounded(prefix, rest, min, max, never)
    }
}
