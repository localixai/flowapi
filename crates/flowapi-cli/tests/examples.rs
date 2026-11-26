//! The examples of real APIs at the repository's root (`examples/`), held to the command: each
//! example whose generated repository is committed (it has `.flowapi/lock.json`) is the bytes
//! `flowapi generate` writes for its config into an empty folder, every file and nothing more,
//! with no issue and no warning on the way; every example's config passes `config check` with
//! none, where its description is at hand (a fetched one is there after `examples/fetch.sh`).
//! `FLOWAPI_BLESS=1 cargo test -p flowapi-cli --test examples` writes the committed repositories
//! anew, deleting what is no longer generated; a change to one is read in review like code.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The files of an example that flowapi does not write: the config, its description, the notes.
const OWN: [&str; 2] = ["flowapi-sdk.yml", "SOURCE.md"];

/// What installing and judging a committed repository leaves in it, ignored by its `.gitignore`
/// or `examples/.gitignore`: never compared.
const LEFT_BY_NPM: [&str; 3] = ["node_modules", "dist", "package-lock.json"];

fn examples() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("examples/")
        .map(|e| e.expect("an entry").path())
        .filter(|p| p.join("flowapi-sdk.yml").is_file())
        .collect();
    out.sort();
    assert!(!out.is_empty(), "no example under {}", root.display());
    out
}

fn name(dir: &Path) -> String {
    dir.file_name()
        .expect("a folder")
        .to_string_lossy()
        .into_owned()
}

/// The config's `spec`, a path relative to it, as the examples write it (one line, no URL).
fn spec(dir: &Path) -> String {
    let config = std::fs::read_to_string(dir.join("flowapi-sdk.yml")).expect("the config");
    let line = config
        .lines()
        .find_map(|l| l.strip_prefix("spec:"))
        .expect("a `spec:` line");
    line.trim().trim_matches('"').to_owned()
}

fn flowapi(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("the command runs")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Every file under `dir` but those `skip` names at its top, with its bytes and whether it is
/// executable, sorted by path.
fn tree(dir: &Path, skip: &[&str]) -> Vec<(String, Vec<u8>, bool)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).expect("a folder") {
            let p = e.expect("an entry").path();
            let rel = p.strip_prefix(dir).expect("inside").display().to_string();
            if skip.contains(&rel.as_str()) {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push((rel, std::fs::read(&p).expect("read"), executable(&p)));
            }
        }
    }
    out.sort();
    out
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = std::fs::metadata(path).expect("a file").permissions().mode();
    mode & 0o111 != 0
}

#[cfg(not(unix))]
fn executable(_: &Path) -> bool {
    false
}

#[test]
fn committed_repositories_are_what_generate_writes() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for dir in examples()
        .into_iter()
        .filter(|d| d.join(".flowapi/lock.json").is_file())
    {
        let example = name(&dir);
        let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("examples")
            .join(&example);
        drop(std::fs::remove_dir_all(&out));
        let run = flowapi(
            &dir,
            &["generate", "flowapi-sdk.yml", "--out", &out.to_string_lossy()],
        );
        let said = stderr(&run);
        assert!(run.status.success(), "{example}: {said}");
        assert_eq!(said.lines().count(), 1, "{example}: no issue, no warning: {said}");
        let written = tree(&out, &[]);
        let spec = spec(&dir);
        let mut skip: Vec<&str> = OWN.to_vec();
        skip.push(&spec);
        skip.extend(LEFT_BY_NPM);
        let kept = tree(&dir, &skip);
        if bless {
            for (path, _, _) in kept.iter().filter(|k| !written.iter().any(|w| w.0 == k.0)) {
                std::fs::remove_file(dir.join(path)).expect("deleted");
            }
            for (path, _, _) in &written {
                let to = dir.join(path);
                std::fs::create_dir_all(to.parent().expect("a folder")).expect("made");
                std::fs::copy(out.join(path), to).expect("copied with its mode");
            }
        } else {
            let paths: BTreeSet<&str> = kept.iter().chain(&written).map(|f| f.0.as_str()).collect();
            for p in paths {
                let at = |t: &[(String, Vec<u8>, bool)]| t.iter().find(|f| f.0 == p).cloned();
                if at(&kept) != at(&written) {
                    changed.push(format!("examples/{example}/{p}"));
                }
            }
        }
    }
    assert!(
        changed.is_empty(),
        "committed otherwise than generated (FLOWAPI_BLESS=1 writes them): {changed:#?}"
    );
}

#[test]
fn every_config_passes_its_check() {
    let mut fetched_missing = Vec::new();
    for dir in examples() {
        let example = name(&dir);
        assert!(dir.join("SOURCE.md").is_file(), "{example}: SOURCE.md");
        if !dir.join(spec(&dir)).is_file() {
            fetched_missing.push(example);
            continue;
        }
        let run = flowapi(&dir, &["config", "check", "flowapi-sdk.yml"]);
        let said = stderr(&run) + &String::from_utf8_lossy(&run.stdout);
        assert!(run.status.success(), "{example}: {said}");
        assert_eq!(said.trim(), "flowapi-sdk.yml: no issues", "{example}");
    }
    if !fetched_missing.is_empty() {
        eprintln!("not checked, the description not fetched (examples/fetch.sh): {fetched_missing:?}");
    }
}
