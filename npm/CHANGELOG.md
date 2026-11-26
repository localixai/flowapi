# @localix-ai/flowapi

## 0.3.0

### Minor Changes

- f3f940c: The SDK writes request bodies by the description's Encoding Objects: by name, by position
  (3.2's `prefixEncoding` and `itemEncoding`) and nested, with `style`, `explode` and
  `allowReserved` for form members, and Swagger 2.0's `formData` `collectionFormat`. A form body
  written from its content is percent-encoded by WHATWG's rules (a space as `+`), as OpenAPI 3.2
  asks.
- 2301860: SDK authentication: the generated client sends every kind of security scheme as OpenAPI and RFC 6749, 6750 and 7617 say (API keys, HTTP bearer and basic, other HTTP schemes, OAuth 2.0 and OpenID Connect tokens, and OAuth 2.0's client credentials grant run by the client itself), follows `security` exactly (alternatives in order, `{}`, `[]`, an operation's own over the description's, Appendix F), sends credentials only over TLS or to a loopback host unless `allowInsecureCredentials` is set, and throws `CredentialError`, `InsecureCredentialError` or `TokenError` before sending what it cannot. `flowapi-sdk.yml` declares the schemes and `security` a description lacks (`auth.<name>.type`, `security`, a method's `security`); `config check` has the `config.auth.*` and `config.security.unknown` codes, `config draft` names a variable for every scheme, and the generated repository's mock, tests and README cover each.
- 32637b7: The generated client follows RFC 9110 on retries and redirects and the OpenAPI text on servers. A
  POST, a PATCH or another method that is not idempotent is retried only when the server answers
  `x-should-retry: true`, or when `defaults.idempotency_key` in `flowapi-sdk.yml` names a header:
  then each call sends a random key in it, the same on its retries. A 403 is never retried, and the
  default `retries` is 1. Every 3xx with a `Location` is followed except 304, 305 and 306; a
  redirect loop is refused at once, and a `Location` with no fragment keeps the request's. Server
  variables take the caller's `serverVariables` (checked against each `enum`), a relative server URL
  is resolved against `descriptionURL`, and a URL with a user name or password is never sent.

### Patch Changes

- fb3337f: The SDK writes Swagger 2.0's `collectionFormat` in a path, a query and a header: `csv`, `ssv`,
  `tsv` and `pipes` in each (`tsv` was written as `csv`), `multi` in a query, and an Items Object's
  own `collectionFormat` for the arrays inside an array.

## 0.2.0

### Minor Changes

- 79e2e37: The command runs the Rust command's commands for an agent that writes an SDK's config,
  `flowapi-sdk.yml`: `flowapi summary <description> [--tag T] [--path P] [--max-bytes N]`,
  `flowapi config draft <description> [-o file]`, `flowapi config schema` and
  `flowapi config check <config> [--spec D] [--json]`, with the same arguments, output and exit
  codes, byte for byte. The API gives them as `summary`, `configDraft`, `configSchema` and
  `configCheck` (every issue as data, as the text `config check` prints and as its `--json`), and
  `loadSdk()` gives them synchronous with `configSpec`, the `spec` a config names, for the host to
  read before the check. They are a second module, `flowapi-sdk.wasm` (about 415 KB gzipped),
  loaded only when one of them is first used: `generate` and `references` load `flowapi.wasm` alone,
  as before.
- ff4c575: `flowapi generate <flowapi-sdk.yml> --out <dir> [--check]` runs through the package as the Rust
  command runs it: the same files byte for byte, the same `.flowapi/lock.json`, the same messages
  and exit codes, by the same rules (all or nothing; a generated file edited by hand is a conflict;
  `src/lib/` and `examples/` kept; an unchanged file no longer generated deleted, a changed one kept
  with a warning; a path out of `--out` or through a symbolic link refused; `--check` writes
  nothing). The rules run in `flowapi-sdk.wasm`; the package reads the disk and makes the changes.
  `loadSdk()` gives the two halves without a disk: `repoFiles` (the repository and its lock) and
  `repoPlan` (what writing it changes, given what is on disk).
- e6b6bd0: YAML descriptions are read, as the Rust command reads them and by the same code: `.yaml` and
  `.yml` files, and under any other name a text that does not open with `{` or `[`, the YAML files a
  `$ref` names too. A diagnostic names the YAML line and column, and a YAML text that JSON cannot
  hold is an error with the code `yaml.syntax`, printed by the command as the Rust command prints
  it. In the API a document's `uri` says whether its text is YAML; `references(text, uri)` takes it
  too (without one the text is JSON, as before). The module is about 43 KB larger gzipped.
- 3e7896a: The TypeScript SDK reads binary responses as bytes, as the Rust SDK does. `<op>Response(status,
  headers, body)` takes `body: string | Uint8Array` and reads it by the type sent: JSON parsed,
  `text/*` a `string`, any other a `Uint8Array` (an `application/octet-stream` or `image/png` body
  was a string before, and bytes that were not UTF-8 were lost on the way). The response union says
  `Uint8Array` for such a type. Bytes given for JSON or text are read as UTF-8, and bytes that are
  not UTF-8 there read as `response: null` with the body as given; a string given for a binary type
  is encoded as UTF-8. `flowapiReadsBytes(contentType)` (exported) says which bodies are read as
  bytes, so a client can read those with `arrayBuffer()` and the others with `text()`.
- 4442ddd: The TypeScript SDK writes `multipart/form-data` request bodies (it took a string you had encoded).
  `<op>Request` takes the body's value as its schema says, a string member also as a file
  (`Uint8Array`, `flowapiFile(bytes or text, name, type)`, or `await flowapiBlob(blobOrFile)`), and
  returns the parts as bytes with `content-type: multipart/form-data; boundary=...`: files as file
  parts (the name's last path segment, `blob` when none), strings, numbers and booleans as text, objects
  as JSON, an array's items one part each under its name in order. The body's `encoding` (the
  Encoding Object's `contentType` and `headers`, given by you: the description's is not read yet)
  sets a part's type and headers. The boundary is the content type's, or 128 random bits from
  `globalThis.crypto`. A request body neither JSON, text nor a form may be given as a `Uint8Array`,
  and `$Request.body` is `string | Uint8Array`. `<op>Request` refuses names, file names, types and
  headers that would change the parts (a CR, LF or NUL, a quote, `..`), `null`, a Blob not read yet
  and a boundary a part holds, with new `FlowapiRequestError` reasons. A schema named `Promise`,
  `ArrayBuffer`, `FlowapiRequestError`, `FlowapiFile` or `FlowapiEncoding` is declared under another
  name with the SDK, with a warning.

## 0.1.0 — 2026-11-24

- First version: flowapi's generator compiled to WebAssembly, with `generate`, `load`,
  `formatDiagnostic`, `@localix-ai/flowapi/node` (`generateFile`, `readDescription`) and the
  `flowapi` command. JSON only.
- The `sdk` option and the command's `--sdk`: each operation's request and response functions
  after the types, with no HTTP library, byte for byte what the Rust command prints. Swagger 2.0
  descriptions are read too.
