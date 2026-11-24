//! The `flowapi` command on OpenAPI descriptions in each version: a response's headers, what is
//! deprecated, and binary bodies, each read from the TypeScript it prints; `--sdk` with each
//! target.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::path::Path;
use std::process::Command;

const VERSIONS: [&str; 3] = ["3.0.4", "3.1.2", "3.2.0"];

/// The TypeScript the command prints for one description, written to a fresh directory.
fn generate(test: &str, text: &str) -> String {
    generate_with(test, text, &[])
}

/// What the command prints for one description with `flags` after it.
fn generate_with(test: &str, text: &str, flags: &[&str]) -> String {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("openapi_cli")
        .join(test);
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let path = dir.join("api.json");
    std::fs::write(&path, text).expect("written");
    let out = Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .arg(&path)
        .args(flags)
        .output()
        .expect("the command runs");
    assert!(
        out.status.success(),
        "flowapi failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("UTF-8")
}

fn headers(version: &str) -> String {
    format!(
        r##"{{"openapi":"{version}","info":{{"title":"t","version":"1"}},"paths":{{"/a":{{"get":{{"operationId":"g",
 "responses":{{"200":{{"description":"ok",
   "headers":{{"X-Ref":{{"$ref":"#/components/headers/Rate"}},
     "X-Content":{{"content":{{"text/plain":{{"schema":{{"type":"integer"}}}}}}}},
     "X-Opt":{{"schema":{{"type":"boolean"}},"x-note":{{"any":"extension"}}}},
     "content-type":{{"required":true,"schema":{{"type":"string"}}}},
     "X-Req":{{"required":true,"x-a":1,"schema":{{"enum":["a","b"]}}}}}}}}}}}}}}}},
 "components":{{"headers":{{"Rate":{{"required":true,"schema":{{"type":"integer"}}}}}}}}}}"##
    )
}

// req: oas-3.0.components-object.headers — X-Ref: a header given by reference to components/headers is its schema's type, required as there
// req: oas-3.1.components-object.headers — the same, 3.1
// req: oas-3.2.components-object.headers — the same, 3.2
// req: oas-3.0.header-object.schema — X-Opt, X-Req: a header's type is its schema's
// req: oas-3.1.header-object.schema — the same, 3.1
// req: oas-3.2.header-object.schema — the same, 3.2
// req: oas-3.0.header-object.content — X-Content: a header given by content is its one media type's schema
// req: oas-3.1.header-object.content — the same, 3.1
// req: oas-3.2.header-object.content — the same, 3.2
// req: oas-3.0.header-object.required — X-Req required, X-Opt optional (the default false)
// req: oas-3.1.header-object.required — the same, 3.1
// req: oas-3.2.header-object.required — the same, 3.2
// req: oas-3.0.header-object.common-fixed-fields.may-object-extended-specification-extensions — X-Opt, X-Req: x- fields beside a header change nothing
// req: oas-3.1.header-object.common-fixed-fields.may-object-extended-specification-extensions — the same, 3.1
// req: oas-3.2.header-object.common-fixed-fields.may-object-extended-specification-extensions — the same, 3.2
// req: oas-3.0.response-object.headers.shall-response-header-defined-name-content-type — a required `content-type` header (any case) is left out
// req: oas-3.1.response-object.headers.shall-response-header-defined-name-content-type — the same, 3.1
// req: oas-3.2.response-object.headers.shall-response-header-defined-name-content-type — the same, 3.2
#[test]
fn a_response_has_its_headers() {
    for v in VERSIONS {
        let ts = generate(&format!("headers-{v}"), &headers(v));
        let want = r#"      headers: {
        "X-Ref": number;
        "X-Content"?: number;
        "X-Opt"?: boolean;
        "X-Req": "a" | "b";
      };"#;
        assert!(ts.contains(want), "{v}:\n{ts}");
        assert!(!ts.to_lowercase().contains("content-type"), "{v}:\n{ts}");
    }
}

// req: oas-3.0.operation-object.deprecated.should-consumers-refrain-usage-declared-operation — the operation's type and its member of Paths carry @deprecated, which editors strike through
// req: oas-3.1.operation-object.deprecated.should-consumers-refrain-usage-declared-operation — the same, 3.1
// req: oas-3.2.operation-object.deprecated.should-consumers-refrain-usage-declared-operation — the same, 3.2
// req: oas-3.0.parameter-object.deprecated.should-specifies-parameter-deprecated-transitioned-out-usage — the parameter's member carries @deprecated
// req: oas-3.1.parameter-object.deprecated.should-specifies-parameter-deprecated-transitioned-out-usage — the same, 3.1
// req: oas-3.2.parameter-object.deprecated.should-specifies-parameter-deprecated-transitioned-out-usage — the same, 3.2
// req: oas-3.0.header-object.deprecated.should-specifies-header-deprecated-transitioned-out-usage — the header's member carries @deprecated
// req: oas-3.1.header-object.deprecated.should-specifies-header-deprecated-transitioned-out-usage — the same, 3.1
// req: oas-3.2.header-object.deprecated.should-specifies-header-deprecated-transitioned-out-usage — the same, 3.2
#[test]
fn what_is_deprecated_says_so() {
    for v in VERSIONS {
        let text = format!(
            r#"{{"openapi":"{v}","info":{{"title":"t","version":"1"}},"paths":{{"/a":{{"get":{{"operationId":"old","deprecated":true,
 "parameters":[{{"name":"q","in":"query","deprecated":true,"schema":{{"type":"string"}}}},{{"name":"r","in":"query","schema":{{"type":"string"}}}}],
 "responses":{{"200":{{"description":"ok","headers":{{"X-Old":{{"deprecated":true,"schema":{{"type":"string"}}}},"X-New":{{"schema":{{"type":"string"}}}}}}}}}}}}}}}}}}"#
        );
        let ts = generate(&format!("deprecated-{v}"), &text);
        for want in [
            "/**\n * @deprecated\n */\nexport type Old = {",
            "      /** @deprecated */\n      q?: string;\n      r?: string;",
            "        /** @deprecated */\n        \"X-Old\"?: string;\n        \"X-New\"?: string;",
            "    /** @deprecated */\n    get: Old;",
        ] {
            assert!(ts.contains(want), "{v}: {want}\n{ts}");
        }
    }
}

// req: oas-3.1.working-with-binary-data.shall-however-contentmediatype-contradicts-relevant-media-type — application/octet-stream with contentMediaType text/plain: the keyword narrows nothing, the body is unknown
// req: oas-3.2.schema-object.working-with-binary-data.shall-however-contentmediatype-contradicts-relevant-media-type — the same, 3.2
// req: oas-3.2.schema-object.schema-evaluation-and-binary-data.must-oas-implementations-do-not-have-access — raw binary (no type) is unknown, base64 in JSON is string, and a multipart part is read apart: file unknown, name string
#[test]
fn binary_data_is_never_narrowed() {
    for v in ["3.1.2", "3.2.0"] {
        let text = format!(
            r#"{{"openapi":"{v}","info":{{"title":"t","version":"1"}},"paths":{{"/a":{{"post":{{"operationId":"up",
 "requestBody":{{"required":true,"content":{{
   "image/png":{{"schema":{{"contentMediaType":"image/png"}}}},
   "application/octet-stream":{{"schema":{{"contentMediaType":"text/plain"}}}},
   "application/json":{{"schema":{{"type":"object","required":["png"],"properties":{{
     "png":{{"type":"string","contentMediaType":"image/png","contentEncoding":"base64"}},
     "doc":{{"type":"string","contentMediaType":"application/json","contentSchema":{{"type":"integer"}}}}}}}}}},
   "multipart/form-data":{{"schema":{{"type":"object","properties":{{"file":{{"contentMediaType":"image/png"}},"name":{{"type":"string"}}}}}}}}}}}},
 "responses":{{"200":{{"description":"ok","content":{{"application/pdf":{{}}}}}}}}}}}}}}}}"#
        );
        let ts = generate(&format!("binary-{v}"), &text);
        for want in [
            "\"image/png\": unknown;",
            "\"application/octet-stream\": unknown;",
            "png: string;\n      doc?: string;",
            "file?: unknown;\n      name?: string;",
            "\"application/pdf\": unknown;",
        ] {
            assert!(ts.contains(want), "{v}: {want}\n{ts}");
        }
    }
    // 3.0 says binary with type string and format binary or byte: the type is string.
    let text = r#"{"openapi":"3.0.4","info":{"title":"t","version":"1"},"paths":{"/a":{"post":{"operationId":"up",
 "requestBody":{"content":{"application/octet-stream":{"schema":{"type":"string","format":"binary"}},
  "application/json":{"schema":{"type":"string","format":"byte"}}}},
 "responses":{"200":{"description":"ok"}}}}}}"#;
    let ts = generate("binary-3.0", text);
    assert!(
        ts.contains("\"application/octet-stream\": string;\n    \"application/json\": string;"),
        "{ts}"
    );
}

/// `--sdk` with each target: the TypeScript functions, or the Rust ones and their runtime.
#[test]
fn the_sdk_with_each_target() {
    let text = r#"{"openapi":"3.1.0","info":{"title":"t","version":"1"},"paths":{"/pets/{id}":{"get":{"operationId":"getPet","parameters":[{"name":"id","in":"path","required":true,"schema":{"type":"integer"}}],"responses":{"200":{"description":"ok"}}}}}}"#;
    let ts = generate_with("sdk_ts", text, &["--sdk"]);
    assert!(ts.contains("export function getPetRequest("));
    let rust = generate_with("sdk_rust", text, &["--target", "rust", "--sdk"]);
    assert!(rust.contains("\npub mod sdk {\n"));
    assert!(rust.contains("pub fn get_pet_request(input: &"));
    assert!(rust.contains("pub fn get_pet_response(status: u16,"));
    let plain = generate_with("plain_rust", text, &["--target", "rust"]);
    assert!(!plain.contains("pub mod sdk"));
}
