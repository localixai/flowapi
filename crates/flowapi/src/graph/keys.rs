//! The names an object may hold: `propertyNames`, read on its own (`local.rs`), and
//! `maxProperties: 0`. No name at all (`false`, a `type` without `string`) is an object with no
//! property; a list of names (`enum`, `const`) is a closed object with those properties, each
//! optional unless required, its value what `properties`, `patternProperties` or
//! `additionalProperties` say of it; a literal pattern (`affix.rs`) is a pattern property that
//! takes what any other property may, no other name allowed. A required name the rule leaves
//! out: no object passes. Any other rule is unchecked (lossy).

use super::build::Builder;
use super::limits::number;
use super::own::Own;
use super::{Affix, Kind, Kinds, Object, Pattern, Prop};
use crate::id::{NodeId, TypeId};
use crate::schema::{Keyword, Resource};
use std::borrow::Cow;

/// The names a rule allows: `None` any, `Ok` those listed, `Err` those a pattern matches.
type Allowed<'t> = Option<Result<Vec<(Cow<'t, str>, Cow<'t, str>)>, (Affix, Cow<'t, str>)>>;

impl<'t> Builder<'_, 't> {
    /// The object narrowed to the names it may hold; `None` when no object passes.
    pub(super) fn keys(
        &mut self,
        at: &Resource,
        o: Object<'t>,
        own: &Own,
        lossy: &mut Vec<Keyword>,
    ) -> Option<Object<'t>> {
        let mut allowed = match own.get(Keyword::PropertyNames) {
            Some(n) => self.allowed(at, n, lossy),
            None => None,
        };
        let tree = &self.docs.get(at.doc)?.tree;
        if let Some((m, _)) = own
            .get(Keyword::MaxProperties)
            .and_then(|n| number(tree, n))
        {
            if m < 1.0 {
                allowed = Some(Ok(Vec::new()));
            } else {
                lossy.push(Keyword::MaxProperties);
            }
        }
        let Some(allowed) = allowed else {
            return Some(o);
        };
        let passes = |name: &str| match &allowed {
            Ok(names) => names.iter().any(|(_, n)| n == name),
            Err((affix, _)) => affix.matches(name),
        };
        if o.props.iter().any(|p| p.required && !passes(&p.name)) {
            return None;
        }
        let never = TypeId::new(1).unwrap_or(self.any);
        let (rest, extra) = (o.rest, o.extra);
        let mut props: Vec<Prop<'t>> = o.props.into_iter().filter(|p| passes(&p.name)).collect();
        let mut patterns = o.patterns;
        match allowed {
            Ok(names) => {
                for (raw, name) in names {
                    if props.iter().all(|p| p.name != name) {
                        let ty = self.value_of(&patterns, rest, extra, &name);
                        let required = false;
                        props.push(Prop {
                            raw,
                            name,
                            ty,
                            required,
                        });
                    }
                }
                Some(Object {
                    props,
                    patterns: Vec::new(),
                    rest: never,
                    extra: never,
                })
            }
            Err((affix, regex)) => {
                // A name it allows takes `additionalProperties` unless a pattern matches it too.
                let ty = if patterns.is_empty() { rest } else { extra };
                if patterns.iter().all(|p| p.affix.as_ref() != Some(&affix)) {
                    patterns.push(Pattern {
                        regex,
                        affix: Some(affix),
                        ty,
                    });
                }
                Some(Object {
                    props,
                    patterns,
                    rest: never,
                    extra,
                })
            }
        }
    }

    /// What `propertyNames` at `node` allows; `None` for any name.
    fn allowed(&mut self, at: &Resource, node: NodeId, lossy: &mut Vec<Keyword>) -> Allowed<'t> {
        let Some(l) = self.local(at, node) else {
            lossy.push(Keyword::PropertyNames);
            return None;
        };
        let Some(s) = &l.values.string else {
            return Some(Ok(Vec::new()));
        };
        if let Some(lits) = &s.lits {
            if !l.exact {
                lossy.push(Keyword::PropertyNames);
            }
            let names = lits.iter().map(|x| (Cow::Borrowed(x.raw), x.norm.clone()));
            return Some(Ok(names.collect()));
        }
        if l.lower().has(Kinds::STRING) {
            return None;
        }
        // Only `pattern` and the lengths narrow: the pattern's names when it is literal (the
        // lengths then unstated).
        let tree = &self.docs.get(at.doc)?.tree;
        let pattern = tree.get(node, "pattern").and_then(|n| tree.as_str(n));
        let affix = pattern.as_deref().and_then(Affix::of);
        let lengths = ["minLength", "maxLength"]
            .iter()
            .any(|k| tree.get(node, k).is_some());
        if affix.is_none() || lengths {
            lossy.push(Keyword::PropertyNames);
        }
        Some(Err((affix?, pattern?)))
    }

    /// The type of the value of a property `name` that `properties` does not name: the patterns
    /// it matches, or `rest`; `extra` when a pattern is not a literal one.
    fn value_of(
        &mut self,
        patterns: &[Pattern<'t>],
        rest: TypeId,
        extra: TypeId,
        name: &str,
    ) -> TypeId {
        let matches = |p: &&Pattern<'t>| p.affix.as_ref().is_some_and(|a| a.matches(name));
        let matching: Vec<TypeId> = patterns.iter().filter(matches).map(|p| p.ty).collect();
        match matching.as_slice() {
            _ if patterns.iter().any(|p| p.affix.is_none()) => extra,
            [] => rest,
            [one] => *one,
            _ => self.make(Kind::Intersection(matching)),
        }
    }
}
