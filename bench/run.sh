#!/bin/sh
# Every benchmark, in one run: the competitors installed and the corpus fetched if they are
# missing (each file checked against corpus/corpus.tsv), flowapi's release build, then `speed`
# writing results/speed.json and results/speed.md, `sdk-repo` writing results/sdk-repo.json and
# .md (the examples' descriptions linked by ../examples/fetch.sh; npm install and the judge reach
# the network), and the results held to budgets.json by `check`, then the README's chart and
# picture (`charts`, `shots`, into ../assets/). Arguments go to `speed` and `sdk-repo` (`--quick`
# for one round, `--rounds N`). A figure published anywhere is copied from results/; nothing else
# is. Nothing else should run meanwhile.
set -eu
cd "$(dirname "$0")"
[ -d competitors/node_modules ] || (cd competitors && npm ci --no-audit --no-fund)
../examples/fetch.sh
(cd .. && cargo build --quiet --release)
cargo build --quiet --release --bins
target/release/speed "$@"
target/release/sdk-repo "$@"
target/release/check
# The README's pictures, drawn from what was just measured and from the oracles' last results
# (../conformance/results/compare.json): the comparison chart, and what flowapi prints.
target/release/charts
target/release/shots
