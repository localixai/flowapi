//! The description and every file its references reach by a relative path, read before the
//! core runs: the core is given JSON text, never a path. A YAML file, the description or one a
//! `$ref` names, is read into JSON first (`source.rs`).

use crate::source::Source;
use flowapi::json::{self, Tree, Value};
use std::path::{Path, PathBuf};

/// Each file (its path as given, then as reached) with its JSON, the description first.
pub(crate) fn all(input: &str) -> Result<Vec<Source>, String> {
    let mut out: Vec<Source> = Vec::new();
    let mut queue: Vec<PathBuf> = vec![PathBuf::from(input)];
    while let Some(path) = queue.pop() {
        let uri = path.to_string_lossy().into_owned();
        if out.iter().any(|s| s.uri == uri) {
            continue;
        }
        let source = Source::read(&path)?;
        if let Ok(tree) = json::parse(&source.json) {
            for reference in external_refs(&tree) {
                let (resource, _) = flowapi::uri::split_fragment(&reference);
                if resource.is_empty() || resource.contains("://") || resource.starts_with("urn:") {
                    continue;
                }
                let next = normal(
                    &path
                        .parent()
                        .unwrap_or_else(|| Path::new(""))
                        .join(resource),
                );
                if next.is_file() {
                    queue.push(next);
                }
            }
        }
        out.push(source);
    }
    Ok(out)
}

/// Every `$ref` of a tree that names another document.
fn external_refs(tree: &Tree<'_>) -> Vec<String> {
    let mut out = Vec::new();
    for i in 0..tree.len() {
        let Some(node) = flowapi::id::NodeId::new(i) else {
            continue;
        };
        if !matches!(tree.value(node), Value::Object(_)) {
            continue;
        }
        // A `$ref`'s target, and a `$schema`'s meta-schema (its `$vocabulary` says what the
        // schema's keywords mean).
        for key in ["$ref", "$schema"] {
            if let Some(r) = tree.get(node, key).and_then(|n| tree.as_str(n))
                && !r.starts_with('#')
            {
                out.push(r.into_owned());
            }
        }
    }
    out
}

/// A path with its `.` and `..` taken out, as written (no file is read): `a/../c.json` is
/// `c.json`, so a file reached two ways is one document under one URI.
fn normal(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                // A `..` cancels the name before it, never another `..`: `../../x` stays so.
                let last = out.components().next_back();
                if matches!(last, Some(std::path::Component::Normal(_))) {
                    out.pop();
                } else {
                    out.push("..");
                }
            }
            std::path::Component::Normal(_)
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                out.push(part.as_os_str());
            }
        }
    }
    out
}
