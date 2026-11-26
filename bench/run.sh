#!/bin/sh
# Every benchmark, in one run: the competitors installed and the corpus fetched if they are
# missing (each file checked against corpus/corpus.tsv), flowapi's release build, then `speed`
# writing results/speed.json and results/speed.md, and the results held to budgets.json by
# `check` (after competitors/sdk-compare.mjs writes results/sdk-compare.*), then the README's chart and picture (`charts`, `shots`, into ../assets/). Arguments go to `speed` (`--quick` for one round, `--rounds N`). A figure published
# anywhere is copied from results/; nothing else is. Nothing else should run meanwhile.
set -eu
cd "$(dirname "$0")"
[ -d competitors/node_modules ] || (cd competitors && npm ci --no-audit --no-fund)
corpus/fetch.sh
(cd .. && cargo build --quiet --release)
cargo build --quiet --release --bins
target/release/speed "$@"
# The whole SDK against the other client generators (results/sdk-compare.*), one round with --quick.
case " $* " in *" --quick "*) rounds=1 ;; *) rounds=5 ;; esac
node competitors/sdk-compare.mjs --rounds "$rounds"
target/release/check
# The README's pictures, drawn from what was just measured and from the oracles' last results
# (../conformance/results/compare.json): the comparison chart, and what flowapi prints.
target/release/charts
target/release/shots
