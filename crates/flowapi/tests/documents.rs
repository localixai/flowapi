//! Several documents given to the core at once, through the public API only: a description whose
//! references reach other documents, targets found anywhere in them, a meta-schema known by its
//! `$id`, and a target written outside the core. A small target here prints each declaration's
//! shape, so no test depends on a real target's output.

use flowapi::graph::{Branches, Kind};
use flowapi::id::TypeId;
use flowapi::lower::Capabilities;
use flowapi::{DocKind, Documents, File, IdentifierRules, Output, Program, Target};
use std::fmt::Write as _;

/// Prints `Name = shape` per declaration: a shape is `any`, `never`, a declared name, the kinds
/// of a value (`string`, `number`, `object{a, b?}`, ...), or a union or intersection of shapes.
struct Shapes;

fn kinds(p: &Program<'_>, b: &Branches<'_>, depth: usize) -> String {
    let mut out = Vec::new();
    if b.null {
        out.push(String::from("null"));
    }
    if b.boolean.allow_true || b.boolean.allow_false {
        out.push(String::from("boolean"));
    }
    if b.number.is_some() {
        out.push(String::from("number"));
    }
    if let Some(s) = &b.string {
        match &s.lits {
            Some(lits) => out.extend(lits.iter().map(|l| l.raw.to_owned())),
            None => out.push(String::from("string")),
        }
    }
    if b.array.is_some() {
        out.push(String::from("array"));
    }
    if let Some(o) = &b.object {
        let props: Vec<String> = o
            .props
            .iter()
            .map(|prop| {
                let opt = if prop.required { "" } else { "?" };
                format!("{}{opt}: {}", prop.name, shape(p, prop.ty, depth + 1))
            })
            .collect();
        out.push(format!("object{{{}}}", props.join(", ")));
    }
    out.join(" | ")
}

fn shape(p: &Program<'_>, id: TypeId, depth: usize) -> String {
    let at = p.resolve(id);
    if depth > 0
        && let Some(name) = p.name_of(at)
    {
        return name.to_owned();
    }
    if depth > 8 {
        return String::from("…");
    }
    let join = |ids: &[TypeId], sep: &str| {
        ids.iter()
            .map(|t| shape(p, *t, depth + 1))
            .collect::<Vec<_>>()
            .join(sep)
    };
    match &p.get(at).kind {
        Kind::Any => String::from("any"),
        Kind::Never => String::from("never"),
        Kind::Value(b) => kinds(p, b, depth),
        Kind::Union(ids) => join(ids, " | "),
        Kind::Intersection(ids) => join(ids, " & "),
        Kind::Alias(next) => shape(p, *next, depth + 1),
    }
}

impl Target for Shapes {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            intersections: true,
            unions: true,
        }
    }
    fn identifiers(&self) -> IdentifierRules {
        IdentifierRules { reserved: &[] }
    }
    fn emit(&self, program: &Program<'_>) -> Output {
        let mut text = String::new();
        for d in program.decls() {
            let _ = writeln!(text, "{} = {}", d.name, shape(program, d.ty, 0));
        }
        Output {
            files: vec![File {
                path: String::from("shapes.txt"),
                text,
            }],
            diagnostics: flowapi::Diagnostics::default(),
        }
    }
}

/// The shapes generated from the documents, the first being the one generated from, or the
/// diagnostics' codes when there is an error.
fn run(documents: &[(&str, &str)]) -> Result<String, Vec<String>> {
    let mut docs = Documents::new();
    for (uri, text) in documents {
        if let Err(d) = docs.add(*uri, text) {
            return Err(vec![d.to_string()]);
        }
    }
    let (output, diags) = flowapi::generate(&docs, &Shapes);
    match output {
        Some(o) => Ok(o.files.into_iter().map(|f| f.text).collect()),
        None => Err(diags.all().iter().map(|d| format!("{d}")).collect()),
    }
}

// req: oas-3.2.openapi-object.parsing-documents.must-not-implementations-treat-reference-unresolvable-before-completely — a $ref to a document added after the referring one resolves; only a document never given leaves it unresolvable
#[test]
fn a_reference_to_a_document_added_later_resolves() {
    let a = r##"{"$ref":"b.json#/$defs/n"}"##;
    let b = r#"{"$defs":{"n":{"type":"number"}}}"#;
    assert_eq!(
        run(&[("https://ex.com/a.json", a), ("https://ex.com/b.json", b)]),
        Ok(String::from("Root = number\n"))
    );
    let alone = run(&[("https://ex.com/a.json", a)]).expect_err("b.json was never given");
    assert!(alone.iter().any(|d| d.contains("b.json")), "{alone:?}");
    let oas = r##"{"openapi":"3.2.0","info":{"title":"t","version":"1"},
      "components":{"schemas":{"A":{"$ref":"b.json#/$defs/n"}}}}"##;
    assert_eq!(
        run(&[
            ("https://ex.com/api.json", oas),
            ("https://ex.com/b.json", b)
        ]),
        Ok(String::from("A = number\n"))
    );
}

// req: oas-3.2.openapi-object.parsing-documents.must-each-document-oad-fully-parsed-order — targets named by an $id and an $anchor deep inside another document, and by another description's $self, are found
#[test]
fn targets_anywhere_in_another_document_are_found() {
    let oas = r#"{"openapi":"3.2.0","info":{"title":"t","version":"1"},"components":{"schemas":{
      "ById":{"$ref":"https://ex.com/inner"},
      "ByAnchor":{"$ref":"https://ex.com/lib.json#here"},
      "BySelf":{"$ref":"https://ex.com/api/common#/components/schemas/C"}}}}"#;
    let lib = r#"{"$defs":{"deep":{"properties":{"x":{"$id":"https://ex.com/inner","type":"boolean"},
      "y":{"$anchor":"here","type":"null"}}}}}"#;
    let common = r#"{"openapi":"3.2.0","$self":"https://ex.com/api/common","info":{"title":"c","version":"1"},
      "components":{"schemas":{"C":{"type":"string"}}}}"#;
    assert_eq!(
        run(&[
            ("https://ex.com/api.json", oas),
            ("https://ex.com/lib.json", lib),
            ("https://files.ex.com/common.json", common),
        ]),
        Ok(String::from(
            "ById = boolean\nByAnchor = null\nBySelf = string\n"
        ))
    );
}

/// An OpenAPI description of `version` whose operation and component reach a second
/// description and a JSON Schema document.
fn distributed(version: &str) -> String {
    format!(
        r#"{{"openapi":"{version}","info":{{"title":"t","version":"1"}},"paths":{{"/pets":{{"get":{{
      "operationId":"list","responses":{{"200":{{"$ref":"common.json#/components/responses/Pets"}}}}}}}}}},
      "components":{{"schemas":{{"Owner":{{"$ref":"owner.schema.json"}}}}}}}}"#
    )
}

// req: oas-3.0.openapi-description-structure.may-openapi-description-oad-made-up-single — a 3.0 description whose response and schema live in other documents
// req: oas-3.1.openapi-description-structure.may-openapi-description-oad-made-up-single — the same in 3.1
// req: oas-3.2.openapi-object.openapi-description-structure.may-oad-made-up-single-document-distributed — the same in 3.2
#[test]
fn a_description_spans_documents() {
    let common = |v: &str| {
        format!(
            r##"{{"openapi":"{v}","info":{{"title":"c","version":"1"}},"components":{{
          "responses":{{"Pets":{{"description":"ok","content":{{"application/json":{{"schema":{{"$ref":"#/components/schemas/Pet"}}}}}}}}}},
          "schemas":{{"Pet":{{"type":"object","required":["name"],"properties":{{"name":{{"type":"string"}}}}}}}}}}}}"##
        )
    };
    let owner = r#"{"type":"object","properties":{"id":{"type":"integer"}}}"#;
    for v in ["3.0.4", "3.1.2", "3.2.0"] {
        let (main, common) = (distributed(v), common(v));
        let shapes = run(&[
            ("https://ex.com/main.json", &main),
            ("https://ex.com/common.json", &common),
            ("https://ex.com/owner.schema.json", owner),
        ])
        .expect("no error");
        assert!(
            shapes.contains("Owner = object{id?: number}"),
            "{v}: {shapes}"
        );
        assert!(shapes.contains("name: string"), "{v}: {shapes}");
        assert!(
            shapes.contains("List = object{responses: "),
            "{v}: {shapes}"
        );
    }
}

// req: oas-3.1.parsing-documents.may-implementations-support-complete-document-parsing-any — a document is an OpenAPI description by its root `openapi` field and a JSON Schema otherwise, by its `$schema`
// req: oas-3.2.appendix-g-parsing-and-resolution-guidance.may-implementations-support-complete-document-parsing-any — the same detection for 3.2
#[test]
fn a_document_is_known_by_its_root() {
    let mut docs = Documents::new();
    let api = docs
        .add(
            "a.json",
            r#"{"openapi":"3.2.0","info":{"title":"t","version":"1"}}"#,
        )
        .expect("JSON");
    let old = docs
        .add(
            "b.json",
            r#"{"$schema":"http://json-schema.org/draft-07/schema#","type":"string"}"#,
        )
        .expect("JSON");
    let plain = docs.add("c.json", r#"{"type":"string"}"#).expect("JSON");
    let kind = |id| docs.get(id).map(|d| d.kind);
    assert!(matches!(kind(api), Some(DocKind::OpenApi(_))));
    assert!(matches!(kind(old), Some(DocKind::Schema(_))));
    assert!(matches!(kind(plain), Some(DocKind::Schema(_))));
    assert_ne!(
        kind(old),
        kind(plain),
        "draft-07 and 2020-12 are told apart"
    );
}

// req: oas-3.2.openapi-object.fixed-fields.not-recommended-implementations-choose-support-referencing-other-uris — flowapi takes the MAY: a description with $self is reached by its $self and by the URI it was given under
#[test]
fn a_description_is_reached_by_self_and_by_its_retrieval_uri() {
    let main = r#"{"openapi":"3.2.0","info":{"title":"t","version":"1"},"components":{"schemas":{
      "BySelf":{"$ref":"https://ex.com/api/common#/components/schemas/C"},
      "ByRetrieval":{"$ref":"https://files.ex.com/common.json#/components/schemas/C"}}}}"#;
    let common = r##"{"openapi":"3.2.0","$self":"https://ex.com/api/common","info":{"title":"c","version":"1"},
      "components":{"schemas":{"C":{"$ref":"#/components/schemas/D"},"D":{"type":"integer"}}}}"##;
    assert_eq!(
        run(&[
            ("https://ex.com/main.json", main),
            ("https://files.ex.com/common.json", common)
        ]),
        Ok(String::from("BySelf = number\nByRetrieval = number\n"))
    );
}

// req: jsonschema-2020-12.core.detecting-a-meta-schema.must-implementations-recognize-schema-meta-schema-being — the document a $schema names by its $id is read as a meta-schema: its required unknown vocabulary is refused
// req: jsonschema-2020-12.core.$vocabulary.must-value-true-implementations-do-not-recognize — the schema declaring that meta-schema is refused, with the $vocabulary code
#[test]
fn a_meta_schema_is_found_by_its_id() {
    let schema = r#"{"$schema":"https://ex.com/meta","type":"string"}"#;
    let meta = r#"{"$id":"https://ex.com/meta","$vocabulary":{
      "https://json-schema.org/draft/2020-12/vocab/core":true,"https://unknown.example/v":true}}"#;
    let refused = run(&[("s.json", schema), ("files/meta.json", meta)]).expect_err("refused");
    assert!(
        refused
            .iter()
            .any(|d| d.contains("error[jsonschema-2020-12.core.$vocabulary]")
                && d.contains("https://unknown.example/v")),
        "{refused:?}"
    );
    // Optional, the unknown vocabulary is passed over; without the validation vocabulary,
    // `type` is an annotation and the root allows every value.
    let optional = meta.replace("/v\":true", "/v\":false");
    assert_eq!(
        run(&[("s.json", schema), ("files/meta.json", &optional)]),
        Ok(String::from("Root = any\n"))
    );
    let validation = optional.replace(
        "\"https://unknown",
        "\"https://json-schema.org/draft/2020-12/vocab/applicator\":true,\"https://json-schema.org/draft/2020-12/vocab/validation\":true,\"https://unknown",
    );
    assert_eq!(
        run(&[("s.json", schema), ("files/meta.json", &validation)]),
        Ok(String::from("Root = string\n"))
    );
}

/// A target outside the core that reads a specification extension: it holds the documents the
/// host gave the core, and finds each declaration's schema by the type's origin.
struct Brands<'d, 't> {
    docs: &'d Documents<'t>,
}

impl Target for Brands<'_, '_> {
    fn capabilities(&self) -> Capabilities {
        Shapes.capabilities()
    }
    fn identifiers(&self) -> IdentifierRules {
        Shapes.identifiers()
    }
    fn emit(&self, program: &Program<'_>) -> Output {
        let mut text = String::new();
        for d in program.decls() {
            let brand = program.get(d.ty).origin.and_then(|o| {
                let tree = &self.docs.get(o.doc)?.tree;
                let value = tree.get(o.node, "x-brand")?;
                Some(tree.as_str(value)?.into_owned())
            });
            if let Some(brand) = brand {
                let _ = writeln!(text, "{} is branded {brand}", d.name);
            }
        }
        Output {
            files: vec![File {
                path: String::from("brands.txt"),
                text,
            }],
            diagnostics: flowapi::Diagnostics::default(),
        }
    }
}

// req: oas-3.0.specification-extensions.recommended-therefore-implementations-designed-extensibility-support-community — a target written outside the core reads an x- field through a type's origin; the core ignores the field
// req: oas-3.1.specification-extensions.recommended-therefore-implementations-designed-extensibility-support-community — the same, 3.1
// req: oas-3.2.specification-extensions.recommended-therefore-implementations-designed-extensibility-support-community — the same, 3.2
#[test]
fn a_target_outside_the_core_reads_an_extension() {
    for v in ["3.0.4", "3.1.2", "3.2.0"] {
        let text = format!(
            r#"{{"openapi":"{v}","info":{{"title":"t","version":"1"}},"x-tool":{{"any":["json"]}},
          "components":{{"schemas":{{"Id":{{"type":"string","x-brand":"UserId"}},"Plain":{{"type":"string"}}}}}}}}"#
        );
        let mut docs = Documents::new();
        docs.add("api.json", &text).expect("JSON");
        let (output, _) = flowapi::generate(&docs, &Brands { docs: &docs });
        let files = output.expect("no error").files;
        assert_eq!(files[0].text, "Id is branded UserId\n", "{v}");
        let (shapes, _) = flowapi::generate(&docs, &Shapes);
        assert_eq!(
            shapes.expect("no error").files[0].text,
            "Id = string\nPlain = string\n",
            "{v}: the core gives the extension no meaning"
        );
    }
}

// req: jsonschema-2020-12.core.the-json-schema-core-vocabulary.recommended-however-implementations-detect-cases-raise-error — a meta-schema that lists the core vocabulary as optional (false) is refused, as one without it is
#[test]
fn a_meta_schema_must_require_the_core_vocabulary() {
    let meta = r#"{"$id":"https://ex.com/meta-optional-core","$vocabulary":{
        "https://json-schema.org/draft/2020-12/vocab/core":false,
        "https://json-schema.org/draft/2020-12/vocab/applicator":true,
        "https://json-schema.org/draft/2020-12/vocab/validation":true}}"#;
    let schema = r#"{"$schema":"https://ex.com/meta-optional-core","type":"string"}"#;
    let refused = run(&[("schema.json", schema), ("meta.json", meta)]).expect_err("refused");
    assert!(
        refused
            .iter()
            .any(|d| d.contains("error[jsonschema-2020-12.core.$vocabulary]")),
        "{refused:?}"
    );
}
