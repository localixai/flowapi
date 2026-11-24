//! Reads a JSON Schema specification (an xml2rfc v2 source) into a [`Doc`].
//!
//! Sections of `<middle>` are numbered 1, 1.1, ...; those of `<back>` are appendices A, A.1, ...
//! A section without an `anchor` gets the one xml2rfc renders (`section-4.3.1`, `appendix-A`).
//! Each `<t>`, each list item and each figure's preamble and postamble is a prose unit; artwork
//! (examples) and editor's comments (`<cref>`) are skipped. A section whose title names a
//! keyword of the 2020-12 vocabularies is a keyword section, and gives a keyword unit.

use crate::doc::{Doc, Heading, Unit, UnitKind};
use crate::text::{collapse, heading_slug};
use crate::xml::{Element, Node};

/// Every keyword of the 2020-12 vocabularies (the meta-schemas of the release, `meta/*.json`).
pub const KEYWORDS: [&str; 57] = [
    // core
    "$id",
    "$schema",
    "$ref",
    "$anchor",
    "$dynamicRef",
    "$dynamicAnchor",
    "$vocabulary",
    "$comment",
    "$defs",
    // applicator
    "prefixItems",
    "items",
    "contains",
    "additionalProperties",
    "properties",
    "patternProperties",
    "dependentSchemas",
    "propertyNames",
    "if",
    "then",
    "else",
    "allOf",
    "anyOf",
    "oneOf",
    "not",
    // unevaluated
    "unevaluatedItems",
    "unevaluatedProperties",
    // validation
    "type",
    "const",
    "enum",
    "multipleOf",
    "maximum",
    "exclusiveMaximum",
    "minimum",
    "exclusiveMinimum",
    "maxLength",
    "minLength",
    "pattern",
    "maxItems",
    "minItems",
    "uniqueItems",
    "maxContains",
    "minContains",
    "maxProperties",
    "minProperties",
    "required",
    "dependentRequired",
    // format-annotation, format-assertion
    "format",
    // content
    "contentEncoding",
    "contentMediaType",
    "contentSchema",
    // meta-data
    "title",
    "description",
    "default",
    "deprecated",
    "readOnly",
    "writeOnly",
    "examples",
];

/// Keywords that no section title names: (section anchor, keywords).
pub const PLACED_BY_ANCHOR: [(&str, &[&str]); 1] = [("anchor", &["$anchor", "$dynamicAnchor"])];

/// The keywords named by a section title: the whole title, or each quoted name in it.
pub fn title_keywords(title: &str) -> Vec<String> {
    if KEYWORDS.contains(&title.trim()) {
        return vec![title.trim().to_owned()];
    }
    title
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|quoted| KEYWORDS.contains(quoted))
        .map(str::to_owned)
        .collect()
}

/// Reads an xml2rfc document.
pub fn read(root: &Element) -> Doc {
    let mut doc = Doc::default();
    for part in root.elements() {
        let appendix = match part.name.as_str() {
            "middle" => false,
            "back" => true,
            _ => continue,
        };
        let sections = part.elements().filter(|e| e.name == "section");
        for (index, section) in sections.enumerate() {
            let number = if appendix {
                letter(index)
            } else {
                (index + 1).to_string()
            };
            read_section(&mut doc, section, None, &number, appendix);
        }
    }
    doc
}

fn letter(index: usize) -> String {
    let code = u32::try_from(index)
        .ok()
        .and_then(|i| char::from_u32(u32::from('A') + i));
    code.map(String::from).unwrap_or_else(|| index.to_string())
}

fn read_section(
    doc: &mut Doc,
    section: &Element,
    parent: Option<usize>,
    number: &str,
    appendix: bool,
) {
    let title = collapse(section.attr("title").unwrap_or_default()).replace('\n', " ");
    let explicit = section.attr("anchor").map(str::to_owned);
    let rendered = if appendix {
        format!("appendix-{number}")
    } else {
        format!("section-{number}")
    };
    let keywords = section_keywords(&title, explicit.as_deref());
    let titled: Vec<String> = title_keywords(&title);
    let (component, base) = match (titled.as_slice(), &explicit) {
        ([single], _) => (single.clone(), true),
        ([_, _, ..], _) => (heading_slug(&title), true),
        ([], Some(anchor)) => (anchor.clone(), true),
        ([], None) => (heading_slug(&title), false),
    };
    let index = doc.headings.len();
    let anchor = explicit.unwrap_or(rendered);
    doc.headings.push(Heading {
        title,
        component,
        base,
        anchor,
        parent,
        line: section.line,
    });
    let own_section = titled.len() == 1;
    for name in keywords {
        let kind = UnitKind::Keyword { name, own_section };
        doc.units.push(Unit {
            heading: Some(index),
            line: section.line,
            kind,
            text: String::new(),
        });
    }
    let mut child = 0;
    for element in section.elements() {
        match element.name.as_str() {
            "t" => paragraph(doc, element, index, ""),
            "figure" => {
                for part in element
                    .elements()
                    .filter(|e| matches!(e.name.as_str(), "preamble" | "postamble"))
                {
                    paragraph(doc, part, index, "");
                }
            }
            "section" => {
                child += 1;
                read_section(
                    doc,
                    element,
                    Some(index),
                    &format!("{number}.{child}"),
                    appendix,
                );
            }
            _ => {}
        }
    }
}

fn section_keywords(title: &str, anchor: Option<&str>) -> Vec<String> {
    let mut found = title_keywords(title);
    for (placed, names) in PLACED_BY_ANCHOR {
        if anchor == Some(placed) {
            found.extend(names.iter().map(|name| (*name).to_owned()));
        }
    }
    found
}

/// A `<t>` (or a preamble): its own text is one unit; each item of a list in it is another.
fn paragraph(doc: &mut Doc, element: &Element, heading: usize, prefix: &str) {
    let mut text = String::from(prefix);
    let mut lists = Vec::new();
    inline(element, &mut text, &mut lists);
    let text = collapse(&text.replace('\n', " "));
    if !text.is_empty() {
        doc.units.push(Unit {
            heading: Some(heading),
            line: element.line,
            kind: UnitKind::Prose,
            text,
        });
    }
    for list in lists {
        for item in list.elements().filter(|e| e.name == "t") {
            let label = item
                .attr("hangText")
                .map(|h| format!("{h} "))
                .unwrap_or_default();
            paragraph(doc, item, heading, &label);
        }
    }
}

fn inline<'a>(element: &'a Element, out: &mut String, lists: &mut Vec<&'a Element>) {
    for node in &element.children {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Element(child) => match child.name.as_str() {
                "list" => lists.push(child),
                "cref" | "artwork" | "figure" => {}
                "vspace" => out.push(' '),
                "xref" | "eref" => {
                    let shown = child.text();
                    if shown.trim().is_empty() {
                        out.push_str(&format!("[{}]", child.attr("target").unwrap_or_default()));
                    } else {
                        out.push_str(&shown);
                    }
                }
                _ => inline(child, out, lists),
            },
        }
    }
}
