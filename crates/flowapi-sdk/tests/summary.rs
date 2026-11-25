//! `flowapi summary` of `tests/config/spec.json`, whole, by a tag, by a path, and over a small
//! budget, against the `.json` kept in `tests/summary/` (`FLOWAPI_BLESS=1 cargo test -p
//! flowapi-sdk --test summary` writes them anew). Each is JSON a reader that is not ours reads.
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::Documents;
use flowapi_sdk::Filter;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn summary(filter: &Filter) -> String {
    let text = std::fs::read_to_string(root().join("config/spec.json")).expect("the description");
    let mut docs = Documents::new();
    docs.add("spec.json", &text).expect("JSON");
    flowapi_sdk::summary(&docs, filter).expect("a description")
}

fn cases() -> Vec<(&'static str, Filter)> {
    let tag = |t: &str| Filter {
        tag: Some(t.to_owned()),
        ..Filter::default()
    };
    vec![
        ("shop", Filter::default()),
        ("shop-tag", tag("Pets")),
        (
            "shop-path",
            Filter {
                path: Some(String::from("/pets/")),
                ..Filter::default()
            },
        ),
        (
            "shop-index",
            Filter {
                max_bytes: 1_000,
                ..Filter::default()
            },
        ),
    ]
}

#[test]
fn summaries_print_as_kept() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for (name, filter) in cases() {
        let printed = summary(&filter);
        let _: Value = serde_json::from_str(&printed)
            .unwrap_or_else(|e| panic!("{name} is no JSON: {e}\n{printed}"));
        let kept = root().join("summary").join(format!("{name}.json"));
        if bless {
            std::fs::write(&kept, &printed).expect("written");
        } else if std::fs::read_to_string(&kept).ok().as_deref() != Some(printed.as_str()) {
            changed.push(format!("{}:\n{printed}", kept.display()));
        }
    }
    assert!(
        changed.is_empty(),
        "summarized otherwise than kept:\n{}",
        changed.join("\n")
    );
}

/// A tag is matched as words; over the budget the tags alone remain, with how to read on.
#[test]
fn filters_and_the_budget() {
    let pets: Value = serde_json::from_str(&summary(&cases()[1].1)).expect("JSON");
    assert_eq!(pets["counts"]["shown"], 6);
    let index: Value = serde_json::from_str(&summary(&cases()[3].1)).expect("JSON");
    assert!(index.get("operations").is_none());
    assert_eq!(index["truncated"]["max"], 1_000);
    assert_eq!(index["tags"][0]["name"], "pets");
}

/// `user_id` and `userId` are two parameters, each kept by its own name (study 03, pitfall 7):
/// flowapi never prints a description's names as words, so no two of them can meet.
#[test]
fn names_the_description_gives_are_kept_apart() {
    let all: Value = serde_json::from_str(&summary(&Filter::default())).expect("JSON");
    let users = all["operations"]
        .as_array()
        .and_then(|ops| ops.iter().find(|o| o["id"] == "listUsers"))
        .expect("listUsers");
    let names: Vec<&str> = users["params"]
        .as_array()
        .expect("params")
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert_eq!(names, ["user_id", "userId"]);
}

/// The pagination an operation looks like it has, with its reason and how sure.
#[test]
fn pagination_candidates() {
    let all: Value = serde_json::from_str(&summary(&Filter::default())).expect("JSON");
    let op = |id: &str| {
        all["operations"]
            .as_array()
            .and_then(|ops| ops.iter().find(|o| o["id"] == id))
            .cloned()
            .expect("an operation")
    };
    let pets = op("listPets");
    assert_eq!(pets["pagination"][0]["scheme"], "cursor");
    assert_eq!(pets["pagination"][0]["confidence"], "high");
    assert_eq!(
        pets["pagination"][0]["fields"]["response"]["next"],
        "next_page_token"
    );
    assert_eq!(op("listPhotos")["pagination"][0]["scheme"], "offset");
    assert_eq!(
        op("listOwners")["pagination"][0]["fields"]["response"]["header"],
        "Link"
    );
    assert!(op("createPet").get("pagination").is_none());
}
