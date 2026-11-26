//! The second module's answers, called natively: the SDK commands give what `flowapi-sdk` gives
//! the command (its own tests hold the bytes; the npm package's `test/parity.test.ts` holds them
//! to the command's), and a request that cannot be read is one error.

use flowapi_wasm_sdk::sdk;

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

const PETS: &str = r#"{"openapi":"3.1.0","info":{"title":"Pets","version":"1"},"paths":{"/pets":{"get":{"operationId":"listPets","tags":["pets"],"responses":{"200":{"description":"ok"}}}}}}"#;

fn command(body: &str) -> String {
    sdk(format!("{{{body}}}").as_bytes())
}

#[test]
fn summary_and_draft_give_the_sdk_layers_text() {
    let docs = format!("\"documents\":[{}]", document("pets.json", PETS));
    let answer = command(&format!(
        "\"command\":\"summary\",{docs},\"filter\":{{\"tag\":\"pets\",\"maxBytes\":\"900\"}}"
    ));
    assert!(answer.starts_with("{\"text\":\"{\\\"info\\\":"), "{answer}");
    assert!(
        answer.ends_with(",\"json\":null,\"diagnostics\":[]}"),
        "{answer}"
    );
    let draft = command(&format!("\"command\":\"draft\",{docs}"));
    assert!(draft.contains("spec: pets.json"), "{draft}");
    let named = command(&format!(
        "\"command\":\"draft\",{docs},\"spec\":\"../pets.json\""
    ));
    assert!(named.contains("spec: \\\"../pets.json\\\""), "{named}");
    let schema = command("\"command\":\"schema\"");
    assert!(schema.starts_with("{\"text\":\"{"), "{schema}");
}

#[test]
fn what_is_not_a_description_has_no_text() {
    let docs = format!(
        "\"documents\":[{}]",
        document("s.json", r#"{"type":"string"}"#)
    );
    let answer = command(&format!("\"command\":\"summary\",{docs}"));
    assert_eq!(answer, "{\"text\":null,\"json\":null,\"diagnostics\":[]}");
    let bad = format!(
        "\"documents\":[{}]",
        document("b.yaml", "a: 1\nb:\n  c: 2\n d: 3\n")
    );
    let answer = command(&format!("\"command\":\"draft\",{bad}"));
    assert!(answer.starts_with("{\"text\":null,\"json\":null,\"diagnostics\":[{\"severity\":\"error\",\"code\":\"yaml.syntax\""), "{answer}");
}

#[test]
fn check_reads_the_config_first_and_gives_text_and_json() {
    let config = document(
        "c.yml",
        "version: 1\nspec: pets.json\npackage: {name: a, client: A}\nresources: {}\n",
    );
    let spec = command(&format!("\"command\":\"spec\",\"config\":{config}"));
    assert_eq!(
        spec,
        "{\"text\":\"pets.json\",\"json\":null,\"diagnostics\":[]}"
    );
    let docs = format!("\"documents\":[{}]", document("pets.json", PETS));
    let answer = command(&format!("\"command\":\"check\",{docs},\"config\":{config}"));
    assert!(answer.contains("error[config.coverage]"), "{answer}");
    assert!(answer.contains("  at c.yml:"), "{answer}");
    assert!(answer.contains("\"json\":\"{\\\"ok\\\":false,"), "{answer}");
    let bad = document("c.yml", "a:\n  b: 1\n c: 2\n");
    let unread = command(&format!("\"command\":\"check\",{docs},\"config\":{bad}"));
    assert!(
        unread.contains("\"code\":\"yaml.syntax\",\"message\""),
        "{unread}"
    );
    assert!(
        unread.starts_with("{\"text\":null,\"json\":null,"),
        "{unread}"
    );
}

#[test]
fn an_sdk_request_that_cannot_be_read_is_one_error() {
    for bad in [
        "",
        "{}",
        "{\"command\":\"make\"}",
        "{\"command\":\"summary\",\"documents\":[]}",
        "{\"command\":\"check\"}",
        "{\"command\":\"summary\",\"documents\":[{\"uri\":\"a\",\"text\":\"{}\"}],\"filter\":{\"maxBytes\":-1}}",
    ] {
        let answer = sdk(bad.as_bytes());
        assert!(
            answer.starts_with("{\"text\":null,\"json\":null,\"diagnostics\":[{\"severity\":\"error\",\"code\":\"flowapi.request\""),
            "{bad}: {answer}"
        );
    }
}
