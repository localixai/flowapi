//! One file the command read: its URI, the JSON text the core is given and, for a YAML file,
//! where in the YAML each JSON node came from, so a diagnostic names the line a person wrote.

use flowapi_cli::yaml::{self, Marks};
use std::path::Path;

/// A file as the core reads it.
pub(crate) struct Source {
    /// Its path, as given or as reached by a `$ref`.
    pub(crate) uri: String,
    /// Its JSON text: the file itself, or its YAML read into JSON.
    pub(crate) json: String,
    /// For a YAML file, the YAML place of each JSON node.
    marks: Option<Marks>,
}

impl Source {
    /// Reads a file: YAML by its extension (`.yaml`, `.yml`), or, under any other name than
    /// `.json`, when its text does not open with `{` or `[`.
    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let uri = path.to_string_lossy().into_owned();
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {uri}: {e}"))?;
        if !is_yaml(path, &text) {
            return Ok(Self {
                uri,
                json: text,
                marks: None,
            });
        }
        let json = yaml::to_json(&text).map_err(|e| format!("{uri}:{e}"))?;
        Ok(Self {
            uri,
            json: json.text,
            marks: Some(json.marks),
        })
    }

    /// A byte offset of the JSON text as `uri:line:column`, the YAML's line for a YAML file.
    pub(crate) fn describe(&self, offset: u32) -> String {
        if let Some((line, column)) = self.marks.as_ref().and_then(|m| m.place(offset)) {
            return format!("{}:{line}:{column}", self.uri);
        }
        let before = self.json.get(..offset as usize).unwrap_or("");
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        format!("{}:{line}:{column}", self.uri)
    }
}

/// Whether a file is read as YAML.
fn is_yaml(path: &Path, text: &str) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("yaml" | "yml") => true,
        Some("json") => false,
        Some(_) | None => !text
            .trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
            .starts_with(['{', '[']),
    }
}
