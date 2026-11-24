//! One group through flowapi: its schema first, the schemas it references after it, the type
//! printed into a TypeScript file followed by one line per case.

use crate::load::Group;
use std::fmt::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

/// What generating a group gave.
pub(crate) enum Generated {
    /// flowapi reported an error, refused a document, or panicked.
    Error(String),
    /// The file written: its name in the output directory, and the line of its first case (the
    /// lines before it are the generated types).
    Written {
        file: String,
        first_case_line: usize,
    },
}

/// The URI a group's schema is added under.
pub(crate) fn uri(file: &str, group: usize) -> String {
    format!("https://json-schema.org/flowapi-suite/{file}/{group}.json")
}

/// The TypeScript file of a group, written into `out`.
pub(crate) fn group(file: &str, g: &Group, out: &Path) -> Generated {
    let name = format!("{file}-{}.ts", g.index);
    let types = match catch_unwind(AssertUnwindSafe(|| types(file, g, &name))) {
        Ok(Ok(text)) => text,
        Ok(Err(message)) => return Generated::Error(message),
        Err(payload) => {
            return Generated::Error(format!("panic: {}", panic_message(payload.as_ref())));
        }
    };
    let mut text = types;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let first_case_line = text.lines().count() + 1;
    for (i, case) in g.cases.iter().enumerate() {
        let data = serde_json::to_string(&case.data).expect("a case's data prints as JSON");
        writeln!(text, "const c{i}: Root = {data};").expect("writing to a String");
    }
    std::fs::write(out.join(&name), text).expect("the output directory is writable");
    Generated::Written {
        file: name,
        first_case_line,
    }
}

/// The generated types, or every error flowapi reported.
fn types(file: &str, g: &Group, path: &str) -> Result<String, String> {
    let mut texts = vec![(uri(file, g.index), pretty(&g.schema))];
    texts.extend(g.external.iter().map(|(u, s)| (u.clone(), pretty(s))));
    let mut docs = flowapi::Documents::new();
    for (u, text) in &texts {
        docs.add(u.clone(), text)
            .map_err(|d| format!("{d} (document {u})"))?;
    }
    let (output, diags) = flowapi::generate(&docs, &flowapi_ts::TypeScript::new(path));
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

/// A schema as text, one key per line, so a diagnostic's line and column point somewhere.
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
