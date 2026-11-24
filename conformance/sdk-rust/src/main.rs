//! Oracle O8 for the Rust target: the SDK run, not read. The Style Examples table and the cases
//! of `conformance/sdk/cases.json` are made into descriptions as for TypeScript
//! (`describe.rs`); flowapi's Rust target with the SDK generates each version's module into one
//! crate (`krate.rs`), built once; every call is run and compared exactly (`compare.rs`). The
//! crate's warnings about the SDK are differences too. Run from here:
//!   cargo run --release -j 1 --bin sdk-rust
//! Writes `conformance/results/sdk-rust.md`; exits 1 when anything differs.

mod compare;
mod describe;
mod krate;

use describe::{Call, Check, Docs};
use serde_json::Value;
use std::fmt::Write as _;
use std::path::Path;

fn read(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).expect("the file is readable");
    serde_json::from_str(&text).expect("the file is JSON")
}

/// A version's SDK: the module's text, its diagnostics, and each operation's function base.
fn generate(version: &str, doc: &Value) -> (String, String, Vec<String>) {
    let text = serde_json::to_string_pretty(doc).expect("a description prints");
    let mut docs = flowapi::Documents::new();
    docs.add(format!("file:///v{version}.json"), &text)
        .expect("the description is JSON");
    let target = flowapi_rust::Rust::new("sdk.rs").with_sdk();
    let (output, diags) = flowapi::generate(&docs, &target);
    let module: String = output
        .map(|o| o.files.into_iter().map(|f| f.text).collect())
        .unwrap_or_default();
    let warnings: String = diags.all().iter().map(|d| format!("{d}\n")).collect();
    let bases = module
        .lines()
        .filter_map(|l| {
            l.strip_prefix("pub fn ")?
                .split_once("_request(")
                .map(|(b, _)| b.to_owned())
        })
        .collect();
    (module, warnings, bases)
}

fn main() -> std::process::ExitCode {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = here.parent().expect("conformance/").to_path_buf();
    let table = read(&root.join("suites/style-examples.json"));
    let cases_file = read(&root.join("sdk/cases.json"));
    let cases = cases_file["cases"].as_array().cloned().unwrap_or_default();
    let order = read(&here.join("order.json"))["calls"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let own = read(&here.join("cases.json"))["cases"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut docs = Docs::default();
    let mut calls: Vec<Call> = Vec::new();
    describe::table(&mut docs, &table, &mut calls);
    let merged = describe::cases(&mut docs, "c", &cases, &order, &mut calls);
    let shared = calls.len();
    describe::cases(&mut docs, "r", &own, &[], &mut calls);

    let mut failures: Vec<String> = Vec::new();
    let mut modules = Vec::new();
    let mut diagnostics = std::collections::BTreeMap::new();
    let mut bases = std::collections::BTreeMap::new();
    for (version, doc, ids) in &docs.docs {
        let name = format!("v{}", version.replace('.', "_"));
        let (text, warnings, b) = generate(version, doc);
        if b.len() != ids.len() {
            failures.push(format!(
                "{version}: {} operations, {} request functions",
                ids.len(),
                b.len()
            ));
        }
        modules.push((name.clone(), text));
        diagnostics.insert(version.clone(), warnings);
        bases.insert(version.clone(), (name, b));
    }
    modules.extend(kept(&root));
    let entries: Vec<krate::Entry> = calls
        .iter()
        .map(|c| {
            let (module, b) = &bases[&c.version];
            krate::Entry {
                module: module.clone(),
                base: b.get(c.op).cloned().unwrap_or_default(),
                bare: c.bare,
                response: c.response,
            }
        })
        .collect();
    let gen_dir = here.join("out/gen");
    krate::write(&gen_dir, &modules, &entries);
    match krate::build(&gen_dir) {
        Ok(warnings) => failures.extend(
            warnings
                .into_iter()
                .map(|w| format!("the generated crate warns: {w}")),
        ),
        Err(stderr) => {
            eprintln!("the generated crate does not build:\n{stderr}");
            return std::process::ExitCode::FAILURE;
        }
    }
    let got = answers(here, &gen_dir, &calls);
    let mut counts = [(0, 0); 3];
    for (i, (c, g)) in calls.iter().zip(&got).enumerate() {
        let before = failures.len();
        match g {
            Some(v) => compare::check(c, v, &mut failures),
            None => failures.push(format!("{}: no answer, the binary stopped", c.what)),
        }
        let failed = failures.len() - before;
        let row = if matches!(c.check, Check::Table { .. }) {
            0
        } else if i < shared {
            1
        } else {
            2
        };
        if let Some(r) = counts.get_mut(row) {
            r.0 += 1;
            r.1 += failed;
        }
    }
    warned(cases.iter().chain(&own), &diagnostics, &mut failures);
    let report = report(&table, (cases.len(), own.len()), counts, merged, &failures);
    std::fs::write(root.join("results/sdk-rust.md"), &report).expect("sdk-rust.md is writable");
    print!("{report}");
    if failures.is_empty() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Every call run by the generated binary: what each gave, `None` where it gave nothing.
fn answers(here: &Path, gen_dir: &Path, calls: &[Call]) -> Vec<Option<Value>> {
    let mut list = String::new();
    for (i, c) in calls.iter().enumerate() {
        let _ = writeln!(list, "{i}\t{}", c.data);
    }
    let list_path = here.join("out/calls.tsv");
    std::fs::write(&list_path, list).expect("the calls' file is writable");
    let mut got: Vec<Option<Value>> = vec![None; calls.len()];
    for line in krate::run(gen_dir, &list_path).lines() {
        if let Some((n, json)) = line.split_once('\t')
            && let (Ok(n), Ok(v)) = (n.parse::<usize>(), serde_json::from_str(json))
            && let Some(slot) = got.get_mut(n)
        {
            *slot = Some(v);
        }
    }
    got
}

/// The SDKs kept by crates/flowapi-rust/tests/sdk.rs, built too: each must compile.
fn kept(root: &Path) -> Vec<(String, String)> {
    let mut modules = Vec::new();
    let kept = root.join("../crates/flowapi-rust/tests/sdk");
    let mut kept_files: Vec<_> = std::fs::read_dir(&kept)
        .expect("the kept SDKs' directory")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    kept_files.sort();
    for p in &kept_files {
        let stem = p
            .file_stem()
            .map(|s| s.to_string_lossy().replace('-', "_"))
            .unwrap_or_default();
        let text = std::fs::read_to_string(p).expect("a kept SDK is readable");
        modules.push((format!("kept_{stem}"), text));
    }
    modules
}

/// Each case that names a diagnostic has it, in each version it is generated in.
fn warned<'c>(
    cases: impl Iterator<Item = &'c Value>,
    diagnostics: &std::collections::BTreeMap<String, String>,
    failures: &mut Vec<String>,
) {
    for c in cases {
        let Some(code) = c["warns"].as_str() else {
            continue;
        };
        for (version, d) in diagnostics {
            let applies = c["versions"].as_array().map_or(version != "2.0", |vs| {
                vs.iter().any(|v| v == version.as_str())
            });
            if applies && !d.contains(code) {
                failures.push(format!(
                    "{} ({version}): no `{code}` diagnostic",
                    c["name"].as_str().unwrap_or("")
                ));
            }
        }
    }
}

fn report(
    table: &Value,
    (cases, own): (usize, usize),
    [
        (table_run, table_failed),
        (cases_run, cases_failed),
        (own_run, own_failed),
    ]: [(usize, usize); 3],
    merged: usize,
    failures: &[String],
) -> String {
    let mut out = String::from(
        "# O8: the Rust SDK, run\n\nWritten by `cargo run --release -j 1 --bin sdk-rust` in \
         `conformance/sdk-rust`; do not edit. Every request and response function is generated by \
         `flowapi --target rust --sdk` into one crate, built (a warning about it is a difference), \
         and called; strings are compared exactly.\n\n| Oracle | Checked | Differ |\n| --- | ---: | ---: |\n",
    );
    let _ = writeln!(
        out,
        "| OpenAPI 3.2.1 \"Style Examples\" ({}:{}), each cell not _n/a_ | {table_run} | {table_failed} |",
        table["source"].as_str().unwrap_or(""),
        table["heading"]
    );
    let _ = writeln!(
        out,
        "| conformance/sdk/cases.json: {cases} cases, each in its versions | {cases_run} | {cases_failed} |"
    );
    let _ = writeln!(
        out,
        "| conformance/sdk-rust/cases.json, the Rust types' own: {own} cases | {own_run} | {own_failed} |"
    );
    let _ = writeln!(
        out,
        "\n{merged} calls are held to `conformance/sdk-rust/order.json`: an object with no named \
         properties is a `BTreeMap` in Rust, so its members are written in key order.\n\n## Differences\n"
    );
    if failures.is_empty() {
        out.push_str("None.\n");
    }
    for f in failures {
        let _ = writeln!(out, "- {f}");
    }
    out
}
