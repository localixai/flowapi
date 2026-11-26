//! Reads an RFC's RFCXML v3 source, as the RFC Editor publishes it (`rfcNNNN.xml`), into a [`Doc`].
//!
//! The published v3 file is "prepped": every section carries its number in `pn`
//! (`section-11.6.1`, `section-appendix.b.1`). A section's component is that number (`11.6.1`,
//! `B.1`), so the ids are `rfc9110.11.6.1.<term>-<slug>`, as for an RFC read from its text
//! (`rfc_text`); its anchor is `section-11.6.1` or `appendix-B.1`, as the RFC Editor's HTML
//! renders it. Each `<t>`, list item, definition and table cell is a prose unit (a `<dt>`'s
//! text leads its `<dd>`); code and artwork (`<sourcecode>`, `<artwork>`) are examples and
//! grammar, and index entries (`<iref>`) are not text, so neither is read. `<bcp14>` holds a
//! keyword in capitals; `terms` finds it as it finds one in plain text.
//!
//! The boilerplate (`<front>`: Status of This Memo, the copyright notice) and `<references>`
//! are outside `<middle>` and `<back>`'s sections; the unnumbered sections (Acknowledgements,
//! the index, Authors' Addresses) are not read.

use crate::doc::{Doc, Heading, Unit, UnitKind};
use crate::text::collapse;
use crate::xml::{Element, Node};

/// Reads a prepped RFCXML v3 document.
pub fn read(root: &Element) -> Doc {
    let mut doc = Doc::default();
    for part in root.elements() {
        if matches!(part.name.as_str(), "middle" | "back") {
            for section in part.elements().filter(|e| e.name == "section") {
                read_section(&mut doc, section, None);
            }
        }
    }
    doc
}

/// `section-11.6.1` gives `11.6.1`; `section-appendix.b.1` gives `B.1`.
pub fn number(pn: &str) -> Option<String> {
    let rest = pn.strip_prefix("section-")?;
    match rest.strip_prefix("appendix.") {
        Some(appendix) => {
            let (letter, tail) = appendix.split_at_checked(1)?;
            Some(format!("{}{tail}", letter.to_ascii_uppercase()))
        }
        None => rest
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
            .then(|| rest.to_owned()),
    }
}

fn read_section(doc: &mut Doc, section: &Element, parent: Option<usize>) {
    let numbered = section.attr("numbered") != Some("false");
    let Some(label) = section.attr("pn").and_then(number).filter(|_| numbered) else {
        return;
    };
    let title = section
        .elements()
        .find(|e| e.name == "name")
        .map(|name| collapse(&inline_text(name)).replace('\n', " "))
        .unwrap_or_default();
    let anchor = match label.starts_with(|c: char| c.is_ascii_uppercase()) {
        true => format!("appendix-{label}"),
        false => format!("section-{label}"),
    };
    let index = doc.headings.len();
    doc.headings.push(Heading {
        title,
        component: label,
        base: true,
        anchor,
        parent,
        line: section.line,
    });
    for child in section.elements() {
        match child.name.as_str() {
            "section" => read_section(doc, child, Some(index)),
            "name" => {}
            _ => block(doc, child, index, ""),
        }
    }
}

/// A block element: a paragraph is a unit; a list, a definition list, a table or an aside holds
/// blocks; code, artwork and index entries are not read. `label` leads the first unit (a `<dt>`).
fn block(doc: &mut Doc, element: &Element, heading: usize, label: &str) {
    match element.name.as_str() {
        "t" | "dt" | "td" | "th" => unit(doc, element, heading, label),
        "li" | "dd" | "blockquote" | "aside" => {
            let holds_blocks = element.elements().any(|e| is_block(&e.name));
            if holds_blocks {
                let mut lead = label.to_owned();
                for child in element.elements() {
                    block(doc, child, heading, &lead);
                    lead.clear();
                }
            } else {
                unit(doc, element, heading, label);
            }
        }
        "dl" => {
            let mut term = String::new();
            for child in element.elements() {
                match child.name.as_str() {
                    "dt" => term = collapse(&inline_text(child)).replace('\n', " "),
                    _ => block(doc, child, heading, &std::mem::take(&mut term)),
                }
            }
        }
        "ul" | "ol" | "table" | "thead" | "tbody" | "tfoot" | "tr" => {
            for child in element.elements() {
                block(doc, child, heading, "");
            }
        }
        _ => {}
    }
}

fn is_block(name: &str) -> bool {
    matches!(
        name,
        "t" | "ul"
            | "ol"
            | "dl"
            | "table"
            | "figure"
            | "sourcecode"
            | "artwork"
            | "aside"
            | "blockquote"
    )
}

fn unit(doc: &mut Doc, element: &Element, heading: usize, label: &str) {
    let mut text = String::new();
    if !label.is_empty() {
        text.push_str(label);
        text.push(' ');
    }
    text.push_str(&inline_text(element));
    let text = collapse(&text.replace('\n', " "));
    if !text.is_empty() {
        doc.units.push(Unit {
            heading: Some(heading),
            line: element.line,
            kind: UnitKind::Prose,
            text,
        });
    }
}

/// The text of inline content: references as the RFC renders them, index entries and examples
/// left out.
fn inline_text(element: &Element) -> String {
    let mut out = String::new();
    for node in &element.children {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Element(child) => match child.name.as_str() {
                "iref" | "sourcecode" | "artwork" | "figure" | "cref" => {}
                "br" => out.push(' '),
                "xref" => out.push_str(&reference(child)),
                "contact" => out.push_str(child.attr("fullname").unwrap_or_default()),
                _ => out.push_str(&inline_text(child)),
            },
        }
    }
    out
}

/// An `<xref>`: its own text, else what xml2rfc derived (`Section 6.2`, `[RFC1952]`, `Section
/// 3.5 of [URI]`).
fn reference(xref: &Element) -> String {
    let own = inline_text(xref);
    if !own.trim().is_empty() {
        return own;
    }
    let derived = xref.attr("derivedContent").unwrap_or_default();
    if let Some(section) = xref.attr("section") {
        return match xref.attr("sectionFormat") {
            Some("comma") => format!("[{derived}], Section {section}"),
            Some("bare") => section.to_owned(),
            Some("parens") => format!("[{derived}] (Section {section})"),
            _ => format!("Section {section} of [{derived}]"),
        };
    }
    let plain = xref.attr("format") == Some("counter")
        || derived.starts_with("Section")
        || derived.starts_with("Appendix");
    match (plain, derived.is_empty()) {
        (_, true) => String::new(),
        (true, false) => derived.to_owned(),
        (false, false) => format!("[{derived}]"),
    }
}
