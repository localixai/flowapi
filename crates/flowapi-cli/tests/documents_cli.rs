//! The `flowapi` command on descriptions spread over several files: the files a `$ref` reaches
//! by a relative path, meta-schemas and their `$vocabulary`, and the base every document is read
//! against (the path it was given by).
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fresh directory holding the files given, by their relative paths.
fn files(test: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("documents_cli")
        .join(test);
    drop(std::fs::remove_dir_all(&dir));
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(&path, text).expect("written");
    }
    dir
}

/// The command run in `dir` on the description at `input`, a path relative to it.
fn flowapi(dir: &Path, input: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .current_dir(dir)
        .arg(input)
        .output()
        .expect("the command runs")
}

fn stdout(out: &Output) -> String {
    assert!(
        out.status.success(),
        "flowapi failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).expect("UTF-8")
}

fn stderr(out: &Output) -> String {
    assert!(!out.status.success(), "flowapi succeeded");
    String::from_utf8(out.stderr.clone()).expect("UTF-8")
}

// req: oas-3.2.openapi-object.openapi-description-structure.may-oad-made-up-single-document-distributed — three files, two directories deep, each reached by a relative $ref from the one before
// req: oas-3.2.openapi-object.parsing-documents.must-not-implementations-treat-reference-unresolvable-before-completely — every file is read before the core resolves a reference; only a file that is not there is an error
#[test]
fn a_description_reads_every_file_its_references_reach() {
    let api = r#"{"openapi":"3.2.0","info":{"title":"t","version":"1"},"paths":{"/pets":{"get":{
      "operationId":"list","responses":{"200":{"description":"ok","content":{"application/json":{
      "schema":{"$ref":"schemas/pet.json"}}}}}}}}}"#;
    let pet = r#"{"type":"object","required":["owner"],"properties":{"owner":{"$ref":"../common/owner.json"}}}"#;
    let owner = r#"{"type":"object","properties":{"name":{"type":"string"}}}"#;
    let dir = files(
        "distributed",
        &[
            ("api.json", api),
            ("schemas/pet.json", pet),
            ("common/owner.json", owner),
        ],
    );
    let ts = stdout(&flowapi(&dir, "api.json"));
    assert!(ts.contains("owner: {\n"), "{ts}");
    assert!(ts.contains("name?: string;"), "{ts}");
    std::fs::remove_file(dir.join("common/owner.json")).expect("removed");
    let err = stderr(&flowapi(&dir, "api.json"));
    assert!(err.contains("owner.json"), "{err}");
}

// req: jsonschema-2020-12.core.initial-base.recommended-implementations-document-any-default-base-uri — flowapi assumes none: a document's base is the URI the host gives it (Documents::add, documented on Document::uri); the command's is the path as typed, which a diagnostic names and a relative $ref resolves against
#[test]
fn a_document_is_read_against_the_path_it_was_given_by() {
    let dir = files(
        "base",
        &[
            ("dir/s.json", r#"{"$ref":"t.json"}"#),
            ("dir/t.json", r#"{"type":"string"}"#),
            ("dir/bad.json", r##"{"$ref":"#/$defs/none"}"##),
        ],
    );
    assert!(stdout(&flowapi(&dir, "dir/s.json")).contains("export type Root = string;"));
    let err = stderr(&flowapi(&dir, "./dir/bad.json"));
    assert!(err.contains("at ./dir/bad.json:1:"), "{err}");
}

const CORE: &str = r#""https://json-schema.org/draft/2020-12/vocab/core":true"#;
const APPLICATOR: &str = r#""https://json-schema.org/draft/2020-12/vocab/applicator":true"#;
const VALIDATION: &str = r#""https://json-schema.org/draft/2020-12/vocab/validation":true"#;

/// The command on `schema` beside `meta.json`, a meta-schema with the vocabularies given.
fn with_meta(test: &str, vocabularies: &[&str], schema: &str) -> Output {
    let meta = format!(r#"{{"$vocabulary":{{{}}}}}"#, vocabularies.join(","));
    let dir = files(test, &[("meta.json", &meta), ("s.json", schema)]);
    flowapi(&dir, "s.json")
}

// req: jsonschema-2020-12.core.$vocabulary — the meta-schema $schema names is read for its $vocabulary
// req: jsonschema-2020-12.core.$vocabulary.must-used-indicate-whether-each-vocabulary-required — an unknown vocabulary marked true is refused, one marked false is passed over
// req: jsonschema-2020-12.core.$vocabulary.must-value-true-implementations-do-not-recognize — the schema declaring the meta-schema is refused with the $vocabulary code
// req: jsonschema-2020-12.core.detecting-a-meta-schema.must-implementations-recognize-schema-meta-schema-being — meta.json is read as a meta-schema because s.json names it in $schema
#[test]
fn an_unknown_vocabulary_is_refused_only_when_required() {
    let schema = r#"{"$schema":"meta.json","type":"string"}"#;
    let unknown = r#""https://unknown.example/v":true"#;
    let err = stderr(&with_meta("required", &[CORE, unknown], schema));
    assert!(
        err.contains("error[jsonschema-2020-12.core.$vocabulary]")
            && err.contains("https://unknown.example/v"),
        "{err}"
    );
    let optional = r#""https://unknown.example/v":false"#;
    let ts = stdout(&with_meta(
        "optional",
        &[CORE, APPLICATOR, VALIDATION, optional],
        schema,
    ));
    assert!(ts.contains("export type Root = string;"), "{ts}");
}

// lossy: jsonschema-2020-12.core.$vocabulary.must-any-vocabulary-understood-implementation-processed-manner — without the validation vocabulary `type` and `required` are annotations and the applicator still applies (tested); without the applicator vocabulary the whole schema is unknown, recorded lossy on $vocabulary, wider than validation alone would give
#[test]
fn only_the_vocabularies_in_use_give_types() {
    let schema = r#"{"$schema":"meta.json","type":"object","required":["a"],
      "properties":{"a":{"type":"string"}}}"#;
    let ts = stdout(&with_meta("no-validation", &[CORE, APPLICATOR], schema));
    assert!(
        ts.contains("a?: unknown;")
            && ts.contains("| unknown[] | string | number | boolean | null"),
        "{ts}"
    );
    let ts = stdout(&with_meta("no-applicator", &[CORE, VALIDATION], schema));
    assert!(
        ts.contains("Not checked by this type: $vocabulary.")
            && ts.contains("export type Root = unknown;"),
        "{ts}"
    );
}

/// A `$vocabulary` without the core vocabulary is an error. The requirement's other case, core
/// marked false, is not refused yet (triage-2.md §6, T1), so this test carries no marker.
#[test]
fn a_meta_schema_without_core_is_refused() {
    let schema = r#"{"$schema":"meta.json","type":"string"}"#;
    let err = stderr(&with_meta("no-core", &[APPLICATOR, VALIDATION], schema));
    assert!(
        err.contains("error[jsonschema-2020-12.core.$vocabulary]"),
        "{err}"
    );
}
