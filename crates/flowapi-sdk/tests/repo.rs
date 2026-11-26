//! The repository printed for each case of `tests/client/` (`flowapi_sdk::repo_files`): every
//! file but `src/` (the client, kept by `--test client`) kept under `tests/repo/<case>/`, the lock
//! with them, what was found in `issues.txt`. `FLOWAPI_BLESS=1 cargo test -p flowapi-sdk --test
//! repo` writes them anew; a change to one is read in review like code, the README above all.
//! Beside them: the same input prints the same bytes (G4), the lock reads back as written and
//! lists every generated file, the release tool's files are printed only without a previous lock,
//! a license flowapi has no text of is an error, and every text file opens with the header.
#![expect(
    clippy::expect_used,
    reason = "a fixture that cannot be read fails the test by panicking"
)]

use flowapi::Documents;
use flowapi_sdk::{Input, Lock, Owner, Repo};
use std::borrow::Cow;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/client")
}

fn kept() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/repo")
}

/// The repository of a description (JSON) and a config's YAML, with what was found.
fn generate(spec: &str, config: &str, previous: Option<&Lock>) -> (Repo, String) {
    let mut docs = Documents::new();
    docs.add("spec.json", spec).expect("JSON");
    let yaml = flowapi_yaml::Text::read("flowapi-sdk.yml", Cow::Borrowed(config)).expect("YAML");
    let target = flowapi_ts::TypeScript::new("src/api.ts").with_sdk();
    let (program, _) = flowapi::program(&docs, &target);
    let program = program.expect("no error");
    let input = Input {
        config: yaml.json(),
        config_bytes: config.as_bytes(),
        spec_bytes: spec.as_bytes(),
    };
    let repo = flowapi_sdk::repo_files(&docs, &program, &input, previous);
    let mut found = flowapi_sdk::issues_text(&repo.issues, "flowapi-sdk.yml", &|o| yaml.place(o));
    for d in repo.diagnostics.all() {
        let _ = writeln!(found, "{d}");
    }
    (repo, found)
}

fn case(name: &str) -> (String, String) {
    let dir = cases().join(name);
    (
        std::fs::read_to_string(dir.join("spec.json")).expect("the description"),
        std::fs::read_to_string(dir.join("flowapi-sdk.yml")).expect("the config"),
    )
}

#[test]
fn repositories_print_as_kept() {
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut changed = Vec::new();
    for name in [
        "auth", "edge", "github", "hosts", "shop", "swagger", "tokens",
    ] {
        let (spec, config) = case(name);
        let (repo, found) = generate(&spec, &config, None);
        assert!(!found.contains("error["), "{name}: {found}");
        let (again, _) = generate(&spec, &config, None);
        assert_eq!(repo.files, again.files, "{name}: G4, printed twice differs");
        assert_eq!(repo.lock, again.lock, "{name}: G4, the lock twice differs");
        let dir = kept().join(name);
        let mut out = vec![
            (dir.join("issues.txt"), found),
            (dir.join(".flowapi/lock.json"), repo.lock.json()),
        ];
        for f in repo.files.iter().filter(|f| !f.path.starts_with("src/")) {
            out.push((dir.join(&f.path), f.text.clone()));
        }
        for (path, text) in out {
            if bless {
                std::fs::create_dir_all(path.parent().expect("a folder")).expect("made");
                std::fs::write(&path, &text).expect("written");
            } else if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                changed.push(path.display().to_string());
            }
        }
    }
    assert!(
        changed.is_empty(),
        "printed otherwise than kept: {changed:#?}"
    );
}

#[test]
fn the_lock_lists_every_generated_file_and_reads_back() {
    let (spec, config) = case("shop");
    let (repo, _) = generate(&spec, &config, None);
    let lock = &repo.lock;
    assert_eq!(Lock::read(&lock.json()).as_ref(), Ok(lock));
    let generated: Vec<&str> = repo
        .files
        .iter()
        .filter(|f| f.owner == Owner::Generated)
        .map(|f| f.path.as_str())
        .collect();
    let listed: Vec<&str> = lock.files.keys().map(String::as_str).collect();
    assert_eq!(
        generated, listed,
        "the lock lists the generated files, sorted"
    );
    for f in &repo.files {
        if let Some(hash) = lock.files.get(&f.path) {
            assert_eq!(
                *hash,
                flowapi_sdk::sha256_hex(f.text.as_bytes()),
                "{}",
                f.path
            );
        }
    }
    assert!(!lock.files.contains_key("CHANGELOG.md"));
    assert!(!lock.files.contains_key("flowapi-sdk.yml"));
    assert_eq!(lock.spec_sha256, flowapi_sdk::sha256_hex(spec.as_bytes()));
    assert_eq!(
        lock.config_sha256,
        flowapi_sdk::sha256_hex(config.as_bytes())
    );
    assert_eq!(lock.configured_endpoints, 14);
    assert!(Lock::read("{\"version\": 2}").is_err());
    let climbing = lock.json().replace("\"api.md\"", "\"../api.md\"");
    assert!(
        Lock::read(&climbing).is_err_and(|e| e.contains("../api.md")),
        "{climbing}"
    );
}

#[test]
fn the_release_tools_files_are_printed_once() {
    let (spec, config) = case("shop");
    let (first, _) = generate(&spec, &config, None);
    let seeds: Vec<&str> = first
        .files
        .iter()
        .filter(|f| f.owner == Owner::Seed)
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(seeds, [".release-please-manifest.json", "CHANGELOG.md"]);
    let (next, _) = generate(&spec, &config, Some(&first.lock));
    assert!(next.files.iter().all(|f| f.owner == Owner::Generated));
    assert_eq!(
        next.lock, first.lock,
        "the lock does not depend on the previous one"
    );
}

#[test]
fn a_license_without_a_text_is_an_error() {
    let (spec, config) = case("shop");
    let config = config.replace("license: MIT", "license: GPL-3.0-only");
    let (repo, found) = generate(&spec, &config, None);
    assert!(repo.files.is_empty());
    assert!(found.contains("error[repo.license]"), "{found}");
    let config = config.replace(", license: GPL-3.0-only", "");
    let (repo, found) = generate(&spec, &config, None);
    assert!(!found.contains("error["), "{found}");
    assert!(repo.files.iter().all(|f| f.path != "LICENSE"));
}

#[test]
fn every_text_file_opens_with_the_header() {
    const HEADER: &str = "Generated by flowapi";
    let (spec, config) = case("shop");
    let (repo, _) = generate(&spec, &config, None);
    let bare = [".keep", "LICENSE", "CHANGELOG.md"];
    for f in &repo.files {
        let name = f.path.rsplit('/').next().unwrap_or("");
        if f.path.ends_with(".json") || bare.contains(&name) {
            continue;
        }
        let head: String = f.text.lines().take(2).collect::<Vec<_>>().join("\n");
        assert!(head.contains(HEADER), "{} opens without the header", f.path);
    }
    let scripts: Vec<&str> = repo
        .files
        .iter()
        .filter(|f| f.executable)
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(
        scripts,
        [
            "scripts/build",
            "scripts/check",
            "scripts/lint",
            "scripts/mock",
            "scripts/test"
        ]
    );
}

#[test]
fn the_package_version_is_exact_and_printed() {
    let (spec, config) = case("shop");
    let (_, found) = generate(
        &spec,
        &config.replace("version: 1.2.0", "version: 1.2.x"),
        None,
    );
    assert!(found.contains("error[config.package.version]"), "{found}");
    let (repo, found) = generate(&spec, &config.replace("version: 1.2.0, ", ""), None);
    assert!(!found.contains("error["), "{found}");
    let package = repo
        .files
        .iter()
        .find(|f| f.path == "package.json")
        .expect("printed");
    assert!(
        package.text.contains("\"version\": \"0.1.0\""),
        "{}",
        package.text
    );
}

/// The README's authentication section has a row for every security scheme the client knows
/// (the table's `flowapiSecuritySchemes`: the description's and the config's), whatever its kind.
#[test]
fn the_readme_names_every_security_scheme() {
    for name in ["auth", "github", "shop", "swagger", "tokens"] {
        let (spec, config) = case(name);
        let (repo, _) = generate(&spec, &config, None);
        let text = |path: &str| {
            repo.files
                .iter()
                .find(|f| f.path == path)
                .map(|f| f.text.clone())
                .expect(path)
        };
        let api = text("src/api.ts");
        let table = api
            .split("export const flowapiSecuritySchemes = {\n")
            .nth(1)
            .and_then(|t| t.split("} as const;").next())
            .expect("the schemes");
        let schemes: Vec<&str> = table
            .lines()
            .filter_map(|l| l.trim().strip_prefix('"')?.split('"').next())
            .collect();
        assert!(!schemes.is_empty(), "{name}: no scheme");
        let readme = text("README.md");
        let section = readme
            .split("## Authentication\n")
            .nth(1)
            .and_then(|t| t.split("\n## ").next())
            .expect("an authentication section");
        for s in schemes {
            assert!(
                section.contains(&format!("| `{s}` |")),
                "{name}: `{s}` is not in:\n{section}"
            );
        }
    }
}
