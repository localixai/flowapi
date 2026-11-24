//! The answers the module gives, called natively: the same functions its exports call.

use flowapi_wasm::{generate, references};

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn request(uri: &str, text: &str) -> String {
    format!(
        "{{\"documents\":[{{\"uri\":{},\"text\":{}}}],\"options\":{{\"output\":\"out.ts\"}}}}",
        quoted(uri),
        quoted(text)
    )
}

#[test]
fn a_schema_is_printed() {
    let schema = r#"{"title":"Pet","type":"object","properties":{"id":{"type":"integer"}}}"#;
    let answer = generate(request("pet.json", schema).as_bytes());
    assert!(
        answer.starts_with("{\"files\":[{\"path\":\"out.ts\",\"text\":\""),
        "{answer}"
    );
    assert!(answer.ends_with(",\"diagnostics\":[]}"), "{answer}");
}

#[test]
fn a_document_that_is_not_json_has_its_line_and_column() {
    let answer = generate(request("bad.json", "{\n  \"a\": tru\n}").as_bytes());
    assert!(
        answer.starts_with(
            "{\"files\":[],\"diagnostics\":[{\"severity\":\"error\",\"code\":\"json.syntax\""
        ),
        "{answer}"
    );
    assert!(answer.contains("\"uri\":\"bad.json\""), "{answer}");
    assert!(answer.contains("\"line\":2,\"column\":8}"), "{answer}");
}

#[test]
fn a_request_that_cannot_be_read_is_one_error() {
    for bad in [
        "",
        "{",
        "[]",
        "{\"documents\":[]}",
        "{\"documents\":[{\"uri\":\"a\"}]}",
    ] {
        let answer = generate(bad.as_bytes());
        assert!(
            answer.contains("\"code\":\"flowapi.request\""),
            "{bad}: {answer}"
        );
    }
    let answer = generate(&[0xff, 0xfe]);
    assert!(answer.contains("not UTF-8"), "{answer}");
}

#[test]
fn control_characters_are_escaped() {
    let schema = "{\"title\":\"T\",\"description\":\"a\\u0001b\\\"c\\\\\",\"type\":\"string\"}";
    let answer = generate(request("t.json", schema).as_bytes());
    assert!(answer.contains("a\\u0001b\\\"c\\\\"), "{answer}");
}

#[test]
fn references_name_other_documents_only() {
    let doc = r##"{"$schema":"https://json-schema.org/draft/2020-12/schema","a":{"$ref":"#/b"},"c":[{"$ref":"other.json#/x"}]}"##;
    assert_eq!(
        references(doc.as_bytes()),
        r#"["https://json-schema.org/draft/2020-12/schema","other.json#/x"]"#
    );
    assert_eq!(references(b"not json"), "[]");
}
