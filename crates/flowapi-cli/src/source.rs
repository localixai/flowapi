//! One file the command read: its URI and its text as the core reads it (`flowapi_yaml::Text`: a
//! YAML file read into JSON, with where in the YAML each JSON node came from, so a diagnostic
//! names the line a person wrote). The WebAssembly module reads a document by the same crate.

use flowapi_yaml::Text;
use std::borrow::Cow;
use std::path::Path;

/// A file as the core reads it.
pub(crate) struct Source {
    /// Its path, as given or as reached by a `$ref`.
    pub(crate) uri: String,
    /// Its JSON: the file itself, or its YAML read into JSON.
    text: Text<'static>,
}

impl Source {
    /// Reads a file, YAML as `flowapi_yaml::is_yaml` decides. The error is what the command
    /// prints: a file that cannot be read, or a YAML text JSON cannot hold, as a diagnostic.
    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let uri = path.to_string_lossy().into_owned();
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("flowapi: cannot read {uri}: {e}"))?;
        match Text::read(&uri, Cow::Owned(text)) {
            Ok(text) => Ok(Self { uri, text }),
            Err(e) => Err(format!(
                "error[{}]: {}\n  at {uri}:{}:{}",
                flowapi_yaml::CODE,
                e.message,
                e.line,
                e.column
            )),
        }
    }

    /// The JSON text the core is given.
    pub(crate) fn json(&self) -> &str {
        self.text.json()
    }

    /// A byte offset of the JSON text as `uri:line:column`, the YAML's line for a YAML file.
    pub(crate) fn describe(&self, offset: u32) -> String {
        let (line, column) = self.place(offset);
        format!("{}:{line}:{column}", self.uri)
    }

    /// The line and column of a byte offset of the JSON text, the YAML's for a YAML file.
    pub(crate) fn place(&self, offset: u32) -> (usize, usize) {
        self.text.place(offset)
    }
}
