# Changelog

## Unreleased

- First version: OpenAPI 3.0, 3.1 and 3.2 and JSON Schema (2020-12, and drafts 4 to 2019-09 by
  their `$schema`) read into a type graph and printed as TypeScript: components, operations
  (parameters by location, request bodies and responses by media type) and `Paths`.
- Every type is sound on the official JSON Schema Test Suite (`conformance/results/suite.md`).
- `$ref` by JSON Pointer, `$id`, `$anchor` and relative files; cycles; `allOf` merged and a
  shape distributed over `anyOf`/`oneOf`; `enum` and `const` as exact types, objects and arrays
  among them; small `minItems`/`maxItems` as tuples; `not: {type}`; `unevaluatedProperties` and
  `unevaluatedItems` `false` as closed shapes; `dependentRequired` and `dependentSchemas` as
  unions.
- Fixed against the specification: `then` and `else` mean nothing without `if`; a meta-schema's
  `$vocabulary` is read (validation off makes its keywords annotations; an unknown required
  vocabulary is an error); OpenAPI 3.2's `$self` names the document; in OpenAPI 3.0 a read-only
  or write-only property is not required in the one type, and only 3.0's own keywords are read;
  a repeated `$id` is a warning.
- Operations: a response is `{ headers?, content }`, its headers typed by name (`Content-Type`
  left out); `deprecated` operations, parameters and headers say so; a reference to a parameter,
  body, response, header or media type that names nothing is an error; fields beside a Path
  Item's `$ref` are kept; 3.2's `additionalOperations`, `query` and media types by reference are
  read; header parameters `Accept`, `Content-Type` and `Authorization` are ignored, and in 3.0 a
  body on GET, HEAD or DELETE. A meta-schema is found by its `$id` too.
- An object that holds no property prints as `Record<string, never>`.
- Objects are printed one member a line, each with its schema's description above it.
- The `flowapi` command reads YAML descriptions (and YAML files a `$ref` names) as YAML 1.2's
  core schema into JSON, aliases expanded within a bound; a diagnostic names the YAML line.
