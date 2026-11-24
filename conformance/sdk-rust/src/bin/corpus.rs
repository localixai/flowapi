//! The Rust SDK on large descriptions: how long generation takes and how much it prints, for
//! GitHub's, Stripe's and Cloudflare's (`bench/data`, `bench/corpus/fetch.sh`; Stripe's from the
//! reference clone when bench/data has none). Their output is never compiled here: rustc takes
//! more than this machine's 16 GB on GitHub's types. Slack's
//! (about 1 MB of JSON) is written as a scratch crate, `out/corpus/slack`, to build with
//! `cargo build -j 1` under `/usr/bin/time -v`; `--build` builds it so (from clean) and adds
//! the time and the peak memory to the report. Writes `conformance/results/sdk-rust-corpus.md`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

struct Row {
    name: String,
    ms: u128,
    bytes: usize,
    items: usize,
    operations: usize,
    errors: usize,
}

#[expect(
    clippy::disallowed_methods,
    reason = "the time generation takes is what this measures; the output does not depend on it"
)]
fn generate(name: &str, path: &Path) -> Option<(Row, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let started = Instant::now();
    let mut docs = flowapi::Documents::new();
    docs.add("file:///api.json", &text).ok()?;
    let (output, diags) = flowapi::generate(&docs, &flowapi_rust::Rust::new("sdk.rs").with_sdk());
    let ms = started.elapsed().as_millis();
    let out: String = output?.files.into_iter().map(|f| f.text).collect();
    let count = |needle: &str| out.matches(needle).count();
    let row = Row {
        name: name.to_owned(),
        ms,
        bytes: out.len(),
        items: count("\npub struct ") + count("\npub enum ") + count("\npub type "),
        operations: count("_request("),
        errors: diags
            .all()
            .iter()
            .filter(|d| d.severity == flowapi::Severity::Error)
            .count(),
    };
    Some((row, out))
}

fn main() -> std::process::ExitCode {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = here.join("../..");
    let data = repo.join("bench/data");
    let stripe = if data.join("stripe.json").exists() {
        data.join("stripe.json")
    } else {
        repo.join("../.references/codegen/corpus/stripe/openapi/spec3.json")
    };
    let inputs: [(&str, PathBuf); 4] = [
        ("github", data.join("github.json")),
        ("stripe", stripe),
        ("cloudflare", data.join("cloudflare.json")),
        ("slack", data.join("slack.json")),
    ];
    let mut report = String::from(
        "# The Rust SDK on large descriptions\n\nWritten by `cargo run --release -j 1 --bin corpus` \
         in `conformance/sdk-rust`; do not edit. Generation only (types and SDK, \
         `Rust::with_sdk`, one run on this machine); the output of the three largest is not \
         compiled (rustc needs more than 16 GB for GitHub's types).\n\n| Description | Generation (ms) | Bytes | \
         Items | Operations | Errors |\n| --- | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for (name, path) in &inputs {
        let Some((row, out)) = generate(name, path) else {
            eprintln!(
                "{name}: {} is missing or not JSON (bench/corpus/fetch.sh)",
                path.display()
            );
            return std::process::ExitCode::FAILURE;
        };
        let _ = writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} |",
            row.name, row.ms, row.bytes, row.items, row.operations, row.errors
        );
        if *name == "slack" {
            let dir = here.join("out/corpus/slack");
            std::fs::create_dir_all(dir.join("src")).expect("the scratch crate is creatable");
            std::fs::write(
                dir.join("Cargo.toml"),
                "[package]\nname = \"flowapi-sdk-corpus-slack\"\nversion = \"0.0.0\"\nedition = \
                 \"2024\"\n\n[workspace]\n\n[dependencies]\nserde = { version = \"1\", features = \
                 [\"derive\"] }\nserde_json = \"1\"\n\n[profile.dev]\ndebug = false\nincremental = false\n",
            )
            .expect("the manifest is writable");
            std::fs::write(dir.join("src/lib.rs"), &out).expect("lib.rs is writable");
        }
    }
    if std::env::args().any(|a| a == "--build") {
        let dir = here.join("out/corpus/slack");
        let _ = std::fs::remove_dir_all(dir.join("target"));
        let run = std::process::Command::new("/usr/bin/time")
            .current_dir(&dir)
            .args([
                "-v",
                "cargo",
                "build",
                "-j",
                "1",
                "--message-format",
                "short",
            ])
            .output()
            .expect("/usr/bin/time runs");
        let err = String::from_utf8_lossy(&run.stderr);
        let field = |name: &str| {
            err.lines()
                .find_map(|l| l.trim().strip_prefix(name))
                .map_or_else(String::new, |v| v.trim().to_owned())
        };
        let warnings = err
            .lines()
            .filter(|l| l.starts_with("src/") && l.contains("warning"))
            .count();
        let rss: u64 = field("Maximum resident set size (kbytes):")
            .parse()
            .unwrap_or(0);
        let _ = writeln!(
            report,
            "
Slack's output built (`cargo build -j 1`, debug, from clean, serde and serde_json included): {}, {} wall clock, peak resident memory {} MB, {warnings} warnings.",
            if run.status.success() { "built" } else { "FAILED" },
            field("Elapsed (wall clock) time (h:mm:ss or m:ss):"),
            rss / 1024
        );
    }
    let results = repo.join("conformance/results/sdk-rust-corpus.md");
    std::fs::write(&results, &report).expect("the report is writable");
    print!("{report}");
    std::process::ExitCode::SUCCESS
}
