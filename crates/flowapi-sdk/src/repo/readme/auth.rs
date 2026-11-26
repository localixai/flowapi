//! The README's authentication: for each scheme of `auth`, its kind, where it goes, the option
//! it is given by and the environment variables it is read from, with their exact format; for
//! OAuth 2.0's client credentials the token URL and the scopes; then how the client chooses among
//! an operation's alternatives, and that credentials go only over TLS. A test holds that every
//! scheme of `auth` is in it.

use crate::client::camel;
use crate::config::auth::{CHALLENGED, has_token_flow, is_basic};
use crate::config::model::Credential;
use crate::model::{Auth, KeyLocation, Scheme};
use crate::repo::Ctx;
use crate::repo::suite::options;
use crate::words;
use std::fmt::Write as _;

const HOW: &str = "An operation's security is a list of alternatives: the client sends the first one it holds every credential of, and an operation that needs none sends none. Each credential is an option of the client, else read from its environment variables when a call needs it; a call none of whose alternatives can be met throws `CredentialError`, naming the options and variables, before anything is sent. Credentials go only over `https:`, or to `localhost`, `127.0.0.1` and `[::1]`: elsewhere a call over plain `http:` throws `InsecureCredentialError`, unless the client is given `allowInsecureCredentials: true`. The log writes each as `***`, and a redirect to another origin is not followed unless `allowCrossOriginRedirects` is set, and then without them.";

/// A scheme's kind, where it goes, what its option takes and its variables' format.
fn row(a: &Credential, s: &Scheme) -> [String; 4] {
    let var = |member: &str| a.var(member).map(|v| format!("`{v}`"));
    let one = |what: &str| var("env_var").map(|v| format!("{v}: {what}"));
    let (kind, sent, option, vars) = match &s.auth {
        Auth::ApiKey { location, name } => {
            let place = match location {
                KeyLocation::Header => "the header",
                KeyLocation::Query => "the query parameter",
                KeyLocation::Cookie => "the cookie",
            };
            (
                String::from("API key"),
                format!("{place} `{name}`"),
                String::from("the key"),
                one("the key"),
            )
        }
        auth if is_basic(auth) => {
            let pair = var("username_env_var")
                .zip(var("password_env_var"))
                .map(|(u, p)| format!("{u}: the user-id; {p}: the password"));
            let vars = [one("`username:password`"), pair].into_iter().flatten();
            (
                String::from("HTTP basic"),
                String::from("`Authorization: Basic <base64 of user-id:password, UTF-8>`"),
                String::from("`{ username, password }` (a user-id without `:`)"),
                Some(vars.collect::<Vec<_>>().join("; ")).filter(|v| !v.is_empty()),
            )
        }
        Auth::Http { scheme, .. } if scheme == "bearer" => (
            String::from("HTTP bearer"),
            String::from("`Authorization: Bearer <token>`"),
            String::from("the token"),
            one("the token"),
        ),
        Auth::Http { scheme, .. } if CHALLENGED.contains(&scheme.as_str()) => (
            format!("HTTP `{scheme}`"),
            String::from(
                "not sent: it answers a challenge; give `Authorization` in a call's `headers`",
            ),
            String::from("—"),
            None,
        ),
        Auth::Http { scheme, .. } => (
            format!("HTTP `{scheme}`"),
            format!("`Authorization: {scheme} <credential>`"),
            String::from("the credential"),
            one("the credential"),
        ),
        Auth::OAuth2 {
            client_credentials: Some(flow),
            ..
        } => {
            let scopes = a.scopes.as_ref().map_or_else(
                || String::from("the scopes the operation needs"),
                |(s, _)| words_of(s, "no scope"),
            );
            let pair = var("client_id_env_var")
                .zip(var("client_secret_env_var"))
                .map(|(i, s)| format!("{i}: the client's id; {s}: its secret"));
            let vars = [one("a token"), pair].into_iter().flatten();
            (
                String::from("OAuth 2.0 client credentials"),
                format!(
                    "`Authorization: Bearer <token>`, the token requested at `{}` for {scopes}",
                    flow.token_url
                ),
                String::from("a token, or `{ clientId, clientSecret }` to request one"),
                Some(vars.collect::<Vec<_>>().join("; ")).filter(|v| !v.is_empty()),
            )
        }
        Auth::OAuth2 { .. } | Auth::OpenIdConnect { .. } => (
            String::from(if matches!(s.auth, Auth::OAuth2 { .. }) {
                "OAuth 2.0"
            } else {
                "OpenID Connect"
            }),
            String::from("`Authorization: Bearer <token>`, a token you obtain"),
            String::from("the token, or a function that gives it"),
            one("the token"),
        ),
        Auth::MutualTls => (
            String::from("mutual TLS"),
            String::from("a client certificate, which your `fetch` presents"),
            String::from("`true`"),
            None,
        ),
    };
    let deprecated = if s.deprecated { " (deprecated)" } else { "" };
    [
        format!("{kind}{deprecated}"),
        sent,
        option,
        vars.unwrap_or_else(|| String::from("—")),
    ]
}

/// Scopes as words: `` `a`, `b` ``.
fn words_of(scopes: &[String], none: &str) -> String {
    if scopes.is_empty() {
        return String::from(none);
    }
    let list: Vec<String> = scopes.iter().map(|s| format!("`{s}`")).collect();
    format!("the scopes {}", list.join(", "))
}

pub(crate) fn section(ctx: &Ctx<'_>, out: &mut String) {
    if ctx.config.auth.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n## Authentication\n\n| Scheme | Kind | Sent as | Option | Environment variables |\n| --- | --- | --- | --- | --- |"
    );
    let mut flow = false;
    for a in &ctx.config.auth {
        let Some(s) = ctx.service.schemes.iter().find(|s| s.name == a.name.name) else {
            continue;
        };
        flow |= has_token_flow(&s.auth);
        let [kind, sent, option, vars] = row(a, s);
        let _ = writeln!(
            out,
            "| `{}` | {kind} | {sent} | `{}`: {option} | {vars} |",
            a.name.name,
            camel(&words::words(&a.name.name))
        );
    }
    let _ = writeln!(out, "\n{HOW}");
    if flow {
        out.push_str("\nA token of OAuth 2.0's client credentials grant is requested at the token URL (relative to the base URL where it is relative) with the client's id and secret in HTTP basic, cached until a little before it expires, shared by the calls that wait for it, and requested again once when the API answers 401. A refusal of the token endpoint throws `TokenError` with its `error` and `description`.\n");
    }
    let _ = writeln!(
        out,
        "\n```ts\nconst client = new {}({})\n```",
        ctx.client(),
        options(ctx, None)
    );
}
