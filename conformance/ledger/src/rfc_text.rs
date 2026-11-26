//! Reads an RFC's plain text, as the RFC Editor publishes it (`rfcNNNN.txt`), into a [`Doc`].
//!
//! For the RFCs that have no XML source at the RFC Editor (those before RFCXML v3). A section
//! heading starts at column 0 with its number (`4.4.2.  Access Token Response`, `Appendix A.  ...`,
//! `A.1.  ...`); its component is that number, so the ids are `rfc6749.4.4.2.<term>-<slug>` and
//! its anchor `section-4.4.2` or `appendix-A.1` as the RFC Editor's HTML renders them. Each run
//! of non-blank lines is a prose unit, and so is each bulleted item (`o  `), its bullet left
//! out; a word the line breaks after its hyphen is joined again. Page breaks (the footer, the form feed and the running
//! header) are removed, and a paragraph cut by one is joined again.
//!
//! What states no requirement of the protocol is not read: everything before section 1 (the
//! abstract, Status of This Memo, the copyright notice, the table of contents), the References
//! sections, and the unnumbered sections at the end (Acknowledgements, Authors' Addresses). The
//! sentence that defines the keywords quotes them, so `terms` finds none in it. Only keywords in
//! capitals count, as BCP 14 says for these RFCs too (RFC 2119's own use is capitals).

use crate::doc::{Doc, Heading, Unit, UnitKind};
use crate::text::collapse;

/// Reads an RFC's text.
pub fn read(source: &str) -> Doc {
    let mut reader = Reader::default();
    for (number, line) in without_pages(source) {
        reader.line(number, line);
    }
    reader.flush();
    reader.doc
}

/// The lines of the text, page breaks removed: (line number, line). A paragraph a page break
/// cuts runs on: the line after the break is as indented as the one before it, and that one does
/// not end a sentence. Otherwise one blank line keeps the two apart.
fn without_pages(source: &str) -> Vec<(usize, &str)> {
    let mut kept: Vec<(usize, &str)> = Vec::new();
    let mut header_next = false;
    // After a break: the indent of the last line before it, when that line ends no sentence.
    let mut cut: Option<Option<usize>> = None;
    for (index, line) in source.lines().enumerate() {
        let number = index + 1;
        if std::mem::take(&mut header_next) && line.starts_with("RFC ") {
            continue;
        }
        if line.starts_with('\u{c}') {
            header_next = true;
            continue;
        }
        if is_footer(line) {
            while kept.last().is_some_and(|(_, l)| l.trim().is_empty()) {
                kept.pop();
            }
            let open = kept
                .last()
                .filter(|(_, l)| !l.trim_end().ends_with(['.', ':']))
                .map(|(_, l)| indent(l));
            cut = Some(open);
            continue;
        }
        if line.trim().is_empty() && cut.is_some() {
            continue;
        }
        if let Some(open) = cut.take()
            && open != Some(indent(line))
        {
            kept.push((number, ""));
        }
        kept.push((number, line));
    }
    kept
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// `Hardt                        Standards Track                    [Page 4]`.
fn is_footer(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with(']') && trimmed.rfind("[Page ").is_some_and(|at| at > 0)
}

#[derive(Default)]
struct Reader {
    doc: Doc,
    /// Whether the lines read belong to a section whose text is read.
    reading: bool,
    /// The open heading.
    current: Option<usize>,
    /// The paragraph being read: its first line and its lines.
    block: Option<(usize, Vec<String>)>,
    /// The indent of the paragraph's last line: a line less indented starts a new one (prose
    /// after an example, a list's next term).
    last_indent: usize,
}

impl Reader {
    fn line(&mut self, number: usize, line: &str) {
        if let Some((label, title)) = heading(line) {
            self.flush();
            self.heading(number, &label, title);
            return;
        }
        let unnumbered = line.starts_with(|c: char| c.is_ascii_alphabetic());
        if unnumbered && !self.doc.headings.is_empty() {
            self.flush();
            self.reading = false;
            return;
        }
        if !self.reading {
            return;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            self.flush();
            return;
        }
        let fell_back = indent(line) < std::mem::replace(&mut self.last_indent, indent(line));
        if fell_back {
            self.flush();
        }
        if let Some(item) = bullet(trimmed) {
            self.flush();
            self.block = Some((number, vec![item.to_owned()]));
            return;
        }
        match &mut self.block {
            Some((_, lines)) => lines.push(trimmed.to_owned()),
            None => self.block = Some((number, vec![trimmed.to_owned()])),
        }
    }

    fn heading(&mut self, number: usize, label: &str, title: &str) {
        let parent_label = label.rsplit_once('.').map(|(head, _)| head);
        let parent = parent_label.and_then(|wanted| {
            self.doc
                .headings
                .iter()
                .rposition(|h| h.component == wanted)
        });
        let appendix = label.starts_with(|c: char| c.is_ascii_uppercase());
        let anchor = match appendix {
            true => format!("appendix-{label}"),
            false => format!("section-{label}"),
        };
        let title = collapse(title);
        self.reading = !is_references(&title);
        self.current = Some(self.doc.headings.len());
        self.doc.headings.push(Heading {
            title,
            component: label.to_owned(),
            base: true,
            anchor,
            parent,
            line: number,
        });
    }

    fn flush(&mut self) {
        let Some((line, lines)) = self.block.take() else {
            return;
        };
        let mut text = String::new();
        for part in lines {
            let hyphen = text.ends_with('-')
                && text.chars().rev().nth(1).is_some_and(char::is_alphanumeric)
                && part.starts_with(|c: char| c.is_ascii_alphanumeric());
            if !text.is_empty() && !hyphen {
                text.push(' ');
            }
            text.push_str(&part);
        }
        self.doc.units.push(Unit {
            heading: self.current,
            line,
            kind: UnitKind::Prose,
            text: collapse(&text),
        });
    }
}

/// A list item's text without its bullet (`o  `, `*  `, `-  `): each item is a unit of its own.
fn bullet(line: &str) -> Option<&str> {
    ["o ", "* ", "- "]
        .iter()
        .find_map(|marker| line.strip_prefix(marker))
        .map(str::trim_start)
}

fn is_references(title: &str) -> bool {
    title.ends_with("References")
}

/// A section heading at column 0: `4.4.2.  Title` gives (`4.4.2`, `Title`), `Appendix A.  Title`
/// gives (`A`, `Title`), `A.1.  Title` gives (`A.1`, `Title`).
pub fn heading(line: &str) -> Option<(String, &str)> {
    let (label, rest) = match line.strip_prefix("Appendix ") {
        Some(rest) => rest.split_once(". ")?,
        None => line.split_once(". ")?,
    };
    let title = rest.trim();
    let mut parts = label.split('.');
    let first = parts.next()?;
    let numbered = !first.is_empty() && first.chars().all(|c| c.is_ascii_digit());
    let lettered = first.len() == 1 && first.chars().all(|c| c.is_ascii_uppercase());
    let rest_numbers = parts.all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    let appendix_word = line.starts_with("Appendix ");
    let shape =
        (numbered && !appendix_word) || (lettered && (appendix_word || label.contains('.')));
    (shape && rest_numbers && !title.is_empty() && !title.ends_with(['.', ',']))
        .then(|| (label.to_owned(), title))
}
