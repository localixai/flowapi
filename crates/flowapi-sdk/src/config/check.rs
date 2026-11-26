//! The config held to its description (study 03, section 6, items 2, 5 and 6): every reference
//! names an operation, no operation is in two places, every operation is in a resource or in
//! `skip`; each method's pagination fits its operation; a streamed method's response is a
//! stream; `default` names an environment; the README's methods exist. `auth` and `security` are
//! held in `auth.rs`.

use super::issue::{At, Issues, closest};
use super::model::{Config, Method, PagingRef};
use crate::ops::{Lookup, Ops};
use crate::paging::{Paging, fit};
use crate::words;

/// The operation each method calls, by index into `c.methods`; `None` where it names none.
pub(crate) fn references(c: &Config, ops: &Ops<'_, '_>, issues: &mut Issues) -> Vec<Option<usize>> {
    let mut placed: Vec<Option<String>> = vec![None; ops.list.len()];
    let mut out = Vec::new();
    let places = c
        .methods
        .iter()
        .map(|m| (&m.op, format!("{}.{}", c.path(m.resource), m.name.name)))
        .chain(c.skip.iter().map(|s| (s, String::from("skip"))));
    for (op, place) in places {
        let found = match ops.find(&op.name) {
            Lookup::Found(i) => Some(i),
            Lookup::Ambiguous => {
                let message = format!("more than one operation has the operationId `{}`", op.name);
                let hint = String::from("name it by its route, `\"METHOD /path\"`");
                issues.error_hint("config.op.ambiguous", &op.at, message, Some(hint));
                None
            }
            Lookup::Missing => {
                let message = format!("`{}` names no operation of the description", op.name);
                let hint =
                    closest(&op.name, ops.references()).map(|n| format!("did you mean `{n}`?"));
                issues.error_hint("config.op.unknown", &op.at, message, hint);
                None
            }
        };
        if let Some(i) = found
            && let Some(slot) = placed.get_mut(i)
        {
            if let Some(first) = slot {
                let message = format!("`{}` is in two places: {first} and {place}", op.name);
                issues.error("config.op.twice", &op.at, message);
            } else {
                *slot = Some(place);
            }
        }
        if out.len() < c.methods.len() {
            out.push(found);
        }
    }
    for (i, slot) in placed.iter().enumerate() {
        if slot.is_none() {
            let message = format!("`{}` is in no resource and not in `skip`", ops.id(i));
            let hint = String::from(
                "name it under a resource's `methods`, or add it to `skip` with a reason",
            );
            issues.error_hint("config.coverage", &c.resources_at, message, Some(hint));
        }
    }
    out
}

/// The scheme a method pages by: its own, else the default for a method named `list`.
pub(crate) fn effective<'c>(c: &'c Config, m: &'c Method) -> Option<&'c super::model::Named> {
    match &m.paging {
        PagingRef::Scheme(n) => Some(n),
        PagingRef::Off => None,
        PagingRef::Unset => c
            .default_paging
            .as_ref()
            .filter(|_| words::words(&m.name.name) == ["list"]),
    }
}

/// Each method's pagination and streaming held to its operation.
pub(crate) fn methods(
    c: &Config,
    ops: &Ops<'_, '_>,
    resolved: &[Option<usize>],
    issues: &mut Issues,
) {
    let names: Vec<&str> = c.schemes.iter().map(|s| s.name.name.as_str()).collect();
    if let Some(d) = &c.default_paging
        && find(c, &d.name).is_none()
    {
        let hint = closest(&d.name, names.iter().copied()).map(|n| format!("did you mean `{n}`?"));
        issues.error_hint(
            "config.pagination.unknown",
            &d.at,
            format!("no pagination scheme is named `{}`", d.name),
            hint,
        );
    }
    for (m, op) in c.methods.iter().zip(resolved) {
        let op = op.and_then(|i| ops.list.get(i));
        if let (Some(at), Some(op)) = (&m.stream, op)
            && !streams(&op.operation)
        {
            let message = format!(
                "`{}` answers no 2xx response as a stream (`text/event-stream`, JSON lines)",
                op.route
            );
            issues.error("config.stream.media", at, message);
        }
        let Some(name) = effective(c, m) else {
            continue;
        };
        let Some(paging) = find(c, &name.name) else {
            if matches!(m.paging, PagingRef::Scheme(_)) {
                let hint = closest(&name.name, names.iter().copied())
                    .map(|n| format!("did you mean `{n}`?"));
                issues.error_hint(
                    "config.pagination.unknown",
                    &name.at,
                    format!("no pagination scheme is named `{}`", name.name),
                    hint,
                );
            }
            continue;
        };
        let Some(op) = op else { continue };
        for p in fit::fit(&ops.doc, &op.operation, paging) {
            let message = format!(
                "`{}.{}` ({}) pages by `{}`: `{}.{}`: {}",
                c.path(m.resource),
                m.name.name,
                op.route,
                name.name,
                p.part,
                p.role,
                p.message
            );
            if p.warning {
                issues.warn(p.code, &m.name.at, message);
            } else {
                issues.error_hint(p.code, &m.name.at, message, p.hint);
            }
        }
    }
}

/// The scheme of a name, as words.
pub(crate) fn find<'c>(c: &'c Config, name: &str) -> Option<&'c Paging> {
    c.schemes
        .iter()
        .find(|s| words::same(&s.name.name, name))
        .map(|s| &s.paging)
}

/// Whether a 2xx response is a stream of events or of JSON lines.
pub(crate) fn streams(op: &flowapi::spec::Operation) -> bool {
    const STREAMS: &[&str] = &[
        "text/event-stream",
        "application/x-ndjson",
        "application/jsonl",
        "application/x-jsonlines",
        "application/json-seq",
    ];
    op.responses
        .iter()
        .filter(|r| r.status.starts_with('2'))
        .flat_map(|r| &r.content)
        .any(|m| {
            let bare = m
                .media_type
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            STREAMS.contains(&bare.as_str())
        })
}

/// `environments.default` and `readme`, each held to what it names (`auth` is `auth.rs`'s).
pub(crate) fn rest(c: &Config, resolved: &[Option<usize>], issues: &mut Issues) {
    if let Some(d) = &c.default_environment
        && !c.environments.iter().any(|e| words::same(&e.name, &d.name))
    {
        let hint = closest(&d.name, c.environments.iter().map(|e| e.name.as_str()))
            .map(|n| format!("did you mean `{n}`?"));
        issues.error_hint(
            "config.env.default",
            &d.at,
            format!("no environment is named `{}`", d.name),
            hint,
        );
    }
    let all: Vec<(String, usize)> = c
        .methods
        .iter()
        .enumerate()
        .map(|(i, m)| (format!("{}.{}", c.path(m.resource), m.name.name), i))
        .collect();
    for (key, n) in &c.readme {
        let wanted: Vec<Vec<String>> = n.name.split('.').map(words::words).collect();
        let found = all
            .iter()
            .find(|(path, _)| path.split('.').map(words::words).eq(wanted.iter().cloned()));
        let Some((_, i)) = found else {
            let hint = closest(&n.name, all.iter().map(|(p, _)| p.as_str()))
                .map(|p| format!("did you mean `{p}`?"));
            issues.error_hint(
                "config.readme.method",
                &n.at,
                format!("`{}` is no method of the config", n.name),
                hint,
            );
            continue;
        };
        let paged = c.methods.get(*i).and_then(|m| effective(c, m)).is_some();
        if *key == "pagination_method" && !paged && resolved.get(*i).is_some_and(Option::is_some) {
            issues.error(
                "config.readme.pagination",
                &n.at,
                format!("`{}` has no pagination", n.name),
            );
        }
    }
}

/// A warning when the config pins another flowapi than this one.
pub(crate) fn generator(c: &Config, issues: &mut Issues) {
    let Some(g) = &c.generator else { return };
    let ours = env!("CARGO_PKG_VERSION");
    let pinned = g.name.strip_suffix(".x").map_or_else(
        || g.name == ours,
        |minor| ours.starts_with(&format!("{minor}.")),
    );
    if !pinned {
        let message = format!(
            "the config pins flowapi {}, and this is flowapi {ours}",
            g.name
        );
        issues.warn("config.generator.version", &g.at, message);
    }
}

/// Where a config's member is, for an issue about the whole config.
pub(crate) fn root() -> At {
    At::default()
}
