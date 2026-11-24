//! The generated crate: a module per version holding its SDK, as a library (so nothing in it
//! is dead code and every warning is the SDK's), and a binary that runs one call per line of a
//! list and prints what it gave as JSON. It is built once, `cargo build -j 1`.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

const MANIFEST: &str = r#"# Written by conformance/sdk-rust: the Rust SDK of every version's description.
[package]
name = "flowapi-sdk-rust-gen"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[lib]
path = "src/lib.rs"

[[bin]]
name = "calls"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
# Objects of `serde_json::Value` kept in the order written, as a JavaScript object is.
serde_json = { version = "1", features = ["preserve_order"] }

[profile.dev]
debug = false
incremental = false
"#;

const RUNNER: &str = r#"
use std::io::{BufRead, Write};

fn show(method: &str, path: &str, query: &str, headers: &[(String, String)], body: Option<&(String, Vec<u8>)>) -> String {
    let body = body.map(|b| (b.0.clone(), String::from_utf8_lossy(&b.1).into_owned()));
    serde_json::json!({ "method": method, "path": path, "query": query, "headers": headers, "body": body }).to_string()
}

fn answer(response: Option<&str>, media: Option<&str>, payload: serde_json::Value, error: Option<String>) -> String {
    serde_json::json!({ "response": response, "mediaType": media, "payload": payload, "error": error }).to_string()
}

fn broken(e: serde_json::Error) -> String {
    serde_json::json!({ "input": e.to_string() }).to_string()
}

#[derive(serde::Deserialize)]
struct Answer {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

macro_rules! request {
    ($m:ident, $f:ident, $i:expr) => {
        match serde_json::from_str($i) {
            Ok(input) => {
                let r = flowapi_sdk_rust_gen::$m::$f(&input);
                show(r.method, &r.path, &r.query, &r.headers, r.body.as_ref())
            }
            Err(e) => broken(e),
        }
    };
}

macro_rules! bare {
    ($m:ident, $f:ident) => {{
        let r = flowapi_sdk_rust_gen::$m::$f();
        show(r.method, &r.path, &r.query, &r.headers, r.body.as_ref())
    }};
}

macro_rules! response {
    ($m:ident, $f:ident, $i:expr) => {
        match serde_json::from_str::<Answer>($i) {
            Ok(a) => match flowapi_sdk_rust_gen::$m::$f(a.status, &a.headers, a.body.as_bytes()) {
                Ok(r) => {
                    use flowapi_sdk_rust_gen::$m::sdk::Response as _;
                    answer(Some(r.response()), r.media_type(), serde_json::to_value(&r).unwrap_or_default(), None)
                }
                Err(e) => answer(None, e.content_type.as_deref(), serde_json::Value::String(String::from_utf8_lossy(&e.body).into_owned()), Some(e.to_string())),
            },
            Err(e) => broken(e),
        }
    };
}

fn main() {
    let path = std::env::args().nth(1).expect("the calls' file");
    let file = std::fs::File::open(path).expect("the calls' file is readable");
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in std::io::BufReader::new(file).lines() {
        let line = line.expect("a line");
        let Some((n, data)) = line.split_once('\t') else { continue };
        let at: usize = n.parse().expect("a call number");
        writeln!(out, "{n}\t{}", CALLS[at](data)).expect("stdout");
    }
}
"#;

/// One call's entry of the binary's table: the module, the function's name before
/// `_request` or `_response`, and which it is.
pub(crate) struct Entry {
    pub(crate) module: String,
    pub(crate) base: String,
    pub(crate) bare: bool,
    pub(crate) response: bool,
}

/// Writes the crate: `modules` (each a name and its text) and the binary's table of calls.
pub(crate) fn write(dir: &Path, modules: &[(String, String)], entries: &[Entry]) {
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("the sources' directory is creatable");
    std::fs::write(dir.join("Cargo.toml"), MANIFEST).expect("the manifest is writable");
    let mut lib = String::from("//! Written by conformance/sdk-rust.\n\n");
    for (name, text) in modules {
        std::fs::write(src.join(format!("{name}.rs")), text).expect("a module is writable");
        // A kept SDK names its types as its description does (`getPet`): the types' case.
        if name.starts_with("kept_") {
            lib.push_str("#[allow(non_camel_case_types)]\n");
        }
        let _ = writeln!(lib, "pub mod {name};");
    }
    std::fs::write(src.join("lib.rs"), lib).expect("lib.rs is writable");
    let mut main = String::from("//! Written by conformance/sdk-rust.\n");
    main.push_str(RUNNER);
    main.push_str("\nconst CALLS: &[fn(&str) -> String] = &[\n");
    for e in entries {
        let (m, b) = (&e.module, &e.base);
        let _ = if e.response {
            writeln!(main, "    |i| response!({m}, {b}_response, i),")
        } else if e.bare {
            writeln!(main, "    |_| bare!({m}, {b}_request),")
        } else {
            writeln!(main, "    |i| request!({m}, {b}_request, i),")
        };
    }
    main.push_str("];\n");
    std::fs::write(src.join("main.rs"), main).expect("main.rs is writable");
}

/// Builds the crate; its compiler's messages about the SDK's modules (warnings included), or
/// the whole error output when it does not build.
pub(crate) fn build(dir: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("cargo")
        .current_dir(dir)
        .args(["build", "-j", "1", "--message-format", "short"])
        .output()
        .expect("cargo runs");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(stderr);
    }
    Ok(stderr
        .lines()
        .filter(|l| l.starts_with("src/") && l.contains("warning"))
        .map(str::to_owned)
        .collect())
}

/// Runs the binary over the calls' file; its output lines.
pub(crate) fn run(dir: &Path, calls: &Path) -> String {
    let output = Command::new(dir.join("target/debug/calls"))
        .arg(calls)
        .output()
        .expect("the generated binary runs");
    String::from_utf8_lossy(&output.stdout).into_owned()
}
