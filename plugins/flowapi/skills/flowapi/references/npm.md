<!-- generated from npm/README.md by plugins/flowapi/scripts/docs.mjs: edit the source, not this file -->

# @localix-ai/flowapi

TypeScript types from OpenAPI 3.0, 3.1 and 3.2 and JSON Schema that follow the specification:
every value a schema accepts, its type accepts. This is flowapi's Rust generator compiled to
WebAssembly (about 208 KB gzipped), so it runs in Node 20+, Bun, Deno and browsers and has no
runtime dependency. Its output is the `flowapi` command's, byte for byte. What it promises, how
that is checked and how fast it is: [the main README](https://github.com/localixai/flowapi#readme).

```sh
npx @localix-ai/flowapi openapi.json -o types.ts
```

```ts
import { generate } from '@localix-ai/flowapi'
import { generateFile } from '@localix-ai/flowapi/node'

// The description's text, or [{ uri, text }, ...] with the documents its $refs reach.
const { files, diagnostics } = await generate(text, { output: 'types.ts' })

// From disk, with every file a relative $ref names (Node, Bun, Deno), JSON or YAML.
const result = await generateFile('openapi.yaml')
```

`files` is empty when a diagnostic is an error. Each diagnostic has its `severity`, `code`,
`message`, the document's `uri`, the byte `offset`, and the `line` and `column` the command
prints (`formatDiagnostic` prints it the same way).

`load()` returns a generator whose `generate` and `references` are synchronous. The module is
`flowapi.wasm`, found beside the package's code with `new URL('./flowapi.wasm', import.meta.url)`,
which Vite, webpack and Parcel follow. With another bundler, serve
`@localix-ai/flowapi/flowapi.wasm` yourself and pass its URL, its bytes or a compiled
`WebAssembly.Module` to `load(source)` before the first call.

YAML is read as the `flowapi` command reads it, by the same code: `.yaml` and `.yml` files, and
under any other name a text that does not open with `{` or `[`. A document's `uri` says which, so
give a YAML description as `{ uri: 'openapi.yaml', text }`: bare text is read as JSON, and
`references(text, uri)` takes the URI too. A diagnostic in a YAML document names its YAML line and
column, and a YAML text that JSON cannot hold is an error with the code `yaml.syntax`. The main
README says how YAML maps to JSON.

## A config for a whole SDK

The command has the Rust command's commands for an agent that writes an SDK's config,
`flowapi-sdk.yml`, and the SDK's repository from it, with the same arguments, output and exit
codes:

```sh
npx -y @localix-ai/flowapi summary openapi.json --tag pets      # the operations in compact JSON
npx -y @localix-ai/flowapi config draft openapi.json -o flowapi-sdk.yml   # a first config
npx -y @localix-ai/flowapi config check flowapi-sdk.yml --json # every issue; 1 on an error
npx -y @localix-ai/flowapi config schema > flowapi-sdk.schema.json
npx -y @localix-ai/flowapi generate flowapi-sdk.yml --out sdk  # the whole repository
npx -y @localix-ai/flowapi generate flowapi-sdk.yml --out sdk --check   # 1 when it differs
```

`generate` writes the same files, byte for byte, and the same `.flowapi/lock.json` as the Rust
command, by the same rules: all or nothing (each file staged beside its target first); a
generated file changed by hand since is a conflict that writes nothing and exits 1, except under
`src/lib/` and `examples/`, which are yours and kept; a file the lock does not list is never
touched; one no longer generated is deleted unless changed, kept with a warning when it was; a
path out of `--out` or through a symbolic link is refused. Which of those applies is decided in
the module, by the code the Rust command runs; the package only reads the disk (SHA-256 by
`node:crypto`) and makes the changes.

`summary` stays under `--max-bytes` (60000): over it, it gives the tags with their counts, to be
read one at a time with `--tag` or `--path`. `config check` reads the description the config's
`spec` names, relative to the config, or the one `--spec` names; it does not fetch a URL.

The same from the API, over documents you read (`readDescription` reads them from disk):

```ts
import { configCheck, configDraft, configSchema, summary } from '@localix-ai/flowapi'

const { text } = await summary(documents, { tag: 'pets', maxBytes: 60_000 }) // null: not OpenAPI
const draft = await configDraft(documents, '../openapi.json') // `spec` as the config will name it
const found = await configCheck({ uri: 'flowapi-sdk.yml', text: yaml }, documents)
// found.ok, found.issues: [{ severity, code, pointer, line, column, message, hint? }],
// found.text and found.json: the issues as `config check` and `config check --json` print them.
const schema = await configSchema()
```

`generate`'s two halves are there too, with no disk: `repoFiles(config, documents, lock?)` gives
every file of the repository, its new lock and the paths to look at, and `repoPlan(...)`, given
what is at each of them, the changes to make, the conflicts and what the command prints.

They are a second module, `flowapi-sdk.wasm` (about 413 KiB gzipped), compiled the first time
one of them is called; the types' `generate` never loads it. `loadSdk(source)` loads it as `load` loads the first
(serve `@localix-ai/flowapi/flowapi-sdk.wasm` with a bundler that does not copy it) and gives them
synchronous, with `configSpec(config)`: the `spec` a config names, for the host to read before
`configCheck`, and `repoFiles` and `repoPlan`. A config or a document that cannot be read gives no text and one diagnostic, as
`generate` does.

MIT
