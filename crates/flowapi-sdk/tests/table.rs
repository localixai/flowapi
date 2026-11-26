//! The table printed for each description in `tests/cases/` (`flowapi_sdk::table`, after the
//! TypeScript target's SDK for the same program), against the `.ts` kept beside it, this layer's
//! diagnostics after it as comments. `FLOWAPI_BLESS=1 cargo test -p flowapi-sdk --test table`
//! writes them anew; a change to one is read in review like code. `tests/tsc.mjs` type-checks
//! each kept table after the `--sdk` output it is appended to, and imports it under Node.
//!
//! servers.json: servers with variables (`enum`, `description`), a relative one, and each one
//! left out (a variable not defined, no `url`, no `default`) or kept with a warning (a `default`
//! outside `enum`, an empty `enum`, a query in the URL); a Path Item's servers, an operation's
//! over them, and empty lists that override nothing. defaults.json: 3.0 with no servers (`/`)
//! and no `security`, a cookie key, `mutualTLS` which 3.0 does not define, and how each body is
//! read (`text`, `json`, a range by what is sent). security.json: every scheme 3.1 has, one
//! behind a `$ref`, each one left out, the description's alternatives, `security: []`, scopes,
//! an empty requirement, a name no scheme has, and malformed lists. swagger20.json: `host`,
//! `basePath` and `schemes` lifted, an operation's own `schemes`, `securityDefinitions` with
//! 2.0's flows named as 3.x's, types 2.0 does not have. swagger-host.json: a host and no
//! `schemes` (the description's own scheme). swagger-nohost.json: no host (a relative URL, an
//! operation's `schemes` changing nothing) and a `basePath` without `/`. v32.json: a server's
//! `name`, a deprecated scheme, a Path Item behind `$ref` with servers and an
//! `additionalOperations` method, a requirement named by URI, the `query` method. encoding.json
//! (3.1): `multipart/form-data`'s and a form's Encoding Objects by name (a part's type with
//! parameters, a list of types kept, styles, a type inferred from `contentEncoding` through
//! `$ref` and `allOf`, never `oneOf`), one behind a request body's `$ref`, responses' and JSON's
//! encoding not read, a multipart body with none. encoding-v30.json: 3.0's fields (no style for
//! `multipart/form-data`, `format: binary` and `byte`). encoding-v32.json: `prefixEncoding`,
//! `itemEncoding`, nesting and its depth bound, `Content-Transfer-Encoding` from
//! `contentEncoding`. swagger-forms.json: 2.0's `formData` `collectionFormat` as styles.
//! A case's `x-flowapi-calls` are requests `tests/tsc.mjs` writes through its table.
//!
//! Each case names the requirements it covers in `x-flowapi-covers` (the ledger reads them).
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use std::fmt::Write;
use std::path::Path;

fn table(uri: &str, text: &str) -> String {
    let mut docs = Documents::new();
    docs.add(uri, text).expect("JSON");
    let target = flowapi_ts::TypeScript::new("out.ts").with_sdk();
    let (program, _) = flowapi::program(&docs, &target);
    let program = program.expect("no error");
    let (mut text, diags) = flowapi_sdk::table(&docs, &program);
    for d in diags.all() {
        let _ = writeln!(text, "// {d}");
    }
    text
}

#[test]
fn tables_print_as_kept() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
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
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let printed = table(&name, &text);
        // G4: the same input prints the same bytes.
        assert_eq!(printed, table(&name, &text), "{name} printed twice differs");
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

/// A description with no operation has no table, as it has no SDK.
#[test]
fn no_operations_no_table() {
    let text =
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1"},"servers":[{"url":"https://x"}]}"#;
    assert_eq!(table("none.json", text), "");
}

/// A JSON Schema document is no description: nothing is read.
#[test]
fn a_schema_reads_nothing() {
    let mut docs = Documents::new();
    docs.add("s.json", r#"{"type":"string"}"#).expect("JSON");
    let target = flowapi_ts::TypeScript::new("out.ts").with_sdk();
    let (program, _) = flowapi::program(&docs, &target);
    let (service, diags) = flowapi_sdk::read(&docs, &program.expect("no error"));
    assert_eq!(service, flowapi_sdk::Service::default());
    assert!(diags.all().is_empty());
}
