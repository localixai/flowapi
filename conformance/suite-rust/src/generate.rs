//! One group through flowapi's Rust target: its schema first, the schemas it references after
//! it, the types printed as one module of the generated crate.

use crate::load::Group;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The URI a group's schema is added under (the same as the TypeScript oracle's).
pub(crate) fn uri(file: &str, group: usize) -> String {
    format!("https://json-schema.org/flowapi-suite/{file}/{group}.json")
}

/// The module text of a group, or why there is none.
pub(crate) fn group(file: &str, g: &Group) -> Result<String, String> {
    match catch_unwind(AssertUnwindSafe(|| types(file, g))) {
        Ok(result) => result,
        Err(payload) => Err(format!("panic: {}", panic_message(payload.as_ref()))),
    }
}

fn types(file: &str, g: &Group) -> Result<String, String> {
    let mut texts = vec![(uri(file, g.index), pretty(&g.schema))];
    texts.extend(g.external.iter().map(|(u, s)| (u.clone(), pretty(s))));
    let mut docs = flowapi::Documents::new();
    for (u, text) in &texts {
        docs.add(u.clone(), text)
            .map_err(|d| format!("{d} (document {u})"))?;
    }
    let (output, diags) = flowapi::generate(&docs, &flowapi_rust::Rust::new("types.rs"));
    let errors: Vec<String> = diags
        .all()
        .iter()
        .filter(|d| d.severity == flowapi::Severity::Error)
        .map(|d| format!("{d} at {}", docs.describe(d.at)))
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let output = output.ok_or_else(|| String::from("no output and no error"))?;
    output
        .files
        .into_iter()
        .next()
        .map(|f| f.text)
        .ok_or_else(|| String::from("no file in the output"))
}

fn pretty(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).expect("a schema prints as JSON")
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        String::from("(no message)")
    }
}
