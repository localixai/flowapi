//! What each kind of string in a config must be, with the code of the issue when it is not and
//! the pattern the JSON Schema says it by.

use super::shape::Str;
use crate::words;

/// The code and the message when `s` is not what `kind` asks for.
pub(crate) fn problem(kind: Str, s: &str) -> Option<(&'static str, String)> {
    let fits = match kind {
        Str::Any => true,
        Str::Text => !s.trim().is_empty(),
        Str::Name => words::is_name(s),
        Str::EnvVar => is_env_var(s),
        Str::Header => is_token(s),
        Str::Url => is_url(s),
        Str::Package => is_package(s),
        Str::Client => is_client(s),
        Str::Version => is_version(s),
        Str::Release => is_version(s) && !s.split('-').next().unwrap_or("").ends_with(".x"),
    };
    if fits {
        return None;
    }
    Some(match kind {
        Str::Any | Str::Text => ("config.schema.empty", String::from("an empty string")),
        Str::Name => ("config.name.invalid", super::read::not_a_name(s)),
        Str::EnvVar => (
            "config.env_var",
            format!(
                "`{s}` is not an environment variable's name: A-Z, 0-9 and `_`, not opening with a digit"
            ),
        ),
        Str::Header => (
            "config.header",
            format!(
                "`{s}` is not a header's name: ASCII letters, digits and any of !#$%&'*+-.^_`|~, no space"
            ),
        ),
        Str::Url => (
            "config.env.url",
            format!("`{s}` is not an absolute http or https URL with a host"),
        ),
        Str::Package => (
            "config.package.name",
            format!("`{s}` is not an npm package name: lower case, `@scope/name` or `name`"),
        ),
        Str::Client => (
            "config.package.client",
            format!("`{s}` is not a class name in PascalCase: a capital, then letters and digits"),
        ),
        Str::Version => (
            "config.generator.version",
            format!("`{s}` is not a version: `0.1.0`, or `0.1.x` for any patch"),
        ),
        Str::Release => (
            "config.package.version",
            format!("`{s}` is not a package's version: `0.1.0`, or `1.0.0-beta.1`"),
        ),
    })
}

/// The pattern of a kind of string, for the JSON Schema.
pub(crate) fn pattern(kind: Str) -> Option<&'static str> {
    match kind {
        Str::Any => None,
        Str::Text => Some("\\S"),
        Str::Name => Some(words::NAME_PATTERN),
        Str::EnvVar => Some("^[A-Z_][A-Z0-9_]*$"),
        Str::Header => Some("^[!#$%&'*+.^_`|~0-9A-Za-z-]+$"),
        Str::Url => Some("^https?://[^\\s/?#]+[^\\s]*$"),
        Str::Package => Some("^(@[a-z0-9-~][a-z0-9-._~]*/)?[a-z0-9-~][a-z0-9-._~]*$"),
        Str::Client => Some("^[A-Z][A-Za-z0-9]*$"),
        Str::Version => Some("^[0-9]+\\.[0-9]+\\.([0-9]+|x)(-[0-9A-Za-z.-]+)?$"),
        Str::Release => Some("^[0-9]+\\.[0-9]+\\.[0-9]+(-[0-9A-Za-z.-]+)?$"),
    }
}

fn is_env_var(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// A token (RFC 9110, 5.6.2), as a field's name is.
fn is_token(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))
}

pub(crate) fn is_url(s: &str) -> bool {
    let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty() && !s.chars().any(char::is_whitespace) && !s.contains(['{', '}'])
}

fn is_package(s: &str) -> bool {
    fn part(p: &str) -> bool {
        p.chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-~".contains(c))
            && p.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-._~".contains(c))
    }
    if s.len() > 214 {
        return false;
    }
    match s.strip_prefix('@') {
        Some(scoped) => scoped
            .split_once('/')
            .is_some_and(|(scope, name)| part(scope) && part(name)),
        None => part(s),
    }
}

fn is_client(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase()) && chars.all(|c| c.is_ascii_alphanumeric())
}

fn is_version(s: &str) -> bool {
    let (core, pre) = s.split_once('-').unwrap_or((s, ""));
    let parts: Vec<&str> = core.split('.').collect();
    let digits = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
    let pre_ok = pre
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c))
        && (pre.is_empty() || s.contains('-'));
    matches!(parts.as_slice(), [a, b, c] if digits(a) && digits(b) && (digits(c) || *c == "x"))
        && pre_ok
        && !(s.ends_with('-'))
}
