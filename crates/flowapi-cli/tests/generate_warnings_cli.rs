//! `flowapi generate`'s warnings about the description, as the command prints them: each once.
//! A Swagger 2.0 form's `collectionFormat` is read by the core alone (`Media::encoding`), and the
//! SDK layer takes the form's encoding from it, so a format 2.0 does not have is reported once,
//! naming the field a `formData` parameter; an Items Object's, once too.

use std::path::Path;
use std::process::Command;

/// A form with an array field of an unknown format and one whose inner array says `multi`.
const SPEC: &str = r#"{
 "swagger": "2.0",
 "info": { "title": "forms", "version": "1" },
 "host": "forms.example",
 "paths": { "/tags": { "post": {
  "operationId": "postTags",
  "consumes": ["application/x-www-form-urlencoded"],
  "parameters": [
   { "name": "odd", "in": "formData", "type": "array", "items": { "type": "string" },
     "collectionFormat": "commas" },
   { "name": "grid", "in": "formData", "type": "array",
     "items": { "type": "array", "collectionFormat": "multi", "items": { "type": "string" } } }
  ],
  "responses": { "204": { "description": "done" } }
 } } }
}"#;

const CONFIG: &str = "version: 1
spec: spec.json
package: { name: forms-sdk, client: Forms, license: MIT }
resources:
  tags:
    methods:
      create: postTags
";

#[test]
fn a_form_fields_bad_collection_format_is_warned_of_once() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("generate_warnings");
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    std::fs::write(dir.join("spec.json"), SPEC).expect("written");
    std::fs::write(dir.join("flowapi-sdk.yml"), CONFIG).expect("written");
    let out = Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .args(["generate", "flowapi-sdk.yml", "--out", "sdk"])
        .current_dir(&dir)
        .output()
        .expect("the command runs");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(
        stderr.matches("`collectionFormat` commas").count(),
        1,
        "{stderr}"
    );
    let named = "warning[swagger.collectionFormat]: `collectionFormat` commas is not one a \
                 parameter in `formData` takes: written as `csv`\n  at spec.json:10:";
    assert!(stderr.contains(named), "{stderr}");
    assert_eq!(
        stderr.matches("`collectionFormat` multi").count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.contains("`collectionFormat` multi is not one an Items Object takes"),
        "{stderr}"
    );
}
