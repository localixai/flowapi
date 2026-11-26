//! The `flowapi` command's commands about an SDK's config: `summary`, `config draft`, `config
//! schema` and `config check`, before a path, while the old form reads a description, even one
//! named `summary`, as it did.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../flowapi-sdk/tests")
        .join(name)
}

/// A fresh directory for one test's files.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("sdk_cli")
        .join(test);
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn flowapi(args: &[&str], dir: Option<&Path>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_flowapi"));
    c.args(args);
    if let Some(d) = dir {
        c.current_dir(d);
    }
    c.output().expect("the command runs")
}

fn text(out: &[u8]) -> String {
    String::from_utf8(out.to_vec()).expect("UTF-8")
}

#[test]
fn a_draft_passes_check_beside_its_description() {
    let dir = scratch("draft");
    std::fs::copy(fixture("config/spec.json"), dir.join("openapi.json")).expect("copied");
    let out = flowapi(
        &["config", "draft", "openapi.json", "-o", "flowapi-sdk.yml"],
        Some(&dir),
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let yaml = std::fs::read_to_string(dir.join("flowapi-sdk.yml")).expect("written");
    assert!(yaml.contains("spec: openapi.json"), "{yaml}");
    let check = flowapi(&["config", "check", "flowapi-sdk.yml"], Some(&dir));
    assert!(check.status.success(), "{}", text(&check.stdout));
    assert_eq!(text(&check.stdout), "flowapi-sdk.yml: no issues\n");
    let printed = flowapi(&["config", "draft", "openapi.json"], Some(&dir));
    assert!(text(&printed.stdout).starts_with("# A first flowapi-sdk.yml"));
}

#[test]
fn check_names_the_yaml_line_and_exits_1() {
    let config = fixture("config/refs.yml");
    let out = flowapi(&["config", "check", config.to_str().expect("a path")], None);
    assert_eq!(out.status.code(), Some(1));
    let printed = text(&out.stdout);
    assert!(
        printed.contains("error[config.op.unknown]: `listPet` names no operation"),
        "{printed}"
    );
    assert!(
        printed.contains("refs.yml:10:13 (/resources/pets/methods/list)"),
        "{printed}"
    );
    assert!(
        printed.contains("hint: did you mean `listPets`?"),
        "{printed}"
    );
    let json = flowapi(
        &[
            "config",
            "check",
            config.to_str().expect("a path"),
            "--json",
        ],
        None,
    );
    assert_eq!(json.status.code(), Some(1));
    let printed = text(&json.stdout);
    assert!(printed.starts_with("{\"ok\":false,"), "{printed}");
    assert!(printed.contains("\"line\":10,\"column\":13"), "{printed}");
}

#[test]
fn check_takes_a_spec_and_refuses_a_url() {
    let dir = scratch("url");
    let config = dir.join("c.yml");
    std::fs::write(&config, "version: 1\nspec: https://example.com/openapi.json\npackage: {name: a, client: A}\nresources: {}\n").expect("written");
    let out = flowapi(&["config", "check", config.to_str().expect("a path")], None);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("does not fetch: name a file with --spec"),
        "{}",
        text(&out.stderr)
    );
    let spec = fixture("config/spec.json");
    let given = flowapi(
        &[
            "config",
            "check",
            config.to_str().expect("a path"),
            "--spec",
            spec.to_str().expect("a path"),
        ],
        None,
    );
    assert!(
        text(&given.stdout).contains("config.coverage"),
        "{}",
        text(&given.stdout)
    );
}

#[test]
fn schema_is_the_kept_one() {
    let out = flowapi(&["config", "schema"], None);
    assert!(out.status.success());
    let kept = std::fs::read_to_string(fixture("schema.json")).expect("kept");
    assert_eq!(text(&out.stdout), kept);
}

#[test]
fn summary_filters_by_tag() {
    let spec = fixture("config/spec.json");
    let out = flowapi(
        &["summary", spec.to_str().expect("a path"), "--tag", "owners"],
        None,
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let printed = text(&out.stdout);
    assert!(printed.contains("\"shown\":1"), "{printed}");
    assert!(printed.contains("\"id\":\"listOwners\""), "{printed}");
    let small = flowapi(
        &[
            "summary",
            spec.to_str().expect("a path"),
            "--max-bytes",
            "500",
        ],
        None,
    );
    assert!(text(&small.stdout).contains("\"truncated\""));
}

/// A description named `summary` is read by the old form through its path.
#[test]
fn a_file_named_summary_is_still_a_description() {
    let dir = scratch("named");
    std::fs::copy(fixture("config/spec.json"), dir.join("summary")).expect("copied");
    let out = flowapi(&["./summary"], Some(&dir));
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("export type Pet"));
    let sub = flowapi(&["summary", "summary"], Some(&dir));
    assert!(
        text(&sub.stdout).starts_with("{\"info\":"),
        "{}",
        text(&sub.stderr)
    );
}

#[test]
fn wrong_use_is_explained() {
    for (args, says) in [
        (&["config"][..], "`config` is followed by"),
        (&["config", "make"][..], "`config make`"),
        (&["config", "check"][..], "needs a config"),
        (
            &["config", "schema", "--json"][..],
            "unknown flag `--json` for `config schema`",
        ),
        (&["summary"][..], "a description to summarize"),
    ] {
        let out = flowapi(args, None);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(
            text(&out.stderr).contains(says),
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
    let help = text(&flowapi(&["--help"], None).stdout);
    assert!(
        help.contains("flowapi config check <flowapi-sdk.yml>"),
        "{help}"
    );
}
