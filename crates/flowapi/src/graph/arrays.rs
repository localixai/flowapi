//! `minItems` and `maxItems` as tuples where they are small: at least one item is
//! `[T, ...T[]]`, at most two is `[T?, T?]`. Past [`SMALL`] items a bound narrows nothing the type
//! shows (and the type stays wider, which is sound).

use super::Array;
use crate::id::TypeId;

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
