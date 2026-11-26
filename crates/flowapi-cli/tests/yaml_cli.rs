//! The `flowapi` command on YAML: a description generates the same TypeScript as its JSON twin,
//! a `$ref` to a YAML file is read, and a diagnostic names the YAML line.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/yaml")
        .join(name)
}

/// A fresh directory for one test's files.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("yaml_cli")
        .join(test);
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn flowapi(input: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_flowapi"))
        .arg(input)
        .output()
        .expect("the command runs")
}

fn stdout(out: &Output) -> String {
    assert!(
        out.status.success(),
        "flowapi failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).expect("UTF-8")
}

#[test]
fn a_yaml_description_generates_what_its_json_twin_does() {
    let yaml = stdout(&flowapi(&fixture("petstore.yaml")));
    let json = stdout(&flowapi(&fixture("petstore.json")));
    assert!(yaml.contains("export type Pet"), "{yaml}");
    assert!(yaml.contains("export type Paths"), "{yaml}");
    assert_eq!(yaml, json);
}

#[test]
fn yaml_is_known_by_its_text_under_another_name() {
    let dir = scratch("by_text");
    let path = dir.join("description");
    std::fs::copy(fixture("petstore.yaml"), &path).expect("a copy");
    assert_eq!(
        stdout(&flowapi(&path)),
        stdout(&flowapi(&fixture("petstore.json")))
    );
}

#[test]
fn a_ref_to_a_yaml_file_is_read() {
    let dir = scratch("external");
    let main = "\
openapi: 3.1.0
info: {title: t, version: '1'}
paths: {}
components:
  schemas:
    Pet:
      $ref: './pet.yml#/Pet'
";
    let pet = "\
Pet:
  type: object
  required: [name]
  properties:
    name: {type: string}
";
    std::fs::write(dir.join("main.yaml"), main).expect("written");
    std::fs::write(dir.join("pet.yml"), pet).expect("written");
    let ts = stdout(&flowapi(&dir.join("main.yaml")));
    assert!(ts.contains("name: string"), "{ts}");
}

#[test]
fn a_diagnostic_names_the_yaml_line() {
    let dir = scratch("diagnostic");
    let text = "\
openapi: 3.1.0
info:
  title: t
  version: '1'
paths: {}
components:
  schemas:
    Pet:
      type: object
      properties:
        owner:
          $ref: '#/components/schemas/Missing'
";
    let path = dir.join("broken.yaml");
    std::fs::write(&path, text).expect("written");
    let out = flowapi(&path);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let at = format!("{}:12:17", path.display());
    assert!(stderr.contains(&at), "wanted {at} in:\n{stderr}");
}

#[test]
fn yaml_syntax_errors_name_the_line() {
    let dir = scratch("syntax");
    let path = dir.join("bad.yaml");
    std::fs::write(&path, "openapi: 3.1.0\ninfo: [a, b\npaths: {}\n").expect("written");
    let out = flowapi(&path);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // A diagnostic as the core's print (G8): its code, then the YAML line and column. The npm
    // package prints the same bytes (npm/test/parity.test.ts).
    assert!(stderr.starts_with("error[yaml.syntax]: "), "{stderr}");
    let at = format!("\n  at {}:3:6\n", path.display());
    assert!(stderr.ends_with(&at), "wanted {at:?} in:\n{stderr}");
}

#[test]
fn an_alias_bomb_is_refused_by_the_command() {
    let dir = scratch("bomb");
    let mut text = String::from("a: &a [x, x, x, x, x, x, x, x, x, x]\n");
    let mut previous = String::from("a");
    for level in 0..9 {
        let name = format!("l{level}");
        let items = vec![format!("*{previous}"); 10].join(", ");
        writeln!(text, "{name}: &{name} [{items}]").unwrap();
        previous = name;
    }
    let path = dir.join("bomb.yaml");
    std::fs::write(&path, text).expect("written");
    let out = flowapi(&path);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("alias bomb"));
}
