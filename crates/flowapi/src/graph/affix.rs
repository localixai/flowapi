//! A regular expression that is a literal text with at most `^` before it and `$` after it, read
//! as data: `^x-` is "starts with `x-`". An unanchored side may carry `.*` (`f.*` is "contains
//! `f`": the `.*` may match nothing). Such a pattern is matched exactly here (ECMA-262 without
//! flags: `^` and `$` are the ends of the input), and a target can state it (a template literal
//! type); any other expression is `None` and stays unchecked.

/// A literal pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Affix {
    /// The literal text, unescaped; never holds a control character, `` ` ``, `$`, `\` or `{`.
    pub text: String,
    /// The text is at the start (`^`).
    pub start: bool,
    /// The text is at the end (`$`).
    pub end: bool,
}

impl Affix {
    /// The affix of a regular expression; `None` for one that is not a literal text.
    #[must_use]
    pub fn of(regex: &str) -> Option<Self> {
        let (start, rest) = regex
            .strip_prefix('^')
            .map_or((false, regex), |r| (true, r));
        let (end, body) = match rest.strip_suffix('$') {
            Some(b) if !b.ends_with('\\') => (true, b),
            Some(_) | None => (false, rest),
        };
        let body = if start {
            body
        } else {
            body.strip_prefix(".*").unwrap_or(body)
        };
        let body = match body.strip_suffix(".*") {
            Some(b) if !end && !b.ends_with('\\') => b,
            Some(_) | None => body,
        };
        let mut text = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            let literal = if c == '\\' {
                let next = chars.next()?;
                if next.is_ascii_alphanumeric() {
                    return None;
                }
                next
            } else if "^$.*+?()[]{}|/".contains(c) {
                return None;
            } else {
                c
            };
            if "`$\\{".contains(literal) || literal.is_control() {
                return None;
            }
            text.push(literal);
        }
        (!text.is_empty() || start || end).then_some(Self { text, start, end })
    }

    /// Whether a string matches.
    #[must_use]
    pub fn matches(&self, s: &str) -> bool {
        match (self.start, self.end) {
            (true, false) => s.starts_with(&self.text),
            (false, true) => s.ends_with(&self.text),
            (true, true) => s == self.text,
            (false, false) => s.contains(&self.text),
        }
    }
}
