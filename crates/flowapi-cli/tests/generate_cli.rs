//! `flowapi generate`, each rule of its writing as the command runs it, on the `shop` client case:
//! a fresh repository written with its lock; a second run that writes nothing; a generated file
//! changed by hand is a conflict that writes nothing; the user's files (`src/lib/`, `examples/`)
//! kept; a file no longer generated deleted, or kept with a warning when changed; `--check`; a
//! path out of the folder or through a symbolic link refused; a missing description.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../flowapi-sdk/tests/client/shop")
        .join(name)
}

/// A fresh folder holding the shop's config and description.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("generate_cli")
        .join(test);
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    for name in ["flowapi-sdk.yml", "spec.json"] {
        std::fs::copy(fixture(name), dir.join(name)).expect("copied");
    }
    dir
}

fn generate(dir: &Path, more: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .args(["generate", "flowapi-sdk.yml", "--out", "sdk"])
        .args(more)
        .current_dir(dir)
        .output()
        .expect("the command runs")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn read(dir: &Path, path: &str) -> String {
    std::fs::read_to_string(dir.join("sdk").join(path)).expect(path)
}

/// Every file under a folder with its bytes, sorted: what a run changed is their difference.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).expect("a folder") {
            let p = e.expect("an entry").path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(dir).expect("inside").display().to_string();
                out.push((rel, std::fs::read(&p).expect("read")));
            }
        }
    }
    out.sort();
    out
}

fn edit_config(dir: &Path, from: &str, to: &str) {
    let path = dir.join("flowapi-sdk.yml");
    let text = std::fs::read_to_string(&path).expect("the config");
    assert!(text.contains(from), "{from}");
    std::fs::write(path, text.replace(from, to)).expect("written");
}

#[test]
fn a_fresh_repository_then_a_second_run_that_writes_nothing() {
    let dir = scratch("fresh");
    let out = generate(&dir, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("written, 0 deleted"),
        "{}",
        stderr(&out)
    );
    let lock = read(&dir, ".flowapi/lock.json");
    assert!(lock.contains("\"src/client.ts\""), "{lock}");
    assert!(lock.contains("\"configured_endpoints\": 14"), "{lock}");
    assert!(
        !lock.contains("CHANGELOG.md"),
        "the release tool's, not in the lock"
    );
    assert!(read(&dir, "CHANGELOG.md").starts_with("# Changelog"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(dir.join("sdk/scripts/check"))
            .expect("written")
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "scripts are executable");
    }
    let before = tree(&dir.join("sdk"));
    let again = generate(&dir, &[]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(
        stderr(&again).contains(": 0 written, 0 deleted"),
        "{}",
        stderr(&again)
    );
    assert_eq!(
        tree(&dir.join("sdk")),
        before,
        "a second run changes nothing"
    );
    let check = generate(&dir, &["--check"]);
    assert!(check.status.success(), "{}", stderr(&check));
}

#[test]
fn a_generated_file_changed_by_hand_is_a_conflict_and_nothing_is_written() {
    let dir = scratch("conflict");
    assert!(generate(&dir, &[]).status.success());
    let readme = dir.join("sdk/README.md");
    std::fs::write(&readme, "# mine\n").expect("edited");
    edit_config(&dir, "version: 1.2.0", "version: 1.3.0");
    let before = tree(&dir.join("sdk"));
    for more in [&[][..], &["--check"][..]] {
        let out = generate(&dir, more);
        assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
        assert!(
            stderr(&out).contains("conflict: README.md was changed"),
            "{}",
            stderr(&out)
        );
        assert!(
            stderr(&out).contains("nothing was written"),
            "{}",
            stderr(&out)
        );
        assert_eq!(
            tree(&dir.join("sdk")),
            before,
            "package.json kept its 1.2.0 too"
        );
    }
    // Restored, it is generated again with the new version.
    std::fs::remove_file(&readme).expect("removed");
    assert!(generate(&dir, &[]).status.success());
    assert!(read(&dir, "package.json").contains("\"version\": \"1.3.0\""));
}

#[test]
fn a_file_in_the_way_that_the_lock_does_not_list_is_a_conflict() {
    let dir = scratch("in_the_way");
    std::fs::create_dir_all(dir.join("sdk")).expect("made");
    std::fs::write(dir.join("sdk/package.json"), "{}\n").expect("written");
    let out = generate(&dir, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("package.json is in the way"),
        "{}",
        stderr(&out)
    );
    assert_eq!(tree(&dir.join("sdk")).len(), 1, "nothing written");
}

#[test]
fn the_users_files_are_kept() {
    let dir = scratch("own");
    assert!(generate(&dir, &[]).status.success());
    std::fs::create_dir_all(dir.join("sdk/src/lib")).expect("made");
    std::fs::write(dir.join("sdk/src/lib/x.ts"), "export const x = 1\n").expect("written");
    std::fs::write(dir.join("sdk/examples/.keep"), "mine\n").expect("written");
    std::fs::write(dir.join("sdk/CHANGELOG.md"), "# Changelog\n\n## 1.2.0\n").expect("written");
    std::fs::write(dir.join("sdk/notes.txt"), "not generated\n").expect("written");
    edit_config(&dir, "version: 1.2.0", "version: 1.3.0");
    let out = generate(&dir, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("examples/.keep was changed: it is yours, and kept"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&dir, "src/lib/x.ts"), "export const x = 1\n");
    assert_eq!(read(&dir, "examples/.keep"), "mine\n");
    assert_eq!(read(&dir, "CHANGELOG.md"), "# Changelog\n\n## 1.2.0\n");
    assert_eq!(read(&dir, "notes.txt"), "not generated\n");
    assert!(read(&dir, "package.json").contains("1.3.0"));
    assert!(!read(&dir, ".flowapi/lock.json").contains("examples/.keep"));
}

#[test]
fn a_file_no_longer_generated_is_deleted_unless_changed() {
    let dir = scratch("orphans");
    assert!(generate(&dir, &[]).status.success());
    // The `system` and `tags` resources leave the config, their operations skipped.
    edit_config(&dir, "  system:\n    methods:\n      health: health\n", "");
    edit_config(
        &dir,
        "  tags:\n    methods:\n      list: { op: listTags, pagination: one }\n",
        "",
    );
    edit_config(
        &dir,
        "readme:",
        "skip:\n  - { op: health, reason: internal }\n  - { op: listTags, reason: internal }\nreadme:",
    );
    std::fs::write(dir.join("sdk/src/resources/tags.ts"), "// mine\n").expect("edited");
    let out = generate(&dir, &[]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!dir.join("sdk/src/resources/system.ts").exists());
    assert!(!dir.join("sdk/tests/api-resources/system.test.mjs").exists());
    assert!(
        stderr(&out).contains("src/resources/tags.ts is no longer generated, but was changed"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&dir, "src/resources/tags.ts"), "// mine\n");
    let lock = read(&dir, ".flowapi/lock.json");
    assert!(
        !lock.contains("system") && !lock.contains("resources/tags.ts"),
        "{lock}"
    );
}

#[test]
fn check_writes_nothing_and_says_what_differs() {
    let dir = scratch("check");
    let out = generate(&dir, &["--check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("write package.json"),
        "{}",
        stderr(&out)
    );
    assert!(!dir.join("sdk").exists(), "nothing written");
    assert!(generate(&dir, &[]).status.success());
    edit_config(&dir, "version: 1.2.0", "version: 1.3.0");
    let out = generate(&dir, &["--check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("differs from what generate writes"),
        "{}",
        stderr(&out)
    );
    assert!(read(&dir, "package.json").contains("1.2.0"));
}

#[test]
fn a_path_out_of_the_folder_is_refused() {
    let dir = scratch("escape");
    assert!(generate(&dir, &[]).status.success());
    let lock = dir.join("sdk/.flowapi/lock.json");
    let text = std::fs::read_to_string(&lock).expect("the lock");
    std::fs::write(&lock, text.replace("\"api.md\"", "\"../outside.md\"")).expect("written");
    std::fs::write(dir.join("outside.md"), "keep me\n").expect("written");
    let before = tree(&dir);
    let out = generate(&dir, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("is not a path inside the repository"),
        "{}",
        stderr(&out)
    );
    assert_eq!(tree(&dir), before, "nothing written or deleted");
}

#[cfg(unix)]
#[test]
fn a_path_through_a_symbolic_link_is_refused() {
    let dir = scratch("symlink");
    std::fs::create_dir_all(dir.join("elsewhere")).expect("made");
    std::fs::create_dir_all(dir.join("sdk")).expect("made");
    std::os::unix::fs::symlink(dir.join("elsewhere"), dir.join("sdk/src")).expect("linked");
    let out = generate(&dir, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("is a symbolic link"),
        "{}",
        stderr(&out)
    );
    assert!(
        std::fs::read_dir(dir.join("elsewhere"))
            .expect("a folder")
            .next()
            .is_none()
    );
    assert_eq!(tree(&dir.join("sdk")).len(), 0, "nothing written");
}

#[test]
fn a_missing_description_writes_nothing() {
    let dir = scratch("no_spec");
    std::fs::remove_file(dir.join("spec.json")).expect("removed");
    let out = generate(&dir, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("cannot read"), "{}", stderr(&out));
    assert!(!dir.join("sdk").exists());
    let config_error = scratch("bad_config");
    edit_config(&config_error, "client: Shop", "client: shop");
    let out = generate(&config_error, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("error[config.package.client]"),
        "{}",
        stderr(&out)
    );
    assert!(!config_error.join("sdk").exists());
}
