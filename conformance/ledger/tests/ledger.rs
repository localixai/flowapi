//! The ledger's own rules, on small inputs.

use flowapi_ledger::markers::{Kind, ids_in, markers_in};
use flowapi_ledger::scope::fits;
use flowapi_ledger::sources::{Format, SOURCES};
use flowapi_ledger::terms::{lead_term, sentence_slug, terms};
use flowapi_ledger::text::{clean_inline, sentences};
use flowapi_ledger::{extract, markdown};

#[test]
fn quoted_terms_are_not_requirements() {
    let boilerplate = r#"The key words "MUST", "MUST NOT", "REQUIRED" and "MAY" are to be interpreted as described in BCP 14."#;
    assert!(terms(boilerplate).is_empty());
    assert_eq!(
        terms("A value MUST NOT be empty and MAY be `MUST`."),
        ["MUST NOT", "MAY"]
    );
    assert_eq!(
        terms("Use of this field is NOT RECOMMENDED."),
        ["NOT RECOMMENDED"]
    );
    assert!(terms("The MUSTARD field must be set.").is_empty());
}

#[test]
fn the_strongest_term_leads_the_id() {
    assert_eq!(lead_term(&["MAY", "SHOULD NOT", "MUST"]), "must");
    assert_eq!(lead_term(&["MAY", "NOT RECOMMENDED"]), "not-recommended");
    assert_eq!(
        sentence_slug("Parameter Objects MUST include either a `content` field", 6),
        "parameter-objects-include-either-content-field"
    );
}

#[test]
fn sentences_split_at_periods_but_not_abbreviations_or_code() {
    let text =
        clean_inline("**REQUIRED**. The name, e.g. `a. B`. See [the object](#schema-object). Done");
    assert_eq!(
        sentences(&text),
        [
            "REQUIRED.",
            "The name, e.g. `a. B`.",
            "See the object.",
            "Done"
        ]
    );
    assert_eq!(sentences("One.<br>Two"), ["One.<br>Two"]);
    assert_eq!(sentences(&clean_inline("One.<br>two")), ["One.", "two"]);
}

#[test]
fn a_field_row_and_its_sentences_get_ids_under_the_object() {
    let text = "# OAS\n\n## Specification\n\n#### Parameter Object\n\n##### Fixed Fields\n\nParameter Objects MUST include either a `content` field or a `schema` field.\n\n| Field Name | Type | Description |\n| ---- | :----: | ---- |\n| <a name=\"parameter-style\"></a>style | `string` | How it is serialized. It MUST be one of the values. |\n\n```yaml\n# MUST in an example\n```\n";
    let source = SOURCES
        .iter()
        .find(|s| s.format == Format::Markdown && s.version == "3.1.2")
        .copied()
        .unwrap();
    let found = extract::requirements(&markdown::read(text), &source);
    let ids: Vec<&str> = found.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "oas-3.1.parameter-object.fixed-fields.must-parameter-objects-include-either-content-field",
            "oas-3.1.parameter-object.style",
            "oas-3.1.parameter-object.style.must-one-values",
        ]
    );
    assert_eq!(
        found.get(1).map(|r| r.anchor.as_str()),
        Some("parameter-style")
    );
}

#[test]
fn a_rule_fits_an_id_or_a_prefix_ending_at_a_dot() {
    assert!(fits(
        "oas-*.schema-object",
        "oas-3.1.schema-object.discriminator"
    ));
    assert!(fits("oas-*.schema-object", "oas-3.2.schema-object"));
    assert!(!fits("oas-*.schema-object", "oas-3.2.schema-objects"));
    assert!(!fits("oas-*.schema-object", "oas-x.schema-object"));
    assert!(fits(
        "jsonschema-2020-12.core",
        "jsonschema-2020-12.core.$ref"
    ));
}

#[test]
fn markers_name_spec_ids_only() {
    assert_eq!(
        ids_in("// req: oas-3.1.schema-object.discriminator."),
        ["oas-3.1.schema-object.discriminator"]
    );
    assert_eq!(
        ids_in("\"req: jsonschema-2020-12.core.$ref\""),
        ["jsonschema-2020-12.core.$ref"]
    );
    assert!(ids_in("let req: Request = prereq: oas-3.1.x;").is_empty());
}

#[test]
fn lossy_markers_carry_their_reason() {
    let line = "- lossy: jsonschema-2020-12.validation.minLength — no TS type has a length";
    assert_eq!(
        markers_in(line),
        [(
            Kind::Lossy,
            "jsonschema-2020-12.validation.minLength".to_owned(),
            "no TS type has a length".to_owned()
        )]
    );
    assert!(ids_in(line).is_empty());
    let both = "// req: oas-3.1.schema-object.format - kept lossy: oas-3.0.schema-object.format";
    let found = markers_in(both);
    assert_eq!(found.len(), 2);
    assert_eq!(
        found.first().map(|m| (m.0, m.2.as_str())),
        Some((Kind::Covered, "kept"))
    );
    assert_eq!(
        found.get(1).map(|m| (m.0, m.2.as_str())),
        Some((Kind::Lossy, ""))
    );
    assert!(markers_in("let lossy: Vec<Keyword> = Vec::new();").is_empty());
}
