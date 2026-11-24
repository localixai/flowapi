# AGENTS.md

Instructions for AI coding agents working on this repository.

flowapi makes types from API descriptions: OpenAPI 3.0, 3.1 and 3.2 and JSON Schema read into a
type graph that no language knows, then printed by a target (TypeScript now, Rust next). Its
promise is that the types follow the specification: every value a schema accepts, its type
accepts, proven on the official suites. [CONTRIBUTING.md](CONTRIBUTING.md) is what a change has
to keep, and it binds an agent as it binds a person. This file says where things are and how to
work here.

## Layout

- `crates/flowapi`: the core. No I/O, no state between calls, no dependency. Its layers, each a
  module that imports only those below it (`tests/repo.rs` holds the table):
  - `id`, `diag`, `uri`, `json`: ids, diagnostics, RFC 3986 references, our own JSON reader that
    keeps every node's byte offset.
  - `source`: the documents, what each is (`openapi` version or `$schema` dialect), JSON Pointers.
  - `schema`: [`Keyword`](crates/flowapi/src/schema/keyword.rs), every keyword of every dialect;
    where a schema holds schemas (`sub.rs`); the index of `$id`, `$anchor` and big objects.
  - `spec`: what a description names: components, a JSON Schema's root, operations.
  - `graph`: schemas into types. `build.rs` (a queue, no recursion), `read.rs` (one keyword),
    `own.rs` (a schema's own values), `narrow.rs` and `literal.rs` (`enum`, `const`), `ops.rs`
    (operations as types).
  - `lower`: the graph made ready for a target: Any/Never edges made canonical, kinds as a
    fixpoint, unions and intersections simplified and merged, direct cycles broken.
  - `program`, `names`: the [`Target`](crates/flowapi/src/program.rs) contract and the one pass
    that names every type.
- `crates/flowapi-ts`: the TypeScript target. `tests/cases/*.json` and the `.ts` kept beside each.
- `crates/flowapi-cli`: the `flowapi` command, all of flowapi's I/O.
- `conformance/`: the oracles that hold flowapi to the specifications
  ([conformance/README.md](conformance/README.md)): the official suites' pinned copies
  (`suites/`), the suite oracle (`suite/`, O3), the same oracle run on the other generators
  (`compare/`), the requirements ledger (`ledger/`, O1), the
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
- **G3** _checked_ (O3, `conformance/results/suite.md`): a type accepts every value its schema
  accepts: soundness is 100% of the suite's valid cases. Precision, the share of invalid cases a
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
