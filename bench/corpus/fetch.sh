#!/bin/sh
# Fetches the descriptions the benchmarks read into bench/data/ (ignored by git), each at the
# commit corpus.tsv pins and checked against its sha256: GitHub's REST description, Stripe's,
# Cloudflare's, and three mid-size ones from APIs-guru's directory. A file is copied from a local
# clone of its repository when one is checked out at that commit (CLONES, by default the
# workspace's reference clones), else downloaded from raw.githubusercontent.com at the commit.
# APIs-guru keeps only YAML: those are converted to JSON by ../competitors/yaml2json.mjs, and both
# the YAML and the JSON are checked. A file already in data/ with the right sha256 is kept.
set -eu
cd "$(dirname "$0")/.."
CLONES=${CLONES:-../../.references/codegen/corpus}
mkdir -p data/source

sha() { sha256sum "$1" | cut -d' ' -f1; }

# The local clone of a repository, by its owner/name on GitHub.
clone_of() {
    case "$1" in
        github/rest-api-description) echo "$CLONES/github" ;;
        stripe/openapi) echo "$CLONES/stripe" ;;
        cloudflare/api-schemas) echo "$CLONES/cloudflare" ;;
        APIs-guru/openapi-directory) echo "$CLONES/apis-guru" ;;
        *) echo "" ;;
    esac
}

# <url> <commit> <expected sha256> <file>: the source file, copied or downloaded, then checked.
get() {
    if [ -f "$4" ] && [ "$(sha "$4")" = "$3" ]; then return; fi
    rest=${1#https://raw.githubusercontent.com/}
    repo=$(echo "$rest" | cut -d/ -f1-2)
    path=$(echo "$rest" | cut -d/ -f4-)
    clone=$(clone_of "$repo")
    if [ -n "$clone" ] && [ -f "$clone/$path" ] &&
        [ "$(git -C "$clone" log -1 --format=%H 2>/dev/null)" = "$2" ] &&
        git -C "$clone" diff --quiet HEAD -- "$path" 2>/dev/null; then
        cp "$clone/$path" "$4"
    else
        curl -fsSL --retry 3 -o "$4" "$1"
    fi
    got=$(sha "$4")
    if [ "$got" != "$3" ]; then
        echo "fetch.sh: $1 has sha256 $got, corpus.tsv pins $3" >&2
        exit 1
    fi
}

grep -v '^#' corpus/corpus.tsv | tail -n +2 |
    while IFS="$(printf '\t')" read -r name url commit sum bytes source; do
        out=data/$name.json
        if [ "$source" = "-" ]; then
            get "$url" "$commit" "$sum" "$out"
        else
            get "$url" "$commit" "$source" "data/source/$name.yaml"
            if [ ! -f "$out" ] || [ "$(sha "$out")" != "$sum" ]; then
                [ -d competitors/node_modules ] || (cd competitors && npm ci --no-audit --no-fund >&2)
                node competitors/yaml2json.mjs "data/source/$name.yaml" "$out"
            fi
            got=$(sha "$out")
            if [ "$got" != "$sum" ]; then
                echo "fetch.sh: $out converted has sha256 $got, corpus.tsv pins $sum" >&2
                exit 1
            fi
        fi
        if [ "$(wc -c <"$out" | tr -d ' ')" != "$bytes" ]; then
            echo "fetch.sh: $out is not $bytes bytes" >&2
            exit 1
        fi
        echo "$out $bytes bytes, $commit"
    done
