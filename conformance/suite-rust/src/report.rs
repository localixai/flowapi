//! The results, written so that the same suite and the same flowapi give the same bytes: files
//! by name, groups and cases by index, no time, no path of this machine.

use crate::evaluate::{Failure, FileReport, percent};
use serde_json::{Value, json};
use std::fmt::Write as _;

#[derive(Default)]
pub(crate) struct Totals {
    pub(crate) groups: usize,
    pub(crate) run: usize,
    pub(crate) generated: usize,
    pub(crate) generation_errors: usize,
    pub(crate) invalid_rust: usize,
    pub(crate) valid: usize,
    pub(crate) valid_accepted: usize,
    pub(crate) invalid: usize,
    pub(crate) invalid_rejected: usize,
}

pub(crate) fn totals(files: &[FileReport]) -> Totals {
    let mut t = Totals::default();
    for f in files {
        t.groups += f.groups;
        t.run += f.groups - f.skipped;
        t.generated += f.generated;
        t.generation_errors += f.generation_errors.len();
        t.invalid_rust += f.invalid_rust.len();
        t.valid += f.valid;
        t.valid_accepted += f.valid_accepted;
        t.invalid += f.invalid;
        t.invalid_rejected += f.invalid_rejected;
    }
    t
}

fn failures(list: &[Failure]) -> Value {
    list.iter()
        .map(|f| json!({ "group": f.group, "description": f.description, "message": f.message }))
        .collect()
}

pub(crate) fn json(files: &[FileReport]) -> String {
    let t = totals(files);
    let per_file: serde_json::Map<String, Value> = files
        .iter()
        .map(|f| {
            let unsound: Value = f
                .unsound
                .iter()
                .map(|u| {
                    json!({ "group": u.group, "group_description": u.group_description,
                        "description": u.description, "data": u.data, "error": u.error })
                })
                .collect();
            let v = json!({
                "groups": f.groups,
                "skipped": f.skipped,
                "generated": f.generated,
                "generation_errors": failures(&f.generation_errors),
                "invalid_rust": failures(&f.invalid_rust),
                "valid_cases": f.valid,
                "valid_accepted": f.valid_accepted,
                "invalid_cases": f.invalid,
                "invalid_rejected": f.invalid_rejected,
                "unsound": unsound,
            });
            (f.name.clone(), v)
        })
        .collect();
    let all = json!({
        "suite": "json-schema-test-suite",
        "dialect": "2020-12",
        "target": "rust (serde)",
        "totals": {
            "groups": t.groups, "groups_run": t.run, "generated": t.generated,
            "generation_errors": t.generation_errors, "invalid_rust": t.invalid_rust,
            "valid_cases": t.valid, "valid_accepted": t.valid_accepted,
            "invalid_cases": t.invalid, "invalid_rejected": t.invalid_rejected,
            "soundness_percent": percent(t.valid_accepted, t.valid),
            "precision_percent": percent(t.invalid_rejected, t.invalid),
        },
        "files": per_file,
    });
    let mut s = serde_json::to_string_pretty(&all).expect("results print as JSON");
    s.push('\n');
    s
}

pub(crate) fn markdown(files: &[FileReport]) -> String {
    let t = totals(files);
    let mut s = String::from("# O3 for the Rust target: JSON-Schema-Test-Suite through serde\n\n");
    s.push_str("Written by `cd conformance/suite-rust && cargo run --release`; do not edit.\n\n");
    s.push_str(
        "A valid case is accepted when `serde_json::from_str::<Root>` reads it and the value \
         written back equals it; an invalid case is rejected when it is not read. Rows named \
         `flowapi-*` are our own cases in the suite's format (`own/`), counted in the totals.\n\n",
    );
    let _ = writeln!(
        s,
        "- Soundness: {}% ({} of {} valid cases accepted)",
        percent(t.valid_accepted, t.valid),
        t.valid_accepted,
        t.valid
    );
    let _ = writeln!(
        s,
        "- Precision: {}% ({} of {} invalid cases rejected)",
        percent(t.invalid_rejected, t.invalid),
        t.invalid_rejected,
        t.invalid
    );
    let _ = writeln!(
        s,
        "- Groups: {} in the suite, {} run, {} generated",
        t.groups, t.run, t.generated
    );
    let _ = writeln!(
        s,
        "- Generation errors: {}; invalid Rust: {}\n",
        t.generation_errors, t.invalid_rust
    );
    s.push_str("| File | Groups | Run | Gen. errors | Invalid Rust | Valid | Accepted | Sound % | Invalid | Rejected | Precision % |\n");
    s.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for f in files {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            f.name,
            f.groups,
            f.groups - f.skipped,
            f.generation_errors.len(),
            f.invalid_rust.len(),
            f.valid,
            f.valid_accepted,
            percent(f.valid_accepted, f.valid),
            f.invalid,
            f.invalid_rejected,
            percent(f.invalid_rejected, f.invalid),
        );
    }
    let _ = writeln!(
        s,
        "| **total** | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        t.groups,
        t.run,
        t.generation_errors,
        t.invalid_rust,
        t.valid,
        t.valid_accepted,
        percent(t.valid_accepted, t.valid),
        t.invalid,
        t.invalid_rejected,
        percent(t.invalid_rejected, t.invalid),
    );
    lists(&mut s, files);
    s
}

fn lists(s: &mut String, files: &[FileReport]) {
    s.push_str("\n## Unsound cases\n\nA valid case the type refuses or changes.\n\n");
    for f in files {
        for u in &f.unsound {
            let error = u.error.lines().next().unwrap_or("");
            let _ = writeln!(
                s,
                "- `{}` #{} {} / {}: `{}`  \n  {}",
                f.name, u.group, u.group_description, u.description, u.data, error
            );
        }
    }
    for (title, pick) in [
        (
            "Invalid Rust",
            (|f: &FileReport| &f.invalid_rust) as fn(&FileReport) -> &Vec<Failure>,
        ),
        ("Generation errors", |f: &FileReport| &f.generation_errors),
    ] {
        let _ = writeln!(s, "\n## {title}\n");
        for f in files {
            for g in pick(f) {
                let first = g.message.lines().next().unwrap_or("");
                let _ = writeln!(
                    s,
                    "- `{}` #{} {}: {}",
                    f.name, g.group, g.description, first
                );
            }
        }
    }
}
