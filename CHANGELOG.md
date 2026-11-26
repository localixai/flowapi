# Changelog

## Unreleased

- The TypeScript SDK writes `multipart/form-data` request bodies: `<op>Request` takes the value
  as its schema says (a string member also as a file: a `Uint8Array`, `flowapiFile(bytes or text,
  name, type)`, or `await flowapiBlob(blobOrFile)`, the one step that waits) and returns the parts
  as bytes, with `content-type: multipart/form-data; boundary=...`; the value was a string you had
  encoded. The parts follow the Encoding Object's defaults: a file a file part (`filename` its
  name's last path segment, `blob` when it has none; `application/octet-stream` when it has no
  type), a string, number or boolean text, an object JSON, an array's items one part each under
  its name in order; `undefined` is left out. The body's `encoding` (`contentType`, `headers`) sets
  a part's type and headers; the description's `encoding` is not read yet, so it is given there.
  The boundary is the content type's, else 128 random bits from `globalThis.crypto`, drawn again
  while a part holds them. 3.2's array of objects is its parts in order. New `FlowapiRequestError`
  reasons refuse what would change the parts: a CR, LF or NUL in a part's name, file name, type or
  header (`control-character`), `null`, a `"` or `\` in a name (`quote`), a file name empty, `.` or
  `..` (`file-name`), a header name not a token or `Content-Disposition` (`header-name`), a list or
  a range as a part's type (`media-range`), a Blob not read (`unread-blob`), an item of the array
  that is not an object (`unnamed-part`), a boundary RFC 2046 does not allow or a part holds
  (`boundary`).
- A request body neither JSON, text nor a form (`application/octet-stream`, `image/png`, a range)
  may be given to the TypeScript SDK as a `Uint8Array`, sent as it is; `$Request.body` is
  `string | Uint8Array`. Another multipart type is still yours to write, now as a string or bytes.
- With the TypeScript SDK a schema named `Promise`, `ArrayBuffer`, `FlowapiRequestError`,
  `FlowapiFile` or `FlowapiEncoding` is declared under another name, with a warning: the SDK names
  them.
- Commands for an agent that writes an SDK's config, `flowapi-sdk.yml` (version 1), before the
  description's path: `flowapi summary <description> [--tag T] [--path P] [--max-bytes N]`, the
  description in compact JSON (servers, security schemes, counts, tags, each operation with its
  parameters, the media types of its body and responses, and the pagination it looks like it has,
  with a reason and how sure; over the budget, 60 KB by default, the tags alone);
  `flowapi config draft <description> [-o file]`, a first config in YAML (resources by tag, else
  by the path's first segment, the verbs `list`, `create`, `retrieve`, `update`, `delete`, the
  suggested pagination, environments from the servers, `auth` from the security schemes,
  `skip: []`), which passes `config check` and does not depend on the order the description is
  written in; `flowapi config schema`, the format as JSON Schema 2020-12, printed from the
  definitions the checker reads by; and `flowapi config check <config> [--spec D] [--json]`,
  every issue at once with a code, the JSON Pointer, the YAML line and column and a hint: members
  the format does not name, types, coverage (every operation in a resource or in `skip` with a
  reason), references that name no operation (the closest named), an operation in two places,
  names equal as words among siblings, names a generated class has itself or a schema has,
  pagination fields missing from the parameters or the response schema or of the wrong type,
  `auth`, environments and the README's methods. It exits 1 on an error. The old form,
  `flowapi <description>`, is unchanged; a description named `summary` is read as `./summary`.
- The npm package runs the same four commands, `npx -y @localix-ai/flowapi summary|config …`,
  with the Rust command's arguments, output and exit codes, byte for byte, and gives them as
  functions: `summary`, `configDraft`, `configSchema` and `configCheck` (issues as data, as text
  and as `--json` prints them), and `loadSdk()` with them synchronous and `configSpec`. They are a
  second module, `flowapi-sdk.wasm`, loaded only when one of them is first used; the generator's
  module is as it was.
- The TypeScript SDK reads binary responses as bytes, as the Rust SDK does: `<op>Response(status,
  headers, body)` takes `body: string | Uint8Array` (it took a `string`) and reads it by the type
  sent: JSON parsed, `text/*` a `string`, any other a `Uint8Array` (an `application/xml` or
  `application/octet-stream` body was text before, and bytes that were not UTF-8 were lost on the
  way). The response union says `Uint8Array` for such a type, and a media range each of those its
  range allows. Bytes given for JSON or text are read as UTF-8; bytes that are not UTF-8 there
  read as `response: null` with the body as given, as the Rust SDK refuses them. A string given
  for a binary type is encoded as UTF-8. `flowapiReadsBytes(contentType)` (exported) says which
  bodies are read as bytes. A schema named `Uint8Array` is declared under another name with the
  SDK, with a warning.
- The npm package reads YAML descriptions as the command does, by the same code: the YAML reader
  moved out of the command into a crate of its own, `flowapi-yaml`, which the command and the
  WebAssembly module share (the core stays JSON-only with no dependency). Its output and its
  diagnostics are the command's byte for byte; the module is about 43 KB larger gzipped.
- A YAML text that JSON cannot hold is printed as a diagnostic, `error[yaml.syntax]: <what>` then
  `  at <file>:<line>:<column>`, as a JSON syntax error is; it was `flowapi: <file>:<line>:<column>:
  <what>`.

## 0.1.0 — 2026-11-24

- Both SDKs refuse a request that would mean something other than it says, rather than write
  it: a CR, LF or NUL in a header's value or a `style: cookie` cookie (header injection; a form
  cookie is percent-encoded as before), a path parameter that leaves its whole segment empty or
  makes it `.` or `..` (`%2E` read as a dot too, each piece between the `/` that
  `allowReserved` keeps held to the same), and NaN or an infinity in a parameter or a JSON or
  form body (written as `NaN` and as `null` before). TypeScript: `<op>Request` throws a
  `FlowapiRequestError` (exported; `location`, `parameter`, `reason`). Rust: `<op>_request` returns
  `Result<sdk::Request, sdk::RequestError>` (`location`, `parameter`, `reason: sdk::Reason`,
  `Reason::code()` the same strings). A JSON response body's leading byte order mark is skipped.
- A header's or a cookie's name that is not an RFC 9110 token is left out of the operation with
  a warning (`parameter.token`); a response key that is not a status (`6XX`, `600`, a word) is
  left out with a warning, and a range written `4xx` is read with one (`responses.status`).
- The Rust `Integer` holds every JSON integer serde_json reads: exact from `i64::MIN` to
  `u64::MAX` (it refused everything past `i64::MAX`), past that the whole `f64` serde_json gives
  (digits past 2^53 are already lost there without its `arbitrary_precision`). It no longer
  dereferences to `i64`: `as_i64()`, `as_u64()`, `as_i128()`, `as_f64()`, `from_f64()`, `From`
  the integer types, `Display`, `Ord` and `Hash` as before. O3 for Rust runs our own cases beside
  the suite (`conformance/suite-rust/own/`: 2^63, 2^64 - 1, 2^64, 1e20, -1e19, 1e300 read and
  written back; 1.5 refused).
- More of a schema is stated by the type (O3 precision 63.3% to 69.0%, soundness still 100%):
  `propertyNames` (`false`, an `enum` or `const` of names as a closed object, `maxLength: 0` as
  the empty key, a literal `pattern` as a template signature) and `maxProperties: 0`; `contains`
  with `minContains` and `maxContains` as the array's length, `contains: false` and
  `maxContains: 0` of a kind test as the items' kinds; `if` that tests kinds or `required` as
  (if and then) or (not if and else); `oneOf` exact where its branches cannot overlap (kinds,
  literals) and kinds two branches both accept ruled out; `not` of any kind test (`enum: [null]`);
  `enum` and `const` values filtered by `minimum`, `maximum`, the exclusive bounds, `multipleOf`,
  `minLength`, `maxLength` and a literal `pattern`, each decided only where it is certain (no
  float tolerance). A keyword the type now states is no longer listed as unchecked; `not` of
  `integer`, which states nothing, now is.
- `patternProperties`: each literal pattern gets its template signature even beside patterns
  that have none, an unanchored `.*` at either end is read as nothing, and patterns of one
  template share a signature. The core reads a literal pattern (`graph::Affix`); the TypeScript
  target prints it.
- A keyword a later JSON Schema draft brought (`const`, `contains`, `propertyNames` in draft 6;
  `if`, `then`, `else` in 7; `minContains`, `maxContains`, `dependent*`, `unevaluated*` in
  2019-09; `prefixItems` in 2020-12) means nothing under an earlier draft's `$schema`.
- O5's random schemas also draw `propertyNames`, `contains` with its counts, `if`/`then`/`else`,
  the numeric and string bounds beside an `enum`, and `maxProperties`.

- An SDK with no HTTP library (`flowapi --sdk`, `TypeScript::with_sdk`): per operation a
  function that writes its request (path, query, headers, body by `style`, `explode`,
  `allowReserved` and media type) and one that reads its response (by status, range, `default`
  and `Content-Type`), sending nothing. Held by the spec's Style Examples table and the pitfalls
  of other generators run under Node (`conformance/sdk`, O8). The core's parameters gain their
  serialization (`spec::Serialization`; Swagger 2.0's `collectionFormat` mapped, `tsv` warned
  about) and the program its operations (`Program::endpoints`).
- The SDK for Rust too (`flowapi --target rust --sdk`, `Rust::with_sdk`): per operation an
  input struct, `<op>_request(&input) -> sdk::Request` (method, path, query, headers, the body's
  content type and bytes) and `<op>_response(status, headers, body)` into an enum of the
  responses (exact status, then `NXX`, then `default`; JSON by serde_json, `text/*` as a
  `String`, other bodies as bytes), or an `sdk::Error` that keeps the answer. A runtime module
  printed once (`pub mod sdk`) needs serde and serde_json only: a serde `Serializer` writes each
  value by RFC 6570 straight into the request, percent-encoding written by byte, numbers as
  JavaScript writes them. Held by oracle O8 for Rust (`conformance/sdk-rust`: the Style Examples,
  `conformance/sdk/cases.json`, and Rust's own cases, compiled into one crate and run).
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
