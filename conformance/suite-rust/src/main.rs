//! Oracle O3 for the Rust target: every group of the JSON-Schema-Test-Suite that applies to
//! 2020-12 is generated with flowapi's Rust target into one crate, built once, and each case's
//! data is read by `serde_json::from_str::<Root>` and written back. A valid case that is not
//! read, or not written back equal, is unsound (a bug); an invalid case that is not read is a
//! rejection (precision).

mod evaluate;
mod generate;
mod krate;
mod load;
mod report;

use evaluate::{Failure, FileReport, Verdict};
use load::Admission;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

/// What a group gave: a module (its index in the crate), or an error.
enum Generated {
    Module(usize),
    Error(String),
}

fn main() -> std::process::ExitCode {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent().expect("conformance/").to_path_buf();
    let mut suite = load::suite(&root.join("suites/json-schema-test-suite/tests"));
    // Our own cases in the suite's format (`own/flowapi-*.json`), reported as rows of their own.
    suite.extend(load::suite(&here.join("own")));
    let out = here.join("out");
    std::fs::create_dir_all(&out).expect("the output directory is creatable");

    std::panic::set_hook(Box::new(|_| {}));
    let mut modules: Vec<(String, String)> = Vec::new();
    let mut generated: Vec<BTreeMap<usize, Generated>> = Vec::new();
    for f in &suite {
        let mut by_group = BTreeMap::new();
        for g in f.groups.iter().filter(|g| g.admission == Admission::Run) {
            let result = match generate::group(&f.name, g) {
                Ok(text) => {
                    modules.push((format!("{}-{}", f.name, g.index), text));
                    Generated::Module(modules.len() - 1)
                }
                Err(message) => Generated::Error(message),
            };
            by_group.insert(g.index, result);
        }
        generated.push(by_group);
    }
    let _ = std::panic::take_hook();

    let gen_dir = out.join("gen");
    if std::env::args().any(|a| a == "--write-only") {
        // The crate written, not built: to look at, or to build by hand.
        krate::write(&gen_dir, &modules, &BTreeSet::new());
        println!("{} modules written to out/gen", modules.len());
        return std::process::ExitCode::SUCCESS;
    }
    let mut skip: BTreeSet<usize> = BTreeSet::new();
    let mut invalid: BTreeMap<usize, String> = BTreeMap::new();
    loop {
        krate::write(&gen_dir, &modules, &skip);
        match krate::build(&gen_dir, &modules) {
            Ok(()) => break,
            Err((bad, stderr)) if !bad.is_empty() && bad.is_disjoint(&skip) => {
                for i in bad {
                    let key = modules.get(i).map_or("", |m| m.0.as_str());
                    let first = stderr
                        .lines()
                        .find(|l| l.starts_with(&format!("src/g/{key}.rs:")))
                        .unwrap_or("");
                    invalid.insert(i, first.to_owned());
                    skip.insert(i);
                }
            }
            Err((_, stderr)) => {
                eprintln!("the generated crate does not build:\n{stderr}");
                return std::process::ExitCode::FAILURE;
            }
        }
    }

    let mut cases = String::new();
    for (f, by_group) in suite.iter().zip(&generated) {
        for g in &f.groups {
            if let Some(Generated::Module(m)) = by_group.get(&g.index) {
                for (c, case) in g.cases.iter().enumerate() {
                    let data = serde_json::to_string(&case.data).expect("data prints as JSON");
                    let _ = writeln!(cases, "{m}\t{c}\t{data}");
                }
            }
        }
    }
    let cases_path = out.join("cases.tsv");
    std::fs::write(&cases_path, cases).expect("the cases' file is writable");
    let verdicts = verdicts(&krate::run(&gen_dir, &cases_path));

    let files: Vec<FileReport> = suite
        .iter()
        .zip(&generated)
        .map(|(f, by_group)| file(f, by_group, &invalid, &verdicts))
        .collect();
    let results = root.join("results");
    std::fs::write(results.join("suite-rust.json"), report::json(&files))
        .expect("suite-rust.json is writable");
    std::fs::write(results.join("suite-rust.md"), report::markdown(&files))
        .expect("suite-rust.md is writable");
    let t = report::totals(&files);
    println!(
        "soundness {}% ({}/{} valid accepted), precision {}% ({}/{} invalid rejected)",
        evaluate::percent(t.valid_accepted, t.valid),
        t.valid_accepted,
        t.valid,
        evaluate::percent(t.invalid_rejected, t.invalid),
        t.invalid_rejected,
        t.invalid,
    );
    println!(
        "groups {} run of {}, {} generated, {} generation errors, {} invalid Rust",
        t.run, t.groups, t.generated, t.generation_errors, t.invalid_rust,
    );
    // G3, as for TypeScript: every valid case read and written back, every group built.
    if t.valid_accepted != t.valid || t.generation_errors > 0 || t.invalid_rust > 0 {
        eprintln!("G3 broken for the Rust target");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

fn verdicts(text: &str) -> BTreeMap<(usize, usize), Verdict> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.splitn(4, '\t').collect();
        let (Some(m), Some(c), Some(kind)) = (cols.first(), cols.get(1), cols.get(2)) else {
            continue;
        };
        let (Ok(m), Ok(c)) = (m.parse(), c.parse()) else {
            continue;
        };
        let rest = cols.get(3).copied().unwrap_or("").to_owned();
        let v = if *kind == "ok" {
            Verdict::Read(rest)
        } else {
            Verdict::Refused(format!("{kind}: {rest}"))
        };
        out.insert((m, c), v);
    }
    out
}

fn file(
    f: &load::SuiteFile,
    by_group: &BTreeMap<usize, Generated>,
    invalid: &BTreeMap<usize, String>,
    verdicts: &BTreeMap<(usize, usize), Verdict>,
) -> FileReport {
    let mut r = FileReport {
        name: f.name.clone(),
        groups: f.groups.len(),
        ..FileReport::default()
    };
    for g in &f.groups {
        if g.admission != Admission::Run {
            r.skipped += 1;
        }
        let failure = |message: String| Failure {
            group: g.index,
            description: g.description.clone(),
            message,
        };
        let m = match by_group.get(&g.index) {
            None => continue,
            Some(Generated::Error(message)) => {
                r.generation_errors.push(failure(message.clone()));
                continue;
            }
            Some(Generated::Module(m)) => *m,
        };
        r.generated += 1;
        if let Some(message) = invalid.get(&m) {
            r.invalid_rust.push(failure(message.clone()));
            continue;
        }
        let missing = Verdict::Refused(String::from("no verdict: the binary stopped"));
        for (c, case) in g.cases.iter().enumerate() {
            let v = verdicts.get(&(m, c)).unwrap_or(&missing);
            evaluate::count(&mut r, (g.index, &g.description), case, v);
        }
    }
    r
}
