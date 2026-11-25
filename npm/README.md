# @localix-ai/flowapi

TypeScript types from OpenAPI 3.0, 3.1 and 3.2 and JSON Schema that follow the specification:
every value a schema accepts, its type accepts. This is flowapi's Rust generator compiled to
WebAssembly (about 207 KB gzipped), so it runs in Node 20+, Bun, Deno and browsers and has no
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

MIT
