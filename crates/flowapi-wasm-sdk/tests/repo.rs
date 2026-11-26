//! `repo` and `plan`, the second module's `generate`, called natively: the repository printed
//! with the paths the plan asks about, and the plan's answers for what is on disk (the npm
//! package's `test/generate.test.ts` holds both to the command's files and messages).
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test, as a test does"
)]

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

fn shop(name: &str) -> String {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../flowapi-sdk/tests/client/shop");
    std::fs::read_to_string(dir.join(name)).expect("the shop fixture")
}

#[test]
fn repo_prints_the_repository_and_names_what_the_plan_reads() {
    let config = format!(
        "{{\"uri\":\"flowapi-sdk.yml\",\"text\":{}}}",
        quoted(&shop("flowapi-sdk.yml"))
    );
    let spec = format!(
        "{{\"uri\":\"spec.json\",\"text\":{}}}",
        quoted(&shop("spec.json"))
    );
    let request = format!(
        "{{\"command\":\"repo\",\"config\":{config},\"documents\":[{spec}],\"lock\":{{\"path\":\"sdk/.flowapi/lock.json\"}}}}"
    );
    let answer = sdk(request.as_bytes());
    assert!(
        answer.starts_with("{\"text\":\"\",\"error\":null,\"files\":[{\"path\":"),
        "{answer}"
    );
    assert!(
        answer.contains("\"path\":\"CHANGELOG.md\""),
        "a first generation seeds it"
    );
    assert!(
        answer.contains("\"executable\":true,\"owner\":\"generated\""),
        "{answer}"
    );
    assert!(answer.contains("\"owner\":\"seed\""), "{answer}");
    assert!(
        answer.ends_with(",\".flowapi/lock.json\"],\"diagnostics\":[]}"),
        "the lock is read last"
    );
    let bad = request.replace(
        "\"path\":\"sdk/.flowapi/lock.json\"",
        "\"path\":\"l.json\",\"text\":\"{}\"",
    );
    let refused = sdk(bad.as_bytes());
    assert!(
        refused.contains(
            "\"error\":\"flowapi: l.json cannot be read: `version` is not a whole number."
        ),
        "{refused}"
    );
}

const LOCK: &str = r#"{\"version\":1,\"flowapi\":\"0.1.0\",\"spec_sha256\":\"\",\"config_sha256\":\"\",\"configured_endpoints\":0,\"files\":{\"package.json\":\"aa\"},\"names\":{}}"#;

fn plan(check: bool, previous: &str, disk: &str) -> String {
    sdk(format!(
        "{{\"command\":\"plan\",\"out\":\"sdk\",\"check\":{check},\"files\":[{{\"path\":\"package.json\",\"owner\":\"generated\",\"executable\":false}}],\"lock\":\"{LOCK}\",\"previous\":{previous},\"disk\":[{disk}]}}"
    )
    .as_bytes())
}

#[test]
fn plan_says_what_to_write_keep_and_refuse() {
    let absent = "{\"path\":\"package.json\",\"kind\":\"absent\"},{\"path\":\".flowapi/lock.json\",\"kind\":\"absent\"}";
    let fresh = plan(false, "null", absent);
    assert!(fresh.starts_with("{\"text\":\"\",\"next\":\"apply\",\"changes\":[{\"change\":\"write\",\"path\":\"package.json\",\"executable\":false},{\"change\":\"write\",\"path\":\".flowapi/lock.json\""), "{fresh}");
    assert!(
        fresh.contains("\"summary\":\"flowapi: sdk: 2 written, 0 deleted, 0 unchanged\""),
        "{fresh}"
    );
    let checked = plan(true, "null", absent);
    assert!(checked.starts_with("{\"text\":\"write package.json\\nwrite .flowapi/lock.json\\nflowapi: sdk differs from what generate writes in 2 files\\n\",\"next\":\"failed\""), "{checked}");
    let same = "{\"path\":\"package.json\",\"kind\":\"file\",\"sha256\":\"aa\",\"executable\":false},{\"path\":\".flowapi/lock.json\",\"kind\":\"absent\"}";
    assert!(
        plan(false, "null", same)
            .contains("\"summary\":\"flowapi: sdk: 1 written, 0 deleted, 1 unchanged\"")
    );
    let edited = same.replace("\"aa\"", "\"bb\"");
    let previous = format!("\"{}\"", LOCK.replace("\\\"aa\\\"", "\\\"cc\\\""));
    let conflict = plan(false, &previous, &edited);
    assert!(
        conflict
            .starts_with("{\"text\":\"conflict: package.json was changed since it was generated"),
        "{conflict}"
    );
    assert!(conflict.contains("\"next\":\"failed\""), "{conflict}");
    let unread = plan(
        false,
        "null",
        "{\"path\":\"package.json\",\"kind\":\"error\",\"message\":\"flowapi: sdk/package.json is a symbolic link\"}",
    );
    assert!(
        unread.starts_with(
            "{\"text\":\"flowapi: sdk/package.json is a symbolic link\\n\",\"next\":\"failed\""
        ),
        "{unread}"
    );
    let outside = plan(
        false,
        &format!("\"{}\"", LOCK.replace("package.json", "../x")),
        absent,
    );
    assert!(outside.contains("\"code\":\"flowapi.request\",\"message\":\"the request's lock cannot be read: the file `../x` is not a path inside the repository\""), "{outside}");
}
