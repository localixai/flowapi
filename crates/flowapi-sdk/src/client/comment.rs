//! A method's doc comment, what an editor shows on hover: the operation's summary, its
//! description's paragraphs, its route, a link to its `externalDocs` and `@deprecated` where the
//! description marks it so; all plain text (`prose.rs`), since a hover shows Markdown its own way
//! and a description's HTML or links would show as markup.

use super::doc;
use crate::ops::Op;
use crate::prose;
use std::fmt::Write as _;

/// A paragraph as a line of a doc comment: no `*/` ends it, and an `@` that opens it is no tag.
fn line(text: &str) -> String {
    let text = doc(text);
    match text.strip_prefix('@') {
        Some(rest) => format!("\\@{rest}"),
        None => text,
    }
}

/// The comment of a method calling `op` (`None` when the method's operation is not found) by
/// `METHOD /path`, indented for a class member.
pub(crate) fn method(op: Option<&Op>, method: &str, path: &str) -> String {
    let route = format!("`{method} {}`", doc(path));
    let mut lines: Vec<String> = Vec::new();
    if let Some(op) = op {
        let summary = op.summary.as_deref().map(prose::inline);
        if let Some(s) = summary.as_ref().filter(|s| !s.is_empty()) {
            lines.push(line(s));
        }
        for p in op
            .description
            .as_deref()
            .map(prose::paragraphs)
            .unwrap_or_default()
        {
            if summary.as_deref() != Some(p.as_str()) {
                lines.push(line(&p));
            }
        }
    }
    let mut tags: Vec<String> = Vec::new();
    if let Some((url, about)) = op.and_then(|o| o.external_docs.as_ref()) {
        let about = about.as_deref().map(prose::inline).unwrap_or_default();
        let see = format!("@see {} {}", doc(url), doc(&about));
        tags.push(see.trim_end().to_owned());
    }
    if op.is_some_and(|o| o.deprecated) {
        tags.push(String::from(
            "@deprecated The API's description marks this operation deprecated.",
        ));
    }
    if lines.is_empty() && tags.is_empty() {
        return format!("  /** {route} */\n");
    }
    let mut out = String::from("  /**\n");
    for l in &lines {
        let _ = writeln!(out, "   * {l}\n   *");
    }
    let _ = writeln!(out, "   * {route}");
    for t in &tags {
        let _ = writeln!(out, "   * {t}");
    }
    out.push_str("   */\n");
    out
}
