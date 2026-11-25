//! Text this layer writes for an agent: a JSON string (RFC 8259), and a YAML scalar, plain where
//! YAML 1.2 reads it back as the same string, else double-quoted as JSON writes it (a JSON string
//! is a YAML double-quoted scalar).

use std::fmt::Write as _;

/// A string as a JSON string literal.
pub(crate) fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 || c == '\u{2028}' || c == '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Strings as a JSON array.
pub(crate) fn json_list<S: AsRef<str>>(items: &[S]) -> String {
    let items: Vec<String> = items.iter().map(|s| json_str(s.as_ref())).collect();
    format!("[{}]", items.join(","))
}

/// A string as a YAML scalar: plain when it is a word YAML reads as that string (letters,
/// digits, `_`, `-`, `.`, `/`, `@`, opening with a letter or `_`, and not one of the words a YAML
/// reader may take for a boolean or null), else quoted.
pub(crate) fn yaml_str(s: &str) -> String {
    let plain = s
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./@".contains(c))
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n"
        );
    if plain { s.to_owned() } else { json_str(s) }
}

/// A text cut to `max` characters, `…` at the end when it was longer, on one line.
pub(crate) fn short(s: &str, max: usize) -> String {
    let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() <= max {
        return one;
    }
    let mut cut: String = one.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}
