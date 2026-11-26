---
description: Prepare an SDK made with the flowapi plugin for its first release - check the package, the judge, the license and the release files - without publishing anything.
argument-hint: "[sdk folder]"
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/bin/flowapi:*) Bash(node ${CLAUDE_PLUGIN_ROOT}/bin/judge.mjs:*) Bash(npm install:*) Bash(npm pack:*)
---

# Prepare a release

The SDK folder is in `$ARGUMENTS`. This command never pushes, tags or publishes: it gets the
repository ready and tells the person what is left.

1. `${CLAUDE_PLUGIN_ROOT}/bin/flowapi generate flowapi-sdk.yml --out . --check` exits 0: the files
   are what the config generates.
2. The judge passes on steps 0-6.
3. `package.name`, `package.license` and `package.repository` in `flowapi-sdk.yml` are the
   person's real choices, not the draft's guesses. Ask when they are not.
4. `npm pack --dry-run`: list what the package would hold and its size.
5. The repository holds `release-please-config.json` and `.release-please-manifest.json`. Say
   what the person does next: put the folder in a git repository with a remote, turn on the
   release workflow they want, and publish under their own npm account.
