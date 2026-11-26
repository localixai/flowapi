//! How a draft names things, from the description alone and whatever its order: an operation's
//! resource is its first tag, else its path's first static segment (a version such as `v1`
//! passed over); a resource's own collection and item (its noun the path's one static
//! segment) get the verbs `list`, `create`,
//! `retrieve`, `update`, `delete`; any other operation is named by its `operationId`'s words
//! without the words its resource already says (`naming/trim.rs`), else by its method and path.
//! Names equal as words both take their longer names, then a number, in the order of path and
//! method.

mod trim;

use crate::config::{CLIENT_MEMBERS, RESOURCE_MEMBERS, reserved};
use crate::ops::Op;
use crate::words::{join, words};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "naming_tests.rs"]
mod tests;

/// The words of a description's `info.title`, twice. First as the package's name and the
/// environment variables read them: the title split where it is split (at what is no ASCII letter
/// or digit), each piece one word whatever its capitals, since a title is prose and its pieces
/// are words already (`GitHub` is `github`, as npm and the platform spell it, never `git hub`;
/// `REST` is `rest`). Then as the client's class: the same text read as a name (`words`), which is
/// how the generator and `config check` read the class (`GitHubV3RestApi`, its words `git hub v3
/// rest api`). Both open with a letter: `api` is put first when the title does not.
pub(crate) fn title(text: &str) -> (Vec<String>, Vec<String>) {
    let lettered = |w: Vec<String>| {
        if usable(&w) {
            w
        } else {
            [vec![String::from("api")], w].concat()
        }
    };
    let prose = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|piece| !piece.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    (lettered(prose), lettered(words(text)))
}

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
/// `admin_apps`, which is a call of its own). A `POST` to an item of one parameter is its
/// `update` where the item has no `PUT` or `PATCH` (`edited`), as APIs that update by `POST`
/// write it (`/v1/prices/{price}`; not `/orgs/{org}/{product}/{action}`, which is an action).
fn verb(op: &Op, group: &[String], edited: bool) -> Option<&'static str> {
    match statics(&op.operation.path).as_slice() {
        [one] if same_noun(&words(one), group) => {}
        _ => return None,
    }
    let path = op.operation.path.trim_end_matches('/');
    let item = path.ends_with('}');
    let one_param = path.matches('{').count() == 1;
    match (op.operation.method.as_str(), item) {
        ("get", false) => Some("list"),
        ("post", false) => Some("create"),
        ("get", true) => Some("retrieve"),
        ("put" | "patch", true) => Some("update"),
        ("post", true) if one_param && !edited => Some("update"),
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

/// An operation's names in a resource, the shortest first, each longer one for when the one
/// before meets another's: the CRUD verb where one applies; the `operationId`'s words without the
/// resource's (`trim.rs`); those words without the resource's at their front alone; all of them
/// (else the method and the path's segments). `edited` says whether the operation's path has a
/// `PUT` or `PATCH`.
pub(crate) fn method(op: &Op, group: &[String], edited: bool) -> Vec<Vec<String>> {
    let id = op
        .operation_id
        .as_deref()
        .map(words)
        .filter(|w| !w.is_empty());
    let method = op.operation.method.to_ascii_lowercase();
    // A name that opens with a digit is given its method in front.
    let lettered = |w: Vec<String>| {
        if usable(&w) {
            w
        } else {
            [vec![method.clone()], w].concat()
        }
    };
    let mut out: Vec<Vec<String>> = Vec::new();
    if let Some(v) = verb(op, group, edited) {
        out.push(vec![String::from(v)]);
    }
    if let Some(w) = &id {
        out.push(trim::trim(w, group));
        out.push(w.strip_prefix(group).unwrap_or(w).to_vec());
    } else {
        out.push(route_words(op, 1));
    }
    out.push(
        id.filter(|w| usable(w))
            .unwrap_or_else(|| route_words(op, 0)),
    );
    // Kept when equal: the names of two operations are raised a kind at a time, so that one whose
    // shorter names are one text is never raised past the others' kind (`list` by `GET /users`
    // meets `GET /user`'s; the first keeps `list` as its trimmed name, the second takes
    // `get_authenticated`).
    out.into_iter()
        .filter(|w| !w.is_empty())
        .map(lettered)
        .collect()
}

/// Each operation's name in one resource, `ops` in the order of path and method: its shortest
/// name where no other has it and no member of a resource is named so; where two meet, both take
/// their next longer name, until none meets or a name has no longer one; a name still shared is
/// told apart by a number, in the order of path and method.
pub(crate) fn methods(names: &[Vec<Vec<String>>]) -> Vec<String> {
    let count = |w: &Vec<String>, all: &[Vec<String>]| all.iter().filter(|x| *x == w).count();
    let mut level = vec![0usize; names.len()];
    let pick = |level: &[usize]| -> Vec<Vec<String>> {
        names
            .iter()
            .zip(level)
            .map(|(n, &l)| n.get(l).or_else(|| n.last()).cloned().unwrap_or_default())
            .collect()
    };
    // Each pass raises at least one name, so the passes are as many as the names' levels at most.
    let passes: usize = names.iter().map(Vec::len).sum();
    for _ in 0..passes {
        let chosen = pick(&level);
        let mut raised = false;
        for ((l, n), w) in level.iter_mut().zip(names).zip(&chosen) {
            let meets = count(w, &chosen) > 1 || reserved(&join(w, "_"), RESOURCE_MEMBERS);
            if meets && *l + 1 < n.len() {
                *l += 1;
                raised = true;
            }
        }
        if !raised {
            break;
        }
    }
    let chosen = pick(&level);
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
