//! `flowapi-sdk.yml`, version 1: the config an agent writes and flowapi generates an SDK by. Its
//! format is written once (`shape.rs`); a config is held to it (`read.rs`), read into a model
//! (`build.rs`, `model.rs`), then to its description (`check.rs`, `names.rs`), every issue found
//! in one pass. The config comes as JSON (a host reads YAML into JSON with `flowapi-yaml`, and
//! turns an issue's offset into the YAML's line and column), the description as the core's
//! [`Documents`]. No I/O (G5).

mod build;
pub(crate) mod check;
pub(crate) mod issue;
pub(crate) mod model;
mod names;
mod read;
mod schema;
pub(crate) mod shape;
mod strings;

pub(crate) use check::streams;
pub use issue::{Issue, issues_json, issues_text};
pub(crate) use names::{CLIENT_MEMBERS, RESOURCE_MEMBERS, reserved};
pub(crate) use strings::is_url;

use crate::ops::Ops;
use flowapi::Documents;
use issue::{At, Issues};
use model::Config;

/// Every issue of a config (its JSON text) against a description: the format first (unknown
/// members, types, required members), then what it names (operations, schemes, environments),
/// coverage, names as words, pagination against the response schemas. In the order of the
/// members they are about.
#[must_use]
pub fn check(docs: &Documents<'_>, config: &str) -> Vec<Issue> {
    checked(docs, config).issues
}

/// A config read and held to its description: what the generator prints by.
pub(crate) struct Checked {
    pub(crate) config: Config,
    /// The operation each method calls, by index into `config.methods` (into [`Ops::read`]'s list).
    pub(crate) resolved: Vec<Option<usize>>,
    /// Every issue, in the order of the members they are about.
    pub(crate) issues: Vec<Issue>,
}

/// [`check`], keeping what it read.
pub(crate) fn checked(docs: &Documents<'_>, config: &str) -> Checked {
    let mut issues = Issues::default();
    let tree = match flowapi::json::parse(config) {
        Ok(tree) => tree,
        Err(e) => {
            let at = At {
                pointer: String::new(),
                offset: e.offset,
            };
            issues.error(
                "config.syntax",
                &at,
                format!("the config is not JSON: {}", e.message),
            );
            return Checked {
                config: Config::default(),
                resolved: Vec::new(),
                issues: issues.list,
            };
        }
    };
    let declared = read::validate(&tree, &shape::CONFIG, &mut issues);
    let c = build::build(&tree, &declared, &mut issues);
    let (ops, _) = Ops::read(docs);
    let mut resolved = Vec::new();
    match ops {
        Some(ops) => {
            resolved = check::references(&c, &ops, &mut issues);
            names::check(&c, &ops.schema_names, &mut issues);
            check::methods(&c, &ops, &resolved, &mut issues);
            check::rest(&c, &ops, &resolved, &mut issues);
        }
        None => {
            let message = String::from("the description is not an OpenAPI or Swagger description");
            issues.error("config.spec", &check::root(), message);
        }
    }
    check::generator(&c, &mut issues);
    issues.list.sort_by_key(|i| i.offset);
    Checked {
        config: c,
        resolved,
        issues: issues.list,
    }
}

/// The description a config names (its `spec`, as written), for a host to read before
/// [`check`]; `None` when the config is not JSON or names none.
#[must_use]
pub fn spec_of(config: &str) -> Option<String> {
    let tree = flowapi::json::parse(config).ok()?;
    let spec = tree.get(tree.root(), "spec")?;
    tree.as_str(spec).map(|s| s.into_owned())
}

/// The JSON Schema (2020-12) of the config's format, printed from the definitions [`check`]
/// reads by.
#[must_use]
pub fn schema() -> String {
    schema::schema()
}
