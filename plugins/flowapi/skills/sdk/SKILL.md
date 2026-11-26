---
name: sdk
description: Build a TypeScript SDK repository from an OpenAPI description with flowapi - write flowapi-sdk.yml, generate, and fix the config until the judge passes - and the rule that generated files change only through flowapi-sdk.yml. Use when asked to make, fix, update or release an SDK, when a file listed in .flowapi/lock.json needs a change, or when the person runs /flowapi:sdk <description> [folder].
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/bin/flowapi:*) Bash(node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs:*) Bash(npm install:*)
---

# An SDK repository with flowapi

flowapi writes every file of the SDK from two inputs: the API's description and
`flowapi-sdk.yml`. You write and fix the config; flowapi and the judge do the rest.

`F` below is `${CLAUDE_PLUGIN_ROOT}/bin/flowapi`, flowapi at the version this plugin is tested
with; where that variable is not filled in, it is `bin/flowapi` at the root of this plugin, two
folders above this file. The judge is `node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs`, beside it.

When the person ran `/flowapi:sdk`, the arguments are the description (a path or a URL), then the
SDK's folder (`sdk` when not given).

## The rule

A file listed in `.flowapi/lock.json` is generated. Never edit, move or delete one: the guard hook
refuses it, and `generate` would call it a conflict. Change the config and generate again. Code of
the person's own goes in `src/lib/` and `examples/`, which flowapi never touches.

## The pipeline

1. **Folder.** Create the SDK folder. Copy the description into it, or keep its URL. The config
   lives in the folder's root and every command runs from there.
2. **Read the API.** `F summary <spec>`: servers, security schemes, each operation with its
   parameters, media types and the pagination its response looks like it has (with a reason and
   how sure). Over 60 KB it prints the tags alone: read on with `--tag <tag>` or
   `--path <prefix>`, one at a time.
3. **Draft.** `F config draft <spec> -o flowapi-sdk.yml`. Every operation lands in a resource,
   with the default verbs and the pagination the responses look like. It already passes
   `config check`; it is a start, not an answer.
4. **Decide.** What the draft cannot know ([references/config.md](references/config.md) lists
   every key):
   - Resource names: plural nouns from the domain (`pets`, `invoices`), nested where the paths
     nest (`client.pets.photos.list()`), not the description's tags when those are teams or
     products.
   - Method names: `list`, `create`, `retrieve`, `update`, `delete` for the plain CRUD; a verb
     for the rest (`cancel`, `archive`, `search`). A `GET` that is not a collection is not `list`
     (`client.health.check()`, not `client.health.list()`).
   - Pagination: keep a suggestion only when the response really pages; `pagination: false` on a
     method turns a default off.
   - `skip`: internal, deprecated or admin operations the SDK should not have, each with its
     reason. Never skip an operation only because its name is hard.
   - `package.name`, `package.client`, `package.license`, `auth.<scheme>.env_var`, `readme`.
5. **Check.** `F config check flowapi-sdk.yml --json`. Each issue has a `code`, a `pointer` into
   the config, a `line` and a `hint`. Fix every error, read every warning, and check again until
   `"errors":0`.
6. **Generate.** `F generate flowapi-sdk.yml --out .`. The summary line says what was written,
   deleted and left unchanged. Exit 1 with a conflict means a generated file was changed by hand:
   report the file and stop; never edit or delete it to get past this. `--check` writes nothing
   and exits 1 when the output differs.
7. **Install.** `npm install --no-audit --no-fund` (TypeScript is the one dependency).
8. **Judge.** `node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs . --json`. Steps: 0 determinism (generate
   twice into empty folders), 1 typecheck, 2 lint, 3 build (ES modules and CommonJS), 4 the built
   types, 5 the package (publint, attw), 6 the tests against a mock server made from the
   description. A red step is fixed in `flowapi-sdk.yml`, then back to step 5. A red step no
   config change can fix is a flowapi bug: report the step and its output.

Done is the judge's verdict `pass` on steps 0-6. Report the steps with their times, the methods
by resource, what was skipped and why, and what the person decides next (the package name, the
license, publishing). Never edit a generated file, push, or publish.

In Claude Code, `/flowapi:refine`, `/flowapi:update` and `/flowapi:release` take an SDK made this
way further; the `sdk-reviewer` agent reads its API from a user's side.

## Issue codes seen often

| Code | What to do |
| --- | --- |
| `config.coverage` | An operation is in no resource: name it under one, or add it to `skip` with a reason |
| `config.op.unknown` | `op` names no operation: the hint says the closest one |
| `config.schema.unknown` | A key the format does not have: the message lists the keys there |
| `config.generator.version` | The config pins another flowapi: set `generator.flowapi` to the version `F --version` prints |

The text output names the same issues, one per paragraph, with `at <file>:<line>:<column>`.
