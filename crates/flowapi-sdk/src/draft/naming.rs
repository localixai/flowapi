//! How a draft names things, from the description alone and whatever its order: an operation's
//! resource is its first tag, else its path's first static segment (a version such as `v1`
//! passed over); a resource's own collection and item (its noun the path's one static
//! segment) get the verbs `list`, `create`,
//! `retrieve`, `update`, `delete`; any other operation is named by its `operationId`'s words,
//! the resource's words taken off the front, else by its method and path. Names equal as words
//! are told apart by the longer name, then by a number, in the order of path and method.

use crate::config::{CLIENT_MEMBERS, RESOURCE_MEMBERS, reserved};
use crate::ops::Op;
use crate::words::{join, words};
use std::collections::BTreeSet;

/// A path's segments that are no parameter, a version (`v1`, `v2beta`) left out.
pub(crate) fn statics(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|s| !s.is_empty() && !s.contains('{'))
        .filter(|s| !is_version(s))
        .collect()
}

fn is_version(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('v' | 'V')) && chars.next().is_some_and(|c| c.is_ascii_digit())
}

/// Words that may open a name: the first opens with a letter.
fn usable(w: &[String]) -> bool {
    w.first()
        .and_then(|f| f.chars().next())
        .is_some_and(|c| c.is_ascii_alphabetic())
}

/// The words of the resource an operation is in.
pub(crate) fn group(op: &Op) -> Vec<String> {
    if let Some(tag) = op.tags.first() {
        let w = words(tag);
        if usable(&w) {
            return w;
        }
    }
    let w = statics(&op.operation.path)
        .first()
        .map(|s| words(s))
        .unwrap_or_default();
    if usable(&w) {
        return w;
    }
    if w.is_empty() {
        return vec![String::from("root")];
    }
    let mut out = vec![String::from("api")];
    out.extend(w);
    out
}

/// A resource's name that no schema, client member or the client itself has: its words, else
/// with an `s`, else with `api` and a number.
pub(crate) fn resource(
    w: &[String],
    models: &BTreeSet<String>,
    client: &[String],
    taken: &BTreeSet<String>,
) -> String {
    let free = |name: &str| {
        !models.contains(name)
            && !reserved(name, CLIENT_MEMBERS)
            && words(name) != client
            && !taken.contains(name)
    };
    let name = join(w, "_");
    if free(&name) {
        return name;
    }
    if !name.ends_with('s') && free(&format!("{name}s")) {
        return format!("{name}s");
    }
    (1..)
        .map(|n| {
            if n == 1 {
                format!("{name}_api")
            } else {
                format!("{name}_api_{n}")
            }
        })
        .find(|n| free(n))
        .unwrap_or(name)
}

/// Whether two names are one noun: equal as words, or but for a plural `s` on the last word.
fn same_noun(a: &[String], b: &[String]) -> bool {
    let bare = |w: &[String]| {
        let mut w = w.to_vec();
        if let Some(last) = w.last_mut()
            && last.len() > 1
            && last.ends_with('s')
        {
            last.pop();
        }
        w
    };
    a == b || bare(a) == bare(b)
}

/// The verb of a resource's own collection or item: an operation whose path has one static
/// segment, the resource's noun (`/pets`, `/pets/{id}` in `pets`; not `/admin.apps.approve` in
/// `admin_apps`, which is a call of its own).
fn verb(op: &Op, group: &[String]) -> Option<&'static str> {
    match statics(&op.operation.path).as_slice() {
        [one] if same_noun(&words(one), group) => {}
        _ => return None,
    }
    let item = op.operation.path.trim_end_matches('/').ends_with('}');
    match (op.operation.method.as_str(), item) {
        ("get", false) => Some("list"),
        ("post", false) => Some("create"),
        ("get", true) => Some("retrieve"),
        ("put" | "patch", true) => Some("update"),
        ("delete", true) => Some("delete"),
        _ => None,
    }
}

/// The method and the path's static segments, as words.
fn route_words(op: &Op, skip: usize) -> Vec<String> {
    let mut w = vec![op.operation.method.to_ascii_lowercase()];
    for s in statics(&op.operation.path).into_iter().skip(skip) {
        w.extend(words(s));
    }
    w
}

/// The short name of an operation in a resource, and its long one for when two short ones meet.
pub(crate) fn method(op: &Op, group: &[String]) -> (Vec<String>, Vec<String>) {
    let id = op
        .operation_id
        .as_deref()
        .map(words)
        .filter(|w| !w.is_empty());
    let long = id
        .clone()
        .filter(|w| usable(w))
        .unwrap_or_else(|| route_words(op, 0));
    let short = if let Some(v) = verb(op, group) {
        vec![String::from(v)]
    } else if let Some(w) = id {
        let rest: Vec<String> = if w.starts_with(group) {
            w.get(group.len()..).unwrap_or(&[]).to_vec()
        } else {
            w.clone()
        };
        let rest = if rest.is_empty() { w } else { rest };
        if usable(&rest) {
            rest
        } else {
            [vec![op.operation.method.to_ascii_lowercase()], rest].concat()
        }
    } else {
        route_words(op, 1)
    };
    (short, long)
}

/// Each operation's name in one resource, `ops` in the order of path and method: the short name
/// where no other has it, else the long one, else the long one and a number.
pub(crate) fn methods(names: &[(Vec<String>, Vec<String>)]) -> Vec<String> {
    let count = |w: &Vec<String>, all: &[Vec<String>]| all.iter().filter(|x| *x == w).count();
    let shorts: Vec<Vec<String>> = names.iter().map(|(s, _)| s.clone()).collect();
    let chosen: Vec<Vec<String>> = names
        .iter()
        .map(|(s, l)| {
            if count(s, &shorts) == 1 && !reserved(&join(s, "_"), RESOURCE_MEMBERS) {
                s.clone()
            } else {
                l.clone()
            }
        })
        .collect();
    let unique =
        |w: &Vec<String>| count(w, &chosen) == 1 && !reserved(&join(w, "_"), RESOURCE_MEMBERS);
    let mut taken: BTreeSet<String> = chosen
        .iter()
        .filter(|w| unique(w))
        .map(|w| join(w, "_"))
        .collect();
    let mut out = Vec::new();
    for w in &chosen {
        let base = join(w, "_");
        if unique(w) {
            out.push(base);
            continue;
        }
        let first =
            (!reserved(&base, RESOURCE_MEMBERS) && !taken.contains(&base)).then(|| base.clone());
        let name = first
            .or_else(|| {
                (2..)
                    .map(|n| format!("{base}_{n}"))
                    .find(|n| !taken.contains(n))
            })
            .unwrap_or_else(|| base.clone());
        taken.insert(name.clone());
        out.push(name);
    }
    out
}
