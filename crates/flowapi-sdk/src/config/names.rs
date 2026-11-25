//! The config's names checked as words (study 03, section 6, items 3 and 4): two siblings equal
//! as words are one name, so `list_all` beside `listAll`, or `abuse-reports` beside
//! `abuse_reports`, is an error rather than one method or file silently written over the other;
//! a name that is one of the members a generated class has itself is an error; and a resource's
//! class may not take the name of the client or of one of the description's schemas.

use super::issue::Issues;
use super::model::{Config, Named};
use crate::words::{self, pascal};
use std::collections::BTreeSet;

/// The members the generated client has beside its resources, as words: a top resource may not
/// be one of them. The generator keeps to this list.
pub(crate) const CLIENT_MEMBERS: &[&str] = &[
    "constructor",
    "base_url",
    "timeout",
    "max_retries",
    "with_options",
    "request",
    "fetch",
];

/// The members a generated resource class has beside its methods and resources.
pub(crate) const RESOURCE_MEMBERS: &[&str] = &["constructor"];

/// Whether `name` is one of `members`, as words.
pub(crate) fn reserved(name: &str, members: &[&str]) -> bool {
    members.iter().any(|m| words::same(m, name))
}

/// Every check of names.
pub(crate) fn check(c: &Config, schema_names: &[String], issues: &mut Issues) {
    let models: BTreeSet<String> = schema_names
        .iter()
        .map(|n| words::join(&words::words(n), "_"))
        .collect();
    let client = c.client.as_ref().map(|n| words::words(&n.name));
    let groups = 1 + c.resources.len();
    for group in 0..groups {
        let parent = group.checked_sub(1);
        let mut names: Vec<(&Named, &'static str)> = Vec::new();
        for r in c.resources.iter().filter(|r| r.parent == parent) {
            names.push((&r.name, "resource"));
        }
        for m in c.methods.iter().filter(|m| Some(m.resource) == parent) {
            names.push((&m.name, "method"));
        }
        let place = parent.map_or_else(
            || String::from("the client"),
            |p| format!("`{}`", c.path(p)),
        );
        let members = if parent.is_some() {
            RESOURCE_MEMBERS
        } else {
            CLIENT_MEMBERS
        };
        siblings(&names, &place, issues);
        for (n, what) in &names {
            if reserved(&n.name, members) {
                let message = format!("the {what} `{}` is a member {place} has itself", n.name);
                issues.error("config.name.reserved", &n.at, message);
            }
        }
    }
    for r in &c.resources {
        let w = words::words(&r.name.name);
        if models.contains(&words::join(&w, "_")) {
            let message = format!(
                "the resource `{}` is the class `{}`, and the description has a schema of that name",
                r.name.name,
                pascal(&w)
            );
            issues.error_hint(
                "config.name.model",
                &r.name.at,
                message,
                Some(String::from(
                    "name the resource otherwise (a plural, as `pets` beside the schema `Pet`)",
                )),
            );
        }
        if r.parent.is_none() && client.as_ref() == Some(&w) {
            let message = format!(
                "the resource `{}` is the class `{}`, the client's name",
                r.name.name,
                pascal(&w)
            );
            issues.error("config.name.client", &r.name.at, message);
        }
    }
    let envs: Vec<(&Named, &'static str)> =
        c.environments.iter().map(|n| (n, "environment")).collect();
    siblings(&envs, "`environments`", issues);
    let schemes: Vec<(&Named, &'static str)> =
        c.schemes.iter().map(|s| (&s.name, "scheme")).collect();
    siblings(&schemes, "`pagination`", issues);
}

/// Names under one parent: the second of two equal as words is an error.
fn siblings(names: &[(&Named, &'static str)], place: &str, issues: &mut Issues) {
    for (i, (n, what)) in names.iter().enumerate() {
        let w = words::words(&n.name);
        let first = names
            .iter()
            .take(i)
            .find(|(m, _)| words::words(&m.name) == w);
        if let Some((m, other)) = first {
            let message = format!(
                "the {what} `{}` and the {other} `{}` are one name (`{}`) in {place}",
                n.name,
                m.name,
                words::join(&w, " ")
            );
            issues.error_hint(
                "config.name.duplicate",
                &n.at,
                message,
                Some(format!("rename one; `{}` is at {}", m.name, m.at.pointer)),
            );
        }
    }
}
