//! The rules of AGENTS.md that a test can hold, over every crate of the workspace: what each
//! failure names is the file and the rule to fix, never this test.
#![expect(
    clippy::expect_used,
    reason = "the suite's helpers fail the test by panicking, as a test does"
)]

use std::path::{Path, PathBuf};

/// G10: the most lines of working code (not blank, not a comment) a file may hold.
const MAX_FILE_LINES: usize = 300;
/// G10: the most lines of working code the core may hold, all files together.
const MAX_CORE_LINES: usize = 6_000;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).expect("a directory") {
            let p = entry.expect("an entry").path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn crate_sources(name: &str) -> Vec<PathBuf> {
    sources(&root().join("crates").join(name).join("src"))
}

fn all_sources() -> Vec<PathBuf> {
    ["flowapi", "flowapi-ts", "flowapi-cli"]
        .iter()
        .flat_map(|c| crate_sources(c))
        .collect()
}

fn working_lines(text: &str) -> usize {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .count()
}

fn name(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

#[test]
fn g10_files_are_small() {
    let over: Vec<String> = all_sources()
        .into_iter()
        .filter_map(|p| {
            let n = working_lines(&std::fs::read_to_string(&p).unwrap());
            (n > MAX_FILE_LINES).then(|| format!("{}: {n} lines", name(&p)))
        })
        .collect();
    assert!(
        over.is_empty(),
        "G10, over {MAX_FILE_LINES} lines of working code: {over:#?}"
    );
    let core: usize = crate_sources("flowapi")
        .iter()
        .map(|p| working_lines(&std::fs::read_to_string(p).unwrap()))
        .sum();
    assert!(
        core <= MAX_CORE_LINES,
        "G10: the core holds {core} lines, over {MAX_CORE_LINES}"
    );
}

#[test]
fn every_file_says_what_it_is() {
    let bare: Vec<String> = all_sources()
        .into_iter()
        .filter(|p| !std::fs::read_to_string(p).unwrap().starts_with("//!"))
        .map(|p| name(&p))
        .collect();
    assert!(
        bare.is_empty(),
        "a file opens with a `//!` comment saying what it is: {bare:#?}"
    );
}

#[test]
fn english_only() {
    let other: Vec<String> = all_sources()
        .into_iter()
        .filter(|p| {
            std::fs::read_to_string(p)
                .unwrap()
                .chars()
                .any(|c| ('\u{400}'..='\u{4ff}').contains(&c))
        })
        .map(|p| name(&p))
        .collect();
    assert!(
        other.is_empty(),
        "code and comments are in English: {other:#?}"
    );
}

// req: jsonschema-2020-12.core.references.should-not-schema-need-not-downloadable-address-network — the core does no I/O; an http $ref is only a name
// req: jsonschema-2020-12.validation.security.must-not-some-implementations-allow-embedding-arbitrary-code — the core runs nothing a document holds: no process, file or socket
#[test]
fn g5_the_core_and_targets_do_no_io() {
    let words = [
        "std::fs",
        "std::net",
        "std::process",
        "std::env",
        "std::thread",
        "Instant",
        "SystemTime",
        "println!",
        "eprintln!",
    ];
    let mut found = Vec::new();
    for p in crate_sources("flowapi")
        .into_iter()
        .chain(crate_sources("flowapi-ts"))
    {
        let text = std::fs::read_to_string(&p).unwrap();
        for w in words {
            if text
                .lines()
                .any(|l| !l.trim_start().starts_with("//") && l.contains(w))
            {
                found.push(format!("{}: {w}", name(&p)));
            }
        }
    }
    assert!(
        found.is_empty(),
        "G5, the core and the targets do no I/O: {found:#?}"
    );
}

/// G9: what each layer of the core may import; a layer names only those below it.
const LAYERS: &[(&str, &[&str])] = &[
    ("id", &[]),
    ("diag", &["id"]),
    ("uri", &[]),
    ("json", &["id", "json"]),
    ("source", &["id", "diag", "json", "uri", "source"]),
    ("schema", &["id", "diag", "json", "uri", "source", "schema"]),
    (
        "spec",
        &["id", "diag", "json", "uri", "source", "schema", "spec"],
    ),
    (
        "graph",
        &[
            "id", "diag", "json", "uri", "source", "schema", "spec", "graph",
        ],
    ),
    ("lower", &["id", "graph", "lower"]),
    ("program", &["id", "diag", "graph", "lower"]),
    (
        "names",
        &["id", "diag", "graph", "lower", "program", "source", "names"],
    ),
];

#[test]
fn g9_layers_import_only_below() {
    let src = root().join("crates/flowapi/src");
    let mut wrong = Vec::new();
    for p in sources(&src) {
        let rel = p.strip_prefix(&src).unwrap();
        let layer = rel
            .iter()
            .next()
            .unwrap()
            .to_string_lossy()
            .trim_end_matches(".rs")
            .trim_end_matches("_tests")
            .to_owned();
        let Some((_, allowed)) = LAYERS.iter().find(|(l, _)| *l == layer) else {
            continue;
        };
        let text = std::fs::read_to_string(&p).unwrap();
        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix("use crate::") else {
                continue;
            };
            let module = rest.split([':', ';', '{']).next().unwrap_or("");
            if !allowed.contains(&module) {
                wrong.push(format!("{}: `{layer}` imports `{module}`", name(&p)));
            }
        }
    }
    assert!(wrong.is_empty(), "G9: {wrong:#?}");
}

#[test]
fn unit_tests_sit_beside_their_file() {
    let mut loose = Vec::new();
    for p in all_sources()
        .into_iter()
        .filter(|p| p.to_string_lossy().ends_with("_tests.rs"))
    {
        let file = p.file_name().unwrap().to_string_lossy().into_owned();
        let owner = p.with_file_name(file.replace("_tests.rs", ".rs"));
        let wired = std::fs::read_to_string(&owner)
            .is_ok_and(|t| t.contains(&format!("#[path = \"{file}\"]")));
        if !wired {
            loose.push(name(&p));
        }
    }
    assert!(
        loose.is_empty(),
        "x_tests.rs is wired from x.rs by #[path]: {loose:#?}"
    );
}
