//! The members of a config that are objects of their own, read for `build.rs`: a pagination
//! scheme (its `type` chooses its members), a method (an operation's name, or an object with
//! `op` and the method's own settings), a credential of `auth`, and a `security`.

use super::B;
use crate::config::issue::At;
use crate::config::model::{Credential, Method, Named, PagingRef, Security};
use crate::config::shape::{self, SCHEMES};
use crate::paging::{Paging, roles};
use flowapi::id::NodeId;

/// A pagination scheme: its `type` chooses its members.
pub(super) fn scheme(b: &mut B<'_, '_>, node: NodeId, at: &At) -> Option<Paging> {
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
        paging.start = b.number(s);
    }
    Some(paging)
}

/// A method: an operation's name, or an object with `op` and the method's own settings.
pub(super) fn method(
    b: &mut B<'_, '_>,
    name: Named,
    value: NodeId,
    resource: usize,
) -> Option<Method> {
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
            retries: None,
            security: None,
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
    let retries = b
        .field(value, &at, shape::METHOD, "retries")
        .and_then(|(n, _)| b.number(n));
    let stream = b
        .field(value, &at, shape::METHOD, "stream")
        .and_then(|(s, at)| (b.tree.as_bool(s) == Some(true)).then_some(at));
    let security = b
        .field(value, &at, shape::METHOD, "security")
        .map(|(s, at)| security(b, s, &at));
    Some(Method {
        name,
        resource,
        op,
        paging,
        stream,
        retries,
        security,
    })
}

/// The environment variables a credential may name.
const VARS: [&str; 5] = [
    "env_var",
    "username_env_var",
    "password_env_var",
    "client_id_env_var",
    "client_secret_env_var",
];

/// A credential of `auth`: its options, and what a scheme it declares is.
pub(super) fn credential(b: &mut B<'_, '_>, name: Named, node: NodeId) -> Credential {
    let at = name.at.clone();
    let mut c = Credential {
        name,
        ..Credential::default()
    };
    if !b.object(node) {
        return c;
    }
    let fields = shape::AUTH;
    c.kind = b.named(node, &at, fields, "type");
    c.scheme = b.named(node, &at, fields, "scheme");
    c.location = b.named(node, &at, fields, "in");
    c.key = b.named(node, &at, fields, "name");
    c.token_url = b.named(node, &at, fields, "token_url");
    c.description = b.text(node, &at, fields, "description");
    for member in VARS {
        if let Some(var) = b.named(node, &at, fields, member) {
            c.env.push((member, var));
        }
    }
    if let Some((list, at)) = b.field(node, &at, fields, "scopes") {
        let tree = b.tree;
        let scopes = tree
            .items(list)
            .iter()
            .filter_map(|&n| b.string(n))
            .collect();
        c.scopes = Some((scopes, at));
    }
    c
}

/// A `security`: each alternative's schemes, by name with where, with their scopes.
pub(super) fn security(b: &mut B<'_, '_>, list: NodeId, at: &At) -> Security {
    let tree = b.tree;
    let mut alternatives = Vec::new();
    for (i, &item) in tree.items(list).iter().enumerate() {
        let item_at = at.child(&i.to_string(), tree.offset(item));
        let mut schemes = Vec::new();
        for (name, scopes, at) in b.entries(item, &item_at, &[]) {
            let scopes = tree
                .items(scopes)
                .iter()
                .filter_map(|&n| b.string(n))
                .collect();
            schemes.push((Named { name, at }, scopes));
        }
        alternatives.push(schemes);
    }
    Security { alternatives }
}
