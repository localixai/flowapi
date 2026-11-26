---
name: flowapi
description: How to use flowapi, which makes types from API descriptions - Swagger 2.0, OpenAPI 3.0 to 3.2 and JSON Schema into TypeScript or Rust (serde) types, optionally with request and response functions per operation (--sdk), from the command line or the npm API (@localix-ai/flowapi - generate, summary, configDraft, configCheck, configSchema). Use when asked for types or a client from an OpenAPI or JSON Schema description, to call flowapi or its npm package, or to read its diagnostics.
---

# flowapi

flowapi reads an API description into a graph of types and prints it as TypeScript or as Rust
(serde). Every value a schema accepts, its type accepts: where a language has no type for a
keyword (`pattern`, `minimum`, `uniqueItems`), the type is wider and a comment above it says
`Not checked by this type: pattern.` It reads Swagger 2.0, OpenAPI 3.0, 3.1 and 3.2 and JSON
Schema (2020-12 unless `$schema` names another draft), in JSON or YAML, with every file a
relative `$ref` names.

## Which way

| The person needs | Use |
| --- | --- |
| Types only | `flowapi openapi.json -o types.ts` (`--target rust -o src/api.rs` for serde types) |
| Types and a function per operation that writes its request and one that reads its response, with no HTTP library | `--sdk` on the same command; Rust: `--target rust --sdk` |
| A whole SDK repository: client classes per resource, retries, pagination, a mock server, tests, package, CI | The `sdk` skill (`/flowapi:sdk <description> [folder]` in Claude Code): `flowapi-sdk.yml`, then `flowapi generate` |
| Types from code (a build step, a browser, a bundler) | The npm API: `generate(text, { output })`, `generateFile(path)` from `@localix-ai/flowapi/node` |

## Running it

- From npm, no install: `npx -y @localix-ai/flowapi openapi.json -o types.ts`. Inside this
  plugin, `${CLAUDE_PLUGIN_ROOT}/bin/flowapi` is that command at the version the plugin pins.
- The npm command prints TypeScript only. `--target rust` is the Rust command's:
  `cargo install --locked --git https://github.com/localixai/flowapi --tag v<version> flowapi-cli`.
- `flowapi --version` prints the version. Both commands print the same bytes for the same input.

## Diagnostics

Each is `error[<code>]: <message>` or `warning[<code>]: ...`, then `at <file>:<line>:<column>`,
on standard error. An error writes no output and exits 1; warnings write the output and exit 0
(`--quiet` hides them). From the npm API, `files` is empty when a diagnostic is an error, and each
diagnostic has `severity`, `code`, `message`, `uri`, `offset`, `line` and `column`
(`formatDiagnostic` prints it as the command does). Fix the description or the `$ref` the error
names; never edit the printed file, which says `Do not edit: generate again.`

## References

Read only the one the task needs:

- [references/cli.md](references/cli.md): every flag and subcommand of both commands, their help
  as printed.
- [references/use.md](references/use.md): what is printed, YAML, `$ref` resolution, the
  TypeScript and Rust `--sdk` functions (parameters, bodies, multipart, responses, the errors they
  refuse with), `summary` and `config`, and `generate`'s repository and lock.
- [references/npm.md](references/npm.md): the npm package's API (`generate`, `load`,
  `summary`, `configDraft`, `configCheck`, `configSchema`, `repoFiles`, `repoPlan`), its two
  WebAssembly modules and bundlers.
- [references/conformance.md](references/conformance.md): how the types are held to the
  specification, and the one TypeScript limit to know.
