//! `auth` and `security`, the credentials a client sends: the schemes the description declares,
//! each given its options (`env_var`, ...), and those it lacks, declared by the config with
//! `type` and the members of their kind, as OpenAPI's Security Scheme Object names them; then
//! `security` at the top, over every operation with none of its own (as the description's
//! `security` is), and a method's own over everything. A scheme of the description may only be
//! given options: what it is stays the description's. `auth/usage.rs` holds the config to what
//! the operations then need. [`apply`] gives the service the client is printed from.

mod usage;

use super::issue::{Issues, closest};
use super::model::{Config, Credential, Method, Security};
use crate::model::{Auth, KeyLocation, Requirement, Scheme, Service, TokenFlow};
use crate::ops::{Op, Ops};
use flowapi::Diagnostics;

/// HTTP authentication schemes (IANA's registry) a client cannot send up front: each answers a
/// challenge of the server's, or signs each request with a proof of its own.
pub(crate) const CHALLENGED: &[&str] = &[
    "concealed",
    "digest",
    "dpop",
    "hoba",
    "mutual",
    "negotiate",
    "oauth",
    "privatetoken",
    "scram-sha-1",
    "scram-sha-256",
];

/// Whether a scheme's credential is HTTP basic's pair.
pub(crate) fn is_basic(auth: &Auth) -> bool {
    matches!(auth, Auth::Http { scheme, .. } if scheme == "basic")
}

/// Whether the client may request a scheme's token itself (OAuth 2.0 client credentials).
pub(crate) fn has_token_flow(auth: &Auth) -> bool {
    matches!(
        auth,
        Auth::OAuth2 {
            client_credentials: Some(_),
            ..
        }
    )
}

/// The scheme a credential declares, when its `type` and the members of its kind are given.
fn declared(a: &Credential) -> Option<Auth> {
    let value = |n: &Option<super::model::Named>| n.as_ref().map(|n| n.name.clone());
    Some(match a.kind.as_ref()?.name.as_str() {
        "apiKey" => Auth::ApiKey {
            location: match value(&a.location)?.as_str() {
                "header" => KeyLocation::Header,
                "query" => KeyLocation::Query,
                "cookie" => KeyLocation::Cookie,
                _ => return None,
            },
            name: value(&a.key)?,
        },
        "http" => Auth::Http {
            scheme: value(&a.scheme)?.to_ascii_lowercase(),
            format: None,
        },
        "oauth2" => {
            let flow = value(&a.token_url).map(|token_url| TokenFlow {
                token_url,
                scopes: Vec::new(),
            });
            Auth::OAuth2 {
                flows: flow
                    .iter()
                    .map(|_| String::from("clientCredentials"))
                    .collect(),
                client_credentials: flow,
            }
        }
        "mutualTLS" => Auth::MutualTls,
        _ => return None,
    })
}

/// The description's schemes, then each one the config declares, in the config's order.
pub(crate) fn schemes(c: &Config, described: Vec<Scheme>) -> Vec<Scheme> {
    let mut out = described;
    for a in &c.auth {
        if out.iter().any(|s| s.name == a.name.name) {
            continue;
        }
        if let Some(auth) = declared(a) {
            out.push(Scheme {
                name: a.name.name.clone(),
                auth,
                description: a.description.clone(),
                deprecated: false,
            });
        }
    }
    out
}

/// A config's `security` as requirements.
pub(crate) fn requirements(s: &Security) -> Vec<Requirement> {
    s.alternatives
        .iter()
        .map(|alt| Requirement {
            schemes: alt
                .iter()
                .map(|(n, scopes)| (n.name.clone(), scopes.clone()))
                .collect(),
        })
        .collect()
}

/// An operation's security once the config is over it: the method's own, else (where the
/// operation has none of its own) the config's top-level, else the description's.
pub(crate) fn over(
    c: &Config,
    method: Option<&Method>,
    security: Vec<Requirement>,
    inherits: bool,
) -> Vec<Requirement> {
    if let Some(own) = method.and_then(|m| m.security.as_ref()) {
        return requirements(own);
    }
    match &c.security {
        Some(top) if inherits => requirements(top),
        Some(_) | None => security,
    }
}

/// What a method's operation needs once the config is over it (`over`), read from the
/// description.
pub(crate) fn needed(
    c: &Config,
    ops: &Ops<'_, '_>,
    method: Option<&Method>,
    op: &Op,
) -> Vec<Requirement> {
    let root = ops
        .doc
        .security(crate::doc::ROOT, &mut Diagnostics::default())
        .unwrap_or_default();
    let (own, inherits) = ops
        .doc
        .effective(&op.operation, &root, &mut Diagnostics::default());
    over(c, method, own, inherits)
}

/// The service a client is printed from: the description's, with the schemes the config
/// declares and its `security` over the operations'. `endpoints` is each method's endpoint.
pub(crate) fn apply(c: &Config, service: &mut Service, endpoints: &[Option<usize>]) {
    service.schemes = schemes(c, std::mem::take(&mut service.schemes));
    for (e, access) in service.operations.iter_mut().enumerate() {
        let method = c
            .methods
            .iter()
            .zip(endpoints)
            .find(|(_, at)| **at == Some(e))
            .map(|(m, _)| m);
        let security = std::mem::take(&mut access.security);
        access.security = over(c, method, security, access.inherits);
    }
}

/// `auth` and `security` held to the description and to what each method's operation needs.
pub(crate) fn check(
    c: &Config,
    ops: &Ops<'_, '_>,
    resolved: &[Option<usize>],
    issues: &mut Issues,
) {
    let described = ops.doc.schemes(&mut Diagnostics::default());
    for a in &c.auth {
        credential(a, &described, issues);
    }
    let all = schemes(c, described.clone());
    let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();
    let securities = c
        .security
        .iter()
        .chain(c.methods.iter().filter_map(|m| m.security.as_ref()));
    for s in securities {
        for (n, _) in s.alternatives.iter().flatten() {
            if !names.contains(&n.name.as_str()) {
                let hint =
                    closest(&n.name, names.iter().copied()).map(|n| format!("did you mean `{n}`?"));
                let message = format!(
                    "`{}` names no security scheme of the description or of `auth`",
                    n.name
                );
                issues.error_hint("config.security.unknown", &n.at, message, hint);
            }
        }
    }
    usage::check(c, ops, resolved, &all, issues);
}

/// One credential: a scheme of the description given options, or one the config declares.
fn credential(a: &Credential, described: &[Scheme], issues: &mut Issues) {
    let name = &a.name.name;
    let auth = match described.iter().find(|s| s.name == *name) {
        Some(s) => {
            if let Some(n) = a.declares() {
                let message = format!(
                    "`{name}` is a security scheme of the description: the config may give it options (its environment variables, `scopes`), not say again what it is"
                );
                issues.error("config.auth.declared", &n.at, message);
            }
            if s.deprecated {
                let message = format!(
                    "`{name}` is deprecated by the description: its consumers should refrain from it"
                );
                issues.warn("config.auth.deprecated", &a.name.at, message);
            }
            s.auth.clone()
        }
        None => {
            let Some(auth) = declaration(a, described, issues) else {
                return;
            };
            auth
        }
    };
    options(a, &auth, issues);
    match &auth {
        Auth::Http { scheme, .. } if CHALLENGED.contains(&scheme.as_str()) => {
            let message = format!(
                "`{name}` is HTTP `{scheme}`, which answers a challenge or signs each request: the client cannot send it; give `Authorization` in a call's `headers`"
            );
            issues.warn("config.auth.refused", &a.name.at, message);
        }
        Auth::MutualTls => {
            let message = format!(
                "`{name}` is a client certificate, which no request carries: give the client a `fetch` (or `fetchOptions`) that presents it, and the option `true` to say so"
            );
            issues.warn("config.auth.mutual_tls", &a.name.at, message);
        }
        Auth::ApiKey { .. }
        | Auth::Http { .. }
        | Auth::OAuth2 { .. }
        | Auth::OpenIdConnect { .. } => {}
    }
}

/// A scheme the description lacks, as the config declares it; `None`, with an error, when it
/// cannot be read.
fn declaration(a: &Credential, described: &[Scheme], issues: &mut Issues) -> Option<Auth> {
    let name = &a.name.name;
    let Some(kind) = &a.kind else {
        let message = if described.is_empty() {
            format!("`{name}` names no security scheme: the description has none")
        } else {
            format!("`{name}` names no security scheme of the description")
        };
        let hint = closest(name, described.iter().map(|s| s.name.as_str()))
            .map(|n| format!("did you mean `{n}`?"))
            .or_else(|| Some(String::from("give `type` (and the members of its kind) to declare a scheme the description lacks")));
        issues.error_hint("config.auth.unknown", &a.name.at, message, hint);
        return None;
    };
    let wants: &[&str] = match kind.name.as_str() {
        "apiKey" => &["in", "name"],
        "http" => &["scheme"],
        "oauth2" | "mutualTLS" => &[],
        _ => return None,
    };
    let given = [
        ("scheme", &a.scheme),
        ("in", &a.location),
        ("name", &a.key),
        ("token_url", &a.token_url),
    ];
    for (member, value) in given {
        let allowed = wants.contains(&member) || (member == "token_url" && kind.name == "oauth2");
        match value {
            Some(v) if !allowed => {
                let message = format!(
                    "`{member}` is no member of a scheme of type `{}`",
                    kind.name
                );
                issues.error("config.auth.field", &v.at, message);
            }
            None if wants.contains(&member) => {
                let message = format!("`{name}`, of type `{}`, needs `{member}`", kind.name);
                issues.error("config.auth.field", &kind.at, message);
            }
            Some(_) | None => {}
        }
    }
    declared(a)
}

/// The options a credential gives, held to its scheme's kind: each pair given whole.
fn options(a: &Credential, auth: &Auth, issues: &mut Issues) {
    let pairs: [(&str, &str, bool, &str); 2] = [
        (
            "username_env_var",
            "password_env_var",
            is_basic(auth),
            "HTTP basic",
        ),
        (
            "client_id_env_var",
            "client_secret_env_var",
            has_token_flow(auth),
            "OAuth 2.0 with a client credentials flow",
        ),
    ];
    for (one, other, fits, kind) in pairs {
        for (member, pair) in [(one, other), (other, one)] {
            let Some((_, n)) = a.env.iter().find(|(m, _)| *m == member) else {
                continue;
            };
            if !fits {
                let message = format!("`{member}` is for {kind} only");
                issues.error("config.auth.option", &n.at, message);
            } else if a.var(pair).is_none() {
                let message = format!("`{member}` is given without `{pair}`");
                issues.error("config.auth.option", &n.at, message);
            }
        }
    }
    if let Some((_, at)) = &a.scopes
        && !has_token_flow(auth)
    {
        let message = String::from(
            "`scopes` is for OAuth 2.0 with a client credentials flow only: other tokens are the caller's",
        );
        issues.error("config.auth.option", at, message);
    }
    if let (Auth::MutualTls, Some((_, n))) = (auth, a.env.first()) {
        let message = String::from("a client certificate is read from no environment variable");
        issues.error("config.auth.option", &n.at, message);
    }
}
