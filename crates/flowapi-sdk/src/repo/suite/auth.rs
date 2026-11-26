//! `tests/auth.test.mjs`: for each kind of credential the config sends, a method whose
//! operation the tests' client meets by it, called against the mock, and what the mock received
//! asserted (the header, the query parameter or the cookie, HTTP basic's pair, the bearer token,
//! the token the mock gave for the client credentials flow); and an operation with no security
//! called by a client that holds no credential, which sends none. Every credential is made up
//! and the same on every run.

use super::super::{Call, Ctx, HEADER, Kind, RepoFile, generated_file};
use super::{args, option_key};
use crate::config::auth::{CHALLENGED, has_token_flow, is_basic, needed};
use crate::model::{Auth, KeyLocation, Requirement, Scheme};
use crate::out::json_str;
use std::fmt::Write as _;

/// The made-up credentials of the tests' client.
pub(crate) const CREDENTIAL: &str = "test-credential";
pub(crate) const USERNAME: &str = "user";
pub(crate) const PASSWORD: &str = "pass";
pub(crate) const CLIENT_ID: &str = "test-client";
pub(crate) const CLIENT_SECRET: &str = "test-secret";
/// The token the mock gives for a client credentials grant (`files/server.mjs`).
const MOCK_TOKEN: &str = "mock-token";

/// The scheme of a name the config sends, when the client can send it.
fn sendable<'s>(ctx: &'s Ctx<'_>, name: &str) -> Option<&'s Scheme> {
    ctx.config.auth.iter().find(|a| a.name.name == name)?;
    let s = ctx.service.schemes.iter().find(|s| s.name == name)?;
    match &s.auth {
        Auth::Http { scheme, .. } if CHALLENGED.contains(&scheme.as_str()) => None,
        Auth::ApiKey { .. }
        | Auth::Http { .. }
        | Auth::OAuth2 { .. }
        | Auth::OpenIdConnect { .. }
        | Auth::MutualTls => Some(s),
    }
}

/// Whether a scheme's credential goes in `Authorization`.
fn authorization(auth: &Auth) -> bool {
    !matches!(auth, Auth::ApiKey { .. } | Auth::MutualTls)
}

/// The alternative the tests' client sends: the first it holds every credential of.
fn chosen<'r>(ctx: &Ctx<'_>, security: &'r [Requirement]) -> Option<&'r Requirement> {
    security
        .iter()
        .find(|r| r.schemes.iter().all(|(n, _)| sendable(ctx, n).is_some()))
}

/// What the mock must have received of a scheme, as assertions on `r`.
fn expected(s: &Scheme) -> Option<String> {
    let line = match &s.auth {
        Auth::ApiKey { location, name } => match location {
            KeyLocation::Header => format!(
                "assert.equal(r.headers[{}], '{CREDENTIAL}')",
                json_str(&name.to_ascii_lowercase())
            ),
            KeyLocation::Query => format!(
                "assert.equal(new URL(r.url, 'http://mock.invalid').searchParams.get({}), '{CREDENTIAL}')",
                json_str(name)
            ),
            KeyLocation::Cookie => format!(
                "assert.ok((r.headers.cookie ?? '').split('; ').includes({}), r.headers.cookie)",
                json_str(&format!("{name}={CREDENTIAL}"))
            ),
        },
        auth if is_basic(auth) => format!(
            "assert.equal(r.headers.authorization, `Basic ${{Buffer.from('{USERNAME}:{PASSWORD}').toString('base64')}}`)"
        ),
        auth if has_token_flow(auth) => format!(
            "assert.equal(r.headers.authorization, 'Bearer {MOCK_TOKEN}')\n  assert.ok(server.requests.some((q) => q.route === 'token'), 'the token was requested of the mock')"
        ),
        Auth::Http { scheme, .. } if scheme != "bearer" => {
            format!("assert.equal(r.headers.authorization, '{scheme} {CREDENTIAL}')")
        }
        Auth::MutualTls => return None,
        Auth::Http { .. } | Auth::OAuth2 { .. } | Auth::OpenIdConnect { .. } => {
            format!("assert.equal(r.headers.authorization, 'Bearer {CREDENTIAL}')")
        }
    };
    Some(line)
}

/// A call of a method, as a statement that runs it to its end.
fn statement(ctx: &Ctx<'_>, client: &str, call: &Call) -> String {
    let args = ctx
        .ops
        .list
        .get(call.op)
        .and_then(|op| args(ctx, &op.operation, false))
        .map(|v| v.js())
        .unwrap_or_default();
    let expr = format!("{client}.{}({args})", call.access);
    match call.kind {
        Kind::Stream => {
            format!("for await (const event of await {expr}) assert.notEqual(event, undefined)")
        }
        Kind::Page | Kind::Call => format!("await {expr}"),
    }
}

/// Each method's security, as the client sends it.
fn security(ctx: &Ctx<'_>, call: &Call) -> Vec<Requirement> {
    ctx.ops.list.get(call.op).map_or_else(Vec::new, |op| {
        needed(ctx.config, ctx.ops, ctx.config.methods.get(call.method), op)
    })
}

pub(crate) fn file(ctx: &Ctx<'_>) -> Option<RepoFile> {
    let mut tests = String::new();
    for a in &ctx.config.auth {
        let Some(s) = sendable(ctx, &a.name.name) else {
            continue;
        };
        let Some(check) = expected(s) else { continue };
        let found = ctx.calls.iter().find(|call| {
            let security = security(ctx, call);
            chosen(ctx, &security).is_some_and(|r| {
                r.schemes.iter().any(|(n, _)| *n == s.name)
                    && !(authorization(&s.auth)
                        && r.schemes.iter().any(|(n, _)| {
                            *n != s.name && sendable(ctx, n).is_some_and(|o| authorization(&o.auth))
                        }))
            })
        });
        let Some(call) = found else { continue };
        let _ = write!(
            tests,
            "\ntest({}, async () => {{\n  {}\n  const r = last()\n  if (r === null) return\n  {check}\n}})\n",
            json_str(&format!("{}: sent by {}", s.name, call.name)),
            statement(ctx, "client", call)
        );
    }
    let open = ctx.calls.iter().find(|call| security(ctx, call).is_empty());
    if let Some(call) = open {
        let nulls: Vec<String> = ctx
            .config
            .auth
            .iter()
            .map(|a| format!(", {}: null", option_key(&a.name.name)))
            .collect();
        let _ = write!(
            tests,
            "\ntest({}, async () => {{\n  const none = new {}({{ baseURL: server.url, maxRetries: 0{} }})\n  {}\n  const r = last()\n  if (r === null) return\n  assert.equal(r.headers.authorization, undefined)\n{}}})\n",
            json_str(&format!("no credential: {} needs none", call.name)),
            ctx.client(),
            nulls.concat(),
            statement(ctx, "none", call),
            absent(ctx)
        );
    }
    if tests.is_empty() {
        return None;
    }
    let client = ctx.client();
    let mut out = format!(
        "// {HEADER}\n//\n// What the mock received of each kind of credential the client sends, and a call that needs none.\n\n"
    );
    out.push_str("import assert from 'node:assert/strict'\nimport { after, before, test } from 'node:test'\n");
    let _ = writeln!(
        out,
        "import {{ {client} }} from {}",
        json_str(ctx.package())
    );
    out.push_str("import { mock } from '../mock/server.mjs'\n\nlet server\nlet client\n\nbefore(async () => {\n  server = await mock()\n");
    let _ = writeln!(
        out,
        "  client = new {client}({})\n}})\n\nafter(() => server.close())\n\n/** The request the mock answered last; `null` against a server of your own. */\nconst last = () => server.requests?.at(-1) ?? null",
        super::options(ctx, Some("server.url"))
    );
    out.push_str(&tests);
    Some(generated_file("tests/auth.test.mjs", out))
}

/// The assertions that a request carries no API key of the config's schemes.
fn absent(ctx: &Ctx<'_>) -> String {
    let mut out = String::new();
    for a in &ctx.config.auth {
        let Some(Auth::ApiKey { location, name }) = ctx
            .service
            .schemes
            .iter()
            .find(|s| s.name == a.name.name)
            .map(|s| &s.auth)
        else {
            continue;
        };
        let line = match location {
            KeyLocation::Header => format!(
                "assert.equal(r.headers[{}], undefined)",
                json_str(&name.to_ascii_lowercase())
            ),
            KeyLocation::Query => format!(
                "assert.equal(new URL(r.url, 'http://mock.invalid').searchParams.has({}), false)",
                json_str(name)
            ),
            KeyLocation::Cookie => format!(
                "assert.ok(!(r.headers.cookie ?? '').split('; ').some((c) => c.startsWith({})))",
                json_str(&format!("{name}="))
            ),
        };
        let _ = writeln!(out, "  {line}");
    }
    out
}

/// The options a call needs when the client's own credentials meet none of its security's
/// alternatives but one whose other schemes answer a challenge: their `Authorization`, given in
/// the call's `headers` as the client's `CredentialError` says to.
pub(crate) fn call_options(ctx: &Ctx<'_>, call: &Call) -> Option<String> {
    let security = security(ctx, call);
    if security.is_empty() || chosen(ctx, &security).is_some() {
        return None;
    }
    security.iter().find_map(|r| {
        let mut challenged = None;
        for (n, _) in &r.schemes {
            if sendable(ctx, n).is_some() {
                continue;
            }
            match ctx
                .service
                .schemes
                .iter()
                .find(|s| s.name == *n)
                .map(|s| &s.auth)
            {
                Some(Auth::Http { scheme, .. }) if CHALLENGED.contains(&scheme.as_str()) => {
                    challenged = Some(scheme.clone());
                }
                Some(_) | None => return None,
            }
        }
        challenged
            .map(|scheme| format!("{{ headers: {{ Authorization: '{scheme} {CREDENTIAL}' }} }}"))
    })
}
