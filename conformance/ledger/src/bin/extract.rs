//! Reads the pinned texts in `conformance/specs/` and writes `conformance/requirements.json`.
//! Prints what was added and removed since the file it replaces. Fails when two requirements
//! share an id or a 2020-12 keyword is found in no section.
//!
//!   cargo run --release --bin extract            write requirements.json
//!   cargo run --release --bin extract -- --check fail when requirements.json is stale

use flowapi_ledger::requirements::{Requirement, parse, render};
use flowapi_ledger::rfc::KEYWORDS;
use flowapi_ledger::sources::{Format, SOURCES};
use flowapi_ledger::{extract, markdown, paths, rfc, xml};
use std::collections::BTreeSet;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("extract: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut all: Vec<Requirement> = Vec::new();
    for source in &SOURCES {
        let path = paths::specs().join(source.file);
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let doc = match source.format {
            Format::Markdown => markdown::read(&text),
            Format::Rfc => {
                rfc::read(&xml::parse(&text).map_err(|e| format!("{}: {e}", source.file))?)
            }
        };
        all.extend(extract::requirements(&doc, source));
    }
    check_unique(&all)?;
    check_keywords(&all)?;
    all.sort_by(|a, b| a.id.cmp(&b.id));
    let listed: Vec<(String, String, String)> = SOURCES
        .iter()
        .map(|s| (s.spec.to_owned(), s.version.to_owned(), s.file.to_owned()))
        .collect();
    let written = render(&all, &listed);
    let target = paths::requirements();
    let before = std::fs::read_to_string(&target).unwrap_or_default();
    if std::env::args().any(|arg| arg == "--check") {
        if before != written {
            eprintln!(
                "extract: conformance/requirements.json is stale; run `cargo run --release --bin extract`."
            );
            return Ok(ExitCode::FAILURE);
        }
    } else {
        print_changes(&before, &all);
        std::fs::write(&target, written).map_err(|e| format!("{}: {e}", target.display()))?;
    }
    println!("{} requirements from {} texts", all.len(), SOURCES.len());
    Ok(ExitCode::SUCCESS)
}

fn check_unique(all: &[Requirement]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let doubled: Vec<&str> = all
        .iter()
        .filter(|r| !seen.insert(r.id.as_str()))
        .map(|r| r.id.as_str())
        .collect();
    match doubled.first() {
        None => Ok(()),
        Some(first) => Err(format!(
            "{} ids are taken twice, the first {first}",
            doubled.len()
        )),
    }
}

/// Every keyword of the 2020-12 vocabularies has an entry: a section that moved fails here.
fn check_keywords(all: &[Requirement]) -> Result<(), String> {
    let placed: BTreeSet<&str> = all
        .iter()
        .filter(|r| r.level == "keyword" && r.spec == "json-schema")
        .filter_map(|r| r.id.rsplit('.').next())
        .collect();
    let named: BTreeSet<String> = all
        .iter()
        .filter(|r| r.level == "keyword" && r.spec == "json-schema")
        .flat_map(|r| rfc::title_keywords(&r.text))
        .collect();
    let missing: Vec<&str> = KEYWORDS
        .iter()
        .copied()
        .filter(|k| !placed.contains(k) && !named.contains(*k))
        .collect();
    match missing.is_empty() {
        true => Ok(()),
        false => Err(format!(
            "these 2020-12 keywords are in no section: {}",
            missing.join(", ")
        )),
    }
}

fn print_changes(before: &str, now: &[Requirement]) {
    let Ok(old) = parse(before) else { return };
    let old: BTreeSet<&str> = old.iter().map(|r| r.id.as_str()).collect::<BTreeSet<_>>();
    let old: BTreeSet<String> = old.into_iter().map(str::to_owned).collect();
    let new: BTreeSet<String> = now.iter().map(|r| r.id.clone()).collect();
    let added = new.difference(&old).count();
    let removed: Vec<&String> = old.difference(&new).collect();
    println!(
        "{added} added, {} removed since the last extraction",
        removed.len()
    );
    for id in removed.iter().take(40) {
        println!("  removed {id}");
    }
}
