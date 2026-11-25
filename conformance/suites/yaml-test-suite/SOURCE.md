# Where these files come from

Copied unchanged from `github.com/yaml/yaml-test-suite`, its data release `data-2022-01-17`
(commit `6e6c296ae9c9d2d5c4134b4b64d01b29ac19ff6f`, 2022-01-17, the newest data release), MIT
(`LICENSE` beside this file, from the repository's `License`). Nothing here is ours: change
nothing in place, so an update stays a plain copy.

Only the files flowapi reads are copied: per case, `===` (its name), `in.yaml` (the input),
`in.json` (the JSON it loads as, when JSON can hold it; several values for several documents)
and `error` (present when the input is not YAML). A case with subcases has them in `00/`,
`01/`, ... The events, the emitter's output and the `name/` and `tags/` indexes are left out.

`crates/flowapi-yaml/tests/yaml_suite.rs` holds the YAML reader that the command and the npm
package share to them: every `error` case is refused, every case with one JSON value converts to
equal JSON, a case of no or several documents is refused (a description is one document).

To update: clone the newest data release, copy the same four files of every case over these,
and put the new tag and commit here.

    git clone --depth 1 -b data-YYYY-MM-DD https://github.com/yaml/yaml-test-suite
    git -C yaml-test-suite log -1 --format='%H %cs'
    find yaml-test-suite -path '*/.git' -prune -o -path '*/name' -prune -o -path '*/tags' -prune \
      -o -type f \( -name in.yaml -o -name in.json -o -name error -o -name '===' \) -print
