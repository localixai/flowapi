//! The generated tests: `tests/api-resources/<resource>.test.mjs`, one file per resource with a
//! method, run by `node --test` over the built package (imported by its own name, so its
//! `exports` are tested too). Each file starts the mock server (`mock.rs`) on a free port and
//! calls every method of its resource with the parameters and body its operation requires,
//! each value an example of its schema; a page is read once, a stream to its end. A call the
//! mock refuses (a wrong path, method, missing parameter or media type) fails its test with the
//! mock's reason, and the route the mock matched is the operation's. `tests/auth.test.mjs`
//! (`suite/auth.rs`) holds what the mock received of each kind of credential.

mod auth;

use super::example::{self, Value};
use super::{Call, Ctx, HEADER, Kind, RepoFile, generated_file};
use super::{mock, sample};
use crate::client::{camel, resources};
use crate::config::auth::{CHALLENGED, is_basic};
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
    out.extend(auth::file(ctx));
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
        let expr = match auth::call_options(ctx, call) {
            Some(o) if args.is_empty() => format!("client.{}({{}}, {o})", call.access),
            Some(o) => format!("client.{}({args}, {o})", call.access),
            None => format!("client.{}({args})", call.access),
        };
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
/// each scheme the config sends (a username and password for HTTP basic, a client's id and
/// secret for the client credentials flow, its token endpoint the mock's) and no retries; for the
/// README, the credentials alone, each `'…'`. A scheme the client cannot send has no option.
pub(crate) fn options(ctx: &Ctx<'_>, base_url: Option<&str>) -> String {
    let mut fields: Vec<String> = base_url
        .map(|b| format!("baseURL: {b}"))
        .into_iter()
        .collect();
    let test = base_url.is_some();
    let quote = |v: &str| {
        if test {
            format!("'{v}'")
        } else {
            String::from("'…'")
        }
    };
    for a in &ctx.config.auth {
        let Some(scheme) = ctx.service.schemes.iter().find(|s| s.name == a.name.name) else {
            continue;
        };
        let value = match &scheme.auth {
            auth if is_basic(auth) => format!(
                "{{ username: {}, password: {} }}",
                quote(auth::USERNAME),
                quote(auth::PASSWORD)
            ),
            Auth::OAuth2 {
                client_credentials: Some(f),
                ..
            } => {
                let url = base_url
                    .map(|b| format!(", tokenURL: `${{{b}}}{}`", mock::token_path(&f.token_url)))
                    .unwrap_or_default();
                format!(
                    "{{ clientId: {}, clientSecret: {}{url} }}",
                    quote(auth::CLIENT_ID),
                    quote(auth::CLIENT_SECRET)
                )
            }
            Auth::Http { scheme, .. } if CHALLENGED.contains(&scheme.as_str()) => continue,
            Auth::MutualTls => String::from("true"),
            Auth::ApiKey { .. }
            | Auth::Http { .. }
            | Auth::OAuth2 { .. }
            | Auth::OpenIdConnect { .. } => quote(auth::CREDENTIAL),
        };
        fields.push(format!("{}: {value}", option_key(&a.name.name)));
    }
    if test {
        fields.push(String::from("maxRetries: 0"));
    }
    format!("{{ {} }}", fields.join(", "))
}

/// A credential's option as a key of an object literal.
pub(crate) fn option_key(scheme: &str) -> String {
    let option = camel(&words::words(scheme));
    if option.starts_with(|c: char| c.is_ascii_alphabetic()) {
        option
    } else {
        json_str(&option)
    }
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
