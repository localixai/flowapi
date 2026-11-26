#!/bin/sh
# The live agent runs, `run.sh [--dry-run] <scenario>... | all`: each scenario of scenarios/ run by
# `claude -p` on Sonnet with this repository's plugin and its release build of flowapi, then judged
# by the runner; results/agent.json and .md written. They cost money and are not deterministic:
# the owner starts them. `--dry-run` prints the commands alone. The runner is lib/run.mjs.
set -eu
exec node "$(dirname "$0")/lib/run.mjs" "$@"
