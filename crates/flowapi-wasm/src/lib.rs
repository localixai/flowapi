//! flowapi compiled to WebAssembly, for the npm package `@localix-ai/flowapi`: the core and the
//! TypeScript target, the same code the `flowapi` command runs, behind an ABI of four exports
//! (`abi`). A request and its answer are JSON text in linear memory; the host reads the files
//! and writes the output, as the CLI does (G5).
//!
//! The request:
//!
//! ```json
//! { "documents": [{ "uri": "openapi.json", "text": "{…}" }], "options": { "output": "types.ts" } }
//! ```
//!
//! The answer, with each diagnostic's place as a byte offset and as the line and column the CLI
//! prints (G8); `files` is empty when a diagnostic is an error:
//!
//! ```json
//! { "files": [{ "path": "types.ts", "text": "…" }],
//!   "diagnostics": [{ "severity": "warning", "code": "…", "message": "…",
//!                     "uri": "openapi.json", "offset": 120, "line": 7, "column": 5 }] }
//! ```

pub mod abi;
mod answer;
mod request;

use answer::Found;
use flowapi::json::{self, Value};
use flowapi::{Diagnostic, Documents};

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
    let request = match request::read(&tree) {
        Ok(r) => r,
        Err(message) => return answer::refused(&message),
    };
    let mut docs = Documents::new();
    for doc in &request.documents {
        if let Err(d) = docs.add(doc.uri.as_ref(), &doc.text) {
            return answer::write(&[], &[found(&request, &d)]);
        }
    }
    let target = flowapi_ts::TypeScript::new(request.output.as_ref());
    let target = if request.sdk {
        target.with_sdk()
    } else {
        target
    };
    let (output, diags) = flowapi::generate(&docs, &target);
    let diags: Vec<Found<'_>> = diags.all().iter().map(|d| found(&request, d)).collect();
    let files = output.map(|o| o.files).unwrap_or_default();
    answer::write(&files, &diags)
}

/// A diagnostic with its document's URI and its place as a line and a column, counted as the
/// CLI counts them (`Documents::describe`): lines from 1, columns in characters from 1.
fn found<'d>(request: &'d request::Request<'_>, d: &'d Diagnostic) -> Found<'d> {
    let doc = request.documents.get(d.at.doc.index());
    let text = doc.map_or("", |doc| doc.text.as_ref());
    let before = text.get(..d.at.offset as usize).unwrap_or("");
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    Found {
        severity: d.severity,
        code: d.code,
        message: &d.message,
        uri: doc.map_or("", |doc| doc.uri.as_ref()),
        offset: d.at.offset,
        line,
        column,
    }
}

/// Every `$ref` and `$schema` of a document that names another document (it does not start with
/// `#`), in the order the CLI finds them, as a JSON array of strings: the host reads those files
/// and hands them to `generate` with the description. A text that is not JSON has none.
#[must_use]
pub fn references(document: &[u8]) -> String {
    let mut found: Vec<String> = Vec::new();
    let tree = std::str::from_utf8(document)
        .ok()
        .and_then(|text| json::parse(text).ok());
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
