# Conformance

## O3: the JSON-Schema-Test-Suite through the TypeScript compiler

The official [JSON-Schema-Test-Suite](suites/json-schema-test-suite/SOURCE.md) is a list of
groups, each a schema and cases of data marked valid or invalid. The oracle generates flowapi's
TypeScript type for every group that applies to JSON Schema 2020-12 and asks the TypeScript
compiler whether each case's data fits that type.

- **Soundness (O3).** Every valid case must fit the type. The target is 100%: a valid case the
  type rejects is a bug in flowapi.
- **Precision (O3′).** The share of invalid cases the type rejects, per keyword file. Many
  keywords (`minimum`, `pattern`, `uniqueItems`, ...) have no TypeScript type, so 100% is out of
  reach; the number must not go down.

How a group is checked:

1. Its `compatibility` must admit 2020-12 (the suite README's algorithm); groups tagged
   `format-assertion` are left out, since 2020-12 reads `format` as an annotation.
2. Its schema is added as the first document, under
   `https://json-schema.org/flowapi-suite/<file>/<group>.json`, and each of its `externalSchemas`
   under its own retrieval URI (`http://localhost:1234/...`). No `$schema` is added: flowapi
   reads a schema without one as 2020-12.
3. The output goes to `out/<file>-<group>.ts`, followed by one line per case,
   `const c<i>: Root = <data>;`. The data is a fresh literal, so TypeScript's excess-property
   check applies to it.
4. One compiler run checks every file: `tsc --strict --exactOptionalPropertyTypes --noEmit
   --target es2022 --skipLibCheck`. Each diagnostic is mapped back to its case by file and line.

A group whose generation reports an error, or panics, is a generation error. A group whose
generated types do not compile (a diagnostic above the first case line) is invalid TypeScript.
Both are failures, listed by group, and their cases are not counted.

## Running it

    cd conformance/ts && npm ci              # once: TypeScript 7, pinned in package-lock.json
    cd conformance/suite && cargo run --release

It takes well under a second. The command prints the totals and writes:

- `results/suite.md`: a row per keyword file, the totals, then every unsound case (file, group,
  case, data, the compiler's first line), every invalid-TypeScript group and every generation
  error.
- `results/suite.json`: the same counts per file, with each unsound case's full compiler message.
- `out/`: the generated files, ignored by git. To see why a case failed, open its file or run the
  compiler on it alone.

Both results are written sorted and without times or paths, so a change to flowapi shows up as
a diff of `results/`. Commit them with the change that moved them.

## Reading the table

| Column | Meaning |
| --- | --- |
| Groups | Groups in the file, for every dialect. |
| Run | Those that apply to 2020-12 and are not `format-assertion`. |
| Gen. errors | Run groups flowapi refused or panicked on. |
| Invalid TS | Generated groups whose types do not compile. |
| Valid / Accepted / Sound % | Valid cases, those the type accepts, the share. |
| Invalid / Rejected / Precision % | Invalid cases, those the type rejects, the share. |

Percentages are rounded down, so 100.0 means every case.

## O8: the SDK, run

`flowapi --sdk` prints functions, so its oracle runs them. [`sdk/run.mjs`](sdk/run.mjs) turns
each row of OpenAPI 3.2.1's "Style Examples" table ([`suites/style-examples.json`](suites/style-examples.json),
written from the pinned text by [`sdk/extract.mjs`](sdk/extract.mjs); `--check` says whether it is
stale) and each case of [`sdk/cases.json`](sdk/cases.json) into an operation, generates the SDK,
type-checks and compiles it with the pinned TypeScript (`strict`, `exactOptionalPropertyTypes`),
and calls every request and response function under Node, comparing the strings exactly; a query
or a form body is also parsed back with WHATWG's `URLSearchParams`, a path percent-decoded. The
table's path parameter sits after a literal `x` in its segment (`/s<line>p/x{color}`): the table
is about the expansion, and a whole segment its undefined column leaves empty or `.` is refused. The
cases are the specification's own examples (Appendix C, the Parameter Object Examples) and the
pitfalls other generators' tests and changelogs show they fixed; each names the requirements it
covers, which the ledger reads. Some are refusals: a value the SDK must not write (a CR, LF or
NUL in a header or a `style: cookie` cookie, a path segment left empty or made `.` or `..`, NaN or
an infinity, written `{ "$number": "NaN" }` since JSON has none) expects
`refused: { location, parameter, reason }`. A case runs in 3.0.4, 3.1.2 and 3.2.1 unless it names its
versions (Swagger 2.0's `collectionFormat`, 3.2's `cookie` and `querystring`). `--corpus` also
generates the SDK of GitHub's, Stripe's and Cloudflare's descriptions (`bench/data`, from
`bench/corpus/fetch.sh`) and type-checks each.

    cargo build --release && node conformance/sdk/run.mjs --corpus

It writes `results/sdk.md` and exits 1 on any difference.

The Rust SDK (`flowapi --target rust --sdk`) has its own run, [`sdk-rust/`](sdk-rust/), a crate
outside the workspace. It makes the same descriptions from the same table and cases, plus its own
([`sdk-rust/cases.json`](sdk-rust/cases.json): numbers as JavaScript writes them, integers past
2^53 and past `i64`, enums, a struct's member order, a media range's body), generates each version's SDK with
the flowapi crates into one generated crate with the SDKs kept by
`crates/flowapi-rust/tests/sdk.rs`, builds it once (`-j 1`; a warning about it is a difference),
and runs every call, comparing as `run.mjs` does. An object with no named properties is a
`BTreeMap` in Rust, its members in key order: the calls that changes are held to
[`sdk-rust/order.json`](sdk-rust/order.json). Its `corpus` binary times generating the SDK of
GitHub's, Stripe's and Cloudflare's descriptions (never compiled: rustc needs more than 16 GB for GitHub's types)
and, with `--build`, builds Slack's under `/usr/bin/time -v`.

    cd conformance/sdk-rust && cargo run --release -j 1 --bin sdk-rust
    cd conformance/sdk-rust && cargo run --release -j 1 --bin corpus -- --build

They write `results/sdk-rust.md` (exit 1 on any difference) and `results/sdk-rust-corpus.md`.
