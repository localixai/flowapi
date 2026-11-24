//! Lists requirements with the class `scope.json` gives them: the way to read the scope while
//! editing it, and to find what waits for a test.
//!
//!   cargo run --release --bin classify                      counts per text and class
//!   cargo run --release --bin classify -- PREFIX            every id under PREFIX, its class, why
//!   cargo run --release --bin classify -- --missing PREFIX  only the in-scope ids no marker names
//!   cargo run --release --bin classify -- --unclassified    only the ids no rule fits

use flowapi_ledger::markers::scan;
use flowapi_ledger::scope::{Class, classify, fits};
use flowapi_ledger::{paths, requirements, scope};
use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("classify: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let read = |p: std::path::PathBuf| {
        std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))
    };
    let all = requirements::parse(&read(paths::requirements())?)?;
    let scope = scope::parse(&read(paths::scope())?)?;
    let (classes, stale) = classify(&scope, &all);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let missing = args.iter().any(|a| a == "--missing");
    let unclassified = args.iter().any(|a| a == "--unclassified");
    let prefix = args.iter().find(|a| !a.starts_with("--"));
    let marked: BTreeSet<String> = match missing {
        true => scan(&paths::root())?.into_iter().map(|m| m.id).collect(),
        false => BTreeSet::new(),
    };
    if prefix.is_none() && !missing && !unclassified {
        let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for (requirement, rule) in all.iter().zip(&classes) {
            let class = match rule.map(|r| r.class) {
                Some(Class::InScope) => "in-scope",
                Some(Class::OutOfScope) => "out-of-scope",
                None => "unclassified",
            };
            *counts
                .entry((requirement.source.as_str(), class))
                .or_default() += 1;
        }
        counts
            .iter()
            .for_each(|((source, class), n)| println!("{source}\t{class}\t{n}"));
        stale
            .iter()
            .for_each(|rule| println!("stale rule\t{}", rule.pattern));
        return Ok(());
    }
    for (requirement, rule) in all.iter().zip(&classes) {
        if prefix.is_some_and(|p| !fits(p, &requirement.id)) {
            continue;
        }
        let shown = match (rule, missing, unclassified) {
            (None, _, _) => Some(("unclassified", "")),
            (Some(_), _, true) => None,
            (Some(r), true, _)
                if r.class == Class::InScope && !marked.contains(&requirement.id) =>
            {
                Some(("missing", ""))
            }
            (Some(_), true, _) => None,
            (Some(r), false, false) if r.class == Class::InScope => {
                Some(("in-scope", r.pattern.as_str()))
            }
            (Some(r), false, false) => Some(("out-of-scope", r.reason.as_str())),
        };
        if let Some((class, why)) = shown {
            println!("{}\t{class}\t{why}", requirement.id);
        }
    }
    Ok(())
}
