//! A literal pattern (`flowapi::graph::Affix`, read by the core) as a TypeScript template literal
//! type: `^x-` is `` `x-${string}` ``, `_id$` is `` `${string}_id` ``. A pattern of one whole name
//! (`^x$`) or of no text is no template; the object keeps an index signature for it.

use flowapi::graph::Affix;

/// The template literal type of a pattern, when it has one.
pub(crate) fn template(affix: &Affix) -> Option<String> {
    if affix.text.is_empty() || (affix.start && affix.end) {
        return None;
    }
    let mut out = String::from("`");
    if !affix.start {
        out.push_str("${string}");
    }
    out.push_str(&affix.text);
    if !affix.end {
        out.push_str("${string}");
    }
    out.push('`');
    Some(out)
}
