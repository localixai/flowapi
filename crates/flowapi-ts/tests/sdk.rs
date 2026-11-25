//! The SDK printed for each description in `tests/sdk/` (`TypeScript::with_sdk`), against the
//! `.ts` kept beside it, diagnostics after it as comments. `FLOWAPI_BLESS=1 cargo test -p
//! flowapi-ts --test sdk` writes them anew; a change to one is read in review like code. What
//! the SDK does when it runs is oracle O8 (`conformance/sdk/run.mjs`): the spec's Style Examples
//! and the cases of `conformance/sdk/cases.json`, which name the requirements they cover.
//!
//! styles.json: every style in its location, a name percent-encoded where it goes into a URI
//! (`q[]`), `allowReserved`, a parameter with `content`, 3.2's `querystring`, `style: cookie`
//! and a form cookie in one header, a style a header does not take (a warning, then `simple`),
//! and `additionalOperations` sent with the method as written. bodies.json: each request media
//! type (JSON, a form, multipart, a range), an optional `+json` body, responses by code,
//! range, `default` and a range of media types, and an operation with no responses.
//! binary.json: responses neither JSON nor text (`octet-stream`, `image/png`, `application/xml`)
//! read as a `Uint8Array`, `text/*` with a parameter as a `string`, and ranges as what each allows.
//! multipart.json: a `multipart/form-data` body as its parts (`$Form`: a string member may be a
//! file, an array's items one part each) with its `encoding` given, 3.2's array of objects,
//! another multipart type and a binary type or a range given as bytes.
//! swagger20.json: `collectionFormat` as 3.x styles, `tsv` warned about. reserved.json: schemas
//! named as the global types the output names (`Record`, `Object`, and the SDK's `NonNullable`,
//! `ReturnType`, `RegExpExecArray`, `Uint8Array`, `ArrayBuffer`, `Promise`, and its exports
//! `FlowapiRequestError`, `FlowapiFile`, `FlowapiEncoding`) are declared under other names, with a
//! warning; `Request` and `JSON` keep theirs (the SDK names no such type; `JSON` is a value there).
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use std::fmt::Write;
use std::path::Path;

fn generate(uri: &str, text: &str) -> String {
    let mut docs = Documents::new();
    docs.add(uri, text).expect("JSON");
    let target = flowapi_ts::TypeScript::new("out.ts").with_sdk();
    let (output, diags) = flowapi::generate(&docs, &target);
    let mut text = output
        .map(|o| o.files.into_iter().map(|f| f.text).collect::<String>())
        .unwrap_or_default();
    for d in diags.all() {
        let _ = writeln!(text, "// {d}");
    }
    text
}

#[test]
fn sdks_print_as_kept() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sdk");
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    let mut changed = Vec::new();
    for case in names {
        let text = std::fs::read_to_string(&case).unwrap();
        let printed = generate(&case.file_name().unwrap().to_string_lossy(), &text);
        let kept = case.with_extension("ts");
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
/// none of the global types the SDK reserves.
#[test]
fn the_sdk_only_adds() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sdk/bodies.json"),
    )
    .unwrap();
    let mut docs = Documents::new();
    docs.add("bodies.json", &text).expect("JSON");
    let plain = flowapi::generate(&docs, &flowapi_ts::TypeScript::new("out.ts"))
        .0
        .expect("output");
    let with = generate("bodies.json", &text);
    let types = &plain.files[0].text;
    assert!(with.starts_with(types.as_str()));
    assert!(with.len() > types.len());
}

/// G4: the same input prints the same bytes, every time.
#[test]
fn sdk_is_deterministic() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sdk/styles.json"),
    )
    .unwrap();
    let first = generate("styles.json", &text);
    for _ in 0..5 {
        assert_eq!(generate("styles.json", &text), first);
    }
}
