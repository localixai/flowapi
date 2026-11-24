//! Oracle O4: the JSON reader against JSONTestSuite. Every `y_` text is read, every `n_` text is
//! refused, and no text, `i_` among them, makes it panic (G1). A text that is not UTF-8 is refused
//! before the reader sees it, as the CLI does.
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "the suite's helpers fail the test by panicking, as a test does"
)]

use std::path::Path;

fn cases() -> Vec<(String, Vec<u8>)> {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance/suites/json-test-suite");
    let mut out: Vec<(String, Vec<u8>)> = std::fs::read_dir(&dir)
        .expect("the suite's copy")
        .map(|e| e.expect("an entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read(&p).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

fn accepts(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok_and(|text| flowapi::json::parse(text).is_ok())
}

#[test]
fn every_y_case_is_read() {
    let refused: Vec<String> = cases()
        .into_iter()
        .filter(|(n, b)| n.starts_with("y_") && !accepts(b))
        .map(|(n, _)| n)
        .collect();
    assert!(refused.is_empty(), "refused: {refused:?}");
}

#[test]
fn every_n_case_is_refused() {
    let read: Vec<String> = cases()
        .into_iter()
        .filter(|(n, b)| n.starts_with("n_") && accepts(b))
        .map(|(n, _)| n)
        .collect();
    assert!(read.is_empty(), "read: {read:?}");
}

/// The cases the specification leaves open, decided: one that is UTF-8 is read (lone surrogates
/// in escapes, huge numbers, 500 levels of nesting: the grammar allows them all); one that is
/// not is refused.
#[test]
fn i_cases_are_decided() {
    let mut seen = 0;
    for (name, bytes) in cases().into_iter().filter(|(n, _)| n.starts_with("i_")) {
        seen += 1;
        assert_eq!(
            accepts(&bytes),
            std::str::from_utf8(&bytes).is_ok(),
            "{name}"
        );
    }
    assert_eq!(seen, 35, "the suite's i_ cases");
}
