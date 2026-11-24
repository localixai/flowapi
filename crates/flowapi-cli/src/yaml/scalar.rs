//! One YAML scalar as JSON: the YAML 1.2 core schema's tags resolved (null, booleans, integers
//! in base 10, 8 and 16, floats), and every other scalar a JSON string, escaped.

use saphyr_parser::{ScalarStyle, Tag};

/// The core schema's tag handle (`!!` in a document).
const CORE: &str = "tag:yaml.org,2002:";

/// A scalar as JSON: its value, and its form as a mapping key (JSON's keys are strings).
#[derive(Clone)]
pub(super) struct Scalar {
    /// The JSON text of the value.
    pub(super) json: String,
    /// The key it makes: a string's own text, any other value's JSON text (`200`, `true`).
    pub(super) key: String,
}

/// What a scalar resolves to.
enum Kind {
    Null,
    Bool,
    Int,
    Float,
    Str,
}

/// A scalar's JSON, by its tag, or by the core schema when it is plain and untagged. Another
/// tag than the core schema's (a local `!thing`) is read as no tag: JSON has nowhere to keep it.
pub(super) fn resolve(text: &str, style: ScalarStyle, tag: Option<&Tag>) -> Result<Scalar, String> {
    // `!!int`, or written out verbatim: `!<tag:yaml.org,2002:int>`.
    let core = tag.and_then(|t| {
        if t.handle == CORE {
            Some(t.suffix.as_str())
        } else if t.handle.is_empty() {
            t.suffix.strip_prefix(CORE)
        } else {
            None
        }
    });
    // The non-specific `!`: a string, whatever it looks like.
    let non_specific = tag.is_some_and(|t| t.handle.is_empty() && t.suffix == "!");
    let wanted = match core {
        Some("null") => Some(Kind::Null),
        Some("bool") => Some(Kind::Bool),
        Some("int") => Some(Kind::Int),
        Some("float") => Some(Kind::Float),
        Some("map" | "seq") => return Err(format!("`!!{}` on a scalar", core.unwrap_or(""))),
        Some(_) => Some(Kind::Str),
        None if non_specific || !matches!(style, ScalarStyle::Plain) => Some(Kind::Str),
        None => None,
    };
    if let Some(Kind::Str) = wanted {
        return Ok(string(text));
    }
    let (kind, json) = match plain(text)? {
        Some(found) => found,
        None => (Kind::Str, String::new()),
    };
    match (wanted, kind) {
        (None, Kind::Str) => Ok(string(text)),
        (None, _)
        | (Some(Kind::Null), Kind::Null)
        | (Some(Kind::Bool), Kind::Bool)
        | (Some(Kind::Int), Kind::Int)
        | (Some(Kind::Float), Kind::Float | Kind::Int) => Ok(Scalar {
            key: json.clone(),
            json,
        }),
        (Some(_), _) => Err(format!(
            "`!!{}` on `{text}`, which the core schema does not read as one",
            core.unwrap_or("")
        )),
    }
}

/// A string scalar.
fn string(text: &str) -> Scalar {
    let mut json = String::with_capacity(text.len() + 2);
    push_string(&mut json, text);
    Scalar {
        json,
        key: text.to_owned(),
    }
}

/// What the core schema reads a plain scalar as, with its JSON; `None` for a string.
fn plain(text: &str) -> Result<Option<(Kind, String)>, String> {
    Ok(Some(match text {
        "" | "~" | "null" | "Null" | "NULL" => (Kind::Null, String::from("null")),
        "true" | "True" | "TRUE" => (Kind::Bool, String::from("true")),
        "false" | "False" | "FALSE" => (Kind::Bool, String::from("false")),
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" | "-.inf" | "-.Inf" | "-.INF"
        | ".nan" | ".NaN" | ".NAN" => {
            return Err(format!("`{text}` is a float JSON has no number for"));
        }
        _ => {
            if let Some(json) = radix(text, "0x", 16)?.or(radix(text, "0o", 8)?) {
                (Kind::Int, json)
            } else if let Some(json) = decimal(text) {
                (Kind::Int, json)
            } else if let Some(json) = float(text) {
                (Kind::Float, json)
            } else {
                return Ok(None);
            }
        }
    }))
}

/// `0x1F` or `0o17` in base 10.
fn radix(text: &str, prefix: &str, base: u32) -> Result<Option<String>, String> {
    let Some(digits) = text.strip_prefix(prefix) else {
        return Ok(None);
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(base)) {
        return Ok(None);
    }
    u128::from_str_radix(digits, base)
        .map(|n| Some(n.to_string()))
        .map_err(|e| format!("`{text}` is an integer too large to read: {e}"))
}

/// `[-+]?[0-9]+`, its leading zeros and `+` taken off; any length.
fn decimal(text: &str) -> Option<String> {
    let (sign, digits) = sign(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(format!("{sign}{}", whole(digits)))
}

/// `[-+]?(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?` as a JSON number: a leading `0` before
/// a bare `.`, a `0` after a bare `.`, and no `+` in front.
fn float(text: &str) -> Option<String> {
    let (minus, rest) = sign(text);
    let (mantissa, exponent) = match rest.find(['e', 'E']) {
        Some(at) => (rest.get(..at)?, Some(rest.get(at + 1..)?)),
        None => (rest, None),
    };
    let (int, fraction) = match mantissa.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (mantissa, None),
    };
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || !fraction.is_none_or(digits) {
        return None;
    }
    if int.is_empty() && fraction.is_none_or(str::is_empty) {
        return None;
    }
    let mut json = format!("{minus}{}", whole(int));
    if let Some(f) = fraction {
        json.push('.');
        json.push_str(if f.is_empty() { "0" } else { f });
    }
    if let Some(e) = exponent {
        let (e_sign, e_digits) = sign(e);
        if e_digits.is_empty() || !digits(e_digits) {
            return None;
        }
        json.push('e');
        json.push_str(e_sign);
        json.push_str(e_digits);
    }
    Some(json)
}

/// The sign JSON keeps (`-` or none) and the rest.
fn sign(text: &str) -> (&'static str, &str) {
    if let Some(rest) = text.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", text.strip_prefix('+').unwrap_or(text))
    }
}

/// Digits without their leading zeros; `0` for none.
fn whole(digits: &str) -> &str {
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}

/// Appends `text` as a JSON string: quotes, backslashes and control characters escaped, the
/// rest as it is (JSON text is UTF-8).
pub(super) fn push_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if u32::from(c) < 0x20 => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let n = usize::from(u8::try_from(c).unwrap_or(0));
                out.push_str("\\u00");
                out.push(char::from(HEX.get(n >> 4).copied().unwrap_or(b'0')));
                out.push(char::from(HEX.get(n & 15).copied().unwrap_or(b'0')));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
