//! The generated tests: `tests/api-resources/<resource>.test.mjs`, one file per resource with a
//! method, run by `node --test` over the built package (imported by its own name, so its
//! `exports` are tested too). Each file starts the mock server (`mock.rs`) on a free port and
//! calls every method of its resource with the parameters and body its operation requires,
//! each value an example of its schema; a page is read once, a stream to its end. A call the
//! mock refuses (a wrong path, method, missing parameter or media type) fails its test with the
//! mock's reason, and the route the mock matched is the operation's.

use super::example::{self, Value};
use super::sample;
use super::{Call, Ctx, HEADER, Kind, RepoFile, generated_file};
use crate::client::{camel, resources};
use crate::model::Auth;
use crate::out::json_str;
use crate::words;
use flowapi::spec::{Location, Operation, Payload};
use std::fmt::Write as _;

pub(crate) fn files(ctx: &Ctx<'_>) -> Vec<RepoFile> {
    let c = ctx.config;
    let mut out = Vec::new();
    for (i, _) in c.resources.iter().enumerate() {
        let calls: Vec<&Call> = ctx
            .calls
            .iter()
            .filter(|k| c.methods.get(k.method).is_some_and(|m| m.resource == i))
            .collect();
        if calls.is_empty() {
            continue;
        }
        let own = resources::path_of(c, i);
        let rel = own.strip_prefix("resources/").unwrap_or(&own);
        let up = "../".repeat(rel.split('/').count() + 1);
        let path = format!("tests/api-resources/{rel}.test.mjs");
        out.push(generated_file(&path, file(ctx, &c.path(i), &up, &calls)));
    }
    out
}

fn file(ctx: &Ctx<'_>, resource: &str, up: &str, calls: &[&Call]) -> String {
    let client = ctx.client();
    let mut out = format!(
        "// {HEADER}\n//\n// Every method of `{resource}` called against the mock server.\n\n"
    );
    out.push_str("import assert from 'node:assert/strict'\nimport { after, before, test } from 'node:test'\n");
    let _ = writeln!(
        out,
        "import {{ {client} }} from {}",
        json_str(ctx.package())
    );
    let _ = writeln!(out, "import {{ mock }} from '{up}mock/server.mjs'\n");
    out.push_str("let server\nlet client\n\nbefore(async () => {\n  server = await mock()\n");
    let _ = writeln!(
        out,
        "  client = new {client}({})\n}})\n\nafter(() => server.close())",
        options(ctx, Some("server.url"))
    );
    for call in calls {
        let Some(op) = ctx.ops.list.get(call.op) else {
            continue;
        };
        let route = format!("{} {}", op.method(), op.operation.path);
        let args = args(ctx, &op.operation, false)
            .map(|v| v.js())
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "\ntest({}, async () => {{",
            json_str(&format!("{}: {route}", call.name))
        );
        let expr = format!("client.{}({args})", call.access);
        match call.kind {
            Kind::Stream => {
                let _ = writeln!(
                    out,
                    "  for await (const event of await {expr}) assert.notEqual(event, undefined)"
                );
            }
            Kind::Page | Kind::Call => {
                let _ = writeln!(out, "  await {expr}");
            }
        }
        let _ = writeln!(
            out,
            "  if (server.requests !== null) assert.equal(server.requests.at(-1)?.route, {})\n}})",
            json_str(&route)
        );
    }
    out
}

/// The client's options: for a test (`base_url` given), that base URL, a made-up credential for
/// each scheme the config sends (a username and password for HTTP basic) and no retries; for the
/// README, the credentials alone, each `'…'`.
pub(crate) fn options(ctx: &Ctx<'_>, base_url: Option<&str>) -> String {
    let mut fields: Vec<String> = base_url
        .map(|b| format!("baseURL: {b}"))
        .into_iter()
        .collect();
    let (key, user, pass) = match base_url {
        Some(_) => ("'test-credential'", "'user'", "'pass'"),
        None => ("'…'", "'…'", "'…'"),
    };
    for a in &ctx.config.auth {
        let option = camel(&words::words(&a.name));
        let basic = ctx.service.schemes.iter().any(|s| {
            s.name == a.name && matches!(&s.auth, Auth::Http { scheme, .. } if scheme == "basic")
        });
        let value = if basic {
            format!("{{ username: {user}, password: {pass} }}")
        } else {
            key.to_owned()
        };
        let name = if option.starts_with(|c: char| c.is_ascii_alphabetic()) {
            option
        } else {
            json_str(&option)
        };
        fields.push(format!("{name}: {value}"));
    }
    if base_url.is_some() {
        fields.push(String::from("maxRetries: 0"));
    }
    format!("{{ {} }}", fields.join(", "))
}

/// A method's argument: the parameters its operation requires, by where they go, and its body
/// when one is required (JSON first, else the first media type); `None` when it needs nothing.
/// For the README (`readme`) a parameter's value is one for a person to read (`sample.rs`).
pub(crate) fn args(ctx: &Ctx<'_>, op: &Operation, readme: bool) -> Option<Value> {
    let doc = &ctx.ops.doc;
    let mut groups: Vec<(String, Value)> = Vec::new();
    for p in op.params.iter().filter(|p| p.required) {
        let value = if readme {
            sample::param(doc, op, p)
        } else {
            example::of(doc, p.schema.as_ref().map(|r| (r.doc, r.node)), false)
        };
        if p.location == Location::QueryString {
            groups.push((String::from("querystring"), value));
            continue;
        }
        let group = p.location.name().to_owned();
        let at = match groups.iter().position(|(g, _)| *g == group) {
            Some(at) => at,
            None => {
                groups.push((group, Value::Obj(Vec::new())));
                groups.len() - 1
            }
        };
        if let Some((_, Value::Obj(members))) = groups.get_mut(at) {
            members.push((p.name.clone(), value));
        }
    }
    if let Some(body) = op.body.as_ref().filter(|b| b.required) {
        let media = body
            .content
            .iter()
            .find(|m| m.media_type.eq_ignore_ascii_case("application/json"))
            .or_else(|| body.content.first());
        if let Some(m) = media {
            let value = match &m.payload {
                Payload::Schema(s) => example::of(doc, s.as_ref().map(|r| (r.doc, r.node)), false),
                Payload::Form(fields) => Value::Obj(
                    fields
                        .iter()
                        .filter(|f| f.required)
                        .map(|f| {
                            let at = f.schema.as_ref().map(|r| (r.doc, r.node));
                            (f.name.clone(), example::of(doc, at, false))
                        })
                        .collect(),
                ),
            };
            let body = vec![
                (
                    String::from("contentType"),
                    Value::Str(m.media_type.clone()),
                ),
                (String::from("value"), value),
            ];
            groups.push((String::from("body"), Value::Obj(body)));
        }
    }
    (!groups.is_empty()).then_some(Value::Obj(groups))
}
