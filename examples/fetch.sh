#!/bin/sh
# The descriptions too large for git, put where each example's config names them: an example
# folder named as a row of ../bench/corpus/corpus.tsv takes that row's file. The corpus is fetched
# and checked against its pinned sha256 by ../bench/corpus/fetch.sh, then linked here as the
# config's `spec` (`openapi.json`). An example whose description is committed is left alone.
set -eu
cd "$(dirname "$0")"
../bench/corpus/fetch.sh
for config in */flowapi-sdk.yml; do
    name=${config%/flowapi-sdk.yml}
    spec=$(sed -n 's/^spec: *"\{0,1\}\([^"]*\)"\{0,1\} *$/\1/p' "$config")
    [ -f "$name/$spec" ] && continue
    if ! grep -q "^$name	" ../bench/corpus/corpus.tsv; then
        echo "fetch.sh: $name/$spec is missing and $name is no row of bench/corpus/corpus.tsv" >&2
        exit 1
    fi
    ln -sf "../../bench/data/$name.json" "$name/$spec"
    echo "$name/$spec -> bench/data/$name.json"
done
