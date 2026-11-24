# Conformance

## O3: the JSON-Schema-Test-Suite through the TypeScript compiler

The official [JSON-Schema-Test-Suite](suites/json-schema-test-suite/SOURCE.md) is a list of
groups, each a schema and cases of data marked valid or invalid. The oracle generates flowapi's
TypeScript type for every group that applies to JSON Schema 2020-12 and asks the TypeScript
compiler whether each case's data fits that type.

- **Soundness (O3).** Every valid case must fit the type. The target is 100%: a valid case the
  type rejects is a bug in flowapi.
- **Precision (O3′).** The share of invalid cases the type rejects, per keyword file. Many
  keywords (`minimum`, `pattern`, `uniqueItems`, ...) have no TypeScript type, so 100% is out of
  reach; the number must not go down.

How a group is checked:

1. Its `compatibility` must admit 2020-12 (the suite README's algorithm); groups tagged
   `format-assertion` are left out, since 2020-12 reads `format` as an annotation.
2. Its schema is added as the first document, under
   `https://json-schema.org/flowapi-suite/<file>/<group>.json`, and each of its `externalSchemas`
   under its own retrieval URI (`http://localhost:1234/...`). No `$schema` is added: flowapi
   reads a schema without one as 2020-12.
3. The output goes to `out/<file>-<group>.ts`, followed by one line per case,
   `const c<i>: Root = <data>;`. The data is a fresh literal, so TypeScript's excess-property
   check applies to it.
4. One compiler run checks every file: `tsc --strict --exactOptionalPropertyTypes --noEmit
   --target es2022 --skipLibCheck`. Each diagnostic is mapped back to its case by file and line.

A group whose generation reports an error, or panics, is a generation error. A group whose
generated types do not compile (a diagnostic above the first case line) is invalid TypeScript.
Both are failures, listed by group, and their cases are not counted.

## Running it

    cd conformance/ts && npm ci              # once: TypeScript 7, pinned in package-lock.json
    cd conformance/suite && cargo run --release

It takes well under a second. The command prints the totals and writes:

- `results/suite.md`: a row per keyword file, the totals, then every unsound case (file, group,
  case, data, the compiler's first line), every invalid-TypeScript group and every generation
  error.
- `results/suite.json`: the same counts per file, with each unsound case's full compiler message.
- `out/`: the generated files, ignored by git. To see why a case failed, open its file or run the
  compiler on it alone.

Both results are written sorted and without times or paths, so a change to flowapi shows up as
a diff of `results/`. Commit them with the change that moved them.

## Reading the table

| Column | Meaning |
| --- | --- |
| Groups | Groups in the file, for every dialect. |
| Run | Those that apply to 2020-12 and are not `format-assertion`. |
| Gen. errors | Run groups flowapi refused or panicked on. |
| Invalid TS | Generated groups whose types do not compile. |
| Valid / Accepted / Sound % | Valid cases, those the type accepts, the share. |
| Invalid / Rejected / Precision % | Invalid cases, those the type rejects, the share. |

Percentages are rounded down, so 100.0 means every case.
