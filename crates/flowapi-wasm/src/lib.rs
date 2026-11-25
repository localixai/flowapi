//! flowapi compiled to WebAssembly, for the npm package `@localix-ai/flowapi`: the core and the
//! TypeScript target, the same code the `flowapi` command runs, behind an ABI of four exports
//! (`abi`). A request and its answer are JSON text in linear memory; the host reads the files
//! and writes the output, as the CLI does (G5). A document's text is JSON or YAML, told apart and
//! read into JSON by `flowapi_yaml`, the crate the command reads its files with.
//!
//! The request:
//!
//! ```json
//! { "documents": [{ "uri": "openapi.yaml", "text": "…" }], "options": { "output": "types.ts" } }
//! ```
//!
//! The answer, with each diagnostic's place as a byte offset (into the JSON the core read: for a
//! YAML document, the JSON it was read into; for a YAML text JSON cannot hold, into the YAML) and
//! as the line and column the CLI prints, the YAML's for a YAML document (G8); `files` is empty
//! when a diagnostic is an error:
//!
//! ```json
//! { "files": [{ "path": "types.ts", "text": "…" }],
//!   "diagnostics": [{ "severity": "warning", "code": "…", "message": "…",
//!                     "uri": "openapi.yaml", "offset": 120, "line": 7, "column": 5 }] }
//! ```

pub mod abi;
mod answer;
mod request;

use answer::Found;
use flowapi::json::{self, Value};
use flowapi::{Diagnostic, Documents, Severity};
use flowapi_yaml::Text;
use std::borrow::Cow;

/// A document of the request as the core reads it.
struct Source<'t> {
    uri: Cow<'t, str>,
    text: Text<'t>,
}

/// Generates TypeScript from a request (the crate's documentation shows it) and returns the
/// answer as JSON. Never fails: a request that cannot be read is an answer with one error.
#[must_use]
pub fn generate(request: &[u8]) -> String {
    let Ok(text) = std::str::from_utf8(request) else {
        return answer::refused("the request is not UTF-8");
    };
    let tree = match json::parse(text) {
        Ok(tree) => tree,
        Err(e) => return answer::refused(&format!("the request is not JSON: {}", e.message)),
    };
    let request::Request {
        documents,
        output,
        sdk,
    } = match request::read(&tree) {
        Ok(r) => r,
        Err(message) => return answer::refused(&message),
    };
    // Every document read into JSON before the core is given one, as the command reads every
    // file first: the same document is the first refused.
    let mut sources = Vec::with_capacity(documents.len());
    for doc in documents {
        match Text::read(&doc.uri, doc.text) {
            Ok(text) => sources.push(Source { uri: doc.uri, text }),
            Err(e) => return answer::write(&[], &[unread(&doc.uri, &e)]),
        }
    }
    let mut docs = Documents::new();
    for source in &sources {
        if let Err(d) = docs.add(source.uri.as_ref(), source.text.json()) {
            return answer::write(&[], &[found(&sources, &d)]);
        }
    }
    let target = flowapi_ts::TypeScript::new(output.as_ref());
    let target = if sdk { target.with_sdk() } else { target };
    let (output, diags) = flowapi::generate(&docs, &target);
    let diags: Vec<Found<'_>> = diags.all().iter().map(|d| found(&sources, d)).collect();
    let files = output.map(|o| o.files).unwrap_or_default();
    answer::write(&files, &diags)
}

/// A diagnostic with its document's URI and its place as a line and a column, counted as the
/// CLI counts them (`flowapi_yaml::Text::place`): lines from 1, columns in characters from 1.
fn found<'d>(sources: &'d [Source<'_>], d: &'d Diagnostic) -> Found<'d> {
    let source = sources.get(d.at.doc.index());
    let (line, column) = source.map_or((1, 1), |s| s.text.place(d.at.offset));
    Found {
        severity: d.severity,
        code: d.code,
        message: &d.message,
        uri: source.map_or("", |s| s.uri.as_ref()),
        offset: d.at.offset,
        line,
        column,
    }
}

/// A YAML text JSON cannot hold, as the error the command prints for it.
fn unread<'d>(uri: &'d str, e: &'d flowapi_yaml::Error) -> Found<'d> {
    Found {
        severity: Severity::Error,
        code: flowapi_yaml::CODE,
        message: &e.message,
        uri,
        offset: u32::try_from(e.offset).unwrap_or(u32::MAX),
        line: e.line,
        column: e.column,
    }
}

/// Every `$ref` and `$schema` of a document (`{"uri": "…", "text": "…"}`, JSON or YAML) that
/// names another document (it does not start with `#`), in the order the CLI finds them, as a
/// JSON array of strings: the host reads those files and hands them to `generate` with the
/// description. A document that cannot be read has none; `generate` says why.
#[must_use]
pub fn references(request: &[u8]) -> String {
    let mut found: Vec<String> = Vec::new();
    let request = std::str::from_utf8(request)
        .ok()
        .and_then(|text| json::parse(text).ok());
    let text = request.as_ref().and_then(|tree| {
        let doc = request::document(tree, tree.root()).ok()?;
        Text::read(&doc.uri, doc.text).ok()
    });
    let tree = text.as_ref().and_then(|text| json::parse(text.json()).ok());
    if let Some(tree) = tree {
        for node in (0..tree.len()).filter_map(flowapi::id::NodeId::new) {
            if !matches!(tree.value(node), Value::Object(_)) {
                continue;
            }
            for key in ["$ref", "$schema"] {
                if let Some(r) = tree.get(node, key).and_then(|n| tree.as_str(n))
                    && !r.starts_with('#')
                {
                    found.push(r.into_owned());
                }
            }
        }
    }
    answer::strings(&found)
}
