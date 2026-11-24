//! The YAML reader held to the YAML test suite (`conformance/suites/yaml-test-suite`): every
//! case the suite marks invalid is refused; every valid case with one JSON value converts to
//! JSON equal to it; a case of no or several documents is refused (a description is one).
//! A valid case the suite gives no JSON for (its keys are mappings, say) must not panic.
#![expect(
    clippy::expect_used,
    reason = "the helpers fail the test by panicking, as a test does"
)]

use flowapi_cli::yaml;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn suite() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance/suites/yaml-test-suite")
}

/// Every case's directory: one with an `in.yaml`.
fn cases() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![suite()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("a directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                stack.push(path);
            }
        }
        if dir.join("in.yaml").is_file() {
            out.push(dir);
        }
    }
    out.sort();
    out
}

fn name(dir: &Path) -> String {
    let id = dir
        .strip_prefix(suite())
        .unwrap_or(dir)
        .display()
        .to_string();
    let label = std::fs::read_to_string(dir.join("===")).unwrap_or_default();
    format!("{id} ({})", label.trim())
}

/// What one case asks of the reader.
enum Want {
    Refused,
    Equal(Value),
    Any,
}

fn want(dir: &Path) -> Want {
    if dir.join("error").exists() {
        return Want::Refused;
    }
    let Ok(text) = std::fs::read_to_string(dir.join("in.json")) else {
        return Want::Any;
    };
    let values: Vec<Value> = serde_json::Deserializer::from_str(&text)
        .into_iter()
        .collect::<Result<_, _>>()
        .expect("the suite's JSON");
    match <[Value; 1]>::try_from(values) {
        Ok([one]) => Want::Equal(one),
        Err(_) => Want::Refused,
    }
}

/// JSON values equal as JSON reads them: `450.00` is the number `450`.
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y || x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

#[test]
fn the_yaml_test_suite() {
    let mut wrong = Vec::new();
    let (mut refused, mut equal, mut other) = (0, 0, 0);
    for dir in cases() {
        let text = std::fs::read_to_string(dir.join("in.yaml")).expect("in.yaml");
        let got = yaml::to_json(&text);
        match (want(&dir), got) {
            (Want::Refused, Err(_)) => refused += 1,
            (Want::Refused, Ok(j)) => wrong.push(format!("{}: read as {}", name(&dir), j.text)),
            (Want::Equal(value), Ok(j)) => match serde_json::from_str::<Value>(&j.text) {
                Ok(v) if same(&v, &value) => equal += 1,
                Ok(v) => wrong.push(format!("{}: {v} for {value}", name(&dir))),
                Err(e) => wrong.push(format!("{}: not JSON ({e}): {}", name(&dir), j.text)),
            },
            (Want::Equal(value), Err(e)) => {
                wrong.push(format!("{}: refused ({e}), wanted {value}", name(&dir)));
            }
            (Want::Any, _) => other += 1,
        }
    }
    println!("refused {refused}, equal {equal}, without JSON {other}");
    assert!(
        refused + equal > 250,
        "the suite was not found: {}",
        suite().display()
    );
    assert!(
        wrong.is_empty(),
        "{} cases:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
