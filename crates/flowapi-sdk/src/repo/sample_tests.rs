//! Each parameter's README value, in the order the module says: the Parameter Object's
//! `example`, the first of its `examples` (by reference too), 2.0's `x-example`, the schema's own
//! value, then the placeholder of its name and type; a `format` or a `pattern` keeps the made
//! value, a header is found whatever the case of its name, a Path Item's parameter too.

use super::param;
use crate::ops::Ops;
use flowapi::Documents;

/// Each parameter of the first operation of `spec`, by name, with the README's value as
/// JavaScript.
fn values(spec: &str) -> Vec<(String, String)> {
    let mut docs = Documents::new();
    assert!(
        docs.add("spec.json", spec).is_ok(),
        "the description is JSON"
    );
    let (ops, _) = Ops::read(&docs);
    let Some(ops) = ops else {
        return Vec::new();
    };
    let Some(op) = ops.list.first() else {
        return Vec::new();
    };
    let o = &op.operation;
    o.params
        .iter()
        .map(|p| (p.name.clone(), param(&ops.doc, o, p).js()))
        .collect()
}

const OPENAPI: &str = r##"{
  "openapi": "3.1.0",
  "info": { "title": "T", "version": "1" },
  "paths": {
    "/pets/{pet_id}/{owner_id}/{kind}/{since}/{code}/{tag}/{when}": {
      "parameters": [
        { "name": "kind", "in": "path", "required": true,
          "schema": { "type": "string", "enum": ["dog", "cat"] } }
      ],
      "get": {
        "operationId": "getPet",
        "parameters": [
          { "name": "pet_id", "in": "path", "required": true, "schema": { "type": "string" } },
          { "name": "owner_id", "in": "path", "required": true, "schema": { "type": "integer" } },
          { "name": "since", "in": "path", "required": true,
            "schema": { "type": "string", "format": "date" } },
          { "name": "code", "in": "path", "required": true,
            "schema": { "type": "string", "pattern": "^[A-Z]{3}$" } },
          { "name": "tag", "in": "path", "required": true, "example": "fluffy",
            "schema": { "type": "string", "example": "ignored" } },
          { "name": "when", "in": "path", "required": true,
            "examples": { "first": { "$ref": "#/components/examples/Noon" }, "second": { "value": "x" } },
            "schema": { "type": "string" } },
          { "name": "verbose", "in": "query", "required": true, "schema": { "type": "boolean" } },
          { "name": "limit", "in": "query", "required": true,
            "schema": { "type": "integer", "default": 20 } },
          { "name": "q", "in": "query", "required": true,
            "schema": { "$ref": "#/components/schemas/Query" } },
          { "name": "letter", "in": "query", "required": true,
            "schema": { "type": "string", "maxLength": 1 } },
          { "name": "X-Request-Id", "in": "header", "required": true,
            "example": "req-1", "schema": { "type": "string" } }
        ],
        "responses": { "200": { "description": "ok" } }
      }
    }
  },
  "components": {
    "examples": { "Noon": { "value": "12:00" } },
    "schemas": { "Query": { "type": "string", "examples": ["rex"] } }
  }
}"##;

#[test]
fn a_parameter_is_shown_by_what_the_description_writes_else_by_its_name() {
    let got = values(OPENAPI);
    let want = [
        ("pet_id", "\"pet_id\""),
        ("owner_id", "1"),
        ("since", "\"2024-01-01\""),
        ("code", "\"string\""),
        ("tag", "\"fluffy\""),
        ("when", "\"12:00\""),
        ("verbose", "true"),
        ("limit", "20"),
        ("q", "\"rex\""),
        ("letter", "\"l\""),
        ("X-Request-Id", "\"req-1\""),
        ("kind", "\"dog\""),
    ];
    for (name, value) in want {
        let found = got.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
        assert_eq!(found, Some(value), "{name}");
    }
}

#[test]
fn swagger_2_0_x_example_is_read() {
    let spec = r#"{
      "swagger": "2.0",
      "info": { "title": "T", "version": "1" },
      "paths": { "/pets/{pet_id}": { "get": {
        "parameters": [
          { "name": "pet_id", "in": "path", "required": true, "type": "string", "x-example": "p-7" }
        ],
        "responses": { "200": { "description": "ok" } }
      } } }
    }"#;
    assert_eq!(
        values(spec),
        vec![(String::from("pet_id"), String::from("\"p-7\""))]
    );
}
