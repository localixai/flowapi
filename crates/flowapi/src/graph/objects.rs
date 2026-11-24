//! A schema's own object keywords as the object branch: `properties` and `required`, the type of
//! any other property (`additionalProperties`, `patternProperties`), and `unevaluatedProperties:
//! false` closing it where every name it may take is known.

use super::build::Builder;
use super::own::Own;
use super::{Kind, Object, Pattern, Prop};
use crate::id::NodeId;
use crate::id::TypeId;
use crate::json::{Tree, Value};
use crate::schema::Resource;
use crate::source::Dialect;
use std::borrow::Cow;

impl<'t> Builder<'_, 't> {
    pub(super) fn object(&mut self, tree: &Tree<'t>, at: &Resource, own: &Own) -> Object<'t> {
        let mut props: Vec<Prop<'t>> = Vec::new();
        for m in own.properties.map_or(&[][..], |n| tree.members(n)) {
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
        for &n in own.required.map_or(&[][..], |n| tree.items(n)) {
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
            if one_way(tree, own.properties, &name, one_way_keys) {
                continue;
            }
            match props.iter_mut().find(|p| p.name == name) {
                Some(p) => p.required = true,
                None => props.push(Prop {
                    raw: Cow::Borrowed(raw),
                    name,
                    ty: self.any,
                    required: true,
                }),
            }
        }
        let rest = own.additional.map_or(self.any, |n| self.sub(at, n));
        let mut extra = rest;
        let mut patterns: Vec<Pattern<'t>> = Vec::new();
        for m in own.patterns.map_or(&[][..], |n| tree.members(n)) {
            let raw = tree.raw(m.key);
            let regex = tree.string(m.key).unwrap_or(Cow::Borrowed(raw));
            patterns.push(Pattern {
                regex,
                ty: self.sub(at, m.value),
            });
        }
        if !patterns.is_empty() && extra != self.any {
            let mut members = vec![extra];
            members.extend(patterns.iter().map(|p| p.ty));
            extra = self.make(Kind::Union(members));
        }
        // `unevaluatedProperties: false`: no name but those some schema applied here could
        // evaluate (unevaluated.rs); `additionalProperties` evaluates every name, so it wins.
        if own.unevaluated_props.is_some()
            && own.additional.is_none()
            && own.patterns.is_none()
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
            return Object {
                props,
                patterns,
                rest: extra,
                extra,
            };
        }
        Object {
            props,
            patterns,
            rest,
            extra,
        }
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
