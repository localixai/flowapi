# AGENTS.md: the flowapi plugin

flowapi's plugin for coding agents, in flowapi's repository (`plugins/flowapi`), installed from
its marketplace (`.claude-plugin/marketplace.json` at the repository root). It carries flowapi's
documentation (the `flowapi` skill) and the SDK pipeline: an agent writes `flowapi-sdk.yml`,
flowapi (`bin/flowapi`: `npx -y @localix-ai/flowapi@<lib/flowapi-version>`) generates a
TypeScript SDK repository from an OpenAPI description and that config, a deterministic judge
checks it, and the agent fixes the config until the judge passes. The repository's rules
(`../../AGENTS.md`) bind here too; this file adds the plugin's.

## Layout

| Path | What it is |
| --- | --- |
| `.claude-plugin/plugin.json`, `hooks/hooks.json` | The Claude Code form: the manifest and its hooks |
| `plugin.json`, `hooks.json` | The Agent Plugins 1.1.0 core form: the same plugin for a core client. A client that reads both loads the Claude Code form alone (Agent Plugins Extended §1.3.1) |
| `hooks/guard.mjs` | The guard, one script for both: no argument is Claude Code's PreToolUse on Edit, Write, MultiEdit, NotebookEdit, answering `hookSpecificOutput` with `permissionDecision: "deny"`; `core` reads the core's `file.modify` tool profile and answers `{ "decision": "deny", "reason" }` (core §7.3.6). Silence is no opinion: it never answers `allow`. Exit code 0 always |
| `lib/guard.mjs` | The guard's decision: `decide` (Claude Code's event), `decideCore` (the core's), the nearest `.flowapi/lock.json`, its `files`, the custom zones |
| `bin/judge.mjs` | The judge's command line: `judge.mjs <sdk-dir> [--steps] [--json] [--keep-going] [--generate] [--timeout]` |
| `lib/judge.mjs` | The judge's steps 0-6 and the report |
| `bin/flowapi`, `lib/flowapi-version` | flowapi at the one pinned version, through `npx`; `FLOWAPI_PLUGIN_FLOWAPI` runs another flowapi instead |
| `lib/run.mjs`, `lib/tree.mjs`, `lib/tools.mjs` | Running a command with a time limit; comparing two trees; the pinned tool versions (flowapi's read from `lib/flowapi-version`) and `FLOWAPI_PLUGIN_NPX` |
| `skills/flowapi/` | How to use flowapi. `references/*.md` are generated (below) |
| `skills/sdk/` | The SDK pipeline, `/flowapi:sdk`, and `references/config.md`, the config's catalogue. A skill, not a command: a command of the same name would hide it (Extended §4.6.1), and a core client loads skills but no commands |
| `commands/`, `agents/sdk-reviewer.md` | `/flowapi:refine`, `update`, `release`; the read-only reviewer. Claude Code's alone |
| `docs/lock.md` | The lock's shape the plugin relies on |
| `scripts/docs.mjs` | Writes `skills/flowapi/references/` from flowapi's docs; `--check` writes nothing and exits 1 when one differs |
| `test/` | Node tests (`npm test`): the guard in both forms and as a hook, the two packagings and the version, the references, the judge on a stand-in SDK, and the pipeline end to end on `test/fixtures/pets.json` with a real flowapi (skipped without one) |

## The references are copies

The `flowapi` skill's references are flowapi's own docs: the README's "Use" and "How it holds to
the specification", `npm/README.md`, and the commands' help (`crates/flowapi-cli/src/args.rs`,
`npm/src/cli.ts`). Those stay the source of truth. An installed plugin is a copy of this folder
alone and cannot read a file outside it (core §4.1), so the skill carries copies, each with a
first line naming its source. After a change to a source, run `node scripts/docs.mjs`; never edit
a reference by hand. `test/docs.test.mjs` fails when one differs from what the script writes.

## Commands

| Command | |
| --- | --- |
| `npm test` | Every test, fast and offline: `npx` is a stand-in (`FLOWAPI_PLUGIN_NPX`); the end-to-end test uses `FLOWAPI_PLUGIN_FLOWAPI` or the repository's `target/release/flowapi` and runs judge step 0 only |
| `npm run test:e2e` | The end-to-end test with judge steps 0-6 (`FLOWAPI_PLUGIN_E2E=1`: installs the SDK's TypeScript, fetches publint and attw) |
| `npm run docs` | Writes the `flowapi` skill's references; `node scripts/docs.mjs --check` checks them |
| `npm run validate` | `claude plugin validate . --strict`; at the repository root, the same validates the marketplace |
| `claude --plugin-dir plugins/flowapi` | The working tree in a real session |

## Rules

- **The agent never edits generated SDK files.** Everything listed in the lock's `files` outside
  `src/lib/` and `examples/` is changed through `flowapi-sdk.yml` and `generate`. Every command,
  skill and agent here says so, and the guard enforces it.
- **The guard fails open and stays cheap.** It reads the event and the lock, starts no process,
  prints nothing but a denial, and exits 0 on any error: a broken lock must not block the person.
- **The judge is deterministic and never passes silently.** A missing script is `skip` with its
  reason; the verdict is `pass` only when every step run passed. It installs nothing, publishes
  nothing, and reaches the network only through `npx` fetching a pinned tool.
- **Tool versions are pinned** in `lib/tools.mjs`; the flowapi version comes from the lock.
- **Both forms stay equivalent.** The two manifests carry the same fields (the core one adds
  `$schema`); an event added to `hooks/hooks.json` that the core defines is added to `hooks.json`;
  skills use the Agent Skills fields alone, so both forms read them alike. `test/manifest.test.mjs`
  and `test/hooks.test.mjs` hold this.
- **One version.** The plugin's version is flowapi's (`[workspace.package] version` in
  `Cargo.toml`): both manifests, `package.json` and `lib/flowapi-version` change with it, and a
  test fails when they differ. `bin/flowapi` then runs that release from npm, so a version is
  raised here when its npm package is published.
- **No dependencies.** Node 20 or newer, ES modules (`.mjs`), the standard library only.
- **Every user-visible change** gets a line in `CHANGELOG.md` under `## Unreleased`, and in the
  repository's `CHANGELOG.md`.
- No source file over 300 lines of working code. English only. No marketing words, and in the plugin's own
  prose no other generator or company named (the references copy flowapi's docs as they are). Commit only when asked; never push unasked.
