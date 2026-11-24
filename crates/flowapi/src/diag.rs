//! What generation found: each diagnostic names where (a document and the byte offset of the
//! node), the rule broken (a requirement id or one of our own) and what the data holds (G8).

use crate::id::DocId;
use std::fmt;

/// A place in the input: a document and a byte offset into its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Loc {
    /// The document.
    pub doc: DocId,
    /// The byte offset of the node's first byte.
    pub offset: u32,
}

/// How much a diagnostic matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The input is invalid, or generation cannot go on: no output is written.
    Error,
    /// The output is written and sound, but wider than the description (a keyword no type in
    /// the target expresses), or a choice was made for the user.
    Warning,
}

/// One finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// How much it matters.
    pub severity: Severity,
    /// The rule: a requirement id of `conformance/requirements.json`, or a code of our own.
    pub code: &'static str,
    /// What was found, with the value it is about.
    pub message: String,
    /// Where.
    pub at: Loc,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{level}[{}]: {}", self.code, self.message)
    }
}

/// The diagnostics of one run, in the order found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    list: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Records an error.
    pub fn error(&mut self, code: &'static str, at: Loc, message: impl Into<String>) {
        self.push(Severity::Error, code, at, message.into());
    }

    /// Records a warning.
    pub fn warn(&mut self, code: &'static str, at: Loc, message: impl Into<String>) {
        self.push(Severity::Warning, code, at, message.into());
    }

    fn push(&mut self, severity: Severity, code: &'static str, at: Loc, message: String) {
        self.list.push(Diagnostic {
            severity,
            code,
            message,
            at,
        });
    }

    /// Whether any is an error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.list.iter().any(|d| d.severity == Severity::Error)
    }

    /// Every diagnostic, in the order found.
    #[must_use]
    pub fn all(&self) -> &[Diagnostic] {
        &self.list
    }

    /// Takes the other run's diagnostics after these.
    pub fn extend(&mut self, other: Self) {
        self.list.extend(other.list);
    }
}
