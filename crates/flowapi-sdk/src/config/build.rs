//! A config's JSON read into [`Config`], after `read.rs` held it to the format: every member is
//! asked for by a name the format gives (`shape.rs`), and asking for another is an issue
//! (`config.internal`), as is a member the format names that nothing here reads. Together they
//! keep the format, its JSON Schema and this reader one thing. A value of the wrong type was
//! reported already and is passed over.

mod members;

use super::issue::{At, Issues};
use super::model::{Config, Named, Resource, Scheme};
use super::shape::{self, Field};
use flowapi::id::NodeId;
use flowapi::json::{Tree, Value};
use members::{credential, method, scheme, security};

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

    /// A string member's value.
    fn text(
        &mut self,
        node: NodeId,
        at: &At,
        fields: &'static [Field],
        key: &'static str,
    ) -> Option<String> {
        self.named(node, at, fields, key).map(|n| n.name)
    }

    /// A number as `T` (an integer as `i64`, seconds as `f64`); `None` for another value.
    fn number<T: std::str::FromStr>(&self, node: NodeId) -> Option<T> {
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
        c.settings.package = b.text(p, &at, shape::PACKAGE, "name");
        c.settings.version = b.text(p, &at, shape::PACKAGE, "version");
        c.client = b.named(p, &at, shape::PACKAGE, "client");
        c.license = b.named(p, &at, shape::PACKAGE, "license");
        c.settings.repository = b.text(p, &at, shape::PACKAGE, "repository");
        c.settings.description = b.text(p, &at, shape::PACKAGE, "description");
        if let Some((k, _)) = b.field(p, &at, shape::PACKAGE, "keywords") {
            let tree = b.tree;
            c.settings.keywords = tree.items(k).iter().filter_map(|&n| b.string(n)).collect();
        }
    }
    if let Some((e, at)) = b.field(root, &top, shape::ROOT, "environments") {
        c.default_environment = b.named(e, &at, shape::ENVIRONMENTS_FIXED, "default");
        c.settings.base_url_env = b
            .named(e, &at, shape::ENVIRONMENTS_FIXED, "env_var")
            .map(|n| n.name);
        for (name, url, at) in b.entries(e, &at, shape::ENVIRONMENTS_FIXED) {
            if let Some(url) = b.string(url) {
                c.settings.urls.push((name.clone(), url));
            }
            c.environments.push(Named { name, at });
        }
    }
    if let Some((a, at)) = b.field(root, &top, shape::ROOT, "auth") {
        for (name, node, at) in b.entries(a, &at, &[]) {
            c.auth.push(credential(&mut b, Named { name, at }, node));
        }
    }
    if let Some((s, at)) = b.field(root, &top, shape::ROOT, "security") {
        c.security = Some(security(&mut b, s, &at));
    }
    if let Some((d, at)) = b.field(root, &top, shape::ROOT, "defaults") {
        let retries = b.field(d, &at, shape::DEFAULTS, "retries");
        c.settings.retries = retries.and_then(|(n, _)| b.number(n));
        let timeout = b.field(d, &at, shape::DEFAULTS, "timeout");
        c.settings.timeout = timeout.and_then(|(n, _)| b.number(n));
        let delay = b.field(d, &at, shape::DEFAULTS, "max_retry_delay");
        c.settings.max_retry_delay = delay.and_then(|(n, _)| b.number(n));
        c.settings.idempotency_key = b.text(d, &at, shape::DEFAULTS, "idempotency_key");
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
        c.settings.title = b.text(r, &at, shape::README, "title");
        c.settings.intro = b.text(r, &at, shape::README, "intro");
    }
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
