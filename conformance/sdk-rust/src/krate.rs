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

// Each module has its own `sdk`, so its own `RequestError`: read by field, in a macro.
macro_rules! refused {
    ($e:expr) => {{
        let e = $e;
        serde_json::json!({ "refused": { "location": e.location, "parameter": e.parameter, "reason": e.reason.code() } }).to_string()
    }};
}

/// An input read from its JSON, where `{"$number": "NaN"}` (or `"Infinity"`, `"-Infinity"`)
/// is the number JSON cannot write: how a case hands the SDK a value it must refuse.
struct V(serde_json::Value);

impl<'de> serde::de::IntoDeserializer<'de, serde_json::Error> for V {
    type Deserializer = V;
    fn into_deserializer(self) -> V {
        self
    }
}

fn marked(m: &serde_json::Map<String, serde_json::Value>) -> Option<f64> {
    if m.len() != 1 {
        return None;
    }
    match m.get("$number")?.as_str()? {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => None,
    }
}

impl<'de> serde::Deserializer<'de> for V {
    type Error = serde_json::Error;
    fn deserialize_any<W: serde::de::Visitor<'de>>(self, w: W) -> Result<W::Value, serde_json::Error> {
        use serde_json::Value;
        match self.0 {
            Value::Object(m) => match marked(&m) {
                Some(x) => w.visit_f64(x),
                None => w.visit_map(serde::de::value::MapDeserializer::new(m.into_iter().map(|(k, v)| (k, V(v))))),
            },
            Value::Array(a) => w.visit_seq(serde::de::value::SeqDeserializer::new(a.into_iter().map(V))),
            other => serde::Deserializer::deserialize_any(other, w),
        }
    }
    fn deserialize_option<W: serde::de::Visitor<'de>>(self, w: W) -> Result<W::Value, serde_json::Error> {
        if self.0.is_null() { w.visit_none() } else { w.visit_some(self) }
    }
    fn deserialize_newtype_struct<W: serde::de::Visitor<'de>>(self, _: &'static str, w: W) -> Result<W::Value, serde_json::Error> {
        w.visit_newtype_struct(self)
    }
    fn deserialize_enum<W: serde::de::Visitor<'de>>(self, name: &'static str, variants: &'static [&'static str], w: W) -> Result<W::Value, serde_json::Error> {
        match self.0 {
            serde_json::Value::Object(m) => {
                let map = serde::de::value::MapDeserializer::new(m.into_iter().map(|(k, v)| (k, V(v))));
                serde::Deserializer::deserialize_enum(serde::de::value::MapAccessDeserializer::new(map), name, variants, w)
            }
            other => serde::Deserializer::deserialize_enum(other, name, variants, w),
        }
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}

fn input<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    T::deserialize(V(serde_json::from_str(text)?))
}

#[derive(serde::Deserialize)]
struct Answer {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

macro_rules! request {
    ($m:ident, $f:ident, $i:expr) => {
        match input($i) {
            Ok(input) => match flowapi_sdk_rust_gen::$m::$f(&input) {
                Ok(r) => show(r.method, &r.path, &r.query, &r.headers, r.body.as_ref()),
                Err(e) => refused!(e),
            },
            Err(e) => broken(e),
        }
    };
}

macro_rules! bare {
    ($m:ident, $f:ident) => {{
        match flowapi_sdk_rust_gen::$m::$f() {
            Ok(r) => show(r.method, &r.path, &r.query, &r.headers, r.body.as_ref()),
            Err(e) => refused!(e),
        }
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
