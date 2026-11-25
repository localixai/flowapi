//! One document as the core is given it: a JSON text as it is, or a YAML text read into JSON with
//! the YAML place of every node, and the line and column of a byte offset into it, counted alike
//! for both.

use crate::reader::{self, Error, Marks};
use std::borrow::Cow;

/// The code of the diagnostic a host reports for a YAML text JSON cannot hold, beside the core's
/// `json.syntax`.
pub const CODE: &str = "yaml.syntax";

/// A document's text as the core reads it.
#[derive(Debug)]
pub struct Text<'t> {
    json: Cow<'t, str>,
    /// For a YAML document, the YAML place of each JSON node.
    marks: Option<Marks>,
}

impl<'t> Text<'t> {
    /// Reads the document named `uri`: YAML when `is_yaml` says so, read into JSON; any other
    /// text is the core's to read as JSON, as it is.
    ///
    /// # Errors
    ///
    /// When a YAML text is not YAML, or holds what JSON cannot.
    pub fn read(uri: &str, text: Cow<'t, str>) -> Result<Self, Error> {
        if !is_yaml(uri, &text) {
            return Ok(Self {
                json: text,
                marks: None,
            });
        }
        let json = reader::to_json(&text)?;
        Ok(Self {
            json: Cow::Owned(json.text),
            marks: Some(json.marks),
        })
    }

    /// The JSON text the core reads.
    #[must_use]
    pub fn json(&self) -> &str {
        &self.json
    }

    /// The line and column (both from 1, the column in characters) of a byte offset into the
    /// JSON text: for a YAML document, the YAML's place of the node there.
    #[must_use]
    pub fn place(&self, offset: u32) -> (usize, usize) {
        if let Some((line, column)) = self.marks.as_ref().and_then(|m| m.place(offset)) {
            return (line as usize, column as usize);
        }
        let before = self.json.get(..offset as usize).unwrap_or("");
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        (line, column)
    }
}

/// Whether the document named `uri` is YAML: by its extension (`.yaml`, `.yml`, in any case), or,
/// under any other name than `.json`, when its text does not open with `{` or `[`. The extension
/// is that of the last segment of a path or URI, after `/` or `\`.
#[must_use]
pub fn is_yaml(uri: &str, text: &str) -> bool {
    match extension(uri).map(str::to_ascii_lowercase).as_deref() {
        Some("yaml" | "yml") => true,
        Some("json") => false,
        Some(_) | None => !text
            .trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
            .starts_with(['{', '[']),
    }
}

/// The extension of a name's last segment, as a path's: none for `.hidden`, none without a dot.
fn extension(uri: &str) -> Option<&str> {
    let name = uri.rsplit(['/', '\\']).next().unwrap_or(uri);
    let (stem, extension) = name.rsplit_once('.')?;
    (!stem.is_empty()).then_some(extension)
}
