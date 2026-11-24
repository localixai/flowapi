# Contributing

```sh
scripts/check        # formatting, lints, tests, the WebAssembly build, the oracles
bench/run.sh         # the benchmarks, then the run held to bench/budgets.json
```

## What a change has to keep

- **Sound by the specification.** A type accepts every value its schema accepts: 100% of the
  valid cases of the official JSON Schema Test Suite type-check against the types printed for
  them, under `strict` and `exactOptionalPropertyTypes` (`conformance/`, O3). Where a type cannot
  narrow as far as its schema, it is wider and says so; it is never narrower.
- **Nothing dropped in silence.** Every keyword of every dialect is read: given a type, recorded
  as one the type does not check, or named an annotation.
- **Precise where the language allows.** The share of the suite's invalid cases the types reject
  never falls.
- **The same bytes for the same input**, on every machine and every run.
- **Fast and small on the largest descriptions.** The budgets are in
  [bench/budgets.json](bench/budgets.json); figures come only from `bench/results/`.
- **No panic, no `unsafe`, no I/O in the core.** The lints in `Cargo.toml` and
  `crates/flowapi/tests/repo.rs` hold them.

The rules, with their ids, are in [AGENTS.md](AGENTS.md).

## Releases

A change a user can see adds a line under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md). A tag
`v<version>`, the version in `Cargo.toml`, will make the release once the repository is public.
