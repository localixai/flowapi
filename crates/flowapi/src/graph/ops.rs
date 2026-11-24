//! Operations as types, so a target prints them by the same rules as any other: an operation is
//! an object of its `parameters` (by location, then by name), its `requestBody` (by media type)
//! and its `responses` (by status, then media type); `Paths` maps each path and method to its
//! operation. These objects are closed: a key they do not name is not part of the operation.
//! Each operation is kept beside its type as an [`Endpoint`], for a target's SDK.

use super::build::Builder;
use super::{Branches, Endpoint, Kind, Kinds, Object, Prop, Root, Type};
use crate::id::TypeId;
use crate::spec::{Location, Media, Operation, Param, Payload, Response};
use std::borrow::Cow;

impl<'t> Builder<'_, 't> {
    /// Each operation's type, named by the operation, and `Paths`.
    pub(super) fn operations(&mut self, ops: &[Operation]) -> Vec<Root> {
        let mut roots = Vec::with_capacity(ops.len() + 1);
        let mut paths: Vec<(String, Vec<(String, TypeId)>)> = Vec::new();
        for op in ops {
            let ty = self.operation(op);
            roots.push(Root {
                name: op.name.clone(),
                ty,
            });
            self.graph.endpoints.push(Endpoint {
                ty,
                operation: op.clone(),
            });
            match paths.iter_mut().find(|(p, _)| *p == op.path) {
                Some((_, methods)) => methods.push((op.method.clone(), ty)),
                None => paths.push((op.path.clone(), vec![(op.method.clone(), ty)])),
            }
        }
        if !paths.is_empty() {
            let props = paths
                .into_iter()
                .map(|(path, methods)| {
                    let methods = methods.into_iter().map(|(m, t)| (m, t, true)).collect();
                    let item = self.closed(methods);
                    (path, item, true)
                })
                .collect();
            let ty = self.closed(props);
            roots.push(Root {
                name: String::from("Paths"),
                ty,
            });
        }
        roots
    }

    fn operation(&mut self, op: &Operation) -> TypeId {
        let mut fields: Vec<(String, TypeId, bool)> = Vec::new();
        let mut locations: Vec<(String, TypeId, bool)> = Vec::new();
        for location in [
            Location::Path,
            Location::Query,
            Location::QueryString,
            Location::Header,
            Location::Cookie,
        ] {
            let params: Vec<_> = op
                .params
                .iter()
                .filter(|p| p.location == location)
                .collect();
            if params.is_empty() {
                continue;
            }
            let required = params.iter().any(|p| p.required);
            let props = params
                .iter()
                .map(|p| (p.name.clone(), self.param(p), p.required))
                .collect();
            let ty = self.closed(props);
            locations.push((location.name().to_owned(), ty, required));
        }
        if !locations.is_empty() {
            let required = locations.iter().any(|l| l.2);
            let ty = self.closed(locations);
            fields.push((String::from("parameters"), ty, required));
        }
        if let Some(body) = &op.body {
            let ty = self.media(&body.content);
            fields.push((String::from("requestBody"), ty, body.required));
        }
        let responses = op
            .responses
            .iter()
            .map(|r| (r.status.clone(), self.response(r), true))
            .collect();
        let ty = self.closed(responses);
        fields.push((String::from("responses"), ty, true));
        let ty = self.closed(fields);
        if let Some(t) = self.graph.types.get_mut(ty.index()) {
            t.deprecated = op.deprecated;
        }
        ty
    }

    /// A parameter's or a header's type; a deprecated one says so in its own alias.
    fn param(&mut self, p: &Param) -> TypeId {
        let ty = p.schema.as_ref().map_or(self.any, |s| self.type_of(s));
        if !p.deprecated {
            return ty;
        }
        let mut alias = Type::of(Kind::Alias(ty));
        alias.deprecated = true;
        self.graph.push(alias)
    }

    /// A response: its `headers` by name, when it has any, and its `content` by media type, when
    /// it has a body.
    fn response(&mut self, r: &Response) -> TypeId {
        let mut fields: Vec<(String, TypeId, bool)> = Vec::new();
        if !r.headers.is_empty() {
            let required = r.headers.iter().any(|h| h.required);
            let props = r
                .headers
                .iter()
                .map(|h| (h.name.clone(), self.param(h), h.required))
                .collect();
            let ty = self.closed(props);
            fields.push((String::from("headers"), ty, required));
        }
        if !r.content.is_empty() {
            let ty = self.media(&r.content);
            fields.push((String::from("content"), ty, true));
        }
        self.closed(fields)
    }

    /// A body's representations by media type; `Never` for none (no body).
    fn media(&mut self, content: &[Media]) -> TypeId {
        if content.is_empty() {
            return TypeId::new(1).unwrap_or(self.any);
        }
        let props = content
            .iter()
            .map(|m| {
                let ty = match &m.payload {
                    Payload::Schema(s) => s.as_ref().map_or(self.any, |s| self.type_of(s)),
                    // A Swagger 2.0 form is closed as the operation's parameters are: its
                    // fields are parameters there.
                    Payload::Form(fields) => {
                        let props = fields
                            .iter()
                            .map(|f| {
                                let ty = f.schema.as_ref().map_or(self.any, |s| self.type_of(s));
                                (f.name.clone(), ty, f.required)
                            })
                            .collect();
                        self.closed(props)
                    }
                };
                (m.media_type.clone(), ty, true)
            })
            .collect();
        self.closed(props)
    }

    /// A closed object of the properties given.
    fn closed(&mut self, props: Vec<(String, TypeId, bool)>) -> TypeId {
        let never = TypeId::new(1).unwrap_or(self.any);
        let props = props
            .into_iter()
            .map(|(name, ty, required)| Prop {
                raw: Cow::Owned(quote(&name)),
                name: Cow::Owned(name),
                ty,
                required,
            })
            .collect();
        let mut b = Branches::of(Kinds::NONE, self.any);
        b.object = Some(Object {
            props,
            patterns: Vec::new(),
            rest: never,
            extra: never,
        });
        self.make(Kind::Value(b))
    }
}

/// A string as a JSON string token.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
