//! `REPORT.md`: per text and per section, how many requirements there are, how many flowapi
//! answers for, how many of those a test covers, how many are lossy (the type is wider, with a
//! reason), and how many wait for either. A requirement with both markers counts as lossy.

use crate::markers::{Kind, Marker};
use crate::requirements::Requirement;
use crate::scope::{Class, Rule, Scope};
use crate::sources::{SOURCES, by_file};
use std::collections::{BTreeMap, BTreeSet};

/// What the report counts.
pub struct Ledger<'a> {
    /// Every requirement, sorted by id.
    pub requirements: &'a [Requirement],
    /// The rule that classifies each, in the same order.
    pub classes: &'a [Option<&'a Rule>],
    /// The scope's own status line.
    pub scope: &'a Scope,
    /// Every marker that names a known id.
    pub markers: &'a [Marker],
}

#[derive(Default, Clone, Copy)]
struct Counts {
    total: usize,
    in_scope: usize,
    out_of_scope: usize,
    covered: usize,
    lossy: usize,
}

/// What the markers say of one requirement.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Marked {
    None,
    Covered,
    Lossy,
}

impl Counts {
    fn add(&mut self, class: Option<Class>, marked: Marked) {
        self.total += 1;
        match class {
            Some(Class::InScope) => {
                self.in_scope += 1;
                self.covered += usize::from(marked == Marked::Covered);
                self.lossy += usize::from(marked == Marked::Lossy);
            }
            Some(Class::OutOfScope) => self.out_of_scope += 1,
            None => {}
        }
    }

    fn missing(&self) -> usize {
        self.in_scope - self.covered - self.lossy
    }

    fn row(&self, label: &str) -> String {
        format!(
            "| {label} | {} | {} | {} | {} | {} | {} | {} |",
            self.total,
            self.in_scope,
            self.out_of_scope,
            self.covered,
            self.lossy,
            self.missing(),
            share(self.covered + self.lossy, self.in_scope)
        )
    }
}

fn share(part: usize, whole: usize) -> String {
    if whole == 0 {
        return "—".to_owned();
    }
    let tenths = (part * 1000 + whole / 2) / whole;
    format!("{}.{}%", tenths / 10, tenths % 10)
}

const HEADER: [&str; 2] = [
    "| {} | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
];

fn header(label: &str) -> Vec<String> {
    vec![HEADER[0].replace("{}", label), HEADER[1].to_owned()]
}

/// The first id segment after the text's prefix: the Object or the top section.
fn group(requirement: &Requirement) -> String {
    let prefix = by_file(&requirement.source)
        .map(|s| s.prefix)
        .unwrap_or_default();
    let rest = requirement
        .id
        .strip_prefix(prefix)
        .unwrap_or(&requirement.id)
        .trim_start_matches('.');
    rest.split('.').next().unwrap_or_default().to_owned()
}

/// Renders the report.
pub fn render(ledger: &Ledger<'_>) -> String {
    let of_kind = |kind: Kind| -> BTreeSet<&str> {
        ledger
            .markers
            .iter()
            .filter(|m| m.kind == kind)
            .map(|m| m.id.as_str())
            .collect()
    };
    let (covered, lossy) = (of_kind(Kind::Covered), of_kind(Kind::Lossy));
    let mut overall = Counts::default();
    let mut by_source: BTreeMap<&str, Counts> = BTreeMap::new();
    let mut by_level: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut by_group: BTreeMap<(&str, String), Counts> = BTreeMap::new();
    let mut outside: Vec<&Marker> = Vec::new();
    for (requirement, rule) in ledger.requirements.iter().zip(ledger.classes) {
        let class = rule.map(|r| r.class);
        let id = requirement.id.as_str();
        let marked = match (lossy.contains(id), covered.contains(id)) {
            (true, _) => Marked::Lossy,
            (false, true) => Marked::Covered,
            (false, false) => Marked::None,
        };
        overall.add(class, marked);
        by_source
            .entry(&requirement.source)
            .or_default()
            .add(class, marked);
        *by_level
            .entry((&requirement.source, &requirement.level))
            .or_default() += 1;
        by_group
            .entry((&requirement.source, group(requirement)))
            .or_default()
            .add(class, marked);
        if class == Some(Class::OutOfScope) {
            outside.extend(ledger.markers.iter().filter(|m| m.id == requirement.id));
        }
    }
    let mut out = vec![
        "# Requirements report".to_owned(),
        String::new(),
        "Written by `cargo run --release --bin report` in `conformance/ledger/` from `requirements.json`,".to_owned(),
        "`scope.json` and the `req:` and `lossy:` markers under `crates/` and `conformance/`. Do not edit by hand.".to_owned(),
        String::new(),
        format!(
            "**Answered: {} of {} requirements in scope ({}): {} covered by a test, {} lossy \
             with a reason; {} missing.** Out of scope: {}. Total: {}.",
            overall.covered + overall.lossy,
            overall.in_scope,
            share(overall.covered + overall.lossy, overall.in_scope),
            overall.covered,
            overall.lossy,
            overall.missing(),
            overall.out_of_scope,
            overall.total
        ),
        format!("Scope: {}.", if ledger.scope.status.is_empty() { "confirmed" } else { &ledger.scope.status }),
        String::new(),
    ];
    out.extend(header("Text"));
    for source in &SOURCES {
        let counts = by_source.get(source.file).copied().unwrap_or_default();
        out.push(counts.row(&format!(
            "{} {} (`{}`)",
            source.spec, source.version, source.file
        )));
    }
    out.push(overall.row("**All**"));
    out.extend(level_table(&by_level));
    for source in &SOURCES {
        out.push(String::new());
        out.push(format!(
            "## {} {} — `{}`",
            source.spec, source.version, source.file
        ));
        out.push(String::new());
        out.extend(header(&format!("Section (id after `{}.`)", source.prefix)));
        for ((file, name), counts) in &by_group {
            if *file == source.file {
                out.push(counts.row(&format!(
                    "`{}`",
                    if name.is_empty() { "(top)" } else { name }
                )));
            }
        }
    }
    out.extend(marker_tables(ledger.markers, &outside));
    out.push(String::new());
    out.join("\n")
}

fn level_table(by_level: &BTreeMap<(&str, &str), usize>) -> Vec<String> {
    let levels = ["MUST", "SHOULD", "MAY", "field", "keyword"];
    let mut out = vec![
        String::new(),
        "## By level".to_owned(),
        String::new(),
        format!("| Text | {} |", levels.join(" | ")),
        format!("| --- |{}", " ---: |".repeat(levels.len())),
    ];
    for source in &SOURCES {
        let cells: Vec<String> = levels
            .iter()
            .map(|l| {
                by_level
                    .get(&(source.file, *l))
                    .copied()
                    .unwrap_or(0)
                    .to_string()
            })
            .collect();
        out.push(format!(
            "| {} {} (`{}`) | {} |",
            source.spec,
            source.version,
            source.file,
            cells.join(" | ")
        ));
    }
    out
}

fn marker_tables(markers: &[Marker], outside: &[&Marker]) -> Vec<String> {
    let mut out = vec![
        String::new(),
        "## Markers per file".to_owned(),
        String::new(),
    ];
    let mut per_file: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for marker in markers {
        per_file.entry(&marker.file).or_default().insert(&marker.id);
    }
    if per_file.is_empty() {
        out.push("None yet.".to_owned());
    } else {
        out.push("| File | Requirements |".to_owned());
        out.push("| --- | ---: |".to_owned());
        out.extend(
            per_file
                .iter()
                .map(|(file, ids)| format!("| `{file}` | {} |", ids.len())),
        );
    }
    out.extend([
        String::new(),
        "## Markers on out-of-scope requirements".to_owned(),
        String::new(),
    ]);
    if outside.is_empty() {
        out.push("None.".to_owned());
    } else {
        out.extend(
            outside
                .iter()
                .map(|m| format!("- `{}` in `{}:{}`", m.id, m.file, m.line)),
        );
    }
    out
}
