//! Identifiers made Rust's: items and variants in upper camel case, fields in snake case, both
//! kept clear of the language's keywords; a name taken once is numbered the next time.

use std::collections::BTreeSet;

/// Rust's keywords, strict and reserved, in every edition up to 2024.
pub(crate) const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// The keywords a raw identifier (`r#type`) cannot spell.
const NOT_RAW: &[&str] = &["crate", "self", "Self", "super"];

/// Names already given in one scope; a name given again is numbered.
#[derive(Default)]
pub(crate) struct Taken(BTreeSet<String>);

impl Taken {
    /// A scope where `names` are already given.
    pub(crate) fn of<'a>(names: impl IntoIterator<Item = &'a str>) -> Self {
        Self(names.into_iter().map(str::to_owned).collect())
    }

    /// `base` if it is free, else `base2`, `base3`, and so on; the one returned is now given.
    pub(crate) fn take(&mut self, base: &str) -> String {
        let mut candidate = base.to_owned();
        let mut n = 1;
        while self.0.contains(&candidate) {
            n += 1;
            candidate = format!("{base}{n}");
        }
        self.0.insert(candidate.clone());
        candidate
    }
}

/// The fields' identifiers of a struct whose keys are `names`, in order, and the scope they
/// leave for what follows (`extra`). The SDK reads a field by the same identifiers.
pub(crate) fn fields<'a>(names: impl IntoIterator<Item = &'a str>) -> (Vec<String>, Taken) {
    let mut taken = Taken::default();
    let idents = names.into_iter().map(|n| taken.take(&snake(n))).collect();
    (idents, taken)
}

/// `pet owner`, `pet-owner`, `pet_owner` and `petOwner` are `PetOwner`; what holds no ASCII
/// letter or digit is `fallback`, and a name that starts with a digit is given `prefix`.
pub(crate) fn pascal(text: &str, fallback: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            out.push(c.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    match out.chars().next() {
        None => fallback.to_owned(),
        Some(c) if c.is_ascii_digit() => format!("{prefix}{out}"),
        Some(_) => out,
    }
}

/// `petOwner`, `Pet-Owner` and `HTTPServer` are `pet_owner`, `pet_owner` and `http_server`; a
/// keyword is written raw (`r#type`), or with `_` after it where Rust allows no raw form.
pub(crate) fn snake(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !out.is_empty() && !out.ends_with('_') {
                out.push('_');
            }
            continue;
        }
        if c.is_ascii_uppercase() && i > 0 && !out.is_empty() && !out.ends_with('_') {
            let prev = chars.get(i - 1).copied().unwrap_or(' ');
            let next = chars.get(i + 1).copied().unwrap_or(' ');
            if prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next.is_ascii_lowercase())
            {
                out.push('_');
            }
        }
        out.push(c.to_ascii_lowercase());
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        return String::from("field");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if NOT_RAW.contains(&out.as_str()) {
        out.push('_');
    } else if KEYWORDS.contains(&out.as_str()) {
        out.insert_str(0, "r#");
    }
    out
}

/// A Rust string literal of `text`: `{:?}` writes one, escapes included.
pub(crate) fn literal(text: &str) -> String {
    format!("{text:?}")
}
