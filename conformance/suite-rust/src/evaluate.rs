//! Each case's verdict, counted per keyword file. A valid case is sound when its root type
//! reads it and writes back a value equal to it (JSON's equality: `1` and `1.0` are one
//! number, an object's keys are a set); an invalid case is rejected when the type does not
//! read it.

use serde_json::Value;

/// What happened to one case.
pub(crate) enum Verdict {
    /// Read, and written back as this text.
    Read(String),
    /// Not read, with serde's message.
    Refused(String),
}

/// A group that did not reach its cases.
pub(crate) struct Failure {
    pub(crate) group: usize,
    pub(crate) description: String,
    pub(crate) message: String,
}

/// A valid case the type refused or changed.
pub(crate) struct Unsound {
    pub(crate) group: usize,
    pub(crate) group_description: String,
    pub(crate) description: String,
    pub(crate) data: String,
    pub(crate) error: String,
}

/// One keyword file's counts.
#[derive(Default)]
pub(crate) struct FileReport {
    pub(crate) name: String,
    pub(crate) groups: usize,
    pub(crate) skipped: usize,
    pub(crate) generated: usize,
    pub(crate) generation_errors: Vec<Failure>,
    pub(crate) invalid_rust: Vec<Failure>,
    pub(crate) valid: usize,
    pub(crate) valid_accepted: usize,
    pub(crate) invalid: usize,
    pub(crate) invalid_rejected: usize,
    pub(crate) unsound: Vec<Unsound>,
}

/// Whether a value read back equals the case's data, by JSON's equality.
pub(crate) fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => number_eq(&x.to_string(), &y.to_string()),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

fn number_eq(x: &str, y: &str) -> bool {
    if let (Ok(a), Ok(b)) = (x.parse::<i128>(), y.parse::<i128>()) {
        return a == b;
    }
    match (x.parse::<f64>(), y.parse::<f64>()) {
        (Ok(a), Ok(b)) => a == b,
        _ => x == y,
    }
}

/// A case's verdict counted into its file's report.
pub(crate) fn count(
    r: &mut FileReport,
    group: (usize, &str),
    case: &crate::load::Case,
    v: &Verdict,
) {
    let data = serde_json::to_string(&case.data).expect("data prints as JSON");
    let problem = match v {
        Verdict::Read(text) => match serde_json::from_str::<Value>(text) {
            Ok(back) if same(&back, &case.data) => None,
            Ok(_) => Some(format!("read, but written back as {text}")),
            Err(e) => Some(format!("written back as no JSON: {e}")),
        },
        Verdict::Refused(message) => Some(message.clone()),
    };
    if case.valid {
        r.valid += 1;
        match problem {
            None => r.valid_accepted += 1,
            Some(error) => r.unsound.push(Unsound {
                group: group.0,
                group_description: group.1.to_owned(),
                description: case.description.clone(),
                data,
                error,
            }),
        }
    } else {
        r.invalid += 1;
        if matches!(v, Verdict::Refused(_)) {
            r.invalid_rejected += 1;
        }
    }
}

/// A share as a percentage with one decimal, rounded down, or a dash for nothing to share.
pub(crate) fn percent(part: usize, whole: usize) -> String {
    if whole == 0 {
        return String::from("-");
    }
    let tenths = part * 1000 / whole;
    format!("{}.{}", tenths / 10, tenths % 10)
}
