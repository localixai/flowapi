# Changelog

## Unreleased

- The plugin, on flowapi 0.2.0, as a Claude Code plugin and an Agent Plugins 1.1.0 core package
  in one folder, installed from flowapi's repository (`flowapi@flowapi`).
- The `flowapi` skill: how to use flowapi, with flowapi's own docs as its references, copied by
  `scripts/docs.mjs` and held to their sources by a test.
- The `sdk` skill (`/flowapi:sdk`): summary, draft, config, check, generate, install, judge;
  the config's catalogue (`references/config.md`). `/flowapi:refine`, `update` and `release`;
  the read-only `sdk-reviewer` agent.
- `bin/flowapi`, flowapi at the version in `lib/flowapi-version` through `npx`, or
  `FLOWAPI_PLUGIN_FLOWAPI`.
- The guard hook, which refuses a hand edit of a file listed in `.flowapi/lock.json` (outside
  `src/lib/` and `examples/`) and of the lock itself: in Claude Code on Edit, Write, MultiEdit
  and NotebookEdit, in a core client on the `file.modify` tool profile.
- The judge, `bin/judge.mjs`, with steps 0-6 (determinism, typecheck, lint, build, dist types,
  package, test) and a JSON report; the lock's shape in `docs/lock.md`; an end-to-end test of the
  pipeline (`npm run test:e2e`).
