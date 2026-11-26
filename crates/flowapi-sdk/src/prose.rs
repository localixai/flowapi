//! A description's prose, CommonMark as OpenAPI writes it, made plain text for the places that
//! show no Markdown: `package.json`'s `description`, a doc comment, a heading. A paragraph keeps
//! its words and loses its markup: a link or an image is its text, code its content, emphasis its
//! words, an HTML tag nothing; a heading, a list item and a quote are paragraphs of their own;
//! fenced and indented code is left out. Never panics, whatever the text (G1).

#[cfg(test)]
#[path = "prose_tests.rs"]
mod tests;

/// The most characters of a sentence that is put where a line is short (`package.json`).
pub(crate) const MAX_SENTENCE: usize = 200;

/// What a paragraph was in the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Text,
    Heading,
    Item,
}

/// The text's paragraphs as plain text, each on one line, none empty.
pub(crate) fn paragraphs(markdown: &str) -> Vec<String> {
    blocks(markdown).into_iter().map(|(_, t)| t).collect()
}

fn blocks(markdown: &str) -> Vec<(Block, String)> {
    let mut out: Vec<(Block, String)> = Vec::new();
    let mut open = String::new();
    let mut fence: Option<&str> = None;
    let push = |kind: Block, text: &mut String, out: &mut Vec<(Block, String)>| {
        let plain = inline(text);
        if !plain.is_empty() {
            out.push((kind, plain));
        }
        text.clear();
    };
    for raw in markdown.lines() {
        let line = raw.trim();
        if let Some(f) = fence {
            if line.starts_with(f) {
                fence = None;
            }
            continue;
        }
        if let Some(f) = ["```", "~~~"].into_iter().find(|f| line.starts_with(f)) {
            push(Block::Text, &mut open, &mut out);
            fence = Some(f);
            continue;
        }
        if line.is_empty() || raw.starts_with("    ") || raw.starts_with('\t') {
            push(Block::Text, &mut open, &mut out);
            continue;
        }
        match block(line) {
            (Block::Text, text) => {
                if !open.is_empty() {
                    open.push(' ');
                }
                open.push_str(&text);
            }
            (kind, mut text) => {
                push(Block::Text, &mut open, &mut out);
                push(kind, &mut text, &mut out);
            }
        }
    }
    push(Block::Text, &mut open, &mut out);
    out
}

/// A line without its block marker, and what it is: a heading and a list item are paragraphs of
/// their own, a quote's line joins the paragraph it is in.
fn block(line: &str) -> (Block, String) {
    let quote = line.trim_start_matches('>').trim_start();
    if quote.len() != line.len() {
        return (Block::Text, quote.to_owned());
    }
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hashes) && line.chars().nth(hashes).is_none_or(|c| c == ' ') {
        let text = line
            .get(hashes..)
            .unwrap_or("")
            .trim()
            .trim_end_matches('#');
        return (Block::Heading, text.trim().to_owned());
    }
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return (Block::Item, format!("- {rest}"));
        }
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits > 0
        && let Some(rest) = line.get(digits..).and_then(|r| r.strip_prefix(". "))
    {
        return (Block::Item, format!("- {rest}"));
    }
    (Block::Text, line.to_owned())
}

/// The first sentence of the text's first paragraph that is no heading, when it is a sentence of
/// at most [`MAX_SENTENCE`] characters; a sentence ends at `.`, `!` or `?` before a space and a
/// capital, or at the paragraph's end.
pub(crate) fn sentence(markdown: &str) -> Option<String> {
    let (_, first) = blocks(markdown)
        .into_iter()
        .find(|(kind, _)| *kind == Block::Text)?;
    let chars: Vec<char> = first.chars().collect();
    let mut end = chars.len();
    for (i, w) in chars.windows(3).enumerate() {
        if let [stop, ' ', next] = w
            && matches!(stop, '.' | '!' | '?')
            && next.is_uppercase()
        {
            end = i + 1;
            break;
        }
    }
    let s: String = chars.iter().take(end).collect();
    let s = s.trim().to_owned();
    (!s.is_empty() && s.chars().count() <= MAX_SENTENCE).then_some(s)
}

/// One paragraph's inline markup taken out, its whitespace made single spaces.
pub(crate) fn inline(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while let Some(&c) = chars.get(i) {
        let at = |k: usize| chars.get(k).copied();
        match c {
            '\\' if at(i + 1).is_some_and(|n| n.is_ascii_punctuation()) => {
                out.extend(at(i + 1));
                i += 2;
            }
            '!' if at(i + 1) == Some('[') => i += 1,
            '[' => i = link(&chars, i, &mut out),
            '<' => i = angle(&chars, i, &mut out),
            '`' => i += 1,
            '*' | '_' => {
                let before = i.checked_sub(1).and_then(at).unwrap_or(' ');
                let after = at(i + 1).unwrap_or(' ');
                let word = |x: char| x.is_alphanumeric();
                // A marker is emphasis only where another closes it: a lone `*` is text.
                let paired = chars.iter().filter(|&&x| x == c).count() > 1;
                let keep = !paired
                    || (before.is_whitespace() && after.is_whitespace())
                    || (c == '_' && word(before) && word(after));
                if keep {
                    out.push(c);
                }
                i += 1;
            }
            '&' => i = entity(&chars, i, &mut out),
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The index of the first `close` after `from` at the same depth of `open`, when there is one.
fn closing(chars: &[char], from: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (k, &c) in chars.iter().enumerate().skip(from) {
        if c == open {
            depth += 1;
        } else if c == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(k);
            }
        }
    }
    None
}

/// `[text](url)`, `[text][ref]` and `[text]` as their text; the index after them.
fn link(chars: &[char], i: usize, out: &mut String) -> usize {
    let Some(end) = closing(chars, i, '[', ']') else {
        out.push('[');
        return i + 1;
    };
    let text: String = chars.get(i + 1..end).unwrap_or(&[]).iter().collect();
    out.push_str(&inline(&text));
    match chars.get(end + 1) {
        Some('(') => closing(chars, end + 1, '(', ')').map_or(end + 1, |k| k + 1),
        Some('[') => closing(chars, end + 1, '[', ']').map_or(end + 1, |k| k + 1),
        Some(_) | None => end + 1,
    }
}

/// An autolink (`<https://…>`) as its URL, an HTML tag as nothing, else a `<`.
fn angle(chars: &[char], i: usize, out: &mut String) -> usize {
    let close = chars.iter().skip(i).position(|&c| c == '>').map(|k| k + i);
    let inner: String = close
        .and_then(|k| chars.get(i + 1..k))
        .unwrap_or(&[])
        .iter()
        .collect();
    let tag = inner
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!');
    match close {
        Some(k) if inner.starts_with("http://") || inner.starts_with("https://") => {
            out.push_str(&inner);
            k + 1
        }
        Some(k) if tag && (!inner.contains(' ') || inner.contains('=')) => k + 1,
        Some(_) | None => {
            out.push('<');
            i + 1
        }
    }
}

/// An HTML entity as its character (the few descriptions write); the index after it.
fn entity(chars: &[char], i: usize, out: &mut String) -> usize {
    const ENTITIES: &[(&str, char)] = &[
        ("&amp;", '&'),
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&quot;", '"'),
        ("&#39;", '\''),
        ("&apos;", '\''),
        ("&nbsp;", ' '),
    ];
    let rest: String = chars.iter().skip(i).take(6).collect();
    for (name, c) in ENTITIES {
        if rest.starts_with(name) {
            out.push(*c);
            return i + name.chars().count();
        }
    }
    out.push('&');
    i + 1
}
