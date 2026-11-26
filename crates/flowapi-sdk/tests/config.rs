//! `config check` on each config of `tests/config/` (YAML, read as the command reads it) against
//! `tests/config/spec.json`, its issues as text against the `.txt` kept beside it, each line's
//! YAML line and column. `FLOWAPI_BLESS=1 cargo test -p flowapi-sdk --test config` writes them
//! anew; a change to one is read in review like code. Each case also names the codes it must
//! raise, so that a blessed file cannot lose one in silence.
//!
//! valid.yml: every member of the format, nothing wrong. format.yml: members the format does not
//! name (typos with hints, `overrides`: no override can rename a schema's neighbours, study 03
//! pitfall 5), types, ranges, kinds of string, required members. refs.yml: references with
//! typos (hints), routes, one operation in two places, coverage, a `skip` without a reason
//! (pitfall 17). names.yml: names as words (pitfalls 2 and 3), reserved members, a resource
//! named as a schema (pitfall 6) or as the client. paging.yml: pagination against the response
//! schemas (pitfall 9), streams, `auth`, `default`, the README, the pinned version.
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::{Documents, Severity};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/config")
}

/// The issues of a config's text (YAML or JSON) against the case description.
fn check(name: &str, text: &str) -> (Vec<flowapi_sdk::Issue>, String) {
    let spec = std::fs::read_to_string(dir().join("spec.json")).expect("the description");
    let mut docs = Documents::new();
    docs.add("spec.json", &spec).expect("JSON");
    let config = flowapi_yaml::Text::read(name, Cow::Borrowed(text)).expect("YAML");
    let issues = flowapi_sdk::check(&docs, config.json());
    let printed = flowapi_sdk::issues_text(&issues, name, &|o| config.place(o));
    (issues, printed)
}

/// The codes each case must raise.
const EXPECTED: &[(&str, &[&str])] = &[
    ("valid", &[]),
    (
        "format",
        &[
            "config.version",
            "config.package.name",
            "config.package.client",
            "config.env.url",
            "config.name.invalid",
            "config.env_var",
            "config.schema.type",
            "config.schema.range",
            "config.pagination.type",
            "config.schema.required",
            "config.schema.unknown",
        ],
    ),
    (
        "refs",
        &[
            "config.op.unknown",
            "config.op.twice",
            "config.coverage",
            "config.schema.empty",
        ],
    ),
    (
        "auth",
        &[
            "config.auth.declared",
            "config.auth.unknown",
            "config.auth.field",
            "config.auth.type",
            "config.auth.in",
            "config.auth.option",
            "config.auth.refused",
            "config.auth.mutual_tls",
            "config.security.unknown",
            "config.auth.env",
            "config.auth.unlisted",
        ],
    ),
    ("auth-none", &["config.auth.none"]),
    (
        "names",
        &[
            "config.name.duplicate",
            "config.name.reserved",
            "config.name.model",
            "config.name.client",
        ],
    ),
    (
        "paging",
        &[
            "config.generator.version",
            "config.env.default",
            "config.auth.unknown",
            "config.pagination.missing",
            "config.pagination.mismatch",
            "config.pagination.link",
            "config.pagination.header",
            "config.pagination.unknown",
            "config.pagination.response",
            "config.stream.media",
            "config.readme.method",
        ],
    ),
];

#[test]
fn configs_check_as_kept() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for (case, codes) in EXPECTED {
        let file = format!("{case}.yml");
        let text = std::fs::read_to_string(dir().join(&file)).expect("the case");
        let (issues, printed) = check(&file, &text);
        assert_eq!(
            printed,
            check(&file, &text).1,
            "{file}: checked twice differs"
        );
        let found: Vec<&str> = issues.iter().map(|i| i.code).collect();
        for code in *codes {
            assert!(found.contains(code), "{file} raises no {code}:\n{printed}");
        }
        assert!(
            !found.contains(&"config.internal"),
            "{file}: the reader and the format drifted:\n{printed}"
        );
        if codes.is_empty() {
            assert!(issues.is_empty(), "{file} has issues:\n{printed}");
        }
        let kept = dir().join(format!("{case}.txt"));
        if bless {
            std::fs::write(&kept, &printed).expect("written");
        } else if std::fs::read_to_string(&kept).ok().as_deref() != Some(printed.as_str()) {
            changed.push(format!("{}:\n{printed}", kept.display()));
        }
    }
    assert!(
        changed.is_empty(),
        "checked otherwise than kept:\n{}",
        changed.join("\n")
    );
}

/// `list_all` and `listAll` are one name; so are `abuse-reports` and `abuse_reports` (study 03,
/// pitfalls 2 and 3): the second of each is an error, at its own key.
#[test]
fn names_are_words() {
    let text = std::fs::read_to_string(dir().join("names.yml")).expect("the case");
    let (issues, _) = check("names.yml", &text);
    let at = |p: &str| {
        issues
            .iter()
            .any(|i| i.code == "config.name.duplicate" && i.pointer == p)
    };
    assert!(at("/resources/pets/methods/listAll"));
    assert!(at("/resources/abuse_reports"));
    assert!(!at("/resources/pets/methods/list_all"));
}

/// A typo names the closest operation; a route differing in its path names the route.
#[test]
fn a_typo_has_a_hint() {
    let text = std::fs::read_to_string(dir().join("refs.yml")).expect("the case");
    let (issues, _) = check("refs.yml", &text);
    let hint = |p: &str| {
        issues
            .iter()
            .find(|i| i.pointer == p)
            .and_then(|i| i.hint.clone())
    };
    assert_eq!(
        hint("/resources/pets/methods/list").as_deref(),
        Some("did you mean `listPets`?")
    );
    assert_eq!(
        hint("/resources/pets/methods/retrieve").as_deref(),
        Some("did you mean `GET /pets/{pet_id}`?")
    );
}

/// A config that is not JSON is one issue; one that is no object, the format's issue; and a
/// description that is no OpenAPI one, its own.
#[test]
fn what_is_no_config() {
    let mut docs = Documents::new();
    docs.add("s.json", r#"{"type":"object"}"#).expect("JSON");
    let syntax = flowapi_sdk::check(&docs, "{");
    assert_eq!(
        syntax.iter().map(|i| i.code).collect::<Vec<_>>(),
        ["config.syntax"]
    );
    let list = flowapi_sdk::check(&docs, "[]");
    assert!(
        list.iter().any(|i| i.code == "config.schema.type"),
        "{list:?}"
    );
    let schema = flowapi_sdk::check(
        &docs,
        r#"{"version":1,"spec":"s.json","package":{"name":"a","client":"A"},"resources":{}}"#,
    );
    assert!(
        schema
            .iter()
            .any(|i| i.code == "config.spec" && i.severity == Severity::Error)
    );
}

/// The JSON for an agent: the issue's YAML place, its pointer, and `ok` false on an error.
#[test]
fn issues_as_json() {
    let text = std::fs::read_to_string(dir().join("refs.yml")).expect("the case");
    let (issues, _) = check("refs.yml", &text);
    let config = flowapi_yaml::Text::read("refs.yml", Cow::Borrowed(text.as_str())).expect("YAML");
    let json = flowapi_sdk::issues_json(&issues, &|o| config.place(o));
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON");
    assert_eq!(value["ok"], false);
    let first = value["issues"]
        .as_array()
        .and_then(|a| a.iter().find(|i| i["code"] == "config.op.unknown"))
        .expect("an issue");
    assert_eq!(first["line"], 10);
    assert_eq!(first["pointer"], "/resources/pets/methods/list");
    assert_eq!(first["hint"], "did you mean `listPets`?");
}
