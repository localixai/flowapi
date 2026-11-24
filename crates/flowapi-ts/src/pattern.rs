//! A `patternProperties` regular expression as a TypeScript template literal type, when it is a
//! literal text with at most `^` before it and `$` after it: `^x-` is `` `x-${string}` ``. Any
//! other expression has no type, and the object keeps one index signature for every name.

/// A pattern made a template.
pub(crate) struct Template {
    /// The literal text.
    text: String,
    /// The name must start with the text (`^`).
    start: bool,
    /// The name must end with it (`$`).
    end: bool,
}

impl Template {
    /// The template of a regular expression; `None` for one no template states.
    pub(crate) fn of(regex: &str) -> Option<Self> {
        let (start, rest) = regex
            .strip_prefix('^')
            .map_or((false, regex), |r| (true, r));
        let (end, body) = match rest.strip_suffix('$') {
            Some(b) if !b.ends_with('\\') => (true, b),
            Some(_) | None => (false, rest),
        };
        if start && end {
            return None;
        }
        let mut text = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            let literal = if c == '\\' {
                let next = chars.next()?;
                if next.is_ascii_alphanumeric() {
                    return None;
                }
                next
            } else if "^$.*+?()[]{}|".contains(c) {
                return None;
            } else {
                c
            };
            if "`$\\{".contains(literal) || literal.is_control() {
                return None;
            }
            text.push(literal);
        }
        if text.is_empty() {
            return None;
        }
        Some(Self { text, start, end })
    }

    /// Whether a name matches.
    pub(crate) fn matches(&self, name: &str) -> bool {
        match (self.start, self.end) {
            (true, false) => name.starts_with(&self.text),
            (false, true) => name.ends_with(&self.text),
            (true, true) => name == self.text,
            (false, false) => name.contains(&self.text),
        }
    }

    /// The template literal type.
    pub(crate) fn print(&self, out: &mut String) {
        out.push('`');
        if !self.start {
            out.push_str("${string}");
        }
        out.push_str(&self.text);
        if !self.end {
            out.push_str("${string}");
        }
        out.push('`');
    }
}
