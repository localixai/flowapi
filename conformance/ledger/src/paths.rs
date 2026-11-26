//! Where the ledger's inputs and outputs are, from this crate's own place in the repository.

use std::path::PathBuf;

/// `conformance/`.
pub fn conformance() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The repository's root.
pub fn root() -> PathBuf {
    conformance().join("..")
}

/// `conformance/specs/`: the pinned texts.
pub fn specs() -> PathBuf {
    conformance().join("specs")
}

/// `conformance/requirements.json`.
pub fn requirements() -> PathBuf {
    conformance().join("requirements.json")
}

/// `conformance/scope.json`.
pub fn scope() -> PathBuf {
    conformance().join("scope.json")
}

/// `conformance/REPORT.md`.
pub fn report() -> PathBuf {
    conformance().join("REPORT.md")
}

/// `conformance/MISSING.md`: the requirements in scope that no marker names yet. Beside
/// `REPORT.md`, not under `coverage/`: that folder holds the markers' Markdown, read by the scan.
pub fn missing() -> PathBuf {
    conformance().join("MISSING.md")
}
