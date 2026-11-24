//! The Rust target on one large description (GitHub's REST API by default): how long
//! generation takes, and a scratch crate under `out/corpus/<name>` holding the file, for
//! `cargo check -j 1` there. Compiling GitHub's output (about 22,000 serde-derived items) takes
//! more than 16 GB of memory: check mid-size descriptions only.

use std::path::Path;
use std::time::Instant;

const DEFAULT: &str =
    "../../../.references/codegen/corpus/github/descriptions/api.github.com/api.github.com.json";

#[expect(
    clippy::disallowed_methods,
    reason = "the time generation takes is what this measures; the output does not depend on it"
)]
fn main() -> std::process::ExitCode {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input: std::path::PathBuf = std::env::args()
        .nth(1)
        .map_or_else(|| here.join(DEFAULT), Into::into);
    let text = std::fs::read_to_string(&input).expect("the description is readable");
    let started = Instant::now();
    let mut docs = flowapi::Documents::new();
    docs.add("file:///api.json", &text)
        .expect("the description is JSON");
    let (output, diags) = flowapi::generate(&docs, &flowapi_rust::Rust::new("types.rs"));
    let took = started.elapsed();
    let errors = diags
        .all()
        .iter()
        .filter(|d| d.severity == flowapi::Severity::Error)
        .count();
    let Some(output) = output else {
        eprintln!("{errors} errors, no output");
        return std::process::ExitCode::FAILURE;
    };
    let types = output
        .files
        .into_iter()
        .next()
        .map(|f| f.text)
        .unwrap_or_default();
    let stem = input.file_stem().map_or_else(
        || String::from("corpus"),
        |s| s.to_string_lossy().into_owned(),
    );
    let dir = here.join("out/corpus").join(stem);
    std::fs::create_dir_all(dir.join("src")).expect("the scratch crate is creatable");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"flowapi-corpus-rust\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [workspace]\n\n[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n\
         serde_json = \"1\"\n",
    )
    .expect("the manifest is writable");
    std::fs::write(dir.join("src/lib.rs"), &types).expect("lib.rs is writable");
    println!(
        "generated in {} ms: {} bytes, {} lines, {} items, {} diagnostics ({errors} errors)",
        took.as_millis(),
        types.len(),
        types.lines().count(),
        types.matches("\npub struct ").count()
            + types.matches("\npub enum ").count()
            + types.matches("\npub type ").count(),
        diags.all().len(),
    );
    std::process::ExitCode::SUCCESS
}
