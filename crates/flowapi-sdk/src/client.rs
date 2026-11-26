//! The client of an SDK, printed from a config (`flowapi-sdk.yml`) and its description: the
//! `src/` of the generated package as TypeScript files. `src/api.ts` is the TypeScript target's
//! SDK (`--sdk`: the types, `<op>Request` and `<op>Response`) with the operations table after it;
//! `src/core/*.ts` the runtime, the same for every API (`client/core/`, printed as written);
//! `src/client.ts` the client class the config names, its settings and its options; and
//! `src/resources/**.ts` a class per resource of the config's tree, each method calling its
//! operation through the runtime (`client/resources.rs`). No I/O (G5): the files are returned.

mod comment;
pub(crate) mod entry;
pub(crate) mod resources;

use crate::config::{Checked, Issue, checked, issue::Issues};
use crate::ops::Ops;
use crate::words;
use flowapi::{Diagnostics, Documents, File, Program, Severity, Target};
use std::collections::BTreeMap;

/// The runtime, file by file: the same for every API.
const CORE: &[(&str, &str)] = &[
    ("auth.ts", include_str!("client/core/auth.ts")),
    ("call.ts", include_str!("client/core/call.ts")),
    ("calls.ts", include_str!("client/core/calls.ts")),
    ("core.ts", include_str!("client/core/core.ts")),
    ("errors.ts", include_str!("client/core/errors.ts")),
    ("index.ts", include_str!("client/core/index.ts")),
    ("log.ts", include_str!("client/core/log.ts")),
    ("pagination.ts", include_str!("client/core/pagination.ts")),
    ("platform.ts", include_str!("client/core/platform.ts")),
    ("request.ts", include_str!("client/core/request.ts")),
    ("retry.ts", include_str!("client/core/retry.ts")),
    ("send.ts", include_str!("client/core/send.ts")),
    ("servers.ts", include_str!("client/core/servers.ts")),
    ("streaming.ts", include_str!("client/core/streaming.ts")),
];

/// What [`client_files`] made.
#[derive(Debug, Default)]
pub struct Generated {
    /// The files, by path from the package's root, in a fixed order; none when an issue is an
    /// error.
    pub files: Vec<File>,
    /// What reading the description found (its servers, schemes, encodings).
    pub diagnostics: Diagnostics,
    /// What holding the config to the description found, as `config check` reports it, and what
    /// printing the client found (`client.*`).
    pub issues: Vec<Issue>,
}

/// The `src/` of an SDK: its client, resources and runtime, after the TypeScript target's SDK for
/// `program` (made with `TypeScript::with_sdk`, so that its names are the SDK's), by the config's
/// JSON text. Nothing is printed when the config has an error: `issues` says what to change.
#[must_use]
pub fn client_files(docs: &Documents<'_>, program: &Program<'_>, config: &str) -> Generated {
    let mut checked = checked(docs, config);
    let issues = std::mem::take(&mut checked.issues);
    print(docs, program, issues, &checked)
}

/// [`client_files`] of a config read and checked already; `issues` are the check's.
pub(crate) fn print(
    docs: &Documents<'_>,
    program: &Program<'_>,
    issues: Vec<Issue>,
    checked: &Checked,
) -> Generated {
    let (service, diagnostics) = crate::read(docs, program);
    let mut out = Generated {
        files: Vec::new(),
        diagnostics,
        issues,
    };
    let (Some(ops), false) = (
        Ops::read(docs).0,
        out.issues.iter().any(|i| i.severity == Severity::Error),
    ) else {
        return out;
    };
    let c = &checked.config;
    let target = flowapi_ts::TypeScript::new("src/api.ts").with_sdk();
    let mut api = target.emit(program);
    let mut text = api.files.pop().map(|f| f.text).unwrap_or_default();
    text.push_str(&crate::print::table(program, &service));
    out.files.push(file("src/api.ts", text));
    for (name, text) in CORE {
        out.files
            .push(file(&format!("src/core/{name}"), (*text).to_owned()));
    }
    // Each method's endpoint in the program, by its route.
    let endpoints: BTreeMap<String, usize> = program
        .endpoints()
        .iter()
        .enumerate()
        .map(|(i, e)| (route(&e.operation), i))
        .collect();
    let methods: Vec<Option<usize>> = checked
        .resolved
        .iter()
        .map(|r| {
            r.and_then(|i| ops.list.get(i))
                .and_then(|op| endpoints.get(&route(&op.operation)).copied())
        })
        .collect();
    let mut issues = Issues::default();
    let ctx = resources::Context {
        config: c,
        program,
        ops: &ops,
        resolved: &checked.resolved,
        endpoints: &methods,
        bases: flowapi_ts::sdk_bases(program),
    };
    out.files.extend(resources::files(&ctx, &mut issues));
    out.files
        .push(file("src/client.ts", entry::client(c, &service)));
    out.files.push(file("src/index.ts", entry::index(c)));
    out.issues.extend(issues.list);
    out
}

fn file(path: &str, text: String) -> File {
    File {
        path: path.to_owned(),
        text,
    }
}

/// `"METHOD /path"`, the method in upper case.
pub(crate) fn route(op: &flowapi::spec::Operation) -> String {
    format!("{} {}", op.method.to_ascii_uppercase(), op.path)
}

/// Words as a camel-case identifier (`listAll`).
pub(crate) fn camel(w: &[String]) -> String {
    let pascal = words::pascal(w);
    let mut chars = pascal.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_ascii_lowercase().to_string() + chars.as_str()
    })
}

/// Text inside a `/** */` comment, on one line: no `*/` can end it early.
pub(crate) fn doc(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("*/", "*\\/")
}
