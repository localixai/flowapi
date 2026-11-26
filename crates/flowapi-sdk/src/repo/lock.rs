//! `.flowapi/lock.json`: the state of a generated repository, in the shape the agent plugin
//! reads (`plugins/flowapi/docs/lock.md`): the lock's format version, the flowapi that wrote it, the
//! hashes of the description and the config, the methods the config maps, every generated file
//! with the SHA-256 of its bytes, and the names pinned when two would collide. Printed as stable
//! JSON (keys in a fixed order, files sorted, two spaces), read back from JSON text.

use crate::out::json_str;
use flowapi::json::{Tree, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The lock of a generated repository.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lock {
    /// The lock's format version: 1.
    pub version: u32,
    /// The flowapi that wrote the repository.
    pub flowapi: String,
    /// SHA-256 (hex) of the description's bytes as read.
    pub spec_sha256: String,
    /// SHA-256 (hex) of the config's bytes.
    pub config_sha256: String,
    /// How many operations the config maps to methods.
    pub configured_endpoints: usize,
    /// Every generated file: its path from the repository's root, and the SHA-256 (hex) of the
    /// bytes written.
    pub files: BTreeMap<String, String>,
    /// Names pinned when two would collide: the operation (`"METHOD /path"` or its
    /// `operationId`) and the name it keeps. Version 1 of the config refuses a collision, so it
    /// is empty.
    pub names: BTreeMap<String, String>,
}

impl Lock {
    /// The lock as JSON text, ending in a newline.
    #[must_use]
    pub fn json(&self) -> String {
        let mut out = String::from("{\n");
        let _ = writeln!(out, "  \"version\": {},", self.version);
        let _ = writeln!(out, "  \"flowapi\": {},", json_str(&self.flowapi));
        let _ = writeln!(out, "  \"spec_sha256\": {},", json_str(&self.spec_sha256));
        let _ = writeln!(
            out,
            "  \"config_sha256\": {},",
            json_str(&self.config_sha256)
        );
        let _ = writeln!(
            out,
            "  \"configured_endpoints\": {},",
            self.configured_endpoints
        );
        map(&mut out, "files", &self.files);
        out.push_str(",\n");
        map(&mut out, "names", &self.names);
        out.push_str("\n}\n");
        out
    }

    /// A lock read from JSON text; the error says what is wrong with it.
    ///
    /// # Errors
    ///
    /// When the text is not JSON, or not a lock of version 1: a member missing or of another
    /// type, or a file's path that is not relative, or climbs with `..`.
    pub fn read(text: &str) -> Result<Self, String> {
        let tree = flowapi::json::parse(text).map_err(|e| format!("not JSON: {}", e.message))?;
        let root = tree.root();
        let string = |key: &str| {
            tree.get(root, key)
                .and_then(|n| tree.as_str(n))
                .map(|s| s.into_owned())
                .ok_or_else(|| format!("`{key}` is not a string"))
        };
        let number = |key: &str| {
            tree.get(root, key)
                .and_then(|n| match tree.value(n) {
                    Value::Number(span) => tree.raw(span).parse::<usize>().ok(),
                    Value::Null
                    | Value::Bool(_)
                    | Value::String(_)
                    | Value::Array(_)
                    | Value::Object(_) => None,
                })
                .ok_or_else(|| format!("`{key}` is not a whole number"))
        };
        let version = number("version")?;
        if version != 1 {
            return Err(format!(
                "its version is {version}; this flowapi reads version 1"
            ));
        }
        let lock = Self {
            version: 1,
            flowapi: string("flowapi")?,
            spec_sha256: string("spec_sha256")?,
            config_sha256: string("config_sha256")?,
            configured_endpoints: number("configured_endpoints")?,
            files: strings(&tree, "files")?,
            names: strings(&tree, "names")?,
        };
        if let Some(bad) = lock.files.keys().find(|p| !is_relative(p)) {
            return Err(format!(
                "the file `{bad}` is not a path inside the repository"
            ));
        }
        Ok(lock)
    }
}

/// Whether a path names a file inside the repository: relative, `/` between parts, no part empty,
/// `.` or `..`, no `\`, no drive.
#[must_use]
pub(crate) fn is_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn map(out: &mut String, key: &str, entries: &BTreeMap<String, String>) {
    if entries.is_empty() {
        let _ = write!(out, "  \"{key}\": {{}}");
        return;
    }
    let _ = writeln!(out, "  \"{key}\": {{");
    let lines: Vec<String> = entries
        .iter()
        .map(|(k, v)| format!("    {}: {}", json_str(k), json_str(v)))
        .collect();
    out.push_str(&lines.join(",\n"));
    out.push_str("\n  }");
}

/// An object of strings, by its key at the root.
fn strings(tree: &Tree<'_>, key: &str) -> Result<BTreeMap<String, String>, String> {
    let node = tree
        .get(tree.root(), key)
        .filter(|&n| matches!(tree.value(n), Value::Object(_)))
        .ok_or_else(|| format!("`{key}` is not an object"))?;
    let mut out = BTreeMap::new();
    for m in tree.members(node) {
        let (Some(k), Some(v)) = (tree.string(m.key), tree.as_str(m.value)) else {
            return Err(format!("`{key}` holds a value that is not a string"));
        };
        out.insert(k.into_owned(), v.into_owned());
    }
    Ok(out)
}
