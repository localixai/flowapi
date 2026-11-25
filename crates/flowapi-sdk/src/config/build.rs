//! A config's JSON read into [`Config`], after `read.rs` held it to the format: every member is
//! asked for by a name the format gives (`shape.rs`), and asking for another is an issue
//! (`config.internal`), as is a member the format names that nothing here reads. Together they
//! keep the format, its JSON Schema and this reader one thing. A value of the wrong type was
//! reported already and is passed over.

use super::issue::{At, Issues};
use super::model::{Config, Method, Named, PagingRef, Resource, Scheme};
use super::shape::{self, Field, SCHEMES};
use crate::paging::{Paging, roles};
use flowapi::id::NodeId;
use flowapi::json::{Tree, Value};

struct B<'a, 't> {
    tree: &'a Tree<'t>,
    read: Vec<NodeId>,
    issues: &'a mut Issues,
}

impl B<'_, '_> {
    /// The member `key` of an object, which `fields` must name, with where.
    fn field(
        &mut self,
        node: NodeId,
        at: &At,
        fields: &'static [Field],
        key: &'static str,
    ) -> Option<(NodeId, At)> {
        if !fields.iter().any(|f| f.name == key) {
            let message = format!(
                "the reader asks `{}` for `{key}`, which the format does not name",
                at.pointer
            );
            self.issues.error("config.internal", at, message);
            return None;
        }
        let m = self
            .tree
            .members(node)
            .iter()
            .rev()
            .find(|m| self.tree.key_is(m.key, key))?;
        self.read.push(m.value);
        Some((m.value, at.child(key, self.tree.offset(m.value))))
    }

    /// The entries of a map: every member but those `fixed` names, each with where its key is.
    fn entries(&mut self, node: NodeId, at: &At, fixed: &[Field]) -> Vec<(String, NodeId, At)> {
        let mut out = Vec::new();
        for m in self.tree.members(node) {
            let Some(key) = self.tree.string(m.key) else {
                continue;
            };
            if fixed.iter().any(|f| f.name == key) {
                continue;
            }
            self.read.push(m.value);
            out.push((
                key.clone().into_owned(),
                m.value,
                at.child(&key, m.key.start),
            ));
        }
        out
    }

    fn string(&self, node: NodeId) -> Option<String> {
        self.tree.as_str(node).map(|s| s.into_owned())
    }

    /// A string member as a [`Named`] at its value.
    fn named(
        &mut self,
        node: NodeId,
        at: &At,
        fields: &'static [Field],
        key: &'static str,
    ) -> Option<Named> {
        let (value, at) = self.field(node, at, fields, key)?;
        Some(Named {
            name: self.string(value)?,
            at,
        })
    }

    fn int(&self, node: NodeId) -> Option<i64> {
        match self.tree.value(node) {
            Value::Number(span) => self.tree.raw(span).parse().ok(),
            Value::Null
            | Value::Bool(_)
            | Value::String(_)
            | Value::Array(_)
            | Value::Object(_) => None,
        }
    }

    fn object(&self, node: NodeId) -> bool {
        matches!(self.tree.value(node), Value::Object(_))
    }
}

/// The config read from `tree`; `declared` are the members `read.rs` found the format names.
pub(crate) fn build(tree: &Tree<'_>, declared: &[(NodeId, At)], issues: &mut Issues) -> Config {
    let mut b = B {
        tree,
        read: Vec::new(),
        issues,
    };
    let mut c = Config::default();
    let root = tree.root();
    let top = At::default();
    if !b.object(root) {
        return c;
    }
    b.field(root, &top, shape::ROOT, "version");
    c.spec = b.named(root, &top, shape::ROOT, "spec").map(|n| n.name);
    if let Some((g, at)) = b.field(root, &top, shape::ROOT, "generator") {
        c.generator = b.named(g, &at, shape::GENERATOR, "flowapi");
    }
    if let Some((p, at)) = b.field(root, &top, shape::ROOT, "package") {
        b.field(p, &at, shape::PACKAGE, "name");
        c.client = b.named(p, &at, shape::PACKAGE, "client");
        b.field(p, &at, shape::PACKAGE, "license");
        b.field(p, &at, shape::PACKAGE, "repository");
    }
    if let Some((e, at)) = b.field(root, &top, shape::ROOT, "environments") {
        c.default_environment = b.named(e, &at, shape::ENVIRONMENTS_FIXED, "default");
        b.field(e, &at, shape::ENVIRONMENTS_FIXED, "env_var");
        for (name, _, at) in b.entries(e, &at, shape::ENVIRONMENTS_FIXED) {
            c.environments.push(Named { name, at });
        }
    }
    if let Some((a, at)) = b.field(root, &top, shape::ROOT, "auth") {
        for (name, node, at) in b.entries(a, &at, &[]) {
            b.field(node, &at, shape::AUTH, "env_var");
            c.auth.push(Named { name, at });
        }
    }
    if let Some((d, at)) = b.field(root, &top, shape::ROOT, "defaults") {
        for key in ["retries", "timeout", "max_retry_delay"] {
            b.field(d, &at, shape::DEFAULTS, key);
        }
        c.default_paging = b.named(d, &at, shape::DEFAULTS, "pagination");
    }
    if let Some((p, at)) = b.field(root, &top, shape::ROOT, "pagination") {
        for (name, node, at) in b.entries(p, &at, &[]) {
            if let Some(paging) = scheme(&mut b, node, &at) {
                c.schemes.push(Scheme {
                    name: Named { name, at },
                    paging,
                });
            }
        }
    }
    if let Some((r, at)) = b.field(root, &top, shape::ROOT, "resources") {
        c.resources_at = at.clone();
        resources(&mut b, r, &at, &mut c);
    }
    rest(&mut b, root, &top, &mut c);
    unread(&mut b, declared);
    c
}

/// `skip` and `readme`.
fn rest(b: &mut B<'_, '_>, root: NodeId, top: &At, c: &mut Config) {
    if let Some((s, at)) = b.field(root, top, shape::ROOT, "skip") {
        let tree = b.tree;
        for (i, &item) in tree.items(s).iter().enumerate() {
            let at = at.child(&i.to_string(), b.tree.offset(item));
            if b.object(item) {
                if let Some(op) = b.named(item, &at, shape::SKIP, "op") {
                    c.skip.push(op);
                }
                b.field(item, &at, shape::SKIP, "reason");
            }
        }
    }
    if let Some((r, at)) = b.field(root, top, shape::ROOT, "readme") {
        for key in ["headline_method", "pagination_method"] {
            if let Some(n) = b.named(r, &at, shape::README, key) {
                c.readme.push((key, n));
            }
        }
        b.field(r, &at, shape::README, "intro");
    }
}

/// A pagination scheme: its `type` chooses its members.
fn scheme(b: &mut B<'_, '_>, node: NodeId, at: &At) -> Option<Paging> {
    let tag = b.tree.get(node, "type").and_then(|t| b.string(t))?;
    let (kind, fields) = SCHEMES.iter().find(|(name, _)| *name == tag)?;
    b.field(node, at, fields, "type");
    let mut paging = Paging::new(kind, &[], &[]);
    for part in ["request", "response"] {
        if !fields.iter().any(|f| f.name == part) {
            continue;
        }
        let Some((obj, at)) = b.field(node, at, fields, part) else {
            continue;
        };
        for role in roles(kind, part) {
            if let Some(value) = b.named(obj, &at, roles(kind, part), role.name) {
                let list = if part == "request" {
                    &mut paging.request
                } else {
                    &mut paging.response
                };
                list.push((role.name, value.name));
            }
        }
    }
    if fields.iter().any(|f| f.name == "start")
        && let Some((s, _)) = b.field(node, at, fields, "start")
    {
        paging.start = b.int(s);
    }
    Some(paging)
}

/// The resource tree, flattened, with each resource's methods.
fn resources(b: &mut B<'_, '_>, map: NodeId, at: &At, c: &mut Config) {
    let mut stack: Vec<(NodeId, At, Option<usize>)> = vec![(map, at.clone(), None)];
    while let Some((map, at, parent)) = stack.pop() {
        for (name, node, at) in b.entries(map, &at, &[]) {
            let index = c.resources.len();
            c.resources.push(Resource {
                name: Named {
                    name,
                    at: at.clone(),
                },
                parent,
            });
            if !b.object(node) {
                continue;
            }
            if let Some((methods, at)) = b.field(node, &at, shape::resource_fields(), "methods") {
                for (name, value, at) in b.entries(methods, &at, &[]) {
                    if let Some(m) = method(b, Named { name, at }, value, index) {
                        c.methods.push(m);
                    }
                }
            }
            if let Some((under, at)) = b.field(node, &at, shape::resource_fields(), "resources") {
                stack.push((under, at, Some(index)));
            }
        }
    }
}

/// A method: an operation's name, or an object with `op` and the method's own settings.
fn method(b: &mut B<'_, '_>, name: Named, value: NodeId, resource: usize) -> Option<Method> {
    let at = name.at.clone();
    let at = At {
        offset: b.tree.offset(value),
        ..at
    };
    if let Some(op) = b.string(value) {
        return Some(Method {
            name,
            resource,
            op: Named { name: op, at },
            paging: PagingRef::Unset,
            stream: None,
        });
    }
    if !b.object(value) {
        return None;
    }
    let op = b.named(value, &at, shape::METHOD, "op")?;
    let paging = match b.field(value, &at, shape::METHOD, "pagination") {
        Some((p, at)) => match (b.string(p), b.tree.as_bool(p)) {
            (Some(s), _) => PagingRef::Scheme(Named { name: s, at }),
            (None, Some(false)) => PagingRef::Off,
            (None, Some(true) | None) => PagingRef::Unset,
        },
        None => PagingRef::Unset,
    };
    b.field(value, &at, shape::METHOD, "retries");
    let stream = b
        .field(value, &at, shape::METHOD, "stream")
        .and_then(|(s, at)| (b.tree.as_bool(s) == Some(true)).then_some(at));
    Some(Method {
        name,
        resource,
        op,
        paging,
        stream,
    })
}

/// A member the format names that nothing read: the reader and the format have drifted.
fn unread(b: &mut B<'_, '_>, declared: &[(NodeId, At)]) {
    b.read.sort_unstable();
    for (node, at) in declared {
        if b.read.binary_search(node).is_err() {
            let message = format!("the format names `{}` and nothing reads it", at.pointer);
            b.issues.error("config.internal", at, message);
        }
    }
}
