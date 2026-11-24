//! A schema's own object keywords as the object branch: `properties` and `required`, the type of
//! any other property (`additionalProperties`, `patternProperties`), and `unevaluatedProperties:
//! false` closing it where every name it may take is known; then the names `propertyNames` and
//! `maxProperties` allow (`keys.rs`).

use super::build::Builder;
use super::own::Own;
use super::{Affix, Kind, Object, Pattern, Prop};
use crate::id::NodeId;
use crate::id::TypeId;
use crate::json::{Tree, Value};
use crate::schema::{Keyword, Resource};
use crate::source::Dialect;
use std::borrow::Cow;

impl<'t> Builder<'_, 't> {
    /// The object branch; `None` when no object passes (`keys.rs`).
    pub(super) fn object(
        &mut self,
        tree: &Tree<'t>,
        at: &Resource,
        own: &Own,
        lossy: &mut Vec<Keyword>,
    ) -> Option<Object<'t>> {
        let mut props: Vec<Prop<'t>> = Vec::new();
        for m in own
            .get(Keyword::Properties)
            .map_or(&[][..], |n| tree.members(n))
        {
            let raw = tree.raw(m.key);
            let name = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
            let ty = self.sub(at, m.value);
            match props.iter_mut().find(|p| p.name == name) {
                Some(p) => p.ty = ty,
                None => props.push(Prop {
                    raw: Cow::Borrowed(raw),
                    name,
                    ty,
                    required: false,
                }),
            }
        }
        let mut unnamed: Vec<usize> = Vec::new();
        for &n in own
            .get(Keyword::Required)
            .map_or(&[][..], |n| tree.items(n))
        {
            let Value::String(span) = tree.value(n) else {
                continue;
            };
            let raw = tree.raw(span);
            let name = tree.string(span).unwrap_or(Cow::Borrowed(raw));
            // OpenAPI 3.0: a required property that is read-only is required in responses only,
            // and a write-only one in requests only; one type serves both, so neither is. Swagger
            // 2.0 says the same of a read-only one (it MUST NOT be sent in a request).
            let one_way_keys: &[&str] = match at.dialect {
                Dialect::Oas30 => &["readOnly", "writeOnly"],
                Dialect::Swagger20 => &["readOnly"],
                Dialect::Draft4
                | Dialect::Draft6
                | Dialect::Draft7
                | Dialect::Draft2019
                | Dialect::Draft2020
                | Dialect::Draft2020Annotations => &[],
            };
            if one_way(tree, own.get(Keyword::Properties), &name, one_way_keys) {
                continue;
            }
            match props.iter_mut().find(|p| p.name == name) {
                Some(p) => p.required = true,
                None => {
                    // Typed below, as any other property: `properties` does not name it.
                    unnamed.push(props.len());
                    props.push(Prop {
                        raw: Cow::Borrowed(raw),
                        name,
                        ty: self.any,
                        required: true,
                    });
                }
            }
        }
        let rest = own
            .get(Keyword::AdditionalProperties)
            .map_or(self.any, |n| self.sub(at, n));
        let mut extra = rest;
        let patterns = self.patterns(tree, at, own);
        if !patterns.is_empty() && extra != self.any {
            let mut members = vec![extra];
            members.extend(patterns.iter().map(|p| p.ty));
            extra = self.make(Kind::Union(members));
        }
        // A required name `properties` leaves out takes what any other property may hold
        // (`additionalProperties` or a matching pattern): `never` when nothing else is allowed.
        for &i in &unnamed {
            if let Some(prop) = props.get_mut(i) {
                prop.ty = extra;
            }
        }
        // `unevaluatedProperties: false`: no name but those some schema applied here could
        // evaluate (unevaluated.rs); `additionalProperties` evaluates every name, so it wins.
        let mut rest = rest;
        if own.get(Keyword::UnevaluatedProperties).is_some()
            && own.get(Keyword::AdditionalProperties).is_none()
            && own.get(Keyword::PatternProperties).is_none()
            && let Some(found) = self.evaluated(at, true)
        {
            for (raw, name) in found.names {
                if props.iter().all(|p| p.name != name) {
                    props.push(Prop {
                        raw,
                        name,
                        ty: self.any,
                        required: false,
                    });
                }
            }
            extra = TypeId::new(1).unwrap_or(extra);
            rest = extra;
        }
        let object = Object {
            props,
            patterns,
            rest,
            extra,
        };
        self.keys(at, object, own, lossy)
    }

    /// `patternProperties`: each regular expression with the type of a matching value.
    fn patterns(&mut self, tree: &Tree<'t>, at: &Resource, own: &Own) -> Vec<Pattern<'t>> {
        let members = own
            .get(Keyword::PatternProperties)
            .map_or(&[][..], |n| tree.members(n));
        let pattern = |m: &crate::json::Member| {
            let raw = tree.raw(m.key);
            let regex = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
            Pattern {
                affix: Affix::of(&regex),
                regex,
                ty: self.sub(at, m.value),
            }
        };
        members.iter().map(pattern).collect()
    }
}

/// Whether the property `name` is `true` under one of `keys` (`readOnly`, `writeOnly`).
fn one_way(tree: &Tree<'_>, properties: Option<NodeId>, name: &str, keys: &[&str]) -> bool {
    let Some(schema) = properties.and_then(|p| tree.get(p, name)) else {
        return false;
    };
    keys.iter()
        .any(|k| tree.get(schema, k).and_then(|n| tree.as_bool(n)) == Some(true))
}
