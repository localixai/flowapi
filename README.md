<h1 align="center">flowapi</h1>

<p align="center"><b>Types from API descriptions that follow the specification.</b><br>
Swagger 2.0, OpenAPI 3.0, 3.1 and 3.2 and JSON Schema in, TypeScript or Rust out. Every value a schema
accepts, its type accepts: checked on the official JSON Schema Test Suite.</p>

<p align="center"><a href="#how-it-holds-to-the-specification">Conformance</a> ·
<a href="#speed">Speed</a> · <a href="#use">Use</a> · <a href="#how-it-works">How it works</a></p>

<p align="center">
  <img src="assets/output-dark.png" alt="An OpenAPI description beside the TypeScript flowapi prints for it: Pet with its described id, the kind as a union of literals, tags as a non-empty tuple, owner as a string or null, x- extensions as a template literal index signature; GetPet's path parameter and 200 response; and Paths" width="860">
</p>

<p align="center">
  <img src="assets/compare-dark.png" alt="flowapi beside openapi-typescript and hey-api: valid values accepted on the official JSON Schema Test Suite (100% against 90.6% and 94.6%), invalid values rejected (63.3% against 51.4% and 42.2%), broken output (0 against 56 and 7 groups), and the time to generate Slack's, Stripe's, GitHub's and Cloudflare's descriptions, peak memory on Cloudflare's and how many outputs type-check" width="860">
</p>

## How it holds to the specification

For every schema in the official
[JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite), flowapi prints
a type, and each test value is written as a literal of that type and handed to the TypeScript
compiler (`strict`, `exactOptionalPropertyTypes`). If the compiler refuses a valid value, the type
is wrong. If it refuses an invalid one, the type caught it.

| Draft 2020-12 | Result |
| --- | --- |
| Valid values accepted (soundness) | **100.0%** (779 of 779) |
| Invalid values rejected (precision) | 63.3% (342 of 540) |
| Groups that failed to generate, or printed invalid TypeScript | 0 of 409 |

From [conformance/results/suite.md](conformance/results/suite.md). The same measurement, each
schema wrapped as an OpenAPI 3.1 component and each tool run with its defaults
([conformance/results/compare.md](conformance/results/compare.md)):

| Generator | Valid accepted | Invalid rejected | Failed to generate | Invalid TypeScript |
| --- | ---: | ---: | ---: | ---: |
| flowapi 0.1.0 | **100.0%** | **63.3%** | 0 | 0 |
| openapi-typescript 7.13.0 | 90.6% | 51.4% | 7 | 49 |
| @hey-api/openapi-ts 0.99.0 | 94.6% | 42.2% | 7 | 0 |

TypeScript has no type for some keywords (`pattern`, `minimum`, `uniqueItems`). There the type
accepts more than the schema does, and a comment above it names the keywords it does not check.
Every keyword of every dialect is handled one of three ways: it becomes part of the type, it is
listed in that comment, or it is read as an annotation. The JSON reader is our own and passes
[JSONTestSuite](https://github.com/nst/JSONTestSuite).

The suite has a few hundred schemas, so two more checks look past it. Thousands of random
schemas and values are generated from fixed seeds; our own validator, flowschema (which passes
the whole suite), says which values are valid, and every valid one must type-check against
flowapi's type ([conformance/results/props.md](conformance/results/props.md)). And every valid
example description the OAI keeps beside its schemas generates and type-checks
([conformance/results/oai.md](conformance/results/oai.md)). A requirements ledger lists every
MUST, SHOULD and MAY of OpenAPI 3.0.4, 3.1.2, 3.2.1 and JSON Schema 2020-12 that a types
generator answers to, each held by a test or named as something a type cannot state
([conformance/REPORT.md](conformance/REPORT.md)).

One TypeScript limit shows in rare cases: an object literal with a key named like a member
every object inherits (`constructor`, `toString`) is checked against `Object`'s member when the
key is not declared, so `{ constructor: [0] }` can be refused by a type that allows it.

## Speed

Each tool ran as a fresh process on real descriptions, five rounds each, on an Intel N150. The
times are medians
([bench/results/speed.md](bench/results/speed.md)):

| Description | Size | flowapi | openapi-typescript 7.13 | @hey-api/openapi-ts 0.99 |
| --- | ---: | ---: | ---: | ---: |
| Slack | 999 KB | 14 ms | 892 ms | 992 ms |
| Stripe | 8.3 MB | 114 ms | 2.44 s | 10.57 s |
| GitHub | 14.3 MB | 185 ms | 3.73 s | 4.49 s |
| Cloudflare | 29.8 MB | 435 ms | 7.96 s | 12.15 s |

On Cloudflare's description flowapi peaks at 140 MB of memory; the other two take 937 MB and
1027 MB. flowapi's output type-checks for all six descriptions in the benchmark. The limits it is
held to are in [bench/budgets.json](bench/budgets.json).

## Use

```sh
cargo run --release -p flowapi-cli -- openapi.json -o types.ts
```

```ts
// One type for each component schema and each operation, and Paths:
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

If the schema lets an object hold other properties, its type has an index signature for them.
Swagger 2.0 descriptions are read too, into the same types as 3.x: `definitions` are the named
types, a `body` or `formData` parameter is the request body under each media type the
operation consumes, and a response's `schema` is its content under each type it produces.
A relative `$ref` resolves against the document's own URI: the path you give the command, or
the URI the library is given with each document; flowapi assumes no other base.
Files that a `$ref` names by relative path are read too, and so are YAML descriptions (`.yaml`,
`.yml`, or any file that does not open with `{` or `[`). YAML is read as JSON's data model under
YAML 1.2's core schema: a key that is not a string becomes its JSON text (`200:` is `"200"`) and
anchors are expanded. What JSON cannot hold is an error with the YAML line and column: `.inf`
and `.nan`, a mapping as a key, a repeated key, `<<` merge keys, a second document, or aliases
that would copy more than a million nodes. The YAML reader passes the YAML test suite
(`crates/flowapi-cli/tests/yaml_suite.rs`).

## How it works

The core (`crates/flowapi`) reads the documents into a graph of types that belongs to no
language. It keeps the byte offset of every JSON node for error messages, resolves `$ref` through
`$id` and `$anchor`, and builds schemas from a work queue, so a long chain of references cannot
overflow the stack. Names are chosen in one pass at the end. The core reads no files and has no
dependencies, and it builds for WebAssembly. Each target language is its own crate; TypeScript is
`crates/flowapi-ts`. The rules are in [AGENTS.md](AGENTS.md).

MIT. Made by [Localix Lab](https://localix.ai).
