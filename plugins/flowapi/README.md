# flowapi plugin

flowapi's plugin for coding agents. It carries flowapi's documentation as a skill, so an agent
can make types and request functions from an OpenAPI or JSON Schema description without reading
the repository. It also carries a pipeline that makes a TypeScript SDK repository: the agent
writes one small file, `flowapi-sdk.yml`, with resource and method names, pagination, auth,
environments, and the operations left out and why. flowapi prints the whole repository from the
description and that file, a deterministic judge checks the result, and the agent changes the
config until the judge passes. It never edits generated files by hand.

It is one package in two forms: a Claude Code plugin (`.claude-plugin/plugin.json`,
`hooks/hooks.json`) and an Agent Plugins 1.1.0 core package (`plugin.json`, `hooks.json`). The
skills and the guard are shared. The commands and the reviewer agent are Claude Code's alone.

It runs flowapi 0.2.0 through `npx` (`bin/flowapi`), so nothing else needs installing.

## Install

```sh
claude plugin marketplace add localixai/flowapi
claude plugin install flowapi@flowapi
```

Node 20 or newer must be on `PATH`.

## What it does

- The `flowapi` skill: what flowapi does, which output fits which need (types, `--sdk`
  functions, a whole SDK repository), the commands, diagnostics, and flowapi's own docs as its
  references: the CLI's help, the README's usage, the npm package's API, and how the types follow
  the specification.
- The `sdk` skill, `/flowapi:sdk <description> [folder]`: summary, draft config, edits, check,
  generate, install and judge, around again until the judge passes.
- `/flowapi:refine`, `/flowapi:update` and `/flowapi:release` improve the config, take a new
  version of the description, and prepare a release. Publishing is left to you. The
  `sdk-reviewer` agent reads a generated SDK's API from its user's side.
- The guard hook refuses an edit of a file listed in `.flowapi/lock.json`, unless it is under
  `src/lib/` or `examples/`, where your own code lives. The refusal tells the agent to change
  `flowapi-sdk.yml` instead.
- The judge checks a generated repository:

```sh
node bin/judge.mjs ./my-sdk           # a table
node bin/judge.mjs ./my-sdk --json    # the report as JSON
```

| # | Step | What it checks |
| --- | --- | --- |
| 0 | determinism | two generations into empty folders are equal byte for byte |
| 1 | typecheck | `tsc --noEmit` |
| 2 | lint | the repository's `scripts/lint` |
| 3 | build | `scripts/build` |
| 4 | dist types | `tsc -p tsconfig.dist.json` over the built declarations |
| 5 | package | `publint --strict` and `attw --pack .` |
| 6 | test | `scripts/test`, against a mock server made from the description |

It stops at the first failure (`--keep-going` runs the rest), shows the first 40 lines of what
failed, and reports a missing script as skipped, never as passed. `--steps 1,3` runs some steps;
`--generate "<command>"` replaces the generate command, with `{out}` for the folder. The judge
installs nothing and publishes nothing; install the SDK's dependency first (`npm install`).

`bin/flowapi` is flowapi at the version this plugin pins (`lib/flowapi-version`), through `npx`.
Every command and the judge's step 0 go through it. `FLOWAPI_PLUGIN_FLOWAPI` names another
flowapi command to use instead, such as a local build.

## Files it relies on

`.flowapi/lock.json` is written by flowapi: the hashes of the description, the config and
every generated file. Its shape is in [docs/lock.md](docs/lock.md).

## License

MIT. See [LICENSE](LICENSE).
