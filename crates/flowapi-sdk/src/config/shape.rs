//! The format of `flowapi-sdk.yml`, version 1, written once: the checker reads a config by
//! these definitions (`read.rs`), the typed reader asks for no member they do not name
//! (`build.rs`), and `flowapi config schema` prints them as JSON Schema (`schema.rs`), so the
//! three cannot drift apart. A member they do not name is an error.

mod package;
mod schemes;

pub(crate) use package::{PACKAGE, README};
use schemes::SCHEME;
pub(crate) use schemes::SCHEMES;

/// What a string must be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Str {
    /// Any string.
    Any,
    /// Any string with something in it.
    Text,
    /// A name, as words (`words::is_name`).
    Name,
    /// An environment variable's name.
    EnvVar,
    /// An absolute `http` or `https` URL.
    Url,
    /// An npm package name.
    Package,
    /// A class name in `PascalCase`.
    Client,
    /// A version, `0.1.0` or `0.1.x`.
    Version,
    /// A package's own version: `0.1.0`, `1.0.0-beta.1`, never `x`.
    Release,
}

/// What a value must be.
#[derive(Debug)]
pub(crate) enum Shape {
    /// A string.
    Str(Str),
    /// An integer in a range.
    Int(i64, i64),
    /// A number of seconds, above zero.
    Seconds,
    /// `true` or `false`.
    Bool,
    /// `false` only.
    False,
    /// The integer 1, the format's version.
    One,
    /// An object of these members, and no other.
    Obj(&'static [Field]),
    /// An object whose keys are chosen (each one of `key`, its value of `value`), beside the
    /// members `fixed` names.
    Map {
        key: Str,
        value: &'static Shape,
        fixed: &'static [Field],
    },
    /// An array of these.
    List(&'static Shape),
    /// The first of these whose JSON type the value has.
    Either(&'static [&'static Shape]),
    /// An object whose `type` member chooses which members it has.
    Tagged(&'static [(&'static str, &'static [Field])]),
    /// A shape named in the JSON Schema's `$defs`.
    Named(&'static str, &'static Shape),
}

/// A member of an object.
#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) name: &'static str,
    pub(crate) shape: &'static Shape,
    pub(crate) required: bool,
    /// What it means, the schema's `description`.
    pub(crate) about: &'static str,
    /// The value when it is not given, as JSON.
    pub(crate) default: Option<&'static str>,
}

const fn f(name: &'static str, shape: &'static Shape, about: &'static str) -> Field {
    Field {
        name,
        shape,
        required: false,
        about,
        default: None,
    }
}

const fn req(name: &'static str, shape: &'static Shape, about: &'static str) -> Field {
    Field {
        name,
        shape,
        required: true,
        about,
        default: None,
    }
}

const fn def(
    name: &'static str,
    shape: &'static Shape,
    value: &'static str,
    about: &'static str,
) -> Field {
    Field {
        name,
        shape,
        required: false,
        about,
        default: Some(value),
    }
}

static ANY: Shape = Shape::Str(Str::Any);
static TEXT: Shape = Shape::Str(Str::Text);
static NAME: Shape = Shape::Str(Str::Name);
static ENV_VAR: Shape = Shape::Str(Str::EnvVar);
static URL: Shape = Shape::Str(Str::Url);
static SECONDS: Shape = Shape::Seconds;
static RETRIES: Shape = Shape::Int(0, 10);
static START: Shape = Shape::Int(0, 1);
static BOOL: Shape = Shape::Bool;
static NO_PAGING: Shape = Shape::False;
static PAGING_REF: Shape = Shape::Either(&[&NAME, &NO_PAGING]);
static OP: Shape = Shape::Str(Str::Text);

pub(crate) const GENERATOR: &[Field] = &[req(
    "flowapi",
    &Shape::Str(Str::Version),
    "The flowapi version the SDK is generated with",
)];

pub(crate) const ENVIRONMENTS_FIXED: &[Field] = &[
    f(
        "default",
        &NAME,
        "The environment used when the caller names none",
    ),
    f(
        "env_var",
        &ENV_VAR,
        "The environment variable that overrides the base URL",
    ),
];

static AUTH_SCHEME: Shape = Shape::Named("auth", &Shape::Obj(AUTH));
pub(crate) const AUTH: &[Field] = &[f(
    "env_var",
    &ENV_VAR,
    "The environment variable the credential is read from",
)];

pub(crate) const DEFAULTS: &[Field] = &[
    def(
        "retries",
        &RETRIES,
        "2",
        "How many times a failed request is retried",
    ),
    def(
        "timeout",
        &SECONDS,
        "60",
        "Seconds a request may take, its body read",
    ),
    def(
        "max_retry_delay",
        &SECONDS,
        "60",
        "The longest wait, in seconds, a server may ask for before a retry",
    ),
    f(
        "pagination",
        &NAME,
        "The pagination scheme of every method named `list` that names none",
    ),
];

pub(crate) const METHOD: &[Field] = &[
    req(
        "op",
        &OP,
        "The operation: its `operationId`, or `\"METHOD /path\"`",
    ),
    f(
        "pagination",
        &PAGING_REF,
        "The pagination scheme, by name; `false` for none",
    ),
    f(
        "retries",
        &RETRIES,
        "How many times a failed request is retried",
    ),
    f(
        "stream",
        &BOOL,
        "Whether the response is read as a stream of events",
    ),
];
static METHOD_VALUE: Shape = Shape::Named("method", &Shape::Either(&[&OP, &Shape::Obj(METHOD)]));

static METHODS: Shape = Shape::Map {
    key: Str::Name,
    value: &METHOD_VALUE,
    fixed: &[],
};
static ENVIRONMENTS: Shape = Shape::Map {
    key: Str::Name,
    value: &URL,
    fixed: ENVIRONMENTS_FIXED,
};
static AUTHS: Shape = Shape::Map {
    key: Str::Any,
    value: &AUTH_SCHEME,
    fixed: &[],
};
static PAGINATION: Shape = Shape::Map {
    key: Str::Name,
    value: &SCHEME,
    fixed: &[],
};
static RESOURCES: Shape = Shape::Map {
    key: Str::Name,
    value: &RESOURCE,
    fixed: &[],
};
static RESOURCE: Shape = Shape::Named("resource", &RESOURCE_OBJECT);
/// A static, not a constant: a resource holds resources, and a constant may not name itself.
static RESOURCE_OBJECT: Shape = Shape::Obj(&[
    f(
        "methods",
        &METHODS,
        "The resource's methods: a name for each operation",
    ),
    f("resources", &RESOURCES, "The resources under it"),
]);

/// A resource's members.
pub(crate) fn resource_fields() -> &'static [Field] {
    match &RESOURCE_OBJECT {
        Shape::Obj(fields) => fields,
        Shape::Str(_)
        | Shape::Int(..)
        | Shape::Seconds
        | Shape::Bool
        | Shape::False
        | Shape::One
        | Shape::Map { .. }
        | Shape::List(_)
        | Shape::Either(_)
        | Shape::Tagged(_)
        | Shape::Named(..) => &[],
    }
}

pub(crate) const SKIP: &[Field] = &[
    req(
        "op",
        &OP,
        "The operation: its `operationId`, or `\"METHOD /path\"`",
    ),
    req("reason", &TEXT, "Why the SDK leaves it out"),
];

/// The config's root.
pub(crate) const ROOT: &[Field] = &[
    req("version", &Shape::One, "The format's version: 1"),
    f(
        "generator",
        &Shape::Obj(GENERATOR),
        "What generates the SDK",
    ),
    req(
        "spec",
        &TEXT,
        "The description: a path, relative to the config, or a URL",
    ),
    req("package", &Shape::Obj(PACKAGE), "The package"),
    f(
        "environments",
        &ENVIRONMENTS,
        "The base URLs by name, with `default` and `env_var`",
    ),
    f(
        "auth",
        &AUTHS,
        "The security schemes the client sends, by their names in the description",
    ),
    f(
        "defaults",
        &Shape::Obj(DEFAULTS),
        "What every method has unless it says otherwise",
    ),
    f("pagination", &PAGINATION, "Pagination schemes, by name"),
    req(
        "resources",
        &RESOURCES,
        "The resources, each a name for a group of methods",
    ),
    f(
        "skip",
        &Shape::List(&Shape::Named("skip", &Shape::Obj(SKIP))),
        "Operations the SDK leaves out, each with its reason",
    ),
    f("readme", &Shape::Obj(README), "What the README shows"),
];

/// The root as a shape.
pub(crate) static CONFIG: Shape = Shape::Obj(ROOT);
