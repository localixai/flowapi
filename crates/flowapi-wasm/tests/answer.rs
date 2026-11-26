//! The answers the module gives, called natively: the same functions its exports call. A YAML
//! document is read as the command reads it: into the same JSON, its diagnostics at YAML lines.

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

fn document(uri: &str, text: &str) -> String {
    format!("{{\"uri\":{},\"text\":{}}}", quoted(uri), quoted(text))
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
        references(document("s.json", doc).as_bytes()),
        r#"["https://json-schema.org/draft/2020-12/schema","other.json#/x"]"#
    );
    assert_eq!(references(document("s.json", "not json").as_bytes()), "[]");
    assert_eq!(references(b"not a request"), "[]");
}

#[test]
fn a_yaml_document_generates_what_its_json_twin_does() {
    let yaml = "title: Pet\ntype: object\nproperties:\n  id: {type: integer}\n";
    let json = r#"{"title":"Pet","type":"object","properties":{"id":{"type":"integer"}}}"#;
    let from_yaml = generate(request("pet.yaml", yaml).as_bytes());
    assert_eq!(from_yaml, generate(request("pet.json", json).as_bytes()));
    // Under a name that is neither `.json` nor `.yaml`, the text says which.
    assert_eq!(generate(request("pet", yaml).as_bytes()), from_yaml);
}

#[test]
fn a_yaml_diagnostic_names_the_yaml_line() {
    let yaml = "title: T\ntype: object\nproperties:\n  a:\n    type: nonsense\n";
    let answer = generate(request("t.yaml", yaml).as_bytes());
    assert!(answer.contains("\"uri\":\"t.yaml\""), "{answer}");
    assert!(answer.contains("\"line\":5,\"column\":11}"), "{answer}");
}

#[test]
fn a_yaml_text_json_cannot_hold_is_one_error_at_its_line() {
    let answer = generate(request("bad.yaml", "a: 1\nb:\n  c: 2\n d: 3\n").as_bytes());
    assert!(
        answer.starts_with(
            "{\"files\":[],\"diagnostics\":[{\"severity\":\"error\",\"code\":\"yaml.syntax\""
        ),
        "{answer}"
    );
    assert!(answer.contains("\"uri\":\"bad.yaml\""), "{answer}");
    assert!(answer.contains("\"line\":4,"), "{answer}");
    assert_eq!(references(document("bad.yaml", "a: [").as_bytes()), "[]");
}

#[test]
fn references_of_a_yaml_document() {
    let yaml = "a:\n  $ref: '#/b'\nc:\n  - $ref: other.yaml#/x\n";
    assert_eq!(
        references(document("s.yaml", yaml).as_bytes()),
        r#"["other.yaml#/x"]"#
    );
}
