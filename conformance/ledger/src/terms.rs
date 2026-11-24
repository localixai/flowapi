//! The RFC 2119 / RFC 8174 keywords in a sentence, the level they give it and its id slug.

/// The keywords of BCP 14, in the order their strength ranks them.
const WORDS: [&str; 8] = [
    "MUST",
    "SHALL",
    "SHOULD",
    "REQUIRED",
    "RECOMMENDED",
    "MAY",
    "OPTIONAL",
    "NOT",
];

/// The BCP 14 terms in a sentence, in order of appearance, each once: `MUST`, `MUST NOT`,
/// `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `NOT RECOMMENDED`,
/// `MAY`, `OPTIONAL`. A term in quotes (`"MUST"`, the boilerplate that defines them) or in a code
/// span is not one.
pub fn terms(sentence: &str) -> Vec<&'static str> {
    let words = capital_words(sentence);
    let mut found: Vec<&'static str> = Vec::new();
    let mut index = 0;
    while let Some(word) = words.get(index) {
        let next = words.get(index + 1).filter(|next| next.adjacent_to(word));
        let (term, used) = match (word.text, next.map(|n| n.text)) {
            ("MUST", Some("NOT")) => ("MUST NOT", 2),
            ("SHALL", Some("NOT")) => ("SHALL NOT", 2),
            ("SHOULD", Some("NOT")) => ("SHOULD NOT", 2),
            ("NOT", Some("RECOMMENDED")) => ("NOT RECOMMENDED", 2),
            ("NOT", _) => ("", 1),
            (other, _) => (other, 1),
        };
        let last = words.get(index + used - 1).unwrap_or(word);
        let quoted = word.before == Some('"') && last.after == Some('"');
        if !term.is_empty() && !quoted && !word.in_code && !found.contains(&term) {
            found.push(term);
        }
        index += used;
    }
    found
}

#[derive(Debug)]
struct Word {
    text: &'static str,
    start: usize,
    end: usize,
    before: Option<char>,
    after: Option<char>,
    in_code: bool,
}

impl Word {
    fn adjacent_to(&self, previous: &Word) -> bool {
        self.start == previous.end + 1
    }
}

fn capital_words(sentence: &str) -> Vec<Word> {
    let chars: Vec<char> = sentence.chars().collect();
    let mut words = Vec::new();
    let mut in_code = false;
    let mut index = 0;
    while let Some(&ch) = chars.get(index) {
        if ch == '`' {
            in_code = !in_code;
        }
        let boundary = index == 0
            || chars
                .get(index - 1)
                .is_none_or(|c| !c.is_alphanumeric() && *c != '_');
        if !(ch.is_ascii_uppercase() && boundary) {
            index += 1;
            continue;
        }
        let end = index
            + chars
                .get(index..)
                .unwrap_or_default()
                .iter()
                .take_while(|c| c.is_ascii_uppercase())
                .count();
        let after = chars.get(end).copied();
        let token: String = chars.get(index..end).unwrap_or_default().iter().collect();
        let closed = after.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if let (true, Some(text)) = (closed, WORDS.iter().find(|w| **w == token)) {
            let before = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            words.push(Word {
                text,
                start: index,
                end,
                before,
                after,
                in_code,
            });
        }
        index = end;
    }
    words
}

/// The strength of a term: 3 for MUST, 2 for SHOULD, 1 for MAY.
pub fn rank(term: &str) -> u8 {
    match term {
        "MUST" | "MUST NOT" | "REQUIRED" | "SHALL" | "SHALL NOT" => 3,
        "SHOULD" | "SHOULD NOT" | "RECOMMENDED" | "NOT RECOMMENDED" => 2,
        _ => 1,
    }
}

/// The level of a sentence: its strongest term's group.
pub fn level(terms: &[&str]) -> &'static str {
    match terms.iter().map(|t| rank(t)).max() {
        Some(3) => "MUST",
        Some(2) => "SHOULD",
        _ => "MAY",
    }
}

/// The first of the strongest terms, as an id word: `must-not`, `should`, `not-recommended`.
pub fn lead_term(terms: &[&str]) -> String {
    let strongest = terms.iter().map(|t| rank(t)).max().unwrap_or(1);
    let lead = terms
        .iter()
        .find(|t| rank(t) == strongest)
        .copied()
        .unwrap_or("MAY");
    lead.to_ascii_lowercase().replace(' ', "-")
}

const STOPWORDS: [&str; 40] = [
    "a", "an", "the", "of", "to", "and", "or", "is", "are", "be", "been", "this", "that", "these",
    "those", "it", "its", "in", "on", "at", "for", "with", "as", "by", "from", "which", "than",
    "then", "there", "their", "if", "when", "where", "so", "such", "also", "can", "into", "was",
    "has",
];

/// A short slug of a sentence: its first `count` words, BCP 14 terms and stop words left out.
pub fn sentence_slug(sentence: &str, count: usize) -> String {
    sentence
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .filter(|word| !WORDS.contains(word))
        .map(str::to_ascii_lowercase)
        .filter(|word| !STOPWORDS.contains(&word.as_str()))
        .take(count)
        .collect::<Vec<_>>()
        .join("-")
}
