//! Writes `conformance/REPORT.md` and `conformance/MISSING.md` (the requirements in scope no
//! marker names yet) from `requirements.json`, `scope.json` and the `req:` and `lossy:` markers. Fails on a marker that names no requirement, a lossy marker without a reason,
//! a requirement no scope rule classifies, and a scope rule that fits no requirement.
//!
//!   cargo run --release --bin report             write REPORT.md and MISSING.md
//!   cargo run --release --bin report -- --check  also fail when either is stale
//!
//! A missing requirement fails nothing: it is counted and listed. What fails is an unknown id,
//! an unclassified one, a stale rule, a reason left out, or a file that does not match.

use flowapi_ledger::markers::{Kind, Marker, scan};
use flowapi_ledger::report::{Ledger, render};
use flowapi_ledger::{missing, paths, requirements, scope};
use std::collections::BTreeSet;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("report: {message}");
            ExitCode::FAILURE
        }
    }
}

fn read(path: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn run() -> Result<ExitCode, String> {
    let all = requirements::parse(&read(&paths::requirements())?)?;
    let scope = scope::parse(&read(&paths::scope())?)?;
    let markers = scan(&paths::root())?;
    let known: BTreeSet<&str> = all.iter().map(|r| r.id.as_str()).collect();
    let (unknown, markers): (Vec<Marker>, Vec<Marker>) = markers
        .into_iter()
        .partition(|m| !known.contains(m.id.as_str()));
    let (classes, stale) = scope::classify(&scope, &all);
    let mut failed = false;
    if !unknown.is_empty() {
        eprintln!("report: markers name ids that are not requirements (G13):");
        unknown
            .iter()
            .for_each(|m| eprintln!("  {}:{}: {}", m.file, m.line, m.id));
        failed = true;
    }
    let unexplained: Vec<&Marker> = markers
        .iter()
        .filter(|m| m.kind == Kind::Lossy && m.reason.is_empty())
        .collect();
    if !unexplained.is_empty() {
        eprintln!("report: lossy markers without a reason:");
        unexplained
            .iter()
            .for_each(|m| eprintln!("  {}:{}: {}", m.file, m.line, m.id));
        failed = true;
    }
    let unclassified: Vec<&str> = all
        .iter()
        .zip(&classes)
        .filter(|(_, c)| c.is_none())
        .map(|(r, _)| r.id.as_str())
        .collect();
    if !unclassified.is_empty() {
        eprintln!(
            "report: {} requirements no rule of scope.json classifies, the first:",
            unclassified.len()
        );
        unclassified
            .iter()
            .take(20)
            .for_each(|id| eprintln!("  {id}"));
        failed = true;
    }
    if !stale.is_empty() {
        eprintln!("report: rules of scope.json that fit no requirement:");
        stale
            .iter()
            .for_each(|rule| eprintln!("  {}", rule.pattern));
        failed = true;
    }
    if failed {
        return Ok(ExitCode::FAILURE);
    }
    let ledger = Ledger {
        requirements: &all,
        classes: &classes,
        scope: &scope,
        markers: &markers,
    };
    let outputs = [
        (paths::report(), "REPORT.md", render(&ledger)),
        (paths::missing(), "MISSING.md", missing::render(&ledger)),
    ];
    let check = std::env::args().any(|arg| arg == "--check");
    for (target, name, written) in &outputs {
        if !check {
            std::fs::write(target, written).map_err(|e| format!("{}: {e}", target.display()))?;
        } else if std::fs::read_to_string(target).unwrap_or_default() != *written {
            eprintln!(
                "report: conformance/{name} is stale; run `cargo run --release --bin report`."
            );
            failed = true;
        }
    }
    if failed {
        return Ok(ExitCode::FAILURE);
    }
    let named = |kind: Kind| {
        markers
            .iter()
            .filter(|m| m.kind == kind)
            .map(|m| m.id.as_str())
            .collect::<BTreeSet<&str>>()
            .len()
    };
    println!(
        "{} requirements, {} markers: {} ids covered, {} lossy",
        all.len(),
        markers.len(),
        named(Kind::Covered),
        named(Kind::Lossy)
    );
    Ok(ExitCode::SUCCESS)
}
