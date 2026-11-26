//! The draft's `auth` and `security`: each security scheme of the description, with the
//! environment variables it is read from (HTTP basic's as `username:password`, a client
//! credentials flow's id and secret beside a ready token); one no variable can give (a client
//! certificate, an HTTP scheme that answers a challenge) listed with none, and it and a
//! deprecated one said why in a comment. `config check` warns of those three whatever the config
//! says, as they are the description's, and so does `config.auth.none` when no operation needs a
//! credential: they are the only issues a draft may have, and none is an error. In that case the
//! draft leaves `security` unset, with a commented example of declaring one, never writing
//! `security: []` (the config's own statement that the API takes no credentials, which would
//! silence the warning): the person, or the agent, decides from the API's own documentation and
//! never by a guess.

use crate::config::auth::{CHALLENGED, has_token_flow, is_basic};
use crate::doc::ROOT;
use crate::model::Auth;
use crate::ops::Ops;
use crate::out::yaml_str;
use crate::words::{upper, words};
use flowapi::Diagnostics;
use std::fmt::Write as _;

/// What a scheme no variable can give is, when it is one.
fn without_env(auth: &Auth) -> Option<&'static str> {
    match auth {
        Auth::MutualTls => Some("a client certificate: give the client a `fetch` that presents it"),
        Auth::Http { scheme, .. } if CHALLENGED.contains(&scheme.as_str()) => Some(
            "an HTTP scheme that answers a challenge: give `Authorization` in a call's `headers`",
        ),
        Auth::ApiKey { .. }
        | Auth::Http { .. }
        | Auth::OAuth2 { .. }
        | Auth::OpenIdConnect { .. } => None,
    }
}

/// Whether some operation needs a credential, by its own `security` or the description's.
fn needed(ops: &Ops<'_, '_>) -> bool {
    let mut quiet = Diagnostics::default();
    let root = ops.doc.security(ROOT, &mut quiet).unwrap_or_default();
    ops.list.iter().any(|op| {
        let (own, _) = ops.doc.effective(&op.operation, &root, &mut quiet);
        own.iter().any(|r| !r.schemes.is_empty())
    })
}

pub(super) fn auth(out: &mut String, ops: &Ops<'_, '_>, title: &[String]) {
    let mut schemes = ops.doc.schemes(&mut Diagnostics::default());
    schemes.sort_by(|a, b| a.name.cmp(&b.name));
    let var = |name: &str, more: &[&str]| {
        let mut w = [title.to_vec(), words(name)].concat();
        w.extend(more.iter().map(|m| (*m).to_owned()));
        upper(&w)
    };
    let mut listed = String::new();
    for s in &schemes {
        if s.deprecated {
            let _ = writeln!(listed, "  # deprecated by the description: refrain from it");
        }
        if let Some(what) = without_env(&s.auth) {
            let _ = writeln!(listed, "  # {what}\n  {}: {{}}", yaml_str(&s.name));
            continue;
        }
        let _ = writeln!(listed, "  {}:", yaml_str(&s.name));
        if is_basic(&s.auth) {
            let _ = writeln!(
                listed,
                "    env_var: {}  # username:password",
                var(&s.name, &[])
            );
        } else {
            let _ = writeln!(listed, "    env_var: {}", var(&s.name, &[]));
        }
        if has_token_flow(&s.auth) {
            let _ = writeln!(
                listed,
                "    client_id_env_var: {}\n    client_secret_env_var: {}",
                var(&s.name, &["client", "id"]),
                var(&s.name, &["client", "secret"])
            );
        }
    }
    if !listed.is_empty() {
        out.push_str("auth:\n");
        out.push_str(&listed);
    }
    if needed(ops) {
        return;
    }
    let first = schemes.first().map(|s| s.name.as_str());
    let example = var("token", &[]);
    out.push_str(
        "# The description declares no security scheme that an operation uses. Most APIs take a\n\
         # credential: find it in the API's documentation, declare it under `auth` and name it in\n\
         # `security`, as in the example below, or write `security: []` when the API truly takes\n\
         # none. Until then `config check` warns `config.auth.none`.\n",
    );
    if first.is_none() {
        let _ = writeln!(
            out,
            "# auth:\n#   token: {{ type: http, scheme: bearer, env_var: {example} }}"
        );
    }
    let _ = writeln!(
        out,
        "# security:\n#   - {}: []",
        yaml_str(first.unwrap_or("token"))
    );
}
