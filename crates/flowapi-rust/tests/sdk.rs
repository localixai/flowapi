//! The Rust SDK printed for each description in `flowapi-ts/tests/sdk/` (the TypeScript SDK's
//! cases: styles, bodies, Swagger 2.0) and in `tests/sdk/` here (`names.json`: schemas named as
//! the SDK's own items, `sdk`, `GetPetInput`, `GetPetResponse` and `Request`, and two operations
//! whose functions would meet), against the `.rs` kept under `tests/sdk/`, diagnostics after it
//! as comments. `FLOWAPI_BLESS=1 cargo test -p flowapi-rust --test sdk` writes them anew; a
//! change to one is read in review like code. What the SDK does when it runs is oracle O8 for
//! Rust (`conformance/sdk-rust`).
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The file printed, then each diagnostic as a comment.
fn generate(uri: &str, text: &str, sdk: bool) -> String {
    let (mut text, diags) = files(uri, text, sdk);
    text.push_str(&diags);
    text
}

fn files(uri: &str, text: &str, sdk: bool) -> (String, String) {
    let mut docs = Documents::new();
    docs.add(uri, text).expect("JSON");
    let target = flowapi_rust::Rust::new("sdk.rs");
    let target = if sdk { target.with_sdk() } else { target };
    let (output, diags) = flowapi::generate(&docs, &target);
    let text = output
        .map(|o| o.files.into_iter().map(|f| f.text).collect::<String>())
        .unwrap_or_default();
    let mut said = String::new();
    for d in diags.all() {
        let _ = writeln!(said, "// {d}");
    }
    (text, said)
}

fn inputs() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<PathBuf> = [root.join("../flowapi-ts/tests/sdk"), root.join("tests/sdk")]
        .iter()
        .flat_map(|d| {
            std::fs::read_dir(d)
                .expect("a directory")
                .map(|e| e.expect("an entry").path())
        })
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter(|p| !p.ends_with("reserved.json"))
        .collect();
    names.sort_by_key(|p| p.file_name().map(std::ffi::OsStr::to_owned));
    names
}

#[test]
fn sdks_print_as_kept() {
    let kept_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sdk");
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for case in inputs() {
        let text = std::fs::read_to_string(&case).unwrap();
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let printed = generate(&name, &text, true);
        let kept = kept_dir.join(&name).with_extension("rs");
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

/// The types are printed as without the SDK, the SDK after them, for a description that names
/// no schema `sdk`.
#[test]
fn the_sdk_only_adds() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../flowapi-ts/tests/sdk/bodies.json");
    let text = std::fs::read_to_string(path).unwrap();
    let (plain, _) = files("bodies.json", &text, false);
    let (with, _) = files("bodies.json", &text, true);
    assert!(with.starts_with(plain.as_str()));
    assert!(with.contains("\npub mod sdk {\n"));
}

/// G4: the same input prints the same bytes, every time.
#[test]
fn sdk_is_deterministic() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../flowapi-ts/tests/sdk/styles.json");
    let text = std::fs::read_to_string(path).unwrap();
    let first = generate("styles.json", &text, true);
    for _ in 0..5 {
        assert_eq!(generate("styles.json", &text, true), first);
    }
}
