//! flowapi's SDK layer, above the targets: what a client needs of a description beside a target's
//! SDK. A target sees only a [`Program`] (G9), so what no type depends on — where the API is
//! served and how a caller proves who it is — is read here, from the documents, through the
//! core's public modules, and matched to the program's endpoints. No I/O (G5).
//!
//! [`read`] gives it in no language ([`Service`]); [`table`] prints it for the TypeScript SDK
//! (`flowapi-ts`, `--sdk`): the servers, the security schemes and one table of operations, text
//! appended after that SDK's output, which a client reads to choose a base URL, attach
//! authentication and read a body as text or bytes.
//!
//! For an agent that writes an SDK's config (`sdk-anything.yml`): [`check`] holds a config to
//! its format and to the description, every issue at once; [`config_schema`] is the format as
//! JSON Schema. Each takes the documents and JSON text and returns text, so the WebAssembly
//! module can call them with JSON in and out as the command does.

mod config;
mod doc;
mod draft;
mod model;
mod ops;
mod out;
mod paging;
mod print;
mod security;
mod servers;
mod summary;
mod words;

pub use config::{Issue, check, issues_json, issues_text, schema as config_schema, spec_of};
pub use draft::draft;
pub use model::{Access, Auth, KeyLocation, Requirement, Scheme, Server, Service, Variable};
pub use summary::{Filter, summary};

use doc::Doc;
use flowapi::{Diagnostics, Documents, Program};

/// What the description says about serving and securing its API, with each of the program's
/// endpoints' own part, in the program's order. Empty for a document that is not an OpenAPI or
/// Swagger description.
#[must_use]
pub fn read(docs: &Documents<'_>, program: &Program<'_>) -> (Service, Diagnostics) {
    let mut diags = Diagnostics::default();
    let Some(doc) = Doc::of(docs) else {
        return (Service::default(), diags);
    };
    let servers = doc.servers(&mut diags);
    let schemes = doc.schemes(&mut diags);
    let security = doc.security(doc::ROOT, &mut diags).unwrap_or_default();
    let operations = program
        .endpoints()
        .iter()
        .map(|e| {
            let items = doc.path_items(&e.operation);
            let op = doc.operation(&items, &e.operation);
            Access {
                servers: doc.own_servers(&items, op, &mut diags),
                security: op
                    .and_then(|op| doc.security(op, &mut diags))
                    .unwrap_or_else(|| security.clone()),
            }
        })
        .collect();
    let service = Service {
        servers,
        schemes,
        operations,
    };
    (service, diags)
}

/// The TypeScript printed after `flowapi-ts`'s SDK for the same program (made with
/// `TypeScript::with_sdk`): `flowapiServers`, `flowapiSecuritySchemes` and `flowapiOperations`.
/// Empty when the program has no endpoint, as that SDK is.
#[must_use]
pub fn table(docs: &Documents<'_>, program: &Program<'_>) -> (String, Diagnostics) {
    let (service, diags) = read(docs, program);
    if program.endpoints().is_empty() {
        return (String::new(), diags);
    }
    (print::table(program, &service), diags)
}
