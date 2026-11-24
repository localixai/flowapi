//! The suite as read from its pinned copy: one file per keyword, each a list of groups (a schema
//! and its cases), each group marked with whether it applies to JSON Schema 2020-12.

use serde_json::Value;
use std::path::Path;

/// The dialect under test, as the suite's `compatibility` field numbers it.
const DRAFT_2020: u32 = 2020;

/// One case: data, and whether the schema accepts it.
pub(crate) struct Case {
    pub(crate) description: String,
    pub(crate) data: Value,
    pub(crate) valid: bool,
}

/// Whether a group is run, or why not.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Admission {
    Run,
    /// Its `compatibility` leaves 2020-12 out.
    Dialect,
    /// It is tagged `format-assertion`: 2020-12 reads `format` as an annotation by default.
    Tags,
}

/// One group: a schema, the schemas it references by retrieval URI, its cases.
pub(crate) struct Group {
    pub(crate) index: usize,
    pub(crate) description: String,
    pub(crate) schema: Value,
    pub(crate) external: Vec<(String, Value)>,
    pub(crate) cases: Vec<Case>,
    pub(crate) admission: Admission,
}

/// One keyword file.
pub(crate) struct SuiteFile {
    /// The file's name without `.json`.
    pub(crate) name: String,
    pub(crate) groups: Vec<Group>,
}

/// Every top-level file of the copy, sorted by name.
pub(crate) fn suite(dir: &Path) -> Vec<SuiteFile> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .expect("the suite's copy is readable")
        .map(|e| e.expect("a directory entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths.iter().map(|p| file(p)).collect()
}

fn file(path: &Path) -> SuiteFile {
    let text = std::fs::read_to_string(path).expect("a suite file is readable");
    let json: Value = serde_json::from_str(&text).expect("a suite file is JSON");
    let name = path
        .file_stem()
        .expect("a file name")
        .to_string_lossy()
        .into_owned();
    let groups = json["tests"].as_array().expect("`tests` is an array");
    SuiteFile {
        name,
        groups: groups
            .iter()
            .enumerate()
            .map(|(i, g)| group(i, g))
            .collect(),
    }
}

fn group(index: usize, g: &Value) -> Group {
    let text = |v: &Value| v.as_str().unwrap_or("").to_owned();
    let tagged = g["tags"]
        .as_array()
        .is_some_and(|t| t.iter().any(|x| x == "format-assertion"));
    let admission = if !admits(g["compatibility"].as_str(), DRAFT_2020) {
        Admission::Dialect
    } else if tagged {
        Admission::Tags
    } else {
        Admission::Run
    };
    let external = g["externalSchemas"]
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let cases = g["tests"]
        .as_array()
        .expect("a group's `tests` is an array");
    Group {
        index,
        description: text(&g["description"]),
        schema: g["schema"].clone(),
        external,
        cases: cases
            .iter()
            .map(|c| Case {
                description: text(&c["description"]),
                data: c["data"].clone(),
                valid: c["valid"].as_bool().expect("`valid` is a boolean"),
            })
            .collect(),
        admission,
    }
}

/// Whether a `compatibility` value admits `version`: the suite README's algorithm, term by term
/// until one reaches the version under test. No value admits every dialect.
pub(crate) fn admits(compatibility: Option<&str>, version: u32) -> bool {
    let Some(terms) = compatibility else {
        return true;
    };
    let mut ok = true;
    for term in terms.split(',').map(str::trim) {
        let (op, number) = if let Some(n) = term.strip_prefix("<=") {
            ("<=", n)
        } else if let Some(n) = term.strip_prefix('=') {
            ("=", n)
        } else {
            ("", term)
        };
        let Ok(v) = number.parse::<u32>() else {
            return false;
        };
        ok = match op {
            "<=" => version <= v,
            "=" => version == v,
            _ => version >= v,
        };
        if version <= v {
            break;
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::admits;

    #[test]
    fn compatibility_follows_the_readme() {
        assert!(admits(None, 2020));
        assert!(admits(Some("7"), 2020));
        assert!(!admits(Some("<=2019"), 2020));
        assert!(admits(Some("=2020"), 2020));
        assert!(!admits(Some("6,<=2019"), 2020));
        assert!(admits(Some("=3,7,<=2020"), 2020));
        assert!(admits(Some("<=3,7"), 2020));
        assert!(!admits(Some("9999"), 2020));
        assert!(!admits(Some("4,<=6,9999"), 2020));
        assert!(admits(Some("3,<=2019"), 2019));
    }
}
