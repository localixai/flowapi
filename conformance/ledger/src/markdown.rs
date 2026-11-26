//! Reads an OpenAPI Specification text (Markdown, as the OAI publishes it) into a [`Doc`].
//!
//! Headings nest by level; the level-1 title is the document and is left out of paths. Fenced
//! code is skipped: examples state no requirement. A table whose first header cell is
//! `Field Name` or `Field Pattern` gives one field unit per row; any other table gives one prose
//! unit per cell. A table's rows may leave out the leading `|` (OpenAPI 2.0 writes them so): a
//! paragraph line with a `|` followed by a delimiter row (`---|:---:|---`) is a table's header,
//! and the lines with a `|` after it are its rows. Under a heading "JSON Schema Keywords", a list
//! item that opens with a keyword gives a keyword unit.

use crate::doc::{Doc, Heading, Unit, UnitKind};
use crate::text::{clean_inline, heading_slug};
use std::collections::BTreeMap;

/// Reads a Markdown specification.
pub fn read(source: &str) -> Doc {
    let mut reader = Reader::default();
    for (index, line) in source.lines().enumerate() {
        reader.line(index + 1, line);
    }
    reader.flush();
    reader.doc
}

#[derive(Default)]
struct Reader {
    doc: Doc,
    /// Open headings: (level, index).
    stack: Vec<(usize, usize)>,
    fence: bool,
    /// The paragraph or list item being read: its first line and its lines.
    block: Option<(usize, Vec<String>, bool)>,
    table: Vec<(usize, String)>,
    anchors: BTreeMap<String, usize>,
}

impl Reader {
    fn line(&mut self, number: usize, line: &str) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            self.flush();
            self.fence = !self.fence;
            return;
        }
        if self.fence {
            return;
        }
        if let Some((level, title)) = heading(line) {
            self.flush();
            self.heading(number, level, title);
        } else if trimmed.is_empty() {
            self.flush();
        } else if trimmed.starts_with('|') || (!self.table.is_empty() && trimmed.contains('|')) {
            self.flush_block();
            self.table.push((number, trimmed.to_owned()));
        } else if let Some(header) = self.bare_header(trimmed) {
            self.table.push(header);
            self.table.push((number, trimmed.to_owned()));
        } else if let Some(item) = list_item(trimmed) {
            self.flush();
            self.block = Some((number, vec![item.to_owned()], true));
        } else {
            self.flush_table();
            match &mut self.block {
                Some((_, lines, _)) => lines.push(trimmed.to_owned()),
                None => self.block = Some((number, vec![trimmed.to_owned()], false)),
            }
        }
    }

    fn heading(&mut self, number: usize, level: usize, raw: &str) {
        while self.stack.last().is_some_and(|(open, _)| *open >= level) {
            self.stack.pop();
        }
        if level == 1 {
            return;
        }
        let title = clean_inline(raw).replace('\n', " ");
        let plain = title.replace('`', "");
        let component = heading_slug(&plain);
        let anchor = explicit_anchor(raw).unwrap_or_else(|| self.respec_anchor(&component));
        let index = self.doc.headings.len();
        self.doc.headings.push(Heading {
            base: plain.ends_with(" Object"),
            title,
            component,
            anchor,
            parent: self.stack.last().map(|(_, open)| *open),
            line: number,
        });
        self.stack.push((level, index));
    }

    /// The id the rendered specification gives a heading: its slug, and from the second time a
    /// slug is used `-0`, `-1`, ... after it.
    fn respec_anchor(&mut self, slug: &str) -> String {
        let seen = self.anchors.entry(slug.to_owned()).or_insert(0);
        let anchor = match *seen {
            0 => slug.to_owned(),
            n => format!("{slug}-{}", n - 1),
        };
        *seen += 1;
        anchor
    }

    /// A delimiter row under a one-line paragraph with a `|`: that line, taken as a table's header.
    fn bare_header(&mut self, line: &str) -> Option<(usize, String)> {
        let delimiter = line.contains('|')
            && line.contains('-')
            && line.chars().all(|c| matches!(c, '-' | ':' | '|' | ' '));
        let header = match &self.block {
            Some((start, lines, false)) if delimiter && lines.len() == 1 => lines
                .first()
                .filter(|l| l.contains('|'))
                .map(|l| (*start, l.clone())),
            _ => None,
        };
        if header.is_some() {
            self.block = None;
        }
        header
    }

    fn current(&self) -> Option<usize> {
        self.stack.last().map(|(_, index)| *index)
    }

    fn flush(&mut self) {
        self.flush_block();
        self.flush_table();
    }

    fn flush_block(&mut self) {
        let Some((line, lines, item)) = self.block.take() else {
            return;
        };
        let text = clean_inline(&lines.join(" "));
        let heading = self.current();
        let under_keywords = heading
            .and_then(|h| self.doc.headings.get(h))
            .is_some_and(|h| h.title == "JSON Schema Keywords");
        let kind = match (item && under_keywords)
            .then(|| keyword_item(&text))
            .flatten()
        {
            Some((name, rest)) => {
                let unit = Unit {
                    heading,
                    line,
                    kind: keyword(name),
                    text: rest,
                };
                self.doc.units.push(unit);
                return;
            }
            None => UnitKind::Prose,
        };
        self.doc.units.push(Unit {
            heading,
            line,
            kind,
            text,
        });
    }

    fn flush_table(&mut self) {
        let rows = std::mem::take(&mut self.table);
        let Some(((_, header), body)) = rows.split_first() else {
            return;
        };
        let header = cells(header);
        let first = header
            .first()
            .map(|cell| clean_inline(cell))
            .unwrap_or_default();
        let fields = matches!(first.as_str(), "Field Name" | "Field Pattern");
        let heading = self.current();
        for (line, row) in body {
            let cells = cells(row);
            if cells.iter().all(|cell| is_rule(cell)) {
                continue;
            }
            if fields {
                let name_cell = cells.first().map(String::as_str).unwrap_or_default();
                let name = clean_inline(name_cell).replace('`', "");
                let anchor = explicit_anchor(name_cell);
                let patterned = first == "Field Pattern";
                let text = cells
                    .last()
                    .map(|cell| clean_inline(cell))
                    .unwrap_or_default();
                let kind = UnitKind::Field {
                    name,
                    patterned,
                    anchor,
                };
                self.doc.units.push(Unit {
                    heading,
                    line: *line,
                    kind,
                    text,
                });
            } else {
                for cell in &cells {
                    let text = clean_inline(cell);
                    self.doc.units.push(Unit {
                        heading,
                        line: *line,
                        kind: UnitKind::Prose,
                        text,
                    });
                }
            }
        }
    }
}

fn keyword(name: String) -> UnitKind {
    UnitKind::Keyword {
        name,
        own_section: false,
    }
}

/// `## Title` gives (2, "Title").
fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    let rest = line.get(level..)?;
    (1..=6).contains(&level).then_some(())?;
    rest.starts_with(' ').then(|| (level, rest.trim()))
}

/// The text of a list item, without its marker.
fn list_item(line: &str) -> Option<&str> {
    for marker in ["* ", "- ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return Some(rest);
        }
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    let rest = line.get(digits..)?;
    (digits > 0).then_some(())?;
    rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") "))
}

/// `title`, `pattern (This string SHOULD ...)` or `type - Value MUST be a string.`
fn keyword_item(text: &str) -> Option<(String, String)> {
    let name: String = text
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '$')
        .collect();
    let rest = text.get(name.len()..)?.trim();
    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '$') {
        return None;
    }
    let rest = if rest.is_empty() {
        String::new()
    } else if let Some(after) = rest.strip_prefix("- ") {
        after.to_owned()
    } else if rest.starts_with('(') && rest.ends_with(')') {
        rest.get(1..rest.len() - 1).unwrap_or_default().to_owned()
    } else {
        return None;
    };
    Some((name, rest))
}

/// `<a name="parameter-style"></a>` gives `parameter-style`.
fn explicit_anchor(cell: &str) -> Option<String> {
    let start = cell.find("<a name=\"")? + "<a name=\"".len();
    let length = cell.get(start..)?.find('"')?;
    cell.get(start..start + length).map(str::to_owned)
}

/// The cells of a table row; `\|` does not split.
fn cells(row: &str) -> Vec<String> {
    let inner = row.trim().trim_start_matches('|');
    let inner = inner
        .strip_suffix('|')
        .filter(|s| !s.ends_with('\\'))
        .unwrap_or(inner);
    let mut found = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if ch == '|' && !escaped {
            found.push(current.trim().to_owned());
            current.clear();
        } else {
            current.push(ch);
        }
        escaped = ch == '\\';
    }
    found.push(current.trim().to_owned());
    found
}

fn is_rule(cell: &str) -> bool {
    !cell.is_empty() && cell.chars().all(|c| matches!(c, '-' | ':' | ' '))
}
