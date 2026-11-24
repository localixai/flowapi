//! Text: inline markup removed, sentences, the RFC 2119 / RFC 8174 keywords, slugs.

/// HTML tags that end a line of text: what follows is a new sentence.
const BREAKING_TAGS: [&str; 6] = ["br", "li", "ul", "ol", "p", "td"];

/// Removes inline Markdown and HTML: `[text](url)` becomes `text`, `[[REF]]` becomes `[REF]`, tags
/// and `**` go, `\|` becomes `|`. Code spans are kept as written. A breaking tag becomes `\n`.
pub fn clean_inline(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    let mut in_code = false;
    while let Some(&ch) = chars.get(index) {
        if ch == '`' {
            in_code = !in_code;
            out.push(ch);
            index += 1;
        } else if in_code {
            out.push(ch);
            index += 1;
        } else if ch == '\\'
            && chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_punctuation())
        {
            chars
                .get(index + 1)
                .into_iter()
                .for_each(|next| out.push(*next));
            index += 2;
        } else if ch == '*' && chars.get(index + 1) == Some(&'*') {
            index += 2;
        } else if ch == '[' && chars.get(index + 1) == Some(&'[') {
            index = respec_reference(&chars, index, &mut out);
        } else if ch == '[' {
            index = link(&chars, index, &mut out);
        } else if ch == '<' {
            index = tag(&chars, index, &mut out);
        } else {
            out.push(ch);
            index += 1;
        }
    }
    collapse(&out)
}

/// `[[?X]]`, `[[X]]` or `[[X|Y]]`: a citation; kept as `[X]` or `[Y]`.
fn respec_reference(chars: &[char], start: usize, out: &mut String) -> usize {
    let rest: String = chars.get(start + 2..).unwrap_or_default().iter().collect();
    let Some(end) = rest.find("]]") else {
        out.push('[');
        return start + 1;
    };
    let inner = rest.get(..end).unwrap_or_default();
    let label = inner
        .rsplit('|')
        .next()
        .unwrap_or(inner)
        .trim_start_matches(['?', '!']);
    out.push('[');
    out.push_str(label);
    out.push(']');
    start + 2 + inner.chars().count() + 2
}

/// `[text](target)`: the text, cleaned; anything else keeps its `[`.
fn link(chars: &[char], start: usize, out: &mut String) -> usize {
    let Some(close) = matching(chars, start, '[', ']') else {
        out.push('[');
        return start + 1;
    };
    if chars.get(close + 1) != Some(&'(') {
        out.push('[');
        return start + 1;
    }
    let Some(paren) = matching(chars, close + 1, '(', ')') else {
        out.push('[');
        return start + 1;
    };
    let inner: String = chars
        .get(start + 1..close)
        .unwrap_or_default()
        .iter()
        .collect();
    out.push_str(&clean_inline(&inner));
    paren + 1
}

fn matching(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in chars.get(start..)?.iter().enumerate() {
        if *ch == open {
            depth += 1;
        } else if *ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

/// An HTML tag is dropped (a breaking one becomes `\n`); a lone `<` is text.
fn tag(chars: &[char], start: usize, out: &mut String) -> usize {
    let after = chars.get(start + 1).copied();
    let opens_tag = after.is_some_and(|c| c.is_ascii_alphabetic() || c == '/');
    let end = chars
        .get(start..)
        .and_then(|rest| rest.iter().position(|c| *c == '>'));
    match (opens_tag, end) {
        (true, Some(length)) => {
            let body: String = chars
                .get(start + 1..start + length)
                .unwrap_or_default()
                .iter()
                .collect();
            let name: String = body
                .trim_start_matches('/')
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect::<String>()
                .to_ascii_lowercase();
            if BREAKING_TAGS.contains(&name.as_str()) {
                out.push('\n');
            }
            start + length + 1
        }
        _ => {
            out.push('<');
            start + 1
        }
    }
}

/// Runs of spaces become one space; each line is trimmed; empty lines go.
pub fn collapse(text: &str) -> String {
    text.split('\n')
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Words before a period that do not end a sentence.
const ABBREVIATIONS: [&str; 8] = ["e.g", "i.e", "etc", "vs", "cf", "eg", "ie", "approx"];

/// Splits text into sentences: at `\n`, and after `.`, `?` or `!` followed by space and a
/// character that starts a sentence. Code spans are never split.
pub fn sentences(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.split('\n') {
        let chars: Vec<char> = line.chars().collect();
        let mut current = String::new();
        let mut in_code = false;
        let mut index = 0;
        while let Some(&ch) = chars.get(index) {
            current.push(ch);
            index += 1;
            if ch == '`' {
                in_code = !in_code;
            }
            if in_code || !matches!(ch, '.' | '?' | '!') {
                continue;
            }
            while let Some(&closer) = chars
                .get(index)
                .filter(|c| matches!(c, ')' | '"' | '\'' | '*' | '_'))
            {
                current.push(closer);
                index += 1;
            }
            let space = chars.get(index).is_some_and(|c| c.is_whitespace());
            let next = chars.get(index + 1).copied();
            let starts = next.is_some_and(|c| c.is_uppercase() || "`[\"*_(<".contains(c));
            if space && starts && !ends_in_abbreviation(&current) {
                found.push(current.trim().to_owned());
                current.clear();
            }
        }
        if !current.trim().is_empty() {
            found.push(current.trim().to_owned());
        }
    }
    found
}

fn ends_in_abbreviation(sentence: &str) -> bool {
    let trimmed = sentence.trim_end_matches(|c: char| !c.is_alphanumeric());
    let word = trimmed
        .rsplit(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or("");
    ABBREVIATIONS.contains(&word.to_ascii_lowercase().as_str())
}

/// A heading's slug: lower case, every run of other characters than letters and digits one `-`.
pub fn heading_slug(title: &str) -> String {
    dashed(&title.to_ascii_lowercase(), |c| c.is_ascii_alphanumeric())
}

/// A field's or keyword's slug: its case kept, `$` and `_` kept.
pub fn name_slug(name: &str) -> String {
    dashed(name, |c| c.is_ascii_alphanumeric() || c == '$' || c == '_')
}

fn dashed(text: &str, keep: impl Fn(char) -> bool) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if keep(ch) {
            out.push(ch);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}
