# AGENTS.md

Instructions for AI coding agents working on this repository.

flowapi makes types from API descriptions: Swagger 2.0, OpenAPI 3.0, 3.1 and 3.2 and JSON Schema
read into a type graph that no language knows, then printed by a target (TypeScript or Rust). Its promise is that the types follow the specification: every value a schema accepts, its
type accepts, proven on the official suites. [CONTRIBUTING.md](CONTRIBUTING.md) is what a change has
to keep, and it binds an agent as it binds a person. This file says where things are and how to
work here.

## Layout

- `crates/flowapi`: the core. No I/O, no state between calls, no dependency. Its layers, each a
  module that imports only those below it (`tests/repo.rs` holds the table):
  - `id`, `diag`, `uri`, `json`: ids, diagnostics, RFC 3986 references, our own JSON reader that
    keeps every node's byte offset.
  - `source`: the documents, what each is (`openapi` or `swagger` version, or `$schema`
    dialect), JSON Pointers.
  - `schema`: [`Keyword`](crates/flowapi/src/schema/keyword.rs), every keyword of every dialect;
    where a schema holds schemas (`sub.rs`); the index of `$id`, `$anchor` and big objects.
  - `spec`: what a description names: components, a JSON Schema's root, operations
    (`operations/reader.rs`, `responses.rs`, `style.rs` how a parameter is serialized; Swagger
    2.0's bodies, forms and media types lifted into the 3.x model in `swagger.rs`).
  - `graph`: schemas into types. `build.rs` (a queue, no recursion), `read.rs` (one keyword),
    `own.rs` (a schema's own values), `narrow.rs` and `literal.rs` (`enum`, `const`), `ops.rs`
    (operations as types).
  - `lower`: the graph made ready for a target: Any/Never edges made canonical, kinds as a
    fixpoint, unions and intersections simplified and merged, direct cycles broken.
  - `program`, `names`: the [`Target`](crates/flowapi/src/program.rs) contract and the one pass
    that names every type.
- `crates/flowapi-ts`: the TypeScript target. `tests/cases/*.json` and the `.ts` kept beside each.
  `src/sdk.rs` the opt-in SDK (`--sdk`): `sdk/request.rs`, `sdk/response.rs` and the runtime
  printed once (`sdk/runtime.ts` writes requests, `sdk/multipart.ts` their `multipart/form-data`
  bodies, `sdk/read.ts` reads responses);
  `tests/sdk/*.json` kept like the cases.
- `crates/flowapi-rust`: the Rust target (serde types). `tests/cases/*.rs`, kept like the TypeScript ones.
  `src/sdk.rs` the opt-in SDK (`--target rust --sdk`): `sdk/request.rs`, `sdk/param.rs`,
  `sdk/response.rs`, and the runtime printed once as `pub mod sdk` (`sdk/runtime.rs` and
  `sdk/errors.rs`, its modules `sdk/write.rs`, `sdk/expand.rs` and `sdk/finite.rs`, never compiled
  here: this crate has no serde); `src/print/integer.rs` the `Integer` type printed with the types;
  `tests/sdk/*.rs` kept for the TypeScript SDK's cases and `tests/sdk/names.json`.
- `crates/flowapi-sdk`: the SDK layer, above the targets: what a client needs beside a target's
  SDK, which no type depends on. It reads the documents through the core's public modules
  (`doc.rs`: references followed, each endpoint's Operation Object found again), `servers.rs`
  (3.x `servers` at three levels with variables; 2.0's `host`, `basePath`, `schemes` lifted) and
  `security.rs` (`securitySchemes` or 2.0's `securityDefinitions`, `security`) into `model.rs`,
  and `print.rs` prints the TypeScript table put after `--sdk`'s output (`flowapiServers`,
  `flowapiSecuritySchemes`, `flowapiOperations`), naming the functions as `flowapi-ts` does. No
  I/O (G5). `tests/cases/*.json` and the `.ts` kept beside each (`--test table`, each case's
  `x-flowapi-covers` the ledger's markers); `tests/tsc.mjs` type-checks and imports them after the
  `--sdk` output. The `flowapi-sdk.yml` config, also no I/O: `config/shape.rs` (and
  `shape/schemes.rs`) its format once, as tables that `config/read.rs` validates by,
  `config/build.rs` reads into `config/model.rs` and `config/schema.rs` prints as JSON Schema;
  `config/check.rs` and `config/names.rs` hold it to the description, `config/issue.rs` the
  issues; `ops.rs` the operations by id and route, `words.rs` names as words, `paging/` the
  pagination a schema fits (`fit.rs`) or suggests (`suggest.rs`), `draft.rs` (`draft/naming.rs`)
  a first config, `summary.rs` the compact summary. `tests/config/*.yml` with the issues kept
  beside each (`--test config`), `tests/schema.json` (`--test schema`, the reader and the schema
  agree both ways), `tests/draft/` (`--test draft`), `tests/summary/` (`--test summary`).
  `encoding.rs` a `multipart/form-data` body's encoding, put in the table. `client.rs` the whole
  TypeScript client from a config (`client_files`): `client/resources.rs` a class per resource,
  `client/entry.rs` the client class and `index.ts`, `client/core/*.ts` the runtime, kept as
  TypeScript and printed as it is (each file held to 300 working lines by `tests/client.mjs`).
  `tests/client/<case>/` a config and a description with the client kept beside them
  (`--test client`); `tests/client.mjs` type-checks them and runs `tests/client/*.mjs` against a
  stub `fetch`, each test naming the pitfall it holds. `repo.rs` the whole repository around the
  client (`repo_files`): `repo/package.rs` `package.json`, the TypeScript configs and the
  license, `repo/texts.rs` the scripts, CI, release-please and the repository's `AGENTS.md`,
  `repo/mock.rs` the mock server's routes, `repo/suite.rs` a test per resource, `repo/readme.rs`
  `README.md` and `api.md`, `repo/example.rs` a schema's example value, `repo/lock.rs`
  `.flowapi/lock.json` (the shape the `sdk-anything` plugin reads), `repo/plan.rs` what writing the
  repository changes on disk (`plan`: by the lock and what the host found at each path; every host
  runs it, none decides alone); `repo/files/` the texts printed
  as they are (scripts, `server.mjs`, the licenses); `sha256.rs` our own SHA-256.
  `tests/repo/<case>/` the repository of each client case but its `src/`, kept (`--test repo`).
- `crates/flowapi-cli`: the `flowapi` command, all of flowapi's I/O. `src/sdk.rs` the subcommands
  put before the path (`summary`, `config draft|schema|check`; `tests/sdk_cli.rs`);
  `src/generate.rs` `generate`, `generate/disk.rs` what is on disk for the SDK layer's `plan`,
  `generate/apply.rs` the writes, staged first (`tests/generate_cli.rs`).
- `crates/flowapi-yaml`: a description's text as the core reads it, shared by the command and the
  WebAssembly module: which document is YAML (`is_yaml`), YAML 1.2 read into JSON with each node's
  YAML place (`reader.rs`, `saphyr-parser`), and the line and column of an offset. No I/O (G5).
- `crates/flowapi-wasm`: the core, the TypeScript target and the YAML reader as a WebAssembly
  module for the npm package, behind four exports of its own (`src/abi.rs`: JSON in and out
  through linear memory, no wasm-bindgen); the one crate with `unsafe` (`deny`, not `forbid`;
  `tests/lints.rs`). Its `sdk` feature adds the SDK layer's config commands (`src/sdk.rs`, the
  export `flowapi_sdk`) and `generate`'s two halves, `repo` and `plan` (`src/repo.rs`).
- `crates/flowapi-wasm-sdk`: the package's second module, `flowapi-wasm` with `sdk` linked into
  a module of its own (no code, no `unsafe`; `tests/sdk.rs`), so the generator's module stays
  without it. Built in a cargo run of its own: with `-p flowapi-wasm` it would turn `sdk` on.
- `npm/`: `@localix-ai/flowapi`, the modules with a typed API (`src/index.ts`, its requests in
  `src/request.ts`), the files a description reaches read from disk (`src/node.ts`) and the
  `flowapi` command (`src/cli.ts`; `summary` and `config` in `src/sdk-cli.ts`, `generate` in
  `src/generate-cli.ts` with the disk it reads and writes in `src/generate-disk.ts`; paths joined
  as Rust's in `src/paths.ts`); JSON and YAML.
  `bun run check` in `npm/` builds it and holds it to the Rust command byte for byte.
- `conformance/`: the oracles that hold flowapi to the specifications
  ([conformance/README.md](conformance/README.md)): the official suites' pinned copies
  (`suites/`), the suite oracle (`suite/`, O3; `suite-rust/` for the Rust target), the same
  oracle run on the other generators (`compare/`; `suite-rust/own/` our own cases in the suite's
  format, run beside it), random schemas against flowschema (`props/`, O5),
  the OAI's example descriptions (`oai/`, O2), the SDK run under Node (`sdk/`, O8; `sdk-rust/` for
  the Rust SDK, its own cases in `sdk-rust/cases.json`), the
  requirements ledger (`ledger/`, O1), the
  TypeScript compiler they check with (`ts/`), and their results (`results/`, `REPORT.md`).
- `bench/`: the benchmarks against the other generators, their budgets and results
  ([bench/README.md](bench/README.md)); `bench/competitors/` the other tools, pinned.

## Commands

```sh
scripts/check                         # fmt, clippy, the tests, the wasm build, the oracles
cargo test -p flowapi --test repo     # the repository's rules; also --test json, --lib
cargo test -p flowapi-ts --test print # the TypeScript kept for each case
FLOWAPI_BLESS=1 cargo test -p flowapi-ts --test print   # write the kept TypeScript anew
(cd conformance/ts && npm ci)         # once: the TypeScript compiler the oracles use
(cd conformance/suite && cargo run --release)           # O3 on the JSON Schema suite
node conformance/compare/compare.mjs                   # O3 on the other generators too
(cd conformance/suite-rust && cargo run --release -j 1 --bin suite-rust)   # O3 for Rust
(cd conformance/props && cargo run --release)           # O5, random schemas; `-- <seed>` for another
node conformance/oai/run.mjs                           # O2, after cargo build --release
node conformance/sdk/run.mjs --corpus                  # O8, the SDK; after cargo build --release
node bench/competitors/sdk.mjs                         # writing requests against openapi-fetch
cargo test -p flowapi-rust --test sdk                  # the Rust SDK kept for each case (FLOWAPI_BLESS=1)
(cd conformance/sdk-rust && cargo run --release -j 1 --bin sdk-rust)   # O8 for the Rust SDK
(cd conformance/sdk-rust && cargo run --release -j 1 --bin corpus -- --build)   # its corpus
(cd bench/sdk-rust && cargo run --release -j 1)        # writing requests against progenitor's way
cargo test -p flowapi-ts --test fuzz                   # O7, no input panics
cargo test -p flowapi-sdk --test table                 # the SDK layer's tables (FLOWAPI_BLESS=1)
cargo test -p flowapi-sdk --test config                # also --test schema, draft, summary (FLOWAPI_BLESS=1)
cargo test -p flowapi-sdk --test client                # the clients kept in tests/client/ (FLOWAPI_BLESS=1)
node crates/flowapi-sdk/tests/client.mjs               # them type-checked and run against a stub fetch
cargo test -p flowapi-cli --test sdk_cli               # summary and config as the command runs them
cargo test -p flowapi-sdk --test repo                  # the repositories kept in tests/repo/ (FLOWAPI_BLESS=1)
cargo test -p flowapi-cli --test generate_cli          # generate's writing rules as the command runs them
node crates/flowapi-sdk/tests/tsc.mjs                  # them type-checked after --sdk; after cargo build --release
cargo run --release -p flowapi-cli -- <description.json> -o types.ts
bench/run.sh                          # the benchmarks, then held to bench/budgets.json
```

## Rules

Each rule has an id; cite it in a commit or a review that touches it. A rule marked _checked_
fails the checks when broken.

- **G1** _checked_ (lints, `tests/print.rs` deep inputs): no panic in working code, and no input
  exhausts the stack: the graph is built from a queue, walks keep their own stacks, the reader
  bounds nesting at 512 levels, inline printing at `names::MAX_INLINE`.
- **G2** _checked_ (`wildcard_enum_match_arm`, O1): no keyword is dropped in silence. Every
  keyword is a variant of `Keyword`, matched without `_`: given a type, recorded in
  `Type::lossy`, or named an annotation.
- **G3** _checked_ (O3 for both targets, O5): a type accepts every value its schema accepts:
  soundness is 100% of the suite's valid cases and of O5's. Precision, the share of invalid cases a
  type rejects, never falls.
- **G4** _checked_ (`tests/print.rs`, `disallowed-types`): the same input prints the same bytes.
  A hash map is looked up, never walked into output.
- **G5** _checked_ (`tests/repo.rs`, the wasm build): the core and the targets do no I/O: no
  file, socket, process, thread, clock or environment. The CLI is the shell.
- **G6** _checked_ (`clippy.toml`): no shared mutable state and no `Rc`: the arenas hold every
  node, an edge is an id.
- **G7**: the input is never changed; each stage takes the one before by value and returns a new
  value.
- **G8**: a diagnostic names the document, the byte offset (the CLI adds line and column), the
  rule (a requirement id or our code) and what the data holds.
- **G9** _checked_ (`tests/repo.rs`): a layer imports only those below it; a target sees only a
  `Program`.
- **G10** _checked_ (`tests/repo.rs`, lints): at most 300 lines of working code per file and 100
  per function, 6,000 in the core; `x.rs` beside `x/`, never `x/mod.rs`; unit tests in
  `x_tests.rs` beside `x.rs`.
- **G11**: no dependency in the core or the targets without a reason written beside it.
- **G12** _checked_ (`bench`, `check`): the budgets of `bench/budgets.json` hold. A figure in the
  README comes from `bench/results/` or `conformance/results/`, never typed by hand.
- **G13** _checked_ (`conformance/ledger`): the requirements ledger is a snapshot of the
  specifications; its report is written by a script; an unknown requirement id is an error.

## Before you say it is done

- `scripts/check` passes; for a change to the graph or a target, O3's soundness is still 100%
  and its precision did not fall (`conformance/results/suite.md` in the diff says so).
- The kept TypeScript of a changed case is read like code (`git diff crates/flowapi-ts/tests`).
- A user-visible change has a line under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md).
