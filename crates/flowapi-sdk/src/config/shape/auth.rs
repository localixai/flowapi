//! The credentials of the format: `auth`, each scheme's options and, for one the description
//! lacks, what it is; and `security`, OpenAPI's Security Requirement Objects as written. Part of
//! `shape.rs`, by which the checker, its reader and the JSON Schema all go.

use super::{ENV_VAR, Field, Shape, Str, TEXT, URL, f};

pub(super) static AUTH_SCHEME: Shape = Shape::Named("auth", &Shape::Obj(AUTH));
static AUTH_TYPE: Shape = Shape::Str(Str::OneOf(
    "config.auth.type",
    &["apiKey", "http", "oauth2", "mutualTLS"],
));
static KEY_IN: Shape = Shape::Str(Str::OneOf("config.auth.in", &["header", "query", "cookie"]));
static TEXTS: Shape = Shape::List(&TEXT);

/// A credential's members: the options the client reads it by, and for a scheme the description
/// does not declare, what it is (`type` and the members of its kind, as OpenAPI's Security
/// Scheme Object names them).
pub(crate) const AUTH: &[Field] = &[
    f(
        "type",
        &AUTH_TYPE,
        "The kind of a scheme the description does not declare: `apiKey`, `http`, `oauth2` (a token, or one the client requests at `token_url`) or `mutualTLS`",
    ),
    f(
        "scheme",
        &TEXT,
        "`http`: the HTTP authentication scheme (`bearer`, `basic`, or another of IANA's registry)",
    ),
    f(
        "in",
        &KEY_IN,
        "`apiKey`: where the key goes: `header`, `query` or `cookie`",
    ),
    f(
        "name",
        &TEXT,
        "`apiKey`: the name of the header, query parameter or cookie",
    ),
    f(
        "token_url",
        &URL,
        "`oauth2`: the token endpoint the client requests a token at by the client credentials flow",
    ),
    f("description", &TEXT, "What the scheme is, for the README"),
    f(
        "env_var",
        &ENV_VAR,
        "The environment variable the credential is read from: the key or the token; for HTTP basic `username:password`",
    ),
    f(
        "username_env_var",
        &ENV_VAR,
        "HTTP basic: the environment variable of the user-id, beside `password_env_var`",
    ),
    f(
        "password_env_var",
        &ENV_VAR,
        "HTTP basic: the environment variable of the password, beside `username_env_var`",
    ),
    f(
        "client_id_env_var",
        &ENV_VAR,
        "OAuth 2.0 client credentials: the environment variable of the client's id",
    ),
    f(
        "client_secret_env_var",
        &ENV_VAR,
        "OAuth 2.0 client credentials: the environment variable of the client's secret",
    ),
    f(
        "scopes",
        &TEXTS,
        "OAuth 2.0 client credentials: the scopes every token is requested with, instead of each operation's",
    ),
];

static SCOPES: Shape = Shape::List(&TEXT);
static REQUIREMENT: Shape = Shape::Map {
    key: Str::Any,
    value: &SCOPES,
    fixed: &[],
};
/// OpenAPI's `security`: alternatives, each the schemes it needs with their scopes.
pub(super) static SECURITY: Shape = Shape::Named("security", &Shape::List(&REQUIREMENT));
