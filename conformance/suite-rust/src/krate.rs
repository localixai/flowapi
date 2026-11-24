//! The generated crate: a module per group, and a binary that reads every case of a list and
//! says, per case, whether its group's root type read it and what it wrote back. It is built
//! once (`cargo build --release -j 2`); a group whose module does not compile is taken out and
//! the crate built again without it.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

const MANIFEST: &str = r#"# Written by conformance/suite-rust: the Rust types of every group of the suite.
[package]
name = "flowapi-suite-rust-gen"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = 0
incremental = false
"#;

const RUNNER: &str = r#"
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{BufRead, Write};

fn check<T: DeserializeOwned + Serialize>(data: &str) -> String {
    match serde_json::from_str::<T>(data) {
        Ok(v) => match serde_json::to_string(&v) {
            Ok(s) => format!("ok\t{s}"),
            Err(e) => format!("unwritten\t{e}"),
        },
        Err(e) => format!("err\t{e}"),
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("the cases' file");
    let file = std::fs::File::open(path).expect("the cases' file is readable");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.expect("a line");
        let mut cols = line.splitn(3, '\t');
        let (Some(g), Some(c), Some(data)) = (cols.next(), cols.next(), cols.next()) else {
            continue;
        };
        let at: usize = g.parse().expect("a group number");
        let verdict = GROUPS[at](data).replace('\n', " ");
        writeln!(out, "{g}\t{c}\t{verdict}").expect("stdout");
    }
}
"#;

/// Writes the crate with `modules` (each a key and its text); `skip` are left out.
pub(crate) fn write(dir: &Path, modules: &[(String, String)], skip: &BTreeSet<usize>) {
    let src = dir.join("src/g");
    if dir.join("src").exists() {
        std::fs::remove_dir_all(dir.join("src")).expect("the old sources are removable");
    }
    std::fs::create_dir_all(&src).expect("the sources' directory is creatable");
    std::fs::write(dir.join("Cargo.toml"), MANIFEST).expect("the manifest is writable");
    let mut main = String::from(
        "//! Written by conformance/suite-rust.\n#![allow(dead_code, non_camel_case_types, \
         non_snake_case, unused_imports, clippy::all)]\n\n",
    );
    for (i, (key, text)) in modules.iter().enumerate() {
        if skip.contains(&i) {
            continue;
        }
        std::fs::write(src.join(format!("{key}.rs")), text).expect("a module is writable");
        let _ = writeln!(main, "#[path = \"g/{key}.rs\"]\nmod g{i};");
    }
    main.push_str(RUNNER);
    main.push_str("\nconst GROUPS: &[fn(&str) -> String] = &[\n");
    for i in 0..modules.len() {
        if skip.contains(&i) {
            main.push_str("    |_| String::from(\"err\\tnot built\"),\n");
        } else {
            let _ = writeln!(main, "    check::<g{i}::Root>,");
        }
    }
    main.push_str("];\n");
    std::fs::write(dir.join("src/main.rs"), main).expect("main.rs is writable");
}

/// Builds the crate; the modules its errors name, by index into `modules`, when it fails.
pub(crate) fn build(
    dir: &Path,
    modules: &[(String, String)],
) -> Result<(), (BTreeSet<usize>, String)> {
    let output = Command::new("cargo")
        .current_dir(dir)
        .args(["build", "--release", "-j", "1", "--message-format", "short"])
        .output()
        .expect("cargo runs");
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let mut bad = BTreeSet::new();
    for line in stderr.lines().filter(|l| l.contains(": error")) {
        for (i, (key, _)) in modules.iter().enumerate() {
            if line.starts_with(&format!("src/g/{key}.rs:")) {
                bad.insert(i);
            }
        }
    }
    Err((bad, stderr))
}

/// Runs the binary over the cases' file; its output lines.
pub(crate) fn run(dir: &Path, cases: &Path) -> String {
    let bin = dir.join("target/release/flowapi-suite-rust-gen");
    let output = Command::new(bin)
        .arg(cases)
        .output()
        .expect("the generated binary runs");
    String::from_utf8_lossy(&output.stdout).into_owned()
}
