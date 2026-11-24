//! The compiler's verdict on each case, counted per keyword file. A group whose generated types
//! do not compile is counted apart: its cases say nothing about soundness.

use crate::generate::Generated;
use crate::load::{Admission, SuiteFile};
use crate::tsc::Diagnostics;
use std::collections::BTreeMap;

/// A group that did not reach the cases: why, in one message.
pub(crate) struct Failure {
    pub(crate) group: usize,
    pub(crate) description: String,
    pub(crate) message: String,
}

/// A valid case the type rejects.
pub(crate) struct Unsound {
    pub(crate) group: usize,
    pub(crate) group_description: String,
    pub(crate) description: String,
    pub(crate) data: String,
    pub(crate) error: String,
}

/// One keyword file's counts.
#[derive(Default)]
pub(crate) struct FileReport {
    pub(crate) name: String,
    pub(crate) groups: usize,
    pub(crate) skipped_dialect: usize,
    pub(crate) skipped_tags: usize,
    pub(crate) generated: usize,
    pub(crate) generation_errors: Vec<Failure>,
    pub(crate) invalid_ts: Vec<Failure>,
    pub(crate) valid: usize,
    pub(crate) valid_accepted: usize,
    pub(crate) invalid: usize,
    pub(crate) invalid_rejected: usize,
    pub(crate) unsound: Vec<Unsound>,
}

/// Counts one keyword file. `generated` holds one entry per group run, by group index.
pub(crate) fn file(
    f: &SuiteFile,
    generated: &BTreeMap<usize, Generated>,
    tsc: &Diagnostics,
) -> FileReport {
    let mut r = FileReport {
        name: f.name.clone(),
        groups: f.groups.len(),
        ..FileReport::default()
    };
    let none = BTreeMap::new();
    for g in &f.groups {
        match g.admission {
            Admission::Dialect => r.skipped_dialect += 1,
            Admission::Tags => r.skipped_tags += 1,
            Admission::Run => {}
        }
        let Some(result) = generated.get(&g.index) else {
            continue;
        };
        let failure = |message: String| Failure {
            group: g.index,
            description: g.description.clone(),
            message,
        };
        let (file, first) = match result {
            Generated::Error(message) => {
                r.generation_errors.push(failure(message.clone()));
                continue;
            }
            Generated::Written {
                file,
                first_case_line,
            } => (file, *first_case_line),
        };
        r.generated += 1;
        let lines = tsc.by_file.get(file).unwrap_or(&none);
        let on_types: Vec<String> = lines
            .range(..first)
            .flat_map(|(n, ms)| ms.iter().map(move |m| format!("line {n}: {m}")))
            .collect();
        if !on_types.is_empty() {
            r.invalid_ts.push(failure(on_types.join("\n")));
            continue;
        }
        for (i, case) in g.cases.iter().enumerate() {
            let errors = lines.get(&(first + i));
            match (case.valid, errors) {
                (true, None) => {
                    r.valid += 1;
                    r.valid_accepted += 1;
                }
                (true, Some(ms)) => {
                    r.valid += 1;
                    r.unsound.push(Unsound {
                        group: g.index,
                        group_description: g.description.clone(),
                        description: case.description.clone(),
                        data: serde_json::to_string(&case.data).expect("data prints as JSON"),
                        error: ms.join("\n"),
                    });
                }
                (false, None) => r.invalid += 1,
                (false, Some(_)) => {
                    r.invalid += 1;
                    r.invalid_rejected += 1;
                }
            }
        }
    }
    r
}

/// A share as a percentage with one decimal, rounded down so that 100.0 means all of them, or a
/// dash when there is nothing to share.
pub(crate) fn percent(part: usize, whole: usize) -> String {
    if whole == 0 {
        return String::from("-");
    }
    let tenths = part * 1000 / whole;
    format!("{}.{}", tenths / 10, tenths % 10)
}
