//! `MISSING.md`: every requirement in scope that no marker names yet, per text and per section of
//! that text, in the text's own order, each with its sentence and the scope rule that put it in
//! scope (its `note` in `scope.json` names the layer that answers for it). The work list of what
//! flowapi answers for and no test shows yet; `REPORT.md` has the counts.

use crate::report::Ledger;
use crate::requirements::Requirement;
use crate::scope::Class;
use crate::sources::SOURCES;
use std::collections::{BTreeMap, BTreeSet};

/// One section's missing requirements: where the section starts, its title, its anchor and the
/// requirements with the pattern of the rule that classifies each.
struct Section<'a> {
    line: usize,
    title: &'a str,
    anchor: &'a str,
    rows: Vec<(&'a Requirement, &'a str)>,
}

/// Renders the list.
pub fn render(ledger: &Ledger<'_>) -> String {
    let marked: BTreeSet<&str> = ledger.markers.iter().map(|m| m.id.as_str()).collect();
    let mut by_source: BTreeMap<&str, BTreeMap<&str, Section<'_>>> = BTreeMap::new();
    let mut total = 0;
    for (requirement, rule) in ledger.requirements.iter().zip(ledger.classes) {
        let Some(rule) = rule.filter(|r| r.class == Class::InScope) else {
            continue;
        };
        if marked.contains(requirement.id.as_str()) {
            continue;
        }
        total += 1;
        let section = by_source
            .entry(&requirement.source)
            .or_default()
            .entry(&requirement.section)
            .or_insert(Section {
                line: requirement.line,
                title: &requirement.section,
                anchor: &requirement.anchor,
                rows: Vec::new(),
            });
        if requirement.line < section.line {
            section.line = requirement.line;
            section.anchor = &requirement.anchor;
        }
        section.rows.push((requirement, &rule.pattern));
    }
    let mut out = vec![
        "# Requirements missing a test".to_owned(),
        String::new(),
        "Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.".to_owned(),
        "Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and".to_owned(),
        "section in the text's order. After each id, the scope rule that classifies it: its `note` names the".to_owned(),
        "layer that answers for it.".to_owned(),
        String::new(),
        format!("**{total} missing.**"),
    ];
    for source in &SOURCES {
        let Some(sections) = by_source.get(source.file) else {
            continue;
        };
        let count: usize = sections.values().map(|s| s.rows.len()).sum();
        out.push(String::new());
        out.push(format!(
            "## {} {} — `{}`: {count} missing",
            source.spec, source.version, source.file
        ));
        let mut ordered: Vec<&Section<'_>> = sections.values().collect();
        ordered.sort_by_key(|s| s.line);
        for section in ordered {
            out.push(String::new());
            out.push(format!(
                "### {} (`#{}`, line {})",
                if section.title.is_empty() {
                    "(top)"
                } else {
                    section.title
                },
                section.anchor,
                section.line
            ));
            out.push(String::new());
            let mut rows = section.rows.clone();
            rows.sort_by_key(|(r, _)| r.line);
            out.extend(rows.iter().map(|(r, pattern)| {
                format!(
                    "- `{}` ({}, by `{pattern}`): {}",
                    r.id,
                    r.level,
                    r.text.replace('\n', " ")
                )
            }));
        }
    }
    out.push(String::new());
    out.join("\n")
}
