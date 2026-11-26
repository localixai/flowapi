//! The markers in flowapi's tests: `// req: oas-3.1.schema-object.discriminator` says a test
//! covers a requirement; `lossy: jsonschema-2020-12.validation.minLength — no TS type has a
//! length` says the type is wider than the schema there, and why. A lossy marker needs its reason.
//!
//! Every `.rs`, `.ts`, `.mjs` and `.json` file under `crates/` and `conformance/` is read (`.mjs`:
//! the SDK client's Node tests), and the `.md`
//! files of `conformance/coverage/` (evidence grouped by oracle), except build output (`target/`,
//! `out/`), installed packages (`node_modules/`), ignored work folders (`scratch/`), the pinned texts and suites
//! (`conformance/specs/`, `conformance/suites/`) and the ledger's own files. A marker is `req:` or
//! `lossy:` followed by an id that starts with `oas-`, `jsonschema-` or `rfc` and a digit
//! (`rfc6749.`), so `req: Request` in code is not one.

use std::path::{Path, PathBuf};

/// What a marker says of its requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// `req:`: a test exercises it.
    Covered,
    /// `lossy:`: the type is wider than the schema there, for the reason given.
    Lossy,
}

/// One marker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marker {
    /// The file, relative to the repository's root, `/`-separated.
    pub file: String,
    /// The line (1-based).
    pub line: usize,
    /// The id it names.
    pub id: String,
    /// Covered or lossy.
    pub kind: Kind,
    /// The text after the id, without the dash before it: a lossy marker's reason.
    pub reason: String,
}

const EXTENSIONS: [&str; 4] = ["rs", "ts", "mjs", "json"];
/// Where `.md` files are read too: the evidence grouped by oracle.
const MARKDOWN_UNDER: &str = "conformance/coverage/";
/// Names of folders never read, wherever they are: build output, installed packages, git's own,
/// and the ignored work folders a tool writes (`out/`, `scratch/`), which a fresh clone lacks:
/// the report must be the same from any checkout.
const SKIPPED_DIRS: [&str; 5] = ["target", "node_modules", ".git", "out", "scratch"];
/// Relative to the repository's root: the ledger itself and what it reads and writes, the pinned
/// texts and suites (other people's files), and generated output.
const SKIPPED: [&str; 6] = [
    "conformance/ledger",
    "conformance/requirements.json",
    "conformance/scope.json",
    "conformance/specs",
    "conformance/suites",
    "conformance/out",
];

/// Every marker under `crates/` and `conformance/` of `root`, sorted by file and line.
pub fn scan(root: &Path) -> Result<Vec<Marker>, String> {
    let mut files = Vec::new();
    for top in ["crates", "conformance"] {
        walk(root, &root.join(top), &mut files)?;
    }
    files.sort();
    let mut found = Vec::new();
    for (relative, path) in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            for (kind, id, reason) in markers_in(line) {
                found.push(Marker {
                    file: relative.clone(),
                    line: index + 1,
                    id,
                    kind,
                    reason,
                });
            }
        }
    }
    Ok(found)
}

fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if SKIPPED.contains(&relative.as_str()) {
            continue;
        }
        if path.is_dir() {
            if !SKIPPED_DIRS.contains(&name.as_str()) {
                walk(root, &path, files)?;
            }
        } else if path.extension().is_some_and(|e| {
            EXTENSIONS.iter().any(|x| e == *x)
                || (e == "md" && relative.starts_with(MARKDOWN_UNDER))
        }) {
            files.push((relative, path));
        }
    }
    Ok(())
}

/// Whether a word is a requirement id: one of OpenAPI (`oas-`), JSON Schema (`jsonschema-`) or
/// an RFC (`rfc` and its number).
fn is_spec_id(id: &str) -> bool {
    let rfc = id
        .strip_prefix("rfc")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()));
    rfc || id.starts_with("oas-") || id.starts_with("jsonschema-")
}

/// The ids the `req:` markers of one line name.
pub fn ids_in(line: &str) -> Vec<String> {
    markers_in(line)
        .into_iter()
        .filter(|(kind, _, _)| *kind == Kind::Covered)
        .map(|(_, id, _)| id)
        .collect()
}

/// The markers of one line: kind, id, and the text after the id up to the next marker.
pub fn markers_in(line: &str) -> Vec<(Kind, String, String)> {
    let mut found: Vec<(usize, usize, Kind, String)> = Vec::new();
    for (tag, kind) in [("req:", Kind::Covered), ("lossy:", Kind::Lossy)] {
        let mut from = 0;
        while let Some(at) = line.get(from..).and_then(|r| r.find(tag)).map(|a| a + from) {
            from = at + tag.len();
            let boundary = line
                .get(..at)
                .and_then(|b| b.chars().last())
                .is_none_or(|c| !c.is_alphanumeric() && c != '_');
            let rest = line.get(from..).unwrap_or_default();
            let start = from + (rest.len() - rest.trim_start().len());
            let id: String = line
                .get(start..)
                .unwrap_or_default()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || "$._-".contains(*c))
                .collect();
            let id = id.trim_end_matches(['.', '-']).to_owned();
            if boundary && is_spec_id(&id) {
                found.push((at, start + id.len(), kind, id));
            }
        }
    }
    found.sort_by_key(|f| f.0);
    let starts: Vec<usize> = found.iter().map(|f| f.0).collect();
    found
        .into_iter()
        .enumerate()
        .map(|(i, (_, end, kind, id))| {
            let until = starts.get(i + 1).copied().unwrap_or(line.len());
            let reason = line.get(end..until).unwrap_or_default();
            let reason = reason.trim().trim_start_matches(['—', '-', ':']).trim();
            (kind, id, reason.to_owned())
        })
        .collect()
}
