# @localix-ai/flowapi

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
