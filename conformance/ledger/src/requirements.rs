//! A requirement, and `requirements.json` written and read.

use serde_json::Value;

/// One normative statement of a specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The stable id markers and `scope.json` name.
    pub id: String,
    /// `openapi` or `json-schema`.
    pub spec: String,
    /// The release: `3.1.2`, `2020-12`.
    pub version: String,
    /// The pinned text, relative to `conformance/specs/`.
    pub source: String,
    /// The line in that text (1-based).
    pub line: usize,
    /// The heading path, titles joined by ` > `.
    pub section: String,
    /// The anchor in the rendered specification.
    pub anchor: String,
    /// `MUST`, `SHOULD`, `MAY` (a sentence), `field` (a row of a field table) or `keyword`.
    pub level: String,
    /// The BCP 14 terms of a sentence, in order; empty for a field or a keyword.
    pub terms: Vec<String>,
    /// The sentence, or the field's description, or the keyword's section title.
    pub text: String,
}

fn string(value: &str) -> String {
    Value::String(value.to_owned()).to_string()
}

/// `requirements.json`: the texts read, then every requirement sorted by id, one field a line.
pub fn render(requirements: &[Requirement], sources: &[(String, String, String)]) -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"about\": ");
    out.push_str(&string(
        "Every normative statement of the pinned texts in specs/ (SOURCE.md). Written by `cargo run --release --bin extract` in conformance/ledger/; never edited by hand.",
    ));
    out.push_str(",\n  \"sources\": [\n");
    let listed: Vec<String> = sources
        .iter()
        .map(|(spec, version, file)| {
            format!(
                "    {{ \"spec\": {}, \"version\": {}, \"file\": {} }}",
                string(spec),
                string(version),
                string(file)
            )
        })
        .collect();
    out.push_str(&listed.join(",\n"));
    out.push_str("\n  ],\n  \"requirements\": [\n");
    let entries: Vec<String> = requirements.iter().map(entry).collect();
    out.push_str(&entries.join(",\n"));
    out.push_str("\n  ]\n}\n");
    out
}

fn entry(requirement: &Requirement) -> String {
    let terms: Vec<String> = requirement.terms.iter().map(|t| string(t)).collect();
    let fields = [
        ("id", string(&requirement.id)),
        ("spec", string(&requirement.spec)),
        ("version", string(&requirement.version)),
        ("source", string(&requirement.source)),
        ("line", requirement.line.to_string()),
        ("section", string(&requirement.section)),
        ("anchor", string(&requirement.anchor)),
        ("level", string(&requirement.level)),
        ("terms", format!("[{}]", terms.join(", "))),
        ("text", string(&requirement.text)),
    ];
    let lines: Vec<String> = fields
        .iter()
        .map(|(key, value)| format!("      \"{key}\": {value}"))
        .collect();
    format!("    {{\n{}\n    }}", lines.join(",\n"))
}

/// Reads `requirements.json`.
pub fn parse(source: &str) -> Result<Vec<Requirement>, String> {
    let value: Value =
        serde_json::from_str(source).map_err(|e| format!("requirements.json: {e}"))?;
    let list = value
        .get("requirements")
        .and_then(Value::as_array)
        .ok_or("requirements.json: no `requirements`")?;
    list.iter().map(one).collect()
}

fn one(value: &Value) -> Result<Requirement, String> {
    let text = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(format!("a requirement has no `{key}`"))
    };
    let terms = value
        .get("terms")
        .and_then(Value::as_array)
        .ok_or("a requirement has no `terms`")?;
    Ok(Requirement {
        id: text("id")?,
        spec: text("spec")?,
        version: text("version")?,
        source: text("source")?,
        line: value
            .get("line")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0),
        section: text("section")?,
        anchor: text("anchor")?,
        level: text("level")?,
        terms: terms
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        text: text("text")?,
    })
}
