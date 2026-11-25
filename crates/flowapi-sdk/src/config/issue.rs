//! What `config check` finds: each issue names its rule (a code of ours, `config.*`), the member
//! of the config it is about as a JSON Pointer and the byte offset of that member in the config's
//! JSON (a host turns it into the YAML line and column, G8), what is wrong, and, where one is
//! known, what to write instead.

use crate::out;
use flowapi::Severity;
use std::fmt::Write as _;

/// One finding about a config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// How much it matters: an error stops generation.
    pub severity: Severity,
    /// The rule: `config.<what>`.
    pub code: &'static str,
    /// The member it is about, as a JSON Pointer into the config (`/resources/pets/methods/list`).
    pub pointer: String,
    /// The byte offset of that member (its key, where it has one) in the config's JSON text.
    pub offset: u32,
    /// What is wrong, with the value it is about.
    pub message: String,
    /// What to write instead (the closest operation for a typo), where one is known.
    pub hint: Option<String>,
}

/// The issues of one check, in the order found.
#[derive(Debug, Default)]
pub(crate) struct Issues {
    pub(crate) list: Vec<Issue>,
}

impl Issues {
    /// Records an error.
    pub(crate) fn error(&mut self, code: &'static str, at: &At, message: String) {
        self.add(Severity::Error, code, at, message, None);
    }

    /// Records an error with what to write instead.
    pub(crate) fn error_hint(
        &mut self,
        code: &'static str,
        at: &At,
        message: String,
        hint: Option<String>,
    ) {
        self.add(Severity::Error, code, at, message, hint);
    }

    /// Records a warning.
    pub(crate) fn warn(&mut self, code: &'static str, at: &At, message: String) {
        self.add(Severity::Warning, code, at, message, None);
    }

    fn add(
        &mut self,
        severity: Severity,
        code: &'static str,
        at: &At,
        message: String,
        hint: Option<String>,
    ) {
        self.list.push(Issue {
            severity,
            code,
            pointer: at.pointer.clone(),
            offset: at.offset,
            message,
            hint,
        });
    }
}

/// A member of the config: its JSON Pointer and its offset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct At {
    pub(crate) pointer: String,
    pub(crate) offset: u32,
}

impl At {
    /// The member `key` under this one, at `offset`.
    pub(crate) fn child(&self, key: &str, offset: u32) -> Self {
        let escaped = key.replace('~', "~0").replace('/', "~1");
        Self {
            pointer: format!("{}/{escaped}", self.pointer),
            offset,
        }
    }
}

fn level(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

/// The issues as text, one block each, as a person reads them: `place(offset)` gives the line and
/// column of an offset (the YAML's, for a YAML config), `file` names the config.
pub fn issues_text(issues: &[Issue], file: &str, place: &dyn Fn(u32) -> (usize, usize)) -> String {
    let mut out = String::new();
    for i in issues {
        let (line, column) = place(i.offset);
        let pointer = if i.pointer.is_empty() {
            "/"
        } else {
            &i.pointer
        };
        let _ = writeln!(out, "{}[{}]: {}", level(i.severity), i.code, i.message);
        let _ = writeln!(out, "  at {file}:{line}:{column} ({pointer})");
        if let Some(h) = &i.hint {
            let _ = writeln!(out, "  hint: {h}");
        }
    }
    out
}

/// The issues as JSON, for an agent: `{"ok", "errors", "warnings", "issues": [{"severity",
/// "code", "pointer", "line", "column", "message", "hint"}]}`, `ok` when no issue is an error.
pub fn issues_json(issues: &[Issue], place: &dyn Fn(u32) -> (usize, usize)) -> String {
    let errors = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    let mut out = format!(
        "{{\"ok\":{},\"errors\":{errors},\"warnings\":{},\"issues\":[",
        errors == 0,
        issues.len() - errors
    );
    for (n, i) in issues.iter().enumerate() {
        let (line, column) = place(i.offset);
        if n > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n{{\"severity\":{},\"code\":{},\"pointer\":{},\"line\":{line},\"column\":{column},\"message\":{}",
            out::json_str(level(i.severity)),
            out::json_str(i.code),
            out::json_str(&i.pointer),
            out::json_str(&i.message)
        );
        if let Some(h) = &i.hint {
            let _ = write!(out, ",\"hint\":{}", out::json_str(h));
        }
        out.push('}');
    }
    out.push_str("\n]}\n");
    out
}

/// The name of `candidates` closest to `wrong`, when one is close enough to be a typo: equal in
/// another case, or within a third of its length in edits (at least two). The first of equals.
pub(crate) fn closest<'a>(
    wrong: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let lower = wrong.to_lowercase();
    let bound = (wrong.chars().count() / 3).max(2);
    let mut best: Option<(usize, &str)> = None;
    for c in candidates {
        let d = if c.to_lowercase() == lower {
            0
        } else {
            distance(&lower, &c.to_lowercase(), bound)
        };
        if d <= bound && best.is_none_or(|(b, _)| d < b) {
            best = Some((d, c));
        }
    }
    best.map(|(_, c)| c.to_owned())
}

/// The Levenshtein distance of two strings, in characters; anything over `bound` is `bound + 1`.
fn distance(a: &str, b: &str, bound: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > bound {
        return bound + 1;
    }
    // One row of the table at a time: `row[j]` is the distance of `a[..i]` and `b[..j]`.
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = i;
        let mut left = i + 1;
        let mut low = left;
        let mut next_row = Vec::with_capacity(row.len());
        next_row.push(left);
        for (cb, &up) in b.iter().zip(row.iter().skip(1)) {
            let d = (up + 1).min(left + 1).min(diagonal + usize::from(ca != cb));
            diagonal = up;
            left = d;
            low = low.min(d);
            next_row.push(d);
        }
        if low > bound {
            return bound + 1;
        }
        row = next_row;
    }
    row.last().map_or(bound + 1, |&d| d.min(bound + 1))
}
