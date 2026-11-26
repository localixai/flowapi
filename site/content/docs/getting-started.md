# Getting started

Run flowapi on an OpenAPI or JSON Schema description and get one TypeScript file of types, from npm or from the Rust command.

## From npm

The package `@localix-ai/flowapi` is the generator compiled to WebAssembly. It runs in Node 20 or later, Bun, Deno and browsers, and has no runtime dependency.

```sh
npx @localix-ai/flowapi openapi.json -o types.ts
```

From code:

```ts
import { generate } from '@localix-ai/flowapi'
import { generateFile } from '@localix-ai/flowapi/node'

// The description's text, or [{ uri, text }, ...] with the documents its $refs reach.
const { files, diagnostics } = await generate(text, { output: 'types.ts' })

// From disk, with every file a relative $ref names (Node, Bun, Deno).
const result = await generateFile('openapi.json')
```

`files` is empty when a diagnostic is an error. Each diagnostic has its `severity`, `code`, `message`, the document's `uri`, the byte `offset`, and the `line` and `column`. The package reads JSON and YAML, as the Rust command does.

## The Rust command

The crates are not on crates.io. Install the command from a release tag:

```sh
cargo install --locked --git https://github.com/localixai/flowapi --tag v0.3.0 flowapi-cli
flowapi openapi.json -o types.ts
```

It prints the same bytes as the npm package. Both read YAML: `.yaml`, `.yml`, or any file that does not open with `{` or `[`. YAML is read as JSON's data model under YAML 1.2's core schema, so `200:` is the key `"200"`. Anything JSON cannot hold (`.inf`, a mapping as a key, a repeated key, `<<` merge keys) is an error that names the YAML line and column.

`--target rust` prints serde types instead of TypeScript, and `--sdk` adds the request and response functions described in [the SDK](sdk.md).

## As a library

A Rust program takes the crates from the repository at a tag:

```toml
[dependencies]
flowapi = { git = "https://github.com/localixai/flowapi", tag = "v0.3.0" }
flowapi-ts = { git = "https://github.com/localixai/flowapi", tag = "v0.3.0" }
```

`flowapi` is the core: it reads the documents into a graph of types and does no I/O. `flowapi-ts` and `flowapi-rust` print that graph.

## What comes out

One type for each component schema and each operation, and `Paths`:

```ts
export type Pet = {
  /** Unique within the store. */
  id: number;
  kind: "cat" | "dog";
  tags?: [string, ...string[]];
  [key: `x-${string}`]: string;
};
export type GetPet = {
  parameters: {
    path: {
      id: number;
    };
  };
  responses: {
    "200": {
      headers: {
        "X-Rate-Limit": number;
      };
      content: {
        "application/json": Pet;
      };
    };
  };
};
export type Paths = {
  "/pets/{id}": {
    get: GetPet;
  };
};
```

If a schema lets an object hold other properties, its type has an index signature for them. Where TypeScript has no type for a keyword (`pattern`, `minimum`, `uniqueItems`), the type accepts more than the schema does, and a comment above it names the keywords it does not check.

## Which descriptions

- OpenAPI 3.0, 3.1 and 3.2, and JSON Schema 2020-12 (drafts 4 to 2019-09 by their `$schema`).
- Swagger 2.0, read into the same types as 3.x: `definitions` are the named types, a `body` or `formData` parameter is the request body under each media type the operation consumes, and a response's `schema` is its content under each type it produces.
- `$ref` by JSON Pointer, `$id` and `$anchor`, and files named by a relative path. A relative `$ref` resolves against the document's own URI: the path you give the command, or the URI the library gets with each document.
