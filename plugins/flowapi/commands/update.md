---
description: Update an SDK made with the flowapi plugin to a new version of its OpenAPI description - regenerate, cover new operations, and report what changed.
argument-hint: "<new spec path or URL> [sdk folder]"
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/bin/flowapi:*) Bash(node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs:*) Bash(npm install:*)
---

# Update an SDK to a new description

The new description and the SDK folder are in `$ARGUMENTS`. `F` is `${CLAUDE_PLUGIN_ROOT}/bin/flowapi`.

1. Replace the description in the folder, or point `spec` in `flowapi-sdk.yml` at the new one.
2. `F config check flowapi-sdk.yml --json`. New operations are `config.coverage` errors: name each
   under a resource, or `skip` it with a reason. A method whose operation is gone is an error too:
   remove it from the config.
3. `F generate flowapi-sdk.yml --out .`, then the judge until it passes.
4. Report what changed for the SDK's users: methods added, removed or changed (from
   `git diff -- api.md src/api.ts`). flowapi does not decide the version yet: say whether the
   change looks breaking (a method removed or renamed, a parameter now required, a response field
   gone) and leave the version to the person.
