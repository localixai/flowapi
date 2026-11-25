//! `flowapi config schema` says what `config check` reads, and nothing else. The schema is kept
//! (`tests/schema.json`, `FLOWAPI_BLESS=1` writes it anew) and read here by a reader that is not
//! ours; from it, configs with every member it names (one for each pagination scheme's `type`)
//! are made and checked: no member is unknown to the checker and none is left unread by its
//! reader (`config.internal`). Then a member it does not name is put into each object of them in
//! turn, and the checker refuses each one there. Every member the checker takes is in the schema,
//! and every member of the schema the checker takes.
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::Documents;
use serde_json::{Map, Value, json};
use std::path::Path;

fn schema() -> Value {
    serde_json::from_str(&flowapi_sdk::config_schema()).expect("the schema is JSON")
}

/// The schema a `$ref` names.
fn resolve<'a>(root: &'a Value, s: &'a Value) -> &'a Value {
    match s.get("$ref").and_then(Value::as_str) {
        Some(r) => r
            .strip_prefix("#/$defs/")
            .and_then(|name| root["$defs"].get(name))
            .expect("a $ref to $defs"),
        None => s,
    }
}

/// A value of a string schema: what its pattern asks for.
fn string(s: &Value) -> Value {
    let v = match s.get("pattern").and_then(Value::as_str) {
        Some("^[A-Z_][A-Z0-9_]*$") => "PET_SHOP_KEY",
        Some("^[A-Z][A-Za-z0-9]*$") => "PetShop",
        Some(p) if p.starts_with("^https?") => "https://api.petshop.example/v1",
        Some(p) if p.starts_with("^(@") => "@petshop/sdk",
        Some(p) if p.starts_with("^[0-9]+") => env!("CARGO_PKG_VERSION"),
        Some(_) | None => "data",
    };
    json!(v)
}

/// A value with every member `s` names, `variant` choosing among `oneOf`; `depth` stops the
/// resources' recursion.
fn sample(root: &Value, s: &Value, variant: usize, depth: usize) -> Value {
    let s = resolve(root, s);
    if let Some(c) = s.get("const") {
        return c.clone();
    }
    if let Some(all) = s.get("oneOf").and_then(Value::as_array) {
        let mut merged = s.clone();
        merged.as_object_mut().expect("an object").remove("oneOf");
        let one = all.get(variant % all.len()).expect("a variant");
        return sample(root, one, variant, depth);
    }
    if let Some(any) = s.get("anyOf").and_then(Value::as_array) {
        return sample(root, any.last().expect("a branch"), variant, depth);
    }
    match s.get("type").and_then(Value::as_str) {
        Some("string") => string(s),
        Some("integer") => s.get("minimum").cloned().unwrap_or_else(|| json!(1)),
        Some("number") => json!(30),
        Some("boolean") => json!(false),
        Some("array") => json!([sample(root, &s["items"], variant, depth)]),
        Some("object") => object(root, s, variant, depth),
        // A type the test does not know: a value the checker refuses, so the test fails.
        Some(_) | None => Value::Null,
    }
}

fn object(root: &Value, s: &Value, variant: usize, depth: usize) -> Value {
    let mut out = Map::new();
    for (k, v) in s
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let deep = k == "resources" && depth > 1;
        if !deep {
            out.insert(k.clone(), sample(root, v, variant, depth + 1));
        }
    }
    if let Some(extra) = s.get("additionalProperties").filter(|a| a.is_object()) {
        out.insert(
            String::from("entry"),
            sample(root, extra, variant, depth + 1),
        );
    }
    Value::Object(out)
}

/// A small description for the samples to name: one operation, one security scheme.
const SPEC: &str = r#"{"openapi":"3.1.0","info":{"title":"t","version":"1"},
"components":{"securitySchemes":{"entry":{"type":"http","scheme":"bearer"}}},
"paths":{"/pets":{"get":{"operationId":"data","responses":{"200":{"description":"ok"}}}}}}"#;

fn check(config: &Value) -> Vec<flowapi_sdk::Issue> {
    let mut docs = Documents::new();
    docs.add("spec.json", SPEC).expect("JSON");
    flowapi_sdk::check(&docs, &config.to_string())
}

fn format_issues(issues: &[flowapi_sdk::Issue]) -> Vec<String> {
    issues
        .iter()
        .filter(|i| {
            i.code.starts_with("config.schema")
                || i.code == "config.internal"
                || i.code == "config.name.invalid"
        })
        .map(|i| format!("{} {} {}", i.code, i.pointer, i.message))
        .collect()
}

/// Every JSON Pointer of an object in the value.
fn objects(v: &Value, at: &str, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            out.push(at.to_owned());
            for (k, v) in m {
                objects(v, &format!("{at}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                objects(v, &format!("{at}/{i}"), out);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[test]
fn the_schema_is_kept() {
    let printed = flowapi_sdk::config_schema();
    let kept = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/schema.json");
    if std::env::var_os("FLOWAPI_BLESS").is_some() {
        std::fs::write(&kept, &printed).expect("written");
    }
    assert_eq!(
        std::fs::read_to_string(&kept).ok().as_deref(),
        Some(printed.as_str()),
        "the schema changed: bless it and read the diff"
    );
    assert_eq!(
        schema()["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
}

#[test]
fn every_member_of_the_schema_is_read() {
    let root = schema();
    let variants = root["$defs"]["pagination"]["oneOf"]
        .as_array()
        .map_or(1, Vec::len);
    for variant in 0..variants {
        let config = sample(&root, &root, variant, 0);
        for deep in [
            "/resources/entry/methods/entry/stream",
            "/pagination/entry/type",
            "/skip/0/reason",
            "/readme/intro",
        ] {
            assert!(
                config.pointer(deep).is_some(),
                "the sample has no {deep}: {config:#}"
            );
        }
        let issues = check(&config);
        assert!(
            format_issues(&issues).is_empty(),
            "variant {variant}: {:#?}\n{config:#}",
            format_issues(&issues)
        );
    }
}

#[test]
fn a_member_the_schema_does_not_name_is_refused_everywhere() {
    let root = schema();
    let config = sample(&root, &root, 0, 0);
    let mut places = Vec::new();
    objects(&config, "", &mut places);
    assert!(places.len() > 12, "{places:?}");
    let maps = [
        "/environments",
        "/auth",
        "/pagination",
        "/resources",
        "/resources/entry/methods",
        "/resources/entry/resources",
    ];
    for at in places.iter().filter(|p| !maps.contains(&p.as_str())) {
        let mut wrong = config.clone();
        if let Some(Value::Object(m)) = wrong.pointer_mut(at) {
            m.insert(String::from("zz_not_a_member"), json!(1));
        }
        let issues = check(&wrong);
        let refused = issues.iter().any(|i| {
            i.code == "config.schema.unknown" && i.pointer == format!("{at}/zz_not_a_member")
        });
        assert!(
            refused,
            "`{at}` took a member the schema does not name: {issues:#?}"
        );
    }
}
