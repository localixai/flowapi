//! `oneOf`: a value passes exactly one branch. The type is the branches' union, which is exact
//! when no value can pass two of them: branches of different kinds, or of disjoint literals
//! (`local.rs` reads each branch). Otherwise the union is wider: a value passing two branches
//! passes it. What can be stated is by kinds: a kind two branches accept every value of has no
//! value that passes, and where every branch tests kinds alone, the kinds exactly one accepts
//! are the type. A branch that is a subset of another drops nothing (its values pass the union).

use super::build::Builder;
use super::local::Local;
use super::{Branches, Kind, Kinds, Lit};
use crate::id::{NodeId, TypeId};
use crate::schema::{Keyword, Resource};

impl Builder<'_, '_> {
    /// The type of a `oneOf` of the schemas at `nodes`, whose union is `union`.
    pub(super) fn one_of(
        &mut self,
        at: &Resource,
        nodes: &[NodeId],
        union: TypeId,
        lossy: &mut Vec<Keyword>,
    ) -> TypeId {
        let tests: Vec<Option<Local<'_>>> = nodes.iter().map(|&n| self.local(at, n)).collect();
        let each = |i: usize, a: &Option<Local<'_>>| {
            let mut later = tests.iter().skip(i + 1);
            later.all(|b| a.as_ref().zip(b.as_ref()).is_some_and(|(a, b)| apart(a, b)))
        };
        if tests.iter().enumerate().all(|(i, a)| each(i, a)) {
            return union;
        }
        let mut allowed = Kinds::NONE;
        for kind in Kinds::ALL.each() {
            let all = tests.iter().flatten().filter(|t| t.lower().has(kind));
            let some = tests
                .iter()
                .filter(|t| t.as_ref().is_none_or(|t| t.values.kinds().has(kind)));
            if some.count() >= 1 && all.count() <= 1 {
                allowed = allowed.union(kind);
            }
        }
        let kinds = self.make(Kind::Value(Branches::of(allowed, self.any)));
        let exact = tests.iter().flatten().count() == tests.len();
        if exact && tests.iter().flatten().all(Local::kinds_only) {
            return kinds;
        }
        lossy.push(Keyword::OneOf);
        if allowed == Kinds::ALL {
            union
        } else {
            self.make(Kind::Intersection(vec![union, kinds]))
        }
    }
}

/// Whether no value passes both, as far as kinds and literals show.
fn apart(a: &Local<'_>, b: &Local<'_>) -> bool {
    let (a, b) = (&a.values, &b.values);
    let lits = |x: Option<&Vec<Lit<'_>>>, y: Option<&Vec<Lit<'_>>>| {
        x.zip(y)
            .is_some_and(|(x, y)| x.iter().all(|l| y.iter().all(|m| m.norm != l.norm)))
    };
    let both = a.kinds().meet(b.kinds());
    let (yes, no) = (&a.boolean, &b.boolean);
    both.meet(Kinds::NULL.union(Kinds::ARRAY).union(Kinds::OBJECT))
        .is_empty()
        && !(both.has(Kinds::BOOLEAN)
            && (yes.allow_true && no.allow_true || yes.allow_false && no.allow_false))
        && (!both.has(Kinds::NUMBER) || lits(numbers(a), numbers(b)))
        && (!both.has(Kinds::STRING) || lits(strings(a), strings(b)))
}

fn numbers<'a, 't>(b: &'a Branches<'t>) -> Option<&'a Vec<Lit<'t>>> {
    b.number.as_ref()?.lits.as_ref()
}

fn strings<'a, 't>(b: &'a Branches<'t>) -> Option<&'a Vec<Lit<'t>>> {
    b.string.as_ref()?.lits.as_ref()
}
