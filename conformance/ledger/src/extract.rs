//! A [`Doc`] into requirements.
//!
//! - A sentence with a BCP 14 term: `<prefix>.<section>.<term>-<slug>`, the slug its first six
//!   words that are neither terms nor stop words (`oas-3.1.parameter-object.fixed-fields.must-parameter-objects-include-either-content-field`).
//! - A field row: `<prefix>.<object>.<name>` (`oas-3.1.schema-object.discriminator`); a patterned
//!   one `pattern-<name>`; its sentences under it (`oas-3.2.parameter-object.style.must-…`).
//! - A keyword: its section's id when the section is its own (`jsonschema-2020-12.core.$ref`),
//!   else `<object or section>.<keyword>`; a listed keyword's sentences under it.
//!
//! An id already taken gets `-2`, `-3`, ... in document order.

use crate::doc::{Doc, Unit, UnitKind};
use crate::ids::{join, section_ids};
use crate::requirements::Requirement;
use crate::sources::Source;
use crate::terms::{lead_term, level, sentence_slug, terms};
use crate::text::{heading_slug, name_slug, sentences};
use std::collections::BTreeMap;

/// How many words of a sentence its slug keeps.
const SLUG_WORDS: usize = 6;

/// Every requirement of one text, in document order.
pub fn requirements(doc: &Doc, source: &Source) -> Vec<Requirement> {
    let sections = section_ids(&doc.headings);
    let mut out = Extractor {
        doc,
        source,
        sections: &sections,
        taken: BTreeMap::new(),
        found: Vec::new(),
    };
    for unit in &doc.units {
        out.unit(unit);
    }
    out.found
}

struct Extractor<'a> {
    doc: &'a Doc,
    source: &'a Source,
    sections: &'a [String],
    taken: BTreeMap<String, usize>,
    found: Vec<Requirement>,
}

impl Extractor<'_> {
    fn section(&self, heading: Option<usize>) -> &str {
        heading
            .and_then(|h| self.sections.get(h))
            .map(String::as_str)
            .unwrap_or_default()
    }

    /// The section id of the nearest base at or above the heading: the Object a field belongs to.
    fn owner(&self, heading: Option<usize>) -> &str {
        let mut at = heading;
        while let Some(index) = at {
            let Some(found) = self.doc.headings.get(index) else {
                break;
            };
            if found.base {
                return self.section(Some(index));
            }
            at = found.parent;
        }
        self.section(heading)
    }

    fn anchor(&self, heading: Option<usize>) -> String {
        heading
            .and_then(|h| self.doc.headings.get(h))
            .map(|h| h.anchor.clone())
            .unwrap_or_default()
    }

    fn unit(&mut self, unit: &Unit) {
        match &unit.kind {
            UnitKind::Prose => {
                let base = join(&[self.source.prefix, self.section(unit.heading)]);
                self.sentences(unit, &base, None);
            }
            UnitKind::Field {
                name,
                patterned,
                anchor,
            } => {
                let slug = if *patterned {
                    format!("pattern-{}", heading_slug(name))
                } else {
                    name_slug(name)
                };
                let id = join(&[self.source.prefix, self.owner(unit.heading), &slug]);
                let anchor = anchor.clone().unwrap_or_else(|| self.anchor(unit.heading));
                let id = self.push(
                    unit,
                    &id,
                    "field",
                    Vec::new(),
                    unit.text.replace('\n', " "),
                    anchor.clone(),
                );
                self.sentences(unit, &id, Some(anchor));
            }
            UnitKind::Keyword { name, own_section } => {
                let id = if *own_section {
                    join(&[self.source.prefix, self.section(unit.heading)])
                } else {
                    join(&[
                        self.source.prefix,
                        self.owner(unit.heading),
                        &name_slug(name),
                    ])
                };
                let text = match unit.text.is_empty() {
                    true => self
                        .doc
                        .headings
                        .get(unit.heading.unwrap_or(usize::MAX))
                        .map(|h| h.title.clone())
                        .unwrap_or_default(),
                    false => unit.text.replace('\n', " "),
                };
                let id = self.push(
                    unit,
                    &id,
                    "keyword",
                    Vec::new(),
                    text,
                    self.anchor(unit.heading),
                );
                if !unit.text.is_empty() {
                    self.sentences(unit, &id, None);
                }
            }
        }
    }

    fn sentences(&mut self, unit: &Unit, base: &str, anchor: Option<String>) {
        for sentence in sentences(&unit.text) {
            let found = terms(&sentence);
            if found.is_empty() {
                continue;
            }
            let slug = sentence_slug(&sentence, SLUG_WORDS);
            let lead = lead_term(&found);
            let last = if slug.is_empty() {
                lead
            } else {
                format!("{lead}-{slug}")
            };
            let id = format!("{base}.{last}");
            let anchor = anchor.clone().unwrap_or_else(|| self.anchor(unit.heading));
            let owned = found.iter().map(|t| (*t).to_owned()).collect();
            self.push(unit, &id, level(&found), owned, sentence, anchor);
        }
    }

    fn push(
        &mut self,
        unit: &Unit,
        id: &str,
        level: &str,
        terms: Vec<String>,
        text: String,
        anchor: String,
    ) -> String {
        let count = self.taken.entry(id.to_owned()).or_default();
        *count += 1;
        let id = if *count == 1 {
            id.to_owned()
        } else {
            format!("{id}-{count}")
        };
        self.found.push(Requirement {
            id: id.clone(),
            spec: self.source.spec.to_owned(),
            version: self.source.version.to_owned(),
            source: self.source.file.to_owned(),
            line: unit.line,
            section: self.doc.section_path(unit.heading),
            anchor,
            level: level.to_owned(),
            terms,
            text,
        });
        id
    }
}
