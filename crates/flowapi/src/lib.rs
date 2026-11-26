//! flowapi reads OpenAPI descriptions and JSON Schemas into a type graph that no language knows,
//! and hands it to a target that prints it: TypeScript in `flowapi-ts`, others the same way.
//!
//! The core does no I/O (G5) and keeps no state between calls (G6): the host gives it the
//! documents' text and writes the files it returns. The stages are types, each taking the one
//! before by value:
//!
//! ```text
//! Documents ─index→ Index ─graph::build→ Graph ─lower→ Graph ─names::assign→ Program ─Target::emit→ Output
//! ```
//!
//! Every type is sound: it accepts every value its schema accepts (G3). Where a target's types
//! cannot narrow as far as the schema, the type is wider and says so.

pub mod diag;
pub mod graph;
pub mod id;
pub mod json;
pub mod lower;
pub mod names;
pub mod program;
pub mod schema;
pub mod source;
pub mod spec;
pub mod uri;

pub use diag::{Diagnostic, Diagnostics, Loc, Severity};
pub use program::{Decl, File, IdentifierRules, Output, Program, Target};
pub use source::{Dialect, DocKind, Document, Documents, Version};

/// Generates a target's files from the documents: the first is the one generated from, the
/// others are what its references reach. The diagnostics come back either way; the output only
/// when none is an error.
#[must_use]
pub fn generate(docs: &Documents<'_>, target: &dyn Target) -> (Option<Output>, Diagnostics) {
    let (program, mut diags) = program(docs, target);
    let Some(program) = program else {
        return (None, diags);
    };
    let output = target.emit(&program);
    diags.extend(output.diagnostics.clone());
    (Some(output), diags)
}

/// The program a target is given, as [`generate`] makes it: for a layer above the targets that
/// prints beside a target's output under the names that target chose. `None` when a diagnostic
/// is an error.
#[must_use]
pub fn program<'t>(
    docs: &Documents<'t>,
    target: &dyn Target,
) -> (Option<Program<'t>>, Diagnostics) {
    let (description, mut diags) = spec::read(docs);
    let index = spec::index(docs, &description.all);
    for (uri, doc, node) in index.duplicates() {
        let at = docs.loc(*doc, *node);
        diags.warn(
            "jsonschema-2020-12.core.$id",
            at,
            format!("`$id` {uri} names another schema already; the first one keeps it"),
        );
    }
    let (graph, found) =
        graph::build_with(docs, &index, &description.named, &description.operations);
    diags.extend(found);
    let lowered = lower::lower(graph, target.capabilities());
    let (program, found) = names::assign(lowered, docs, target.identifiers());
    diags.extend(found);
    if diags.has_errors() {
        return (None, diags);
    }
    (Some(program), diags)
}
