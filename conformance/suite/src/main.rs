//! Oracles O3 and O3': every group of the JSON-Schema-Test-Suite that applies to 2020-12 is
//! generated with flowapi's TypeScript target, each case's data is written as a literal of the
//! root type, and one run of the compiler says which literals fit. A valid case that does not
//! fit is unsound (a bug); an invalid case that does not fit is a rejection (precision).

mod evaluate;
mod generate;
mod load;
mod report;
mod tsc;

use generate::Generated;
use load::Admission;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn main() -> std::process::ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("conformance/")
        .to_path_buf();
    let suite = load::suite(&root.join("suites/json-schema-test-suite/tests"));
    let out = root.join("out");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old output is removable");
    }
    std::fs::create_dir_all(&out).expect("the output directory is creatable");

    // A panic in flowapi is recorded as the group's failure; its default report would only
    // repeat it on stderr.
    std::panic::set_hook(Box::new(|_| {}));
    let mut generated: Vec<BTreeMap<usize, Generated>> = Vec::new();
    let mut written: Vec<String> = Vec::new();
    for f in &suite {
        let mut by_group = BTreeMap::new();
        for g in f.groups.iter().filter(|g| g.admission == Admission::Run) {
            let result = generate::group(&f.name, g, &out);
            if let Generated::Written { file, .. } = &result {
                written.push(file.clone());
            }
            by_group.insert(g.index, result);
        }
        generated.push(by_group);
    }
    let _ = std::panic::take_hook();

    let tsc_path: PathBuf = root.join("ts/node_modules/.bin/tsc");
    let diagnostics = tsc::check(&tsc_path, &out, &written);
    let files: Vec<_> = suite
        .iter()
        .zip(&generated)
        .map(|(f, g)| evaluate::file(f, g, &diagnostics))
        .collect();

    let results = root.join("results");
    std::fs::create_dir_all(&results).expect("the results directory is creatable");
    let before = previous_rejected(&results.join("suite.json"));
    std::fs::write(
        results.join("suite.json"),
        report::json(&files, &diagnostics.other),
    )
    .expect("suite.json is writable");
    std::fs::write(
        results.join("suite.md"),
        report::markdown(&files, &diagnostics.other),
    )
    .expect("suite.md is writable");

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
        "groups {} run of {}, {} generated, {} generation errors, {} invalid TypeScript; results in conformance/results/",
        t.run, t.groups, t.generated, t.generation_errors, t.invalid_ts,
    );
    // G3: soundness is all or nothing; precision never falls below the run committed last.
    let mut broken = Vec::new();
    if t.valid_accepted != t.valid {
        broken.push(format!(
            "{} valid cases rejected (unsound)",
            t.valid - t.valid_accepted
        ));
    }
    if t.generation_errors > 0 || t.invalid_ts > 0 {
        broken.push(format!(
            "{} generation errors, {} groups of invalid TypeScript",
            t.generation_errors, t.invalid_ts
        ));
    }
    if let Some(before) = before.filter(|&b| t.invalid_rejected < b) {
        broken.push(format!(
            "precision fell: {} invalid cases rejected, {before} before",
            t.invalid_rejected
        ));
    }
    if !broken.is_empty() {
        eprintln!("G3 broken: {}", broken.join("; "));
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

/// The invalid cases the committed run rejected, read before it is written over.
fn previous_rejected(path: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value
        .get("totals")?
        .get("invalid_rejected")?
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
}
