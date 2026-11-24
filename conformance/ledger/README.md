# The requirements ledger (O1)

flowapi's promise is that it follows the specifications. The ledger makes that countable: every
normative statement of the pinned texts (`../specs/`, [`SOURCE.md`](../specs/SOURCE.md)) gets an
id, `../scope.json` says which ids flowapi answers for, a test that checks one says so with a
marker, and a script counts the rest as missing.

| File | What it is | Written by |
| --- | --- | --- |
| `../specs/` | OpenAPI 3.0.4, 3.1.2, 3.2.1 and JSON Schema 2020-12 (core, validation), byte for byte | By hand, from the authors' repositories |
| `../requirements.json` | Every requirement: id, text, section, anchor, line | `extract` |
| `../scope.json` | Which ids are in scope, and why the others are not | By hand |
| `../REPORT.md` | Per text and per section: total, in scope, out of scope, covered, lossy, missing | `report` |
| `../coverage/` | Markers grouped by oracle (`suite.md`: O3's keyword files), and `triage.md`: what no marker answers yet | By hand |
| `../triage-2.md` | The second pass (2026-11-24): the 3.2.1 id changes, the scope decided, violations found, what is left; holds no markers | By hand |

## What a requirement is

- **A sentence with a BCP 14 keyword**: MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD,
  SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, OPTIONAL, in capitals (RFC 2119, RFC 8174). Its
  `level` is the strongest group it holds (`MUST`, `SHOULD`, `MAY`) and `terms` lists them all. A
  keyword in quotes (the boilerplate that defines them) or in a code span does not count;
  examples (fenced code, `<artwork>`) and editors' comments (`<cref>`) are not read.
- **A row of an OpenAPI field table** (a first header cell `Field Name` or `Field Pattern`: fixed
  fields, patterned fields, specification extensions): `level` `field`, `text` the description.
- **A JSON Schema keyword**: one per keyword of the 2020-12 vocabularies, at the section that
  defines it; and in OpenAPI, each keyword listed under "JSON Schema Keywords": `level` `keyword`.

`text` is the sentence with links reduced to their text and HTML removed; `line` is where its
paragraph, row or section starts in the pinned file; `anchor` is the fragment of the rendered
specification (a field's own `<a name>`, else the heading's slug as ReSpec makes it, `-0`, `-1`
on repeats; for JSON Schema the section's `anchor`, else `section-N.M` or `appendix-A.N` as
xml2rfc numbers it).

## Ids

```
<text>.<section>[.<field or keyword>][.<term>-<slug>]

oas-3.1.schema-object.discriminator                       a field
oas-3.2.parameter-object.style.must-…                     a sentence in that field's row
oas-3.1.parameter-object.fixed-fields.must-parameter-objects-include-either-content-field
oas-3.0.responses-object.pattern-http-status-code         a patterned field
jsonschema-2020-12.core.$ref                              a keyword
jsonschema-2020-12.core.$ref.must-value-ref-keyword-string-uri-reference
```

- `<text>` is `oas-3.0`, `oas-3.1`, `oas-3.2`, `jsonschema-2020-12.core` or
  `jsonschema-2020-12.validation`.
- `<section>` starts at the nearest base: an OpenAPI Object (`schema-object`), a JSON Schema
  keyword section (`$ref`) or a section with an anchor (`fragments`); without one, at the nearest
  heading whose slug is unique in the text. Then the heading itself; the headings between are
  added only when two sections would otherwise share an id.
- `<term>` is the strongest keyword (`must`, `must-not`, `should`, `not-recommended`, `may`, ...),
  `<slug>` the sentence's first six words that are neither keywords nor stop words.
- An id taken twice gets `-2`, `-3` in document order (none today). Ids only change when the text
  they come from does: a reworded sentence is a new requirement.

## Classes

The research (`map/research/codegen-2026-11/05-specs-and-conformance.md`, §2) puts every
requirement in one class:

| Class | Here |
| --- | --- |
| `conforms` | In scope and named by a `req:` marker in a test: covered. |
| `lossy` | In scope and named by a `lossy:` marker: the type is wider than the schema there, for the reason the marker gives. |
| `out-of-scope` | A rule in `scope.json` with a `reason`. |
| `deviates` | Not allowed without the owner's decision; no marker for it exists. |

`scope.json` rules: `match` is an id or a prefix ending at a `.`, `*` stands for a version
(`oas-*.server-object`), the longest match wins. A rule that fits no id, or an id no rule fits,
fails `report`. The current file was decided 2026-11-24 by the maintainers, owner delegated (its `status`).

## Markers

A marker is `req: <id>` (a test covers it) or `lossy: <id> — <reason>` (the type is wider than the
schema there; the reason is required) in any `.rs`, `.ts` or `.json` file under `crates/` or
`conformance/`, and in the `.md` files of `conformance/coverage/`; usually a comment beside the
test: `// req: oas-3.1.schema-object.discriminator`. Build output, `node_modules/`, the pinned
texts and suites, and the ledger's own files are not read. Only ids that start with `oas-` or
`jsonschema-` are markers, so `req: Request` in code is not one. An id that is not in
`requirements.json` fails `report` (G13), and so does a lossy marker without a reason. An id with
both markers counts as lossy; one with neither is missing.

## Running it

In `conformance/ledger/`; each takes well under a second.

```sh
cargo run --release --bin extract            # write ../requirements.json
cargo run --release --bin extract -- --check # fail when it is stale
cargo run --release --bin report             # write ../REPORT.md
cargo run --release --bin report -- --check  # also fail when REPORT.md is stale
cargo run --release --bin classify           # counts per text and class
cargo run --release --bin classify -- oas-3.1.parameter-object      # each id, class, why
cargo run --release --bin classify -- --missing oas-3.1.schema-object  # in scope, no marker
cargo test --release --test ledger           # the ledger's own rules
```

Updating a text: [`../specs/SOURCE.md`](../specs/SOURCE.md), "Updating a text".
