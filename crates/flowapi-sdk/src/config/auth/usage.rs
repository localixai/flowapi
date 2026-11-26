//! `auth` held to what the methods' operations need, once the config's `security` is over
//! theirs: a scheme they need that `auth` does not list is sent by no credential, one listed
//! with no environment variable only by its option, and a config whose operations need no
//! scheme at all sends nothing (`config.auth.none`), unless its top-level `security: []` says
//! so on purpose: an API that needs a credential its description does not declare needs `auth`
//! and `security` here.

use super::{CHALLENGED, has_token_flow, is_basic, needed};
use crate::config::check::root;
use crate::config::issue::Issues;
use crate::config::model::Config;
use crate::model::{Auth, Scheme};
use crate::ops::Ops;

/// Each scheme some method's operation needs, with the first method that needs it.
fn used(c: &Config, ops: &Ops<'_, '_>, resolved: &[Option<usize>]) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for (i, (m, op)) in c.methods.iter().zip(resolved).enumerate() {
        let Some(op) = op.and_then(|o| ops.list.get(o)) else {
            continue;
        };
        for r in needed(c, ops, Some(m), op) {
            for (name, _) in r.schemes {
                if !out.iter().any(|(n, _)| *n == name) {
                    out.push((name, i));
                }
            }
        }
    }
    out
}

/// Whether a credential names the environment variables its scheme is read from.
fn has_env(c: &Config, name: &str, auth: &Auth) -> bool {
    let Some(a) = c.auth.iter().find(|a| a.name.name == name) else {
        return false;
    };
    let pair = |x: &str, y: &str| a.var(x).is_some() && a.var(y).is_some();
    a.var("env_var").is_some()
        || (is_basic(auth) && pair("username_env_var", "password_env_var"))
        || (has_token_flow(auth) && pair("client_id_env_var", "client_secret_env_var"))
}

pub(super) fn check(
    c: &Config,
    ops: &Ops<'_, '_>,
    resolved: &[Option<usize>],
    schemes: &[Scheme],
    issues: &mut Issues,
) {
    let used = used(c, ops, resolved);
    let said = c
        .security
        .as_ref()
        .is_some_and(|s| s.alternatives.is_empty());
    if used.is_empty() && !c.methods.is_empty() && !said {
        let message = String::from(
            "no operation needs a credential, by the description's `security` or the config's: the client sends none; if the API takes one, declare it under `auth` and name it in `security`; `security: []` says it takes none",
        );
        let at = c.auth.first().map_or_else(root, |a| a.name.at.clone());
        issues.warn("config.auth.none", &at, message);
    }
    for (name, i) in used {
        let Some(scheme) = schemes.iter().find(|s| s.name == name) else {
            continue;
        };
        let Some(m) = c.methods.get(i) else { continue };
        let place = format!("{}.{}", c.path(m.resource), m.name.name);
        let listed = c.auth.iter().find(|a| a.name.name == name);
        match (listed, &scheme.auth) {
            (None, _) => {
                let message = format!(
                    "`{place}` needs `{name}`, which `auth` does not list: the client sends no credential for it"
                );
                issues.warn("config.auth.unlisted", &m.name.at, message);
            }
            // No variable gives a client certificate or answers a challenge: `credential` warned.
            (Some(_), Auth::MutualTls) => {}
            (Some(_), Auth::Http { scheme, .. }) if CHALLENGED.contains(&scheme.as_str()) => {}
            (Some(a), auth) if !has_env(c, &name, auth) => {
                let message = format!(
                    "`{name}` names no environment variable: the client sends it only when given its option"
                );
                issues.warn("config.auth.env", &a.name.at, message);
            }
            (Some(_), _) => {}
        }
    }
}
