# Changelog

## Unreleased

- An SDK with no HTTP library (`flowapi --sdk`, `TypeScript::with_sdk`): per operation a
  function that writes its request (path, query, headers, body by `style`, `explode`,
  `allowReserved` and media type) and one that reads its response (by status, range, `default`
  and `Content-Type`), sending nothing. Held by the spec's Style Examples table and the pitfalls
  of other generators run under Node (`conformance/sdk`, O8). The core's parameters gain their
  serialization (`spec::Serialization`; Swagger 2.0's `collectionFormat` mapped, `tsv` warned
  about) and the program its operations (`Program::endpoints`).
- A schema named `Record` or `Object`, global types the output refers to, is declared under
  another name, with a warning.
- The Rust target (`crates/flowapi-rust`, `flowapi --target rust`): serde types, untagged enums
  ordered narrow to wide, `Option<Option<T>>` for optional and nullable, an `Integer` that reads
  `1.0`, closed objects with `deny_unknown_fields`, cycles boxed. Sound on the official suite
  (`conformance/results/suite-rust.md`).
- Two more oracles: random schemas checked against flowschema's verdicts
  (`conformance/props`, O5), the OAI's own example descriptions (`conformance/oai`, O2), and
  random inputs that must never panic (`crates/flowapi-ts/tests/fuzz.rs`, O7).
- Intersections are merged further: parts simplified before what holds them, to a fixpoint;
  shapes behind unnamed aliases merged; `(A | B) & C` split into `(A & C) | (B & C)`; an
  inherited member's type (`Object["constructor"]`) carried into the index signatures it falls
  under.
- A meta-schema that does not require the core vocabulary is an error; a parameter's or a
  header's `content` with more than one media type is warned about.
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
- Swagger 2.0 (OpenAPI 2.0) is read into the same model as 3.x: `definitions` as the named
  types; `body` parameters as the request body under each media type of `consumes` (the
  operation's, else the description's, else `application/json`), `formData` parameters as a form
  under `application/x-www-form-urlencoded` or `multipart/form-data`; a response's `schema`
  under each type of `produces`; non-body parameters and headers read as their own schemas
  (`type`, `format`, `items`, `enum`); `#/parameters/…` and `#/responses/…` followed. Its Schema
  Object takes only 2.0's keywords, `$ref` overrides its siblings, `type: file` is any value,
  and the `x-nullable` extension adds `null`. A document with both a body and a form is warned
  about (`swagger.body`); `host`, `basePath` and `schemes` are not read.
- The `flowapi` command reads YAML descriptions (and YAML files a `$ref` names) as YAML 1.2's
  core schema into JSON, aliases expanded within a bound; a diagnostic names the YAML line.
- The npm package `@localix-ai/flowapi` (`npm/`): the same generator compiled to WebAssembly
  for Node, Bun, Deno and browsers, with no runtime dependency: `generate`, `load`, the files a
  description reaches read by `@localix-ai/flowapi/node`, and a `flowapi` command whose output
  is the Rust command's byte for byte. It reads JSON only; YAML is the Rust command's.
