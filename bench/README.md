# Benchmark

What it costs to turn a large real API description into TypeScript types, measured the same way
for the `flowapi` command and for the two generators most projects use,
[openapi-typescript](https://github.com/openapi-ts/openapi-typescript) and
[@hey-api/openapi-ts](https://github.com/hey-api/openapi-ts). This is what flowapi is held to:
[`budgets.json`](budgets.json) states the rules, `cargo run --release --bin check` fails when a run
breaks one, and every run's results are committed under [`results/`](results/), written by the
programs here, never by hand.

The latest numbers: [`results/speed.md`](results/speed.md), and for the whole SDK
[`results/sdk-compare.md`](results/sdk-compare.md).

A crate of its own, outside the workspace: neither the other generators nor the corpus reach a
build of flowapi.

## What is measured

| Program | What it runs | Results |
| --- | --- | --- |
| `speed` | Each description of the corpus through each generator, every run a fresh process: its wall time, its peak memory, the size of what it wrote, and whether that output type-checks | `speed.json`, `speed.md` |
| `check` | The committed `results/` held to [`budgets.json`](budgets.json) | (its exit code) |
| `phases` | The time of each stage of flowapi (parse, index, graph, lower, names, emit) on one description, in process | (printed) |
| `competitors/sdk.mjs` | Writing 100,000 requests a round with the SDK flowapi generates (`--sdk`) against openapi-fetch's serializers (pinned in `competitors/package.json`), on the same three operation shapes, each pair first checked to write the same request; one Node process, sequential, the median of 5 rounds | `sdk.md` |
| `competitors/sdk-compare.mjs` | The whole client SDK a user gets from flowapi (`config draft`, then `generate`) and from each other client generator, on GitHub, Stripe, Twilio and two small descriptions: generation time and memory, what it wrote, `tsc --strict` errors, and what each SDK sends to one mock server (below) | `sdk-compare.json`, `sdk-compare.md` |
| `measure` | One command as a fresh process, its wall time and peak memory as `speed` takes them, for the programs written in Node | (printed) |

## The corpus

[`corpus/corpus.tsv`](corpus/corpus.tsv) lists each description with the commit it is taken at and
the sha256 and size of the JSON measured; [`corpus/fetch.sh`](corpus/fetch.sh) fetches them into
`data/`, which git ignores, and fails on any file whose hash differs.

| Name | Description | Source |
| --- | --- | --- |
| `slack` | Slack's Web API, OpenAPI 3.0 | APIs-guru's openapi-directory, YAML |
| `twilio` | Twilio's API (2010), OpenAPI 3.0 | APIs-guru's openapi-directory, YAML |
| `digitalocean` | DigitalOcean's API v2, OpenAPI 3.0 | APIs-guru's openapi-directory, YAML |
| `stripe` | Stripe's API, OpenAPI 3.0 | stripe/openapi, `openapi/spec3.json` |
| `github` | GitHub's REST API, OpenAPI 3.0 | github/rest-api-description, `api.github.com.json` |
| `cloudflare` | Cloudflare's API, OpenAPI 3.0 | cloudflare/api-schemas, `openapi.json` |

APIs-guru keeps its descriptions as YAML only, and flowapi reads JSON, so `fetch.sh` converts
those three with [`competitors/yaml2json.mjs`](competitors/yaml2json.mjs) (the `yaml` package at
its pinned version); both the YAML and the JSON are pinned by hash, and every generator reads the
same JSON. A file is copied from a local clone of its repository when one is checked out at the
pinned commit (`CLONES`, by default the workspace's reference clones), else downloaded from
raw.githubusercontent.com at that commit.

DigitalOcean's description, as APIs-guru keeps it, still names files of DigitalOcean's own
repository in its `discriminator.mapping` values (`models/*.yml`), which were not brought along. A
generator that follows those names as references fails on it; the results record each failure with
what the generator said, and the description is kept as it is, a real one.

## How

- **The generators.** `flowapi` is the release build of the command
  (`../target/release/flowapi <in.json> -q -o <out.ts>`, fat LTO, one codegen unit). The other two
  are pinned to exact versions in [`competitors/package.json`](competitors/package.json) and run
  by [`competitors/run.mjs`](competitors/run.mjs) through their documented Node APIs, each asked
  for types alone: openapi-typescript's `openapiTS` and `astToString` with its defaults, and
  hey-api's `createClient` with the `@hey-api/typescript` plugin and nothing else (no client, no
  SDK, no formatter), its `types.gen.ts` kept.
- **Time.** Each run is a fresh process, timed from before it is started to when it is reaped,
  so a Node tool's start-up is counted, as its user pays it. `speed` runs each generator 5 times
  on each description and keeps the median; `--quick` runs it once. Runs are sequential, one
  process at a time, and the machine should be otherwise idle.
- **Memory** is the peak resident set of the process, as the kernel accounts for it (`ru_maxrss`
  of `wait4`). Read this way on Linux and macOS; the results are taken on Linux.
- **Type-checks.** Each output is checked by the TypeScript compiler of the conformance harness
  (`../conformance/ts`, TypeScript 7) as
  `tsc --ignoreConfig --strict --exactOptionalPropertyTypes --noEmit --target es2022 --skipLibCheck <out.ts>`.
  When that compiler is not installed, the outputs are not checked, the results say so, and
  `check` fails the rule that needs it.
- **Failures.** A generator that exits with an error, writes nothing or runs past the time limit
  (5 minutes by default, `--timeout`) is recorded as failed with what it said, and the run moves
  on. A description no other generator finished counts for flowapi.
- **The machine** (processor, cores, memory, system) and every tool's version are written into the
  results with the figures.

Timings depend on the machine; the ratios between the generators move far less than the times do.

## The whole SDK

[`competitors/sdk-compare.mjs`](competitors/sdk-compare.mjs) compares what a user gets: a client
they call, with its credentials, retries, pages and errors. The descriptions and the calls are
[`competitors/sdk-compare/cases.mjs`](competitors/sdk-compare/cases.mjs): GitHub's, Stripe's and
Twilio's from the corpus, and two small ones of flowapi's own tests (`shop`, `auth`) that carry
the security scheme kinds the large ones lack (an API key in a header, a query and a cookie).

- **The generators**, pinned in [`competitors/package.json`](competitors/package.json), each in
  the client mode its documentation gives for `fetch` (`run.mjs sdk`): flowapi's `generate` on
  the config `config draft` writes (GitHub's description declares no security scheme, so its
  config declares a bearer token, as a user's would); @hey-api/openapi-ts with its fetch client
  and SDK plugins; openapi-typescript's types with openapi-fetch, a client by composition, its
  five lines of composition checked with the types and not counted as generated; orval with its
  `fetch` client; swagger-typescript-api with its `fetch` client. openapi-generator is not run:
  it needs Java or Docker. The results name every generator left out and why.
- **Generation**: each round a fresh process into an empty folder, timed with its peak memory by
  `measure`, the median of 5 (`--rounds`). Files and bytes are everything a generator wrote:
  flowapi writes a whole repository (tests, mock, README), the others the client alone.
- **Type-checks**: `tsc --ignoreConfig --strict --target es2022 --module nodenext --lib
  es2022,dom --skipLibCheck --noEmit` on each SDK's TypeScript, with the conformance harness's
  compiler; the errors counted.
- **Behavior**: each SDK emitted as ES modules and called through its own surface
  ([`competitors/sdk-compare/adapters.mjs`](competitors/sdk-compare/adapters.mjs)) against one
  mock server made from the description ([`competitors/sdk-compare/mock.mjs`](competitors/sdk-compare/mock.mjs)):
  it finds each request's operation, records it as it arrived and answers with an example of the
  operation's success response, or with what a test queued. Every SDK is given the description's
  own server; its `fetch` is sent to the mock with the path, query, headers and body kept. The
  tests ([`competitors/sdk-compare/behavior.mjs`](competitors/sdk-compare/behavior.mjs)): path
  parameters, query arrays, a JSON or form body held to the bytes the specification asks; the
  credential of each security scheme kind; a retry on 429 and on 503 after `Retry-After`; a list
  read to its last page; a 404 given back as a value with its status and body.
- **The feature table** is filled only by what a test observed. `n/a` is a feature the generated
  code has no surface for (no option for a scheme's credential, no retry, no pages): the caller
  writes it. `fail` is a surface that did not do what the specification asks, with what it sent.

The limits: a client by composition is as good as what its user adds, and the table does not
count that; a feature a generator leaves to its user is `n/a`, not `fail`, though the user still
writes it; hey-api's, orval's and swagger-typescript-api's other modes (axios, query hooks,
mutators) are not measured; the calls are a few per description, not every operation.

## Run

```sh
./run.sh                                       # everything (sdk-compare too): results/ written, then held to budgets.json
./run.sh --quick                               # the same, one round per generator and description
corpus/fetch.sh                                # the corpus alone, into data/
cargo run --release --bin speed -- --only stripe --quick    # one description, printed, nothing written
cargo run --release --bin check                # results/ held to budgets.json again
cargo run --release --bin phases -- data/github.json        # where flowapi's time goes
node competitors/sdk-compare.mjs && target/release/check      # the whole SDK alone, held to budgets.json
node competitors/sdk-compare.mjs --only shop,auth --rounds 1      # the small ones, written; sdk-complete fails until a whole run
```

`./run.sh` builds flowapi's release first (`cargo build --release` at the workspace root). The
competitors are installed with `npm ci` in `competitors/` when `node_modules` is missing; Node.js
22.18 or later.

## Budgets

| Rule | Holds |
| --- | --- |
| `flowapi-type-checks` | flowapi's output type-checks, strictly, on every description |
| `faster` | flowapi is at least 10x faster than the fastest other generator on every description |
| `memory` | flowapi's peak memory is no more than the leanest other generator's on every description |
| `sdk-type-checks` | flowapi's SDK type-checks under `tsc --strict` on every description of `sdk-compare` |
| `sdk-behavior` | flowapi's SDK passes every behavior test it is given: none fails, none is `n/a` |
| `sdk-generate-faster` | flowapi's `generate` is at least 10x faster than the fastest other client generator on every description |
| `sdk-complete` | `results/sdk-compare.json` is a whole run: every description, no `--only` |

A number changes only together with its reason, in the change that moves it.

## Pictures

`charts` draws `../assets/compare-dark.svg` and `.png` (the README's chart) from `results/speed.json` and `../conformance/results/compare.json`; `shots` draws `../assets/output-dark.svg` and `.png` from `shots/pets.json` and the TypeScript the library prints for it. Both run at the end of `run.sh`; dark only, as every demo of the lab. Each panel has its own scale.
