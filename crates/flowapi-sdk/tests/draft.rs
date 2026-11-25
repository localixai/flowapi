//! `config draft` on each description of `tests/draft/` and on `tests/config/spec.json`, against
//! the `.yml` kept beside it (`FLOWAPI_BLESS=1 cargo test -p flowapi-sdk --test draft` writes
//! them anew). Each draft passes `config check` with no issue (a draft that fails its own check
//! is a bug), and the same description written in other orders, every object's members sorted
//! and reversed, `paths` among them, gives the same bytes (study 03, pitfall 1).
//!
//! shop (`tests/config/spec.json`): tags, a cursor, an offset and a `Link` header, a stream, a
//! resource named as a schema (`collision`, made `collisions`), an operation with no
//! `operationId`. swagger.json: Swagger 2.0's `host` and `basePath` as the environment, no tags
//! (resources by path), `skip` and `pageSize` as an offset, a page, a title opening with a digit.
//! rpc.json: dotted RPC paths that are no collection, a cursor one level down
//! (`response_metadata.next_cursor`), a server variable, a relative server and `mutualTLS` left
//! out, a tag that is no name (`2fa`), a resource named as the client. stripe.json: `/v1` passed
//! over, the last item's id, a resource named as a client member (`request`).
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::Documents;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn draft(name: &str, text: &str) -> String {
    let mut docs = Documents::new();
    docs.add(name, text).expect("JSON");
    flowapi_sdk::draft(&docs, name).expect("a description")
}

/// The issues of a draft against its description, as text.
fn issues(name: &str, spec: &str, yaml: &str) -> String {
    let mut docs = Documents::new();
    docs.add(name, spec).expect("JSON");
    let config =
        flowapi_yaml::Text::read("draft.yml", Cow::Borrowed(yaml)).expect("the draft is YAML");
    let found = flowapi_sdk::check(&docs, config.json());
    flowapi_sdk::issues_text(&found, "draft.yml", &|o| config.place(o))
}

/// A JSON value written with each object's members in the order `reverse` says: sorted (as
/// `serde_json` keeps them), or the reverse.
fn write(v: &Value, reverse: bool, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut members: Vec<(&String, &Value)> = m.iter().collect();
            if reverse {
                members.reverse();
            }
            out.push('{');
            for (i, (k, v)) in members.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let _ = write!(out, "{}:", Value::String(k.clone()));
                write(v, reverse, out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, v) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(v, reverse, out);
            }
            out.push(']');
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            out.push_str(&v.to_string())
        }
    }
}

fn cases() -> Vec<(String, PathBuf, PathBuf)> {
    let mut out = vec![(
        String::from("spec.json"),
        root().join("config/spec.json"),
        root().join("draft/shop.yml"),
    )];
    let mut names: Vec<PathBuf> = std::fs::read_dir(root().join("draft"))
        .expect("the cases")
        .map(|e| e.expect("an entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    for p in names {
        let name = p
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .into_owned();
        let kept = p.with_extension("yml");
        out.push((name, p, kept));
    }
    out
}

#[test]
fn drafts_print_as_kept_and_pass_check() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for (name, spec, kept) in cases() {
        let text = std::fs::read_to_string(&spec).expect("the description");
        let printed = draft(&name, &text);
        assert_eq!(
            issues(&name, &text, &printed),
            "",
            "{name}: the draft fails its own check:\n{printed}"
        );
        if bless {
            std::fs::write(&kept, &printed).expect("written");
        } else if std::fs::read_to_string(&kept).ok().as_deref() != Some(printed.as_str()) {
            changed.push(format!("{}:\n{printed}", kept.display()));
        }
    }
    assert!(
        changed.is_empty(),
        "drafted otherwise than kept:\n{}",
        changed.join("\n")
    );
}

#[test]
fn the_order_of_the_description_changes_nothing() {
    for (name, spec, _) in cases() {
        let text = std::fs::read_to_string(&spec).expect("the description");
        let first = draft(&name, &text);
        let value: Value = serde_json::from_str(&text).expect("JSON");
        for reverse in [false, true] {
            let mut other = String::new();
            write(&value, reverse, &mut other);
            assert_ne!(other, text, "{name}: the order did not change");
            assert_eq!(
                draft(&name, &other),
                first,
                "{name}, members {}",
                if reverse { "reversed" } else { "sorted" }
            );
        }
    }
}

/// A document that is no description has no draft.
#[test]
fn a_schema_has_no_draft() {
    let mut docs = Documents::new();
    docs.add("s.json", r#"{"type":"string"}"#).expect("JSON");
    assert_eq!(flowapi_sdk::draft(&docs, "s.json"), None);
}
