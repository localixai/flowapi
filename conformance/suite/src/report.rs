//! The results, written so that the same suite and the same flowapi give the same bytes: files
//! by name, groups and cases by index, no time, no path of this machine.

use crate::evaluate::{Failure, FileReport, percent};
use serde_json::{Value, json};
use std::fmt::Write as _;

/// The totals over every file.
#[derive(Default)]
pub(crate) struct Totals {
    pub(crate) groups: usize,
    pub(crate) run: usize,
    pub(crate) generated: usize,
    pub(crate) generation_errors: usize,
    pub(crate) invalid_ts: usize,
    pub(crate) valid: usize,
    pub(crate) valid_accepted: usize,
    pub(crate) invalid: usize,
    pub(crate) invalid_rejected: usize,
}

pub(crate) fn totals(files: &[FileReport]) -> Totals {
    let mut t = Totals::default();
    for f in files {
        t.groups += f.groups;
        t.run += f.groups - f.skipped_dialect - f.skipped_tags;
        t.generated += f.generated;
        t.generation_errors += f.generation_errors.len();
        t.invalid_ts += f.invalid_ts.len();
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

pub(crate) fn json(files: &[FileReport], other: &[String]) -> String {
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
                "skipped_dialect": f.skipped_dialect,
                "skipped_format_assertion": f.skipped_tags,
                "generated": f.generated,
                "generation_errors": failures(&f.generation_errors),
                "invalid_typescript": failures(&f.invalid_ts),
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
        "totals": {
            "groups": t.groups, "groups_run": t.run, "generated": t.generated,
            "generation_errors": t.generation_errors, "invalid_typescript": t.invalid_ts,
            "valid_cases": t.valid, "valid_accepted": t.valid_accepted,
            "invalid_cases": t.invalid, "invalid_rejected": t.invalid_rejected,
            "soundness_percent": percent(t.valid_accepted, t.valid),
            "precision_percent": percent(t.invalid_rejected, t.invalid),
        },
        "compiler_other": other,
        "files": per_file,
    });
    let mut s = serde_json::to_string_pretty(&all).expect("results print as JSON");
    s.push('\n');
    s
}

pub(crate) fn markdown(files: &[FileReport], other: &[String]) -> String {
    let t = totals(files);
    let mut s = String::from("# O3: JSON-Schema-Test-Suite through the TypeScript compiler\n\n");
    s.push_str("Written by `cd conformance/suite && cargo run --release`; do not edit.\n\n");
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
        "- Generation errors: {}; invalid TypeScript: {}\n",
        t.generation_errors, t.invalid_ts
    );
    s.push_str("| File | Groups | Run | Gen. errors | Invalid TS | Valid | Accepted | Sound % | Invalid | Rejected | Precision % |\n");
    s.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for f in files {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            f.name,
            f.groups,
            f.groups - f.skipped_dialect - f.skipped_tags,
            f.generation_errors.len(),
            f.invalid_ts.len(),
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
        t.invalid_ts,
        t.valid,
        t.valid_accepted,
        percent(t.valid_accepted, t.valid),
        t.invalid,
        t.invalid_rejected,
        percent(t.invalid_rejected, t.invalid),
    );
    lists(&mut s, files, other);
    s
}

fn lists(s: &mut String, files: &[FileReport], other: &[String]) {
    s.push_str("\n## Unsound cases\n\nA valid case the type rejects: each is a bug.\n\n");
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
    s.push_str("\n## Invalid TypeScript\n\nThe generated types themselves do not compile.\n\n");
    for f in files {
        for g in &f.invalid_ts {
            let first = g.message.lines().next().unwrap_or("");
            let _ = writeln!(
                s,
                "- `{}` #{} {}: {}",
                f.name, g.group, g.description, first
            );
        }
    }
    s.push_str("\n## Generation errors\n\n");
    for f in files {
        for g in &f.generation_errors {
            let first = g.message.lines().next().unwrap_or("");
            let _ = writeln!(
                s,
                "- `{}` #{} {}: {}",
                f.name, g.group, g.description, first
            );
        }
    }
    if !other.is_empty() {
        s.push_str("\n## Compiler output tied to no case\n\n");
        for line in other {
            let _ = writeln!(s, "    {line}");
        }
    }
}
