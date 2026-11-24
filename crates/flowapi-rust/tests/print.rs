//! The Rust printed for each description in `flowapi-ts/tests/cases/` (one set of cases for
//! every target), against the `.rs` kept under `tests/cases/` here.
//! `FLOWAPI_BLESS=1 cargo test -p flowapi-rust --test print` writes them anew; a change to one
//! is read in review like a change to code.
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use std::fmt::Write;
use std::path::Path;

fn generate(uri: &str, text: &str) -> String {
    let mut docs = Documents::new();
    docs.add(uri, text).expect("JSON");
    let target = flowapi_rust::Rust::new("types.rs");
    let (output, diags) = flowapi::generate(&docs, &target);
    let mut text = output
        .map(|o| o.files.into_iter().map(|f| f.text).collect::<String>())
        .unwrap_or_default();
    for d in diags.all() {
        let _ = writeln!(text, "// {d}");
    }
    text
}

#[test]
fn cases_print_as_kept() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inputs = root.join("../flowapi-ts/tests/cases");
    let kept_dir = root.join("tests/cases");
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut names: Vec<_> = std::fs::read_dir(&inputs)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    let mut changed = Vec::new();
    for case in names {
        let text = std::fs::read_to_string(&case).unwrap();
        let file = case.file_name().unwrap().to_string_lossy().into_owned();
        let printed = generate(&file, &text);
        let kept = kept_dir.join(&file).with_extension("rs");
        if bless {
            std::fs::write(&kept, &printed).unwrap();
        } else if std::fs::read_to_string(&kept).ok().as_deref() != Some(printed.as_str()) {
            changed.push(format!("{}:\n{printed}", kept.display()));
        }
    }
    assert!(
        changed.is_empty(),
        "printed otherwise than kept:\n{}",
        changed.join("\n")
    );
}

/// G1: a chain of references thousands long, and nesting as deep as the reader takes, end in
/// output, not in an overflowed stack.
#[test]
fn deep_inputs_do_not_overflow() {
    let mut defs = String::new();
    for i in 0..20_000 {
        let _ = write!(
            defs,
            "\"d{i}\":{{\"type\":\"object\",\"properties\":{{\"next\":{{\"$ref\":\"#/$defs/d{}\"}}}}}},",
            i + 1
        );
    }
    defs.push_str("\"d20000\":{\"type\":\"string\"}");
    let chain = format!("{{\"$ref\":\"#/$defs/d0\",\"$defs\":{{{defs}}}}}");
    assert!(generate("chain.json", &chain).contains("Root"));
    let nested = format!(
        "{}{{\"type\":\"string\"}}{}",
        "{\"items\":".repeat(500),
        "}".repeat(500)
    );
    assert!(generate("nested.json", &nested).contains("Root"));
}

/// G4: the same input prints the same bytes, every time.
#[test]
fn output_is_deterministic() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../flowapi-ts/tests/cases/schema.json"),
    )
    .unwrap();
    let first = generate("schema.json", &text);
    for _ in 0..5 {
        assert_eq!(generate("schema.json", &text), first);
    }
}
