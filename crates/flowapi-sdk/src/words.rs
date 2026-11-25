//! Names as words, not strings: `list_all`, `listAll` and `list-all` are the one name
//! `[list, all]`. A config names its resources and methods by words and is checked by words (two
//! names equal as words are one name); a target prints the words as its identifiers (`listAll`,
//! `ListAll`), so an identifier never depends on how the config spelled it.

/// A name's words, in lower case: split at every character that is not an ASCII letter or digit,
/// before an upper-case letter that follows a lower-case one or a digit (`listAll`), and before
/// the last capital of a run that a lower-case letter follows (`HTTPServer` is `http server`).
/// A digit stays with the word before it (`v2`, `oauth2`). Characters outside ASCII separate.
pub(crate) fn words(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut word = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            flush(&mut out, &mut word);
            continue;
        }
        if c.is_ascii_uppercase() && !word.is_empty() {
            let before = chars.get(i.wrapping_sub(1)).copied().unwrap_or(' ');
            let after = chars.get(i + 1).copied().unwrap_or(' ');
            let lower_before = before.is_ascii_lowercase() || before.is_ascii_digit();
            let run_ends = before.is_ascii_uppercase() && after.is_ascii_lowercase();
            if lower_before || run_ends {
                flush(&mut out, &mut word);
            }
        }
        word.push(c.to_ascii_lowercase());
    }
    flush(&mut out, &mut word);
    out
}

fn flush(out: &mut Vec<String>, word: &mut String) {
    if !word.is_empty() {
        out.push(std::mem::take(word));
    }
}

/// Whether two names are one name: equal as words.
pub(crate) fn same(a: &str, b: &str) -> bool {
    words(a) == words(b)
}

/// Whether a config may use `name` for a resource, a method, an environment or a scheme: ASCII
/// letters, digits, `_` and `-`, opening with a letter, so that every target prints it as an
/// identifier.
pub(crate) fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The pattern of [`is_name`], for the config's JSON Schema.
pub(crate) const NAME_PATTERN: &str = "^[A-Za-z][A-Za-z0-9_-]*$";

fn capital(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |c| {
        c.to_ascii_uppercase().to_string() + chars.as_str()
    })
}

/// The words as `PascalCase`: a class's identifier.
pub(crate) fn pascal(words: &[String]) -> String {
    words.iter().map(|w| capital(w)).collect()
}

/// The words joined by `sep` (`snake_case` with `_`, `kebab-case` with `-`).
pub(crate) fn join(words: &[String], sep: &str) -> String {
    words.join(sep)
}

/// The words as `UPPER_SNAKE`, an environment variable's name.
pub(crate) fn upper(words: &[String]) -> String {
    words.join("_").to_ascii_uppercase()
}
