# Triage, second pass (2026-11-24)

The second coverage pass of the requirements ledger (`ledger/`, O1): what changed with the pin of
OpenAPI 3.2.1, the scope decided by the maintainers (the owner delegated it), the violations the
pass found, and the in-scope requirements no marker answers yet. The first pass is
`coverage/triage.md`. This file sits beside it, not inside `coverage/`, because the workspace's
Claude Code settings deny reading `**/coverage/**` (a rule written for test-coverage output), so
this pass could neither read nor edit `coverage/`. Merge the two when that rule is narrowed. The
ledger reads markers only from `.md` files under `coverage/`, so this file holds none.

Counts are `REPORT.md`'s, written by `cargo run --release --bin report` in `ledger/`.

## 1. OpenAPI 3.2.1 replaces 3.2.0

`specs/openapi/3.2.1.md`, from the same commit as the other OpenAPI texts (`SOURCE.md` has its
SHA-256). `extract` reported 10 ids added and 9 removed. Each removed id is a sentence that 3.2.1
reworded, and none had a marker:

| Removed (3.2.0) | Added (3.2.1) |
| --- | --- |
| `example-object.criteria-for-serializedexample.should-…` | `example-object.criteria-for-serializedvalue.should-…` (the section was renamed) |
| `example-object.externalValue.must-otherwise-value-valid-serialization-data-value` | `…must-otherwise-value-identify-valid-serialization-data` |
| `media-type-object.additional-encoding-approaches.must-includes-multipart-form-data-encoding-object` | (removed in 3.2.1) |
| `media-type-object.encoding-by-name.must-array-properties-handled-applying-given-encoding` | `…must-serializing-property-values-array-type-handled` |
| `media-type-object.encoding-by-name.must-multipart-encoding-keys-map-name-parameter` | `…must-multipart-types-decode-object-multipart-form` |
| (new in 3.2.1) | `media-type-object.encoding-by-name.must-deserialization-object-multiple-values-same-name` |
| `media-type-object.encoding-by-position.must-use-prefixencoding-itemencoding-fields-either-itemschema` | `…must-use-prefixencoding-itemencoding-fields-each-item` |
| (new in 3.2.1) | `media-type-object.encoding-by-position.shall-applications-wish-preserve-part-order-multipart` |
| `parameter-object.parameter-locations.must-not-example-items-id-query-parameter-id` | `…must-not-query-parameters-appended-url-character-subsequent` |
| `parameter-object.serialization-and-examples.must-cases-name-inherent-part-constructing-serialization` | `…must-cases-parameter-name-inherent-part-constructing` |
| `responses-object.pattern-http-status-code.must-field-enclosed-quotation-marks-example-200` | `…must-field-represented-string-example-200-enclosed` |

All are serialization, examples or rules for authors, and all are out of scope (section 2).

## 2. Scope: 726 ids moved out of scope

`scope.json`'s `status` is now "decided 2026-11-24 by the maintainers, owner delegated". The
first pass proposed 328 ids in `coverage/triage.md`. That list could not be read here, so this
pass rebuilt it from `classify --missing`: every in-scope id without a marker, read one by one and
given the reason it is out of scope, or kept in scope. Among the same texts the first pass covered
(JSON Schema 2020-12, the Schema and Discriminator Objects), 285 ids moved out of scope. Compare
them with the first pass's 328 when `coverage/` can be read again. The other 441 come from this
pass's own work on OpenAPI structure, where serialization went to the SDK stage.

| Reason (`scope.json` gives the full text) | JSON Schema, Schema Object | OpenAPI structure | All |
| --- | ---: | ---: | ---: |
| For the author: what a valid document is | 126 | 141 | 267 |
| SDK stage (map FA-012), not types: serialization | 4 | 189 | 193 |
| A MAY or OPTIONAL flowapi does not take | 38 | 51 | 89 |
| Defining vocabularies, meta-schemas, extension keywords | 33 | 0 | 33 |
| Documentation of an operation, parameter, body or response | 0 | 33 | 33 |
| Annotation collection and output | 20 | 0 | 20 |
| Discriminator selection at run time | 18 | 0 | 18 |
| Servers, publishing, decoding, run-time behaviour | 16 | 2 | 18 |
| format as an assertion | 15 | 0 | 15 |
| Example values | 0 | 15 | 15 |
| Meta-validation | 6 | 3 | 9 |
| Bundling | 8 | 0 | 8 |
| Implicit connections (links, tags, names) | 0 | 7 | 7 |
| Schema-editing tools | 1 | 0 | 1 |

The whole-section rules that were in scope "for the SDK stage" (`encoding-object`, the
serialization appendices, the percent-encoding and interoperability sections) are now out of
scope with the SDK reason. No type depends on style, explode, allowReserved or encoding: a
parameter's type is its schema's. Operations themselves stay in scope.

## 3. Markers added (`crates/flowapi-ts/tests/print.rs`)

New cases in `crates/flowapi-ts/tests/cases/`. Each `.ts` was written by `FLOWAPI_BLESS=1`, read
against the specification and compiled with the oracle's `tsc` flags. The kept cases print the
same bytes as before.

- `oas32.json`: OpenAPI 3.2.1 with `$self` (a `$ref` by the `$self` URI), the `query` method, a
  `querystring` parameter, every other method, path-level and operation parameters (the latter
  winning), `content` parameters, cookie and header parameters, Reference Objects to components
  (parameters, request bodies, responses, path items), status ranges, `default`, `x-` fields
  everywhere, `application/jsonl` with `schema`, `text/event-stream` with only `itemSchema`
  (lossy: unknown), a discriminator, and a Schema Object with its own `$schema` (draft-07).
- `oas30-operations.json`, `oas31-operations.json`: the same structure in 3.0.4 and 3.1.2, with
  a 3.0 Path Item `$ref`, 3.0 `readOnly`/`writeOnly` (lossy: one type for both directions),
  `title`, `default`, `deprecated`.
- `oas31-dialect.json`, `oas32-dialect.json`: `jsonSchemaDialect` draft-07, a schema's own
  `$schema` winning over it, and empty `paths`.
- `annotations.json` (JSON Schema): `$comment` never printed, title, description and deprecated
  as doc comments, unknown keywords and a non-meta-schema's `$vocabulary` ignored, embedded
  resources with and without their own `$schema`, `if` and `then` in different `allOf` branches,
  an unanchored `pattern`, an `$id` under `definitions`.
- `duplicate-id.json`: two schemas with one `$id` give a warning, kept in the `.ts` as a
  `// warning[...]` line, so the case needs no CLI.

The fixed violations of the first pass (its V1 to V7 could not be read; these are the fixes the
CHANGELOG lists) now have markers here:

- `then`/`else` without `if`: `conditional.json`, and across `allOf` branches in `annotations.json`.
- `$self` names the document: `oas32-self.json`, `oas32.json`.
- 3.0 `readOnly`/`writeOnly` not required: `oas30-read-only.json`, `oas30-operations.json`.
- 3.0 reads only its own keywords: `oas30-read-only.json` (`C`). No requirement id states this
  alone, so it has no marker.
- A repeated `$id` is a warning: `duplicate-id.json`, marked on
  `jsonschema-2020-12.core.loading-a-referenced-schema.should-multiple-schemas-try-identify-same-uri`.
  The text says "raise an error condition" and flowapi warns and keeps the first. The maintainers
  should confirm a warning is enough.
- Relative references against the referring document (multi-document, V5): the existing marker in
  `components.json` covers the single-document part. The rest needs two documents, which only the
  CLI loads; see section 5.
- `$vocabulary` (an unknown required vocabulary is an error; validation off makes keywords
  annotations): no test, because the meta-schema must be a second document. A reproduction is in
  section 5, and F6 is a gap that remains.

## 4. Violations found (most important first)

Each reproduction is a file run as `target/release/flowapi <file.json>` after `cargo build
--release`. None is in a kept case: a kept `.ts` must be right by the specification.

**F1. An unresolvable Reference Object for a parameter, request body or response is dropped
silently.** The command exits 0 with no diagnostic, and the operation's type loses that part:
`responses` becomes `{ [key: string]: never }` and the parameters and body disappear. A dangling
schema `$ref` is an error (`schema.ref`), and so is a Path Item's (`oas.ref`). Ids:
`oas-*.reference-object.$ref`; rules G2 and G8. Code: `spec/operations.rs`, `Reader::params`
(`continue`), `Reader::body` (`?`), `Reader::responses` (`?` in `filter_map`).

```json
{"openapi":"3.1.0","info":{"title":"t","version":"1"},"paths":{"/a":{"post":{"operationId":"p",
 "parameters":[{"$ref":"#/components/parameters/Missing"}],
 "requestBody":{"$ref":"#/components/requestBodies/Missing"},
 "responses":{"200":{"$ref":"#/components/responses/Missing"}}}}}}
```

**F2. Header parameters named `Accept`, `Content-Type` or `Authorization` are not ignored.** The
specification says they SHALL be (in all three versions), and header names compare without case
(RFC 9110), so `accept` is ignored too. Today each becomes a member of `parameters.header`, and a
required one makes `header` required. Id: `oas-*.parameter-object.name.shall-header-name-field-accept-content-type`.
Code: `spec/operations.rs`, `Reader::params`.

```json
{"openapi":"3.2.0","info":{"title":"t","version":"1"},"paths":{"/a":{"get":{"operationId":"g",
 "parameters":[{"name":"Authorization","in":"header","required":true,"schema":{"type":"string"}},
   {"name":"accept","in":"header","schema":{"type":"string"}}],
 "responses":{"200":{"description":"ok"}}}}}}
```

**F3. A Path Item with `$ref` loses every field beside the `$ref`.** `Reader::follow` returns the
target and drops the referring object, so `post` below gets no type and nothing says so. The
specification leaves behaviour undefined only for a field that appears in both objects. The same
happens in 3.0, 3.1 and 3.2. Id: `oas-*.path-item-object.$ref`. Its marker covers a `$ref` with
nothing beside it.

```json
{"openapi":"3.1.0","info":{"title":"t","version":"1"},
 "paths":{"/a":{"$ref":"#/components/pathItems/P","post":{"operationId":"sib","responses":{"200":{"description":"ok"}}}}},
 "components":{"pathItems":{"P":{"get":{"operationId":"pg","responses":{"200":{"description":"ok"}}}}}}}
```

**F4. OpenAPI 3.2's `additionalOperations` is not read.** Operations under it (`LINK` below) get
no type and no diagnostic. Id: `oas-3.2.path-item-object.additionalOperations`. Code: `METHODS` in
`spec/operations.rs`.

```json
{"openapi":"3.2.0","info":{"title":"t","version":"1"},"paths":{"/a":{
 "additionalOperations":{"LINK":{"operationId":"link","responses":{"200":{"description":"ok"}}}}}}}
```

**F5. A Media Type Object given as a Reference Object (3.2, `components/mediaTypes`) is not
followed.** The media type's type is `unknown`: sound, but the schema is lost without a word. Id:
`oas-3.2.components-object.mediaTypes`. Code: `Reader::content` reads `schema` without `follow`.

```json
{"openapi":"3.2.0","info":{"title":"t","version":"1"},"paths":{"/a":{"get":{"operationId":"g",
 "responses":{"200":{"description":"ok","content":{"application/json":{"$ref":"#/components/mediaTypes/M"}}}}}}},
 "components":{"mediaTypes":{"M":{"schema":{"type":"number"}}}}}
```

**F6. A meta-schema named by its `$id` is not recognized.** The schema below names its
meta-schema by `$id`. The meta-schema is loaded (a `$ref` reaches the file), but `graph/vocab.rs`
looks it up with `Documents::find`, which matches retrieval URIs only. So the unknown required
vocabulary is not refused: the output is `string` and the exit code 0. It works when `$schema` is
the file's relative path. Ids: `jsonschema-2020-12.core.detecting-a-meta-schema.must-implementations-recognize-schema-meta-schema-being`,
`jsonschema-2020-12.core.$vocabulary.must-value-true-implementations-do-not-recognize`.

```sh
echo '{"$id":"https://ex.com/meta","$vocabulary":{"https://json-schema.org/draft/2020-12/vocab/core":true,"https://unknown.example/v":true}}' > meta3.json
echo '{"$schema":"https://ex.com/meta","type":"string","$defs":{"m":{"$ref":"meta3.json"}}}' > s4.json
target/release/flowapi s4.json   # prints Root = string, exit 0; expected an error
```

**F7. In 3.0, a `requestBody` on GET, HEAD or DELETE is kept.** 3.0.4 says it "SHALL be ignored
by consumers". Low impact, because 3.1 and 3.2 permit it. Id:
`oas-3.0.operation-object.requestBody.shall-other-cases-http-spec-vague-get`.

```json
{"openapi":"3.0.4","info":{"title":"t","version":"1"},"paths":{"/a":{"get":{"operationId":"g",
 "requestBody":{"content":{"application/json":{"schema":{"type":"string"}}}},
 "responses":{"200":{"description":"ok"}}}}}}
```

Not a requirement, but noted: the `query` method is read in 3.0 and 3.1 documents too, where a
Path Item has no such field.

## 5. In scope, not answered (66 ids)

`cargo run --release --bin classify -- --missing oas-3.2` (and the other prefixes) lists them.

- **Response headers are not generated** (27 ids): `header-object.{content,schema,required,deprecated,…}`,
  `components-object.headers`, `response-object.headers` and its Content-Type rule, in each
  version. A response's type has its body only. To decide: a `headers` member beside each media
  type.
- **`deprecated` on operations and parameters is not printed** (12 ids:
  `operation-object.deprecated`, `parameter-object.deprecated`, with their SHOULDs, in each
  version). Schemas get `@deprecated`, operations and parameters do not.
- **F2, F4, F5, F7** (6 ids), section 4.
- **Several documents** (7 ids: `openapi-description-structure.may-…-single`,
  `parsing-documents.*`, `appendix-g-…complete-document-parsing`). The CLI loads every file a
  `$ref` or `$schema` reaches by a relative path, but `tests/print.rs` gives the core one
  document, so no kept case shows it. A test with two documents in `print.rs`, or CLI tests,
  would cover these, V5 and the `$vocabulary` ids below.
- **`$vocabulary` and meta-schemas** (6 ids: the `$vocabulary` keyword, `must-any-vocabulary-understood`,
  `must-used-indicate-whether-required`, `must-value-true-…`, `detecting-a-meta-schema.must-recognize`,
  the core-vocabulary "raise an error" RECOMMENDED). Works through the CLI for a meta-schema
  named by its relative path:
  `echo '{"$vocabulary":{"https://json-schema.org/draft/2020-12/vocab/core":true,"https://unknown.example/v":true}}' > meta.json; echo '{"$schema":"meta.json","type":"string"}' > s.json; flowapi s.json`
  fails with `error[jsonschema-2020-12.core.$vocabulary]`, and a meta-schema without the
  validation vocabulary gives `unknown`. F6 is the gap that remains.
- **Binary data** (3 ids: `schema-evaluation-and-binary-data.must-…`, `working-with-binary-data.shall-however-contentmediatype-contradicts-…`
  in 3.1 and 3.2): untested. What a binary body's type should be (`string`, `Blob`, `unknown`) is
  a target decision still to make.
- **Others** (5 ids): `specification-extensions.recommended-…-designed-extensibility` (3 versions;
  flowapi has no extension mechanism yet), `openapi-object.fixed-fields.not-recommended-…-other-uris`
  (3.2: flowapi also resolves by the retrieval URI when `$self` is set, a MAY, untested),
  `jsonschema-2020-12.core.initial-base.recommended-implementations-document-any-default-base-uri`
  (the host gives every URI and flowapi assumes no default; the README does not say so yet).

## 6. Third pass (2026-11-24): the 48 ids left after the fixes

Markers were added in three new test files, each run as a named target:

- `crates/flowapi/tests/documents.rs` (`cargo test -p flowapi --test documents`): several
  documents given to `Documents::add`, generated through a small target written in the test (it
  prints each declaration's shape, so no real target's output is needed). A `$ref` to a document
  added after the referring one; an `$id` and an `$anchor` deep inside another document and
  another description's `$self` as targets; one description spread over a second description and
  a JSON Schema, in 3.0, 3.1 and 3.2; documents told apart by `openapi` and `$schema`; a
  description reached both by its `$self` and by the URI it was given under; a meta-schema found
  by its `$id`, and a target outside the core reading an `x-` field through a type's origin
  (the specification-extensions RECOMMENDED; the core gives the field no meaning).
- `crates/flowapi-cli/tests/documents_cli.rs` (`--test documents_cli`): three files in two
  directories reached by relative `$ref`s, and a missing one an error; the base a document is
  read against (the path as typed); meta-schemas beside the schema: an unknown vocabulary refused
  when `true` and passed over when `false`, validation off (`type`, `required` annotations),
  applicator off (lossy, below), core absent (an error).
- `crates/flowapi-cli/tests/openapi_cli.rs` (`--test openapi_cli`): response headers in each
  version (by reference to `components/headers`, by `schema`, by `content`, `required`, `x-`
  fields beside them, `Content-Type` in any case left out), `@deprecated` on operations,
  parameters and headers, and binary bodies (raw binary `unknown`, base64 `string`, a
  contradicting `contentMediaType` narrowing nothing, multipart parts read apart, 3.0's
  `format: binary` a `string`).

The tests pass on the last commit's core (a copy made by `git archive HEAD`, while the working
tree's core was being changed for Swagger 2.0) and on the working tree once it compiled again.

One id is lossy: `jsonschema-2020-12.core.$vocabulary.must-any-vocabulary-understood-…`. A
meta-schema without the applicator vocabulary makes the whole schema `unknown` (recorded lossy on
`$vocabulary`), wider than the validation vocabulary alone would give.

One id stays missing, a violation:

**T1. A meta-schema whose `$vocabulary` marks the core vocabulary `false` is not refused.** The
specification leaves the case undefined and RECOMMENDS an error, as for a `$vocabulary` without
core, which flowapi does refuse. Id:
`jsonschema-2020-12.core.the-json-schema-core-vocabulary.recommended-however-implementations-detect-cases-raise-error`.
Code: `graph/vocab.rs`, where the core vocabulary's absence is checked.

```sh
echo '{"$vocabulary":{"https://json-schema.org/draft/2020-12/vocab/core":false,"https://json-schema.org/draft/2020-12/vocab/applicator":true,"https://json-schema.org/draft/2020-12/vocab/validation":true}}' > meta.json
echo '{"$schema":"meta.json","type":"string"}' > s.json
target/release/flowapi s.json   # prints Root = string, exit 0; expected error[jsonschema-2020-12.core.$vocabulary]
```

Noted, not a requirement flowapi answers for: **T2.** A Header Object (and, by the same reader, a
parameter) whose `content` holds two media types, which the specification forbids ("MUST only
contain one entry"), takes the first one's schema without a diagnostic: `X-Two` below prints
`number`, and the `string` of the second entry is lost. An error or a warning naming the rule
would follow G8.

```json
{"openapi":"3.1.2","info":{"title":"t","version":"1"},"paths":{"/a":{"get":{"operationId":"g",
 "responses":{"200":{"description":"ok","headers":{"X-Two":{"content":{
   "text/plain":{"schema":{"type":"integer"}},"application/json":{"schema":{"type":"string"}}}}}}}}}}}
```

**T3.** The default base URI marker rests on the API documentation (`Document::uri`: "a
retrieval URI, a path, or any name the host gives it") and the CLI test. The README says nothing
of it; one sentence there ("flowapi assumes no base URI: each document's is the one the host
gives it, the CLI's the path as typed") would make the documentation easy to find.
