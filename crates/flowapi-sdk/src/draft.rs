//! `flowapi config draft`: a first config for a description, made without an agent, for an agent
//! to edit. Every operation is in a resource (`naming.rs`), each `GET` of a collection that
//! looks paged pages by the likeliest scheme that fits it (`paging::suggest`), the environments
//! are the description's absolute servers, `auth` its security schemes (`draft/auth.rs`), `skip: []`. The same
//! description gives the same text in whatever order its paths are written, and the text passes
//! `config check` (a draft that does not is a bug of this module), but for the warnings of a
//! scheme no config can answer (`draft/auth.rs`).

mod auth;
mod naming;

use crate::doc::Doc;
use crate::model::Server;
use crate::ops::Ops;
use crate::out::yaml_str;
use crate::paging::{Paging, suggest};
use crate::words::{self, join, pascal, upper, words};
use flowapi::{Diagnostics, Documents};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// A method of the draft: its name, the operation it calls, and its scheme's index.
struct Method {
    name: String,
    reference: String,
    route: String,
    paging: Option<usize>,
    stream: bool,
}

/// The draft of a config for the description in `docs`, read from `spec` (written as given),
/// as YAML; `None` when the first document is not an OpenAPI or Swagger description.
#[must_use]
pub fn draft(docs: &Documents<'_>, spec: &str) -> Option<String> {
    let (ops, _) = Ops::read(docs);
    let ops = ops?;
    // `title` the client's words, which a resource may not be; `prose` the package's.
    let (prose, title) = naming::title(&ops.info("title").unwrap_or_default());
    let models: BTreeSet<String> = ops
        .schema_names
        .iter()
        .map(|n| join(&words(n), "_"))
        .collect();
    let mut groups: BTreeMap<Vec<String>, Vec<usize>> = BTreeMap::new();
    for (i, op) in ops.list.iter().enumerate() {
        groups.entry(naming::group(op)).or_default().push(i);
    }
    // A resource whose own name is free keeps it; the others are renamed around them.
    let none = BTreeSet::new();
    let own = |g: &Vec<String>| naming::resource(g, &models, &title, &none) == join(g, "_");
    let mut taken: BTreeSet<String> = groups
        .keys()
        .filter(|g| own(g))
        .map(|g| join(g, "_"))
        .collect();
    // The paths with a `PUT` or `PATCH`: a `POST` to an item is its `update` only without one.
    let edited: BTreeSet<&str> = ops
        .list
        .iter()
        .filter(|op| matches!(op.operation.method.as_str(), "put" | "patch"))
        .map(|op| op.operation.path.as_str())
        .collect();
    let mut schemes: Vec<Paging> = Vec::new();
    let mut resources: Vec<(String, Vec<Method>)> = Vec::new();
    for (group, mut members) in groups {
        members.sort_by_key(|&a| order(&ops, a));
        let name = if own(&group) {
            join(&group, "_")
        } else {
            naming::resource(&group, &models, &title, &taken)
        };
        taken.insert(name.clone());
        let names: Vec<Vec<Vec<String>>> = members
            .iter()
            .filter_map(|&i| ops.list.get(i))
            .map(|op| naming::method(op, &group, edited.contains(op.operation.path.as_str())))
            .collect();
        let mut methods = Vec::new();
        for (&i, method) in members.iter().zip(naming::methods(&names)) {
            let Some(op) = ops.list.get(i) else { continue };
            let collection = op.operation.method == "get"
                && !op.operation.path.trim_end_matches('/').ends_with('}');
            let paging = collection
                .then(|| suggest::suggest(&ops.doc, &op.operation).into_iter().next())
                .flatten()
                .map(|s| index(&mut schemes, s.paging));
            let stream = crate::config::streams(&op.operation);
            methods.push(Method {
                name: method,
                reference: ops.id(i),
                route: op.route.clone(),
                paging,
                stream,
            });
        }
        resources.push((name, methods));
    }
    resources.sort_by(|a, b| a.0.cmp(&b.0));
    let names = scheme_names(&schemes);
    let mut out = String::from(HEADER);
    let client = pascal(&title);
    head(&mut out, spec, &prose, &client);
    environments(&mut out, &ops.doc, &prose);
    auth::auth(&mut out, &ops, &prose);
    out.push_str("defaults:\n  retries: 2\n  timeout: 60\n  max_retry_delay: 60\n");
    if !schemes.is_empty() {
        out.push_str("pagination:\n");
        for (s, name) in schemes.iter().zip(&names) {
            let _ = writeln!(out, "  {name}:");
            s.yaml(4, &mut out);
        }
    }
    out.push_str(if resources.is_empty() {
        "resources: {}\n"
    } else {
        "resources:\n"
    });
    for (name, methods) in &resources {
        let _ = writeln!(out, "  {name}:\n    methods:");
        for m in methods {
            print_method(&mut out, m, &names);
        }
    }
    out.push_str("skip: []\n");
    Some(out)
}

const HEADER: &str =
    "# A first flowapi-sdk.yml, drafted by `flowapi config draft` from the description: rename
# resources and methods, page what pages, then `flowapi config check` it.
version: 1
";

/// An operation's place in its resource: its path, then its method.
fn order(ops: &Ops<'_, '_>, i: usize) -> (String, String) {
    ops.list
        .get(i)
        .map(|op| (op.operation.path.clone(), op.operation.method.clone()))
        .unwrap_or_default()
}

/// A scheme's index, the same scheme written once.
fn index(schemes: &mut Vec<Paging>, p: Paging) -> usize {
    if let Some(i) = schemes.iter().position(|s| *s == p) {
        return i;
    }
    schemes.push(p);
    schemes.len() - 1
}

/// Each scheme's name: its type where one scheme has it, else its type and its items' words
/// (`page_actions`), then a number, in the order first used.
fn scheme_names(schemes: &[Paging]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in schemes {
        let alone = schemes.iter().filter(|o| o.kind == s.kind).count() == 1;
        let items = s
            .get("items")
            .map_or_else(|| vec![String::from("body")], words);
        let base = if alone {
            s.kind.to_owned()
        } else {
            join(&[vec![s.kind.to_owned()], items].concat(), "_")
        };
        let name = (1..)
            .map(|n| {
                if n == 1 {
                    base.clone()
                } else {
                    format!("{base}_{n}")
                }
            })
            .find(|n| !out.iter().any(|o| words::same(o, n)))
            .unwrap_or(base);
        out.push(name);
    }
    out
}

fn print_method(out: &mut String, m: &Method, names: &[String]) {
    let comment = if m.reference == m.route {
        String::new()
    } else {
        format!("  # {}", m.route)
    };
    let scheme = m.paging.and_then(|i| names.get(i));
    if scheme.is_none() && !m.stream {
        let _ = writeln!(out, "      {}: {}{comment}", m.name, yaml_str(&m.reference));
        return;
    }
    let _ = writeln!(
        out,
        "      {}:{comment}\n        op: {}",
        m.name,
        yaml_str(&m.reference)
    );
    if let Some(s) = scheme {
        let _ = writeln!(out, "        pagination: {s}");
    }
    if m.stream {
        out.push_str("        stream: true\n");
    }
}

fn head(out: &mut String, spec: &str, title: &[String], client: &str) {
    let _ = writeln!(
        out,
        "generator:\n  flowapi: {}",
        yaml_str(env!("CARGO_PKG_VERSION"))
    );
    let _ = writeln!(out, "spec: {}", yaml_str(spec));
    let package: String = join(title, "-").chars().take(214).collect();
    let _ = writeln!(
        out,
        "package:\n  name: {}\n  version: 0.1.0\n  client: {client}",
        yaml_str(&package)
    );
}

/// A server's URL with each variable's default put in.
fn url(s: &Server) -> String {
    s.variables.iter().fold(s.url.clone(), |u, v| {
        u.replace(&format!("{{{}}}", v.name), &v.default)
    })
}

/// The description's absolute servers: the first `production`, the others by their name or
/// description where it is a short name of its own, else `server_<n>`.
fn environments(out: &mut String, doc: &Doc<'_, '_>, title: &[String]) {
    let mut seen: Vec<String> = Vec::new();
    let mut lines = Vec::new();
    for s in doc.servers(&mut Diagnostics::default()) {
        let u = url(&s);
        if !crate::config::is_url(&u) || seen.contains(&u) {
            continue;
        }
        seen.push(u.clone());
        let n = seen.len();
        let own = s.name.as_deref().or(s.description.as_deref()).map(words);
        let name = env_name(own, n, &lines);
        lines.push((name, u));
    }
    if lines.is_empty() {
        return;
    }
    out.push_str("environments:\n");
    for (name, u) in &lines {
        let _ = writeln!(out, "  {name}: {}", yaml_str(u));
    }
    let env = upper(
        &[
            title.to_vec(),
            vec![String::from("base"), String::from("url")],
        ]
        .concat(),
    );
    let _ = writeln!(out, "  default: production\n  env_var: {env}");
}

/// An environment's name: `production` for the first; else the server's own name in one to
/// three words, where it is a name no other environment and no member of `environments` has;
/// else `server_<n>`.
fn env_name(own: Option<Vec<String>>, n: usize, before: &[(String, String)]) -> String {
    if n == 1 {
        return String::from("production");
    }
    let free = |w: &str| {
        words::is_name(w)
            && !before.iter().any(|(l, _)| words::same(l, w))
            && !["default", "env_var", "production"]
                .iter()
                .any(|k| words::same(k, w))
    };
    match own
        .filter(|w| (1..=3).contains(&w.len()))
        .map(|w| join(&w, "_"))
    {
        Some(w) if free(&w) => w,
        Some(_) | None => format!("server_{n}"),
    }
}
