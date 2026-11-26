//! The client printed for each case of `tests/client/` (`flowapi_sdk::client_files`): its
//! `flowapi-sdk.yml` (YAML, read as the command reads it) and `spec.json`, the files printed kept
//! beside them under `src/` (the runtime's, `src/core/`, are the crate's own `src/client/core/`
//! and are not kept again), what was found in `issues.txt`. `FLOWAPI_BLESS=1 cargo test -p
//! flowapi-sdk --test client` writes them anew; a change to one is read in review like code.
//! `tests/client.mjs` type-checks the kept files and runs the runtime against a stub `fetch`.
//!
//! shop: every pagination scheme, a stream of events, a multipart upload with the description's
//! encoding, a download as bytes, two alternatives of security and an operation that needs none.
//! auth: each kind of credential, alternatives and schemes needed together, an operation's own
//! servers, a server variable, JSON lines, reserved words as names, resources three levels deep.
//! hosts (3.2): servers with variables and an `enum`, relative ones, a Path Item's and an
//! operation's own, every idempotent method but TRACE and two that are not, content with DELETE
//! and OPTIONS, an idempotency key. edge (3.1): variables and relative servers, the default retries.
//!
//! `generate`, ignored, writes the files for any description and config: `FLOWAPI_CLIENT_SPEC`,
//! `FLOWAPI_CLIENT_CONFIG` and `FLOWAPI_CLIENT_OUT` name them (`-- --ignored generate`).
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::Documents;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/client")
}

/// The files and the findings of a description's text and a config's YAML.
fn generate(spec_name: &str, spec: &str, config: &str) -> (Vec<flowapi::File>, String) {
    let json: String = if spec_name.ends_with(".json") {
        spec.to_owned()
    } else {
        let text = flowapi_yaml::Text::read(spec_name, Cow::Borrowed(spec)).expect("YAML");
        text.json().to_owned()
    };
    let mut docs = Documents::new();
    docs.add(spec_name, &json).expect("JSON");
    let yaml = flowapi_yaml::Text::read("flowapi-sdk.yml", Cow::Borrowed(config)).expect("YAML");
    let target = flowapi_ts::TypeScript::new("src/api.ts").with_sdk();
    let (program, _) = flowapi::program(&docs, &target);
    let program = program.expect("no error");
    let generated = flowapi_sdk::client_files(&docs, &program, yaml.json());
    let mut found =
        flowapi_sdk::issues_text(&generated.issues, "flowapi-sdk.yml", &|o| yaml.place(o));
    for d in generated.diagnostics.all() {
        let _ = writeln!(found, "{d}");
    }
    (generated.files, found)
}

fn runtime() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/client/core");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("the runtime")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn clients_print_as_kept() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut names: Vec<PathBuf> = std::fs::read_dir(cases())
        .expect("the cases")
        .map(|e| e.expect("an entry").path())
        .filter(|p| p.is_dir())
        .collect();
    names.sort();
    let mut changed = Vec::new();
    for case in names {
        let spec = std::fs::read_to_string(case.join("spec.json")).expect("the description");
        let config = std::fs::read_to_string(case.join("flowapi-sdk.yml")).expect("the config");
        let (files, found) = generate("spec.json", &spec, &config);
        // G4: the same input prints the same bytes.
        assert_eq!(
            files,
            generate("spec.json", &spec, &config).0,
            "{} printed twice differs",
            case.display()
        );
        assert!(!found.contains("error["), "{}: {found}", case.display());
        let core: Vec<String> = files
            .iter()
            .filter_map(|f| f.path.strip_prefix("src/core/").map(str::to_owned))
            .collect();
        assert_eq!(
            core,
            runtime(),
            "every file of src/client/core/ is printed, in order"
        );
        let mut kept = vec![(case.join("issues.txt"), found)];
        for f in files.iter().filter(|f| !f.path.starts_with("src/core/")) {
            kept.push((case.join(&f.path), f.text.clone()));
        }
        for (path, text) in kept {
            if bless {
                std::fs::create_dir_all(path.parent().expect("a folder")).expect("made");
                std::fs::write(&path, &text).expect("written");
            } else if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                changed.push(path.display().to_string());
            }
        }
    }
    assert!(
        changed.is_empty(),
        "printed otherwise than kept: {changed:#?}"
    );
}

/// A config with an error prints nothing.
#[test]
fn an_error_prints_nothing() {
    let spec = std::fs::read_to_string(cases().join("shop/spec.json")).expect("the description");
    let config = "version: 1\nspec: spec.json\npackage: { name: x, client: X }\nresources: {}\n";
    let (files, found) = generate("spec.json", &spec, config);
    assert!(files.is_empty());
    assert!(found.contains("config.coverage"), "{found}");
}

/// A scheme's credential is an option of the client: one named as an option the runtime has, or
/// with no letter or digit to name one, is an error (`config.name.option`).
#[test]
fn a_scheme_named_as_an_option_is_an_error() {
    let spec = r#"{"openapi": "3.1.0", "info": {"title": "t", "version": "1"},
      "paths": {"/a": {"get": {"operationId": "a", "responses": {"204": {"description": "none"}}}}},
      "components": {"securitySchemes": {
        "fetch": {"type": "apiKey", "in": "header", "name": "x-a"},
        "--": {"type": "apiKey", "in": "header", "name": "x-b"},
        "Key": {"type": "apiKey", "in": "header", "name": "x-c"}}}}"#;
    let config = "version: 1\nspec: spec.json\npackage: { name: x, client: X }\n\
        auth: { fetch: {}, \"--\": {}, Key: {} }\nresources: { a: { methods: { get: a } } }\n";
    let (files, found) = generate("spec.json", spec, config);
    assert!(files.is_empty());
    assert_eq!(
        found.matches("error[config.name.option]").count(),
        2,
        "{found}"
    );
    assert!(
        found.contains("`fetch` would be the client's own option"),
        "{found}"
    );
    assert!(found.contains("`--` has no letter or digit"), "{found}");
}

/// The files for any description and config, written where `FLOWAPI_CLIENT_OUT` says.
#[test]
#[ignore = "writes files; run by hand with the three variables"]
fn generate_from_env() {
    let var = |name: &str| std::env::var_os(name).map(PathBuf::from).expect(name);
    let spec_path = var("FLOWAPI_CLIENT_SPEC");
    let spec = std::fs::read_to_string(&spec_path).expect("the description");
    let config = std::fs::read_to_string(var("FLOWAPI_CLIENT_CONFIG")).expect("the config");
    let name = spec_path
        .file_name()
        .expect("a name")
        .to_string_lossy()
        .into_owned();
    let (files, found) = generate(&name, &spec, &config);
    let out = var("FLOWAPI_CLIENT_OUT");
    for f in &files {
        let path = out.join(&f.path);
        std::fs::create_dir_all(path.parent().expect("a folder")).expect("made");
        std::fs::write(path, &f.text).expect("written");
    }
    let errors = found.matches("error[").count();
    println!("{} files, {errors} errors\n{found}", files.len());
}
