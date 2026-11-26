---
description: Improve an SDK made with the flowapi plugin - names that read badly, operations left out, pagination missed - by changing flowapi-sdk.yml and generating again.
argument-hint: "[sdk folder] [what to improve]"
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/bin/flowapi:*) Bash(node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs:*) Bash(npm install:*)
---

# Refine an SDK

The SDK folder and what to improve are in `$ARGUMENTS`. `F` is `${CLAUDE_PLUGIN_ROOT}/bin/flowapi`.

1. Read `flowapi-sdk.yml`, `api.md` and `README.md`. Run `F summary <spec>` (by `--tag` when it is
   large) to see what the description has that the config does not use.
2. Use the `sdk-reviewer` agent for a read of the API from its user's side when no specific
   change was asked for.
3. Change `flowapi-sdk.yml` only: names, nesting, pagination, `skip`, `readme`.
4. `F config check flowapi-sdk.yml --json` until no error, `F generate flowapi-sdk.yml --out .`,
   then `node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs . --json` until it passes.
5. Report what changed in the API (from `git diff -- api.md` when the folder is a git repository)
   and that a renamed or removed method breaks its callers.
