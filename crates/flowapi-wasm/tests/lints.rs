//! The crate's lint table is the workspace's, `unsafe_code` aside (`Cargo.toml` says why): a
//! lint added to or changed in one and not the other fails here.

use std::path::Path;

/// The `key = value` lines of the sections whose header is one of `headers`, comments dropped.
fn lints(text: &str, headers: &[&str]) -> Vec<String> {
    let mut on = false;
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            on = headers.contains(&line);
        } else if on && !line.is_empty() && !line.starts_with('#') {
            out.push(line.to_owned());
        }
    }
    out
}

#[test]
fn the_lints_are_the_workspaces() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ours = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    let workspace = std::fs::read_to_string(dir.join("../../Cargo.toml")).unwrap();
    let ours = lints(&ours, &["[lints.rust]", "[lints.clippy]"]);
    let workspace: Vec<String> = lints(
        &workspace,
        &["[workspace.lints.rust]", "[workspace.lints.clippy]"],
    )
    .into_iter()
    .map(|l| l.replace("unsafe_code = \"forbid\"", "unsafe_code = \"deny\""))
    .collect();
    assert_eq!(
        ours, workspace,
        "crates/flowapi-wasm/Cargo.toml: copy the workspace's [workspace.lints] here, with \
         unsafe_code = \"deny\""
    );
}
