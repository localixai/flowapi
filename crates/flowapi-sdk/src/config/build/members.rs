//! The members of a config that are objects of their own, read for `build.rs`: a pagination
//! scheme (its `type` chooses its members) and a method (an operation's name, or an object with
//! `op` and the method's own settings).

use super::B;
use crate::config::issue::At;
use crate::config::model::{Method, Named, PagingRef};
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
    Some(Method {
        name,
        resource,
        op,
        paging,
        stream,
        retries,
    })
}
