# The requirements ledger (O1)

flowapi's promise is that it follows the specifications. The ledger makes that countable: every
normative statement of the pinned texts (`../specs/`, [`SOURCE.md`](../specs/SOURCE.md)) gets an
id, `../scope.json` says which ids flowapi answers for, a test that checks one says so with a
marker, and a script counts the rest as missing.

| File | What it is | Written by |
| --- | --- | --- |
| `../specs/` | OpenAPI 2.0, 3.0.4, 3.1.2, 3.2.1, JSON Schema 2020-12 (core, validation) and RFCs 6749, 6750, 7578, 7617 and 9110, byte for byte | By hand, from the authors' repositories and the RFC Editor |
| `../requirements.json` | Every requirement: id, text, section, anchor, line | `extract` |
| `../scope.json` | Which ids are in scope, and why the others are not | By hand |
| `../REPORT.md` | Per text and per section: total, in scope, out of scope, covered, lossy, missing | `report` |
| `../MISSING.md` | Every id in scope that no marker names, per text and section, with its sentence and the rule that put it in scope | `report` |
| `../coverage/` | Markers grouped by oracle (`suite.md`: O3's keyword files), and `triage.md`: what no marker answers yet | By hand |
| `../triage-2.md` | The second pass (2026-11-24): the 3.2.1 id changes, the scope decided, violations found, what is left; holds no markers | By hand |
| `../triage-3.md` | The SDK stage (2026-11-26): Swagger 2.0 and the RFCs pinned, the scope decided again for the SDK, section by section; holds no markers | By hand |

## What a requirement is

- **A sentence with a BCP 14 keyword**: MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD,
  SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, OPTIONAL, in capitals (RFC 2119, RFC 8174). Its
  `level` is the strongest group it holds (`MUST`, `SHOULD`, `MAY`) and `terms` lists them all. A
  keyword in quotes (the boilerplate that defines them) or in a code span does not count;
  examples (fenced code, `<artwork>`) and editors' comments (`<cref>`) are not read.
- **A row of an OpenAPI field table** (a first header cell `Field Name` or `Field Pattern`: fixed
  fields, patterned fields, specification extensions): `level` `field`, `text` the description.
  OpenAPI 2.0 writes its tables without a leading `|` (`Field Name | Type | Description`); they
  are read alike.
- **A JSON Schema keyword**: one per keyword of the 2020-12 vocabularies, at the section that
  defines it; and in OpenAPI, each keyword listed under "JSON Schema Keywords": `level` `keyword`.

`text` is the sentence with links reduced to their text and HTML removed; `line` is where its
paragraph, row or section starts in the pinned file; `anchor` is the fragment of the rendered
specification (a field's own `<a name>`, else the heading's slug as ReSpec makes it, `-0`, `-1`
on repeats; for JSON Schema the section's `anchor`, else `section-N.M` or `appendix-A.N` as
xml2rfc numbers it; for an RFC `section-N.M` or `appendix-A.N` as the RFC Editor's HTML renders
it).

The RFCs are read by their numbered sections. RFC 9110 from its RFCXML v3 source
(`src/rfc_xml.rs`: each `<t>`, list item, definition and table cell; code, artwork and the index
not read); the others, which have no XML at the RFC Editor, from their text (`src/rfc_text.rs`:
headings at column 0, page breaks removed, a list item a unit of its own, a word broken at its
hyphen joined). Neither reads the boilerplate, the table of contents, the References or the
unnumbered sections at the end. `tests/readers.rs` holds each reader to a fixture.

## Ids

```
<text>.<section>[.<field or keyword>][.<term>-<slug>]

oas-3.1.schema-object.discriminator                       a field
oas-3.2.parameter-object.style.must-…                     a sentence in that field's row
oas-3.1.parameter-object.fixed-fields.must-parameter-objects-include-either-content-field
oas-3.0.responses-object.pattern-http-status-code         a patterned field
oas-2.0.swagger-object.basePath.must-value-start-leading-slash
jsonschema-2020-12.core.$ref                              a keyword
jsonschema-2020-12.core.$ref.must-value-ref-keyword-string-uri-reference
rfc6749.4.4.2.must-value-set-client-credentials           a sentence of RFC 6749 section 4.4.2
rfc9110.B.1.<term>-<slug>                                  appendix B.1
```

- `<text>` is `oas-2.0`, `oas-3.0`, `oas-3.1`, `oas-3.2`, `jsonschema-2020-12.core`,
  `jsonschema-2020-12.validation`, or an RFC: `rfc6749`, `rfc6750`, `rfc7578`, `rfc7617`, `rfc9110`.
- In an RFC, `<section>` is the section's number (`4.4.2`, `A.1`), and each sentence follows it.
- `<section>` starts at the nearest base: an OpenAPI Object (`schema-object`), a JSON Schema
  keyword section (`$ref`) or a section with an anchor (`fragments`); without one, at the nearest
  heading whose slug is unique in the text. Then the heading itself; the headings between are
  added only when two sections would otherwise share an id.
- `<term>` is the strongest keyword (`must`, `must-not`, `should`, `not-recommended`, `may`, ...),
  `<slug>` the sentence's first six words that are neither keywords nor stop words.
- An id taken twice gets `-2`, `-3` in document order (18 today, all in the RFCs). Ids only change when the text
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
(`oas-*.server-object`, which fits `oas-2.0` too), the longest match wins. A rule that fits no id,
or an id no rule fits, fails `report`. An in-scope rule's `note` names the layer that answers for
the ids. The current file was decided 2026-11-24 and for the SDK stage 2026-11-26
(`../triage-3.md`) by the maintainers, owner delegated (its `status`).

## Markers

A marker is `req: <id>` (a test covers it) or `lossy: <id> — <reason>` (the type is wider than the
schema there; the reason is required) in any `.rs`, `.ts`, `.mjs` or `.json` file under `crates/` or
`conformance/`, and in the `.md` files of `conformance/coverage/`; usually a comment beside the
test: `// req: oas-3.1.schema-object.discriminator`. Build output, `node_modules/`, the pinned
texts and suites, and the ledger's own files are not read. Only ids that start with `oas-`,
`jsonschema-` or `rfc` and a digit are markers, so `req: Request` in code is not one. In a JSON
case the markers are strings of `x-flowapi-covers` (`crates/flowapi-sdk/tests/cases/`) or
`covers` (`../sdk/cases.json`). An id that is not in
`requirements.json` fails `report` (G13), and so does a lossy marker without a reason. An id with
both markers counts as lossy; one with neither is missing, and is listed in `../MISSING.md`.

A missing id fails no check: it is counted and listed. `report --check` fails on an unknown id, a
lossy marker without a reason, an unclassified id, a stale rule, and a `REPORT.md` or
`MISSING.md` that is not what `report` writes. A gate on missing ids would be a budget (a number
`report` refuses to rise above); there is none yet.

## Running it

In `conformance/ledger/`; each takes well under a second.

```sh
cargo run --release --bin extract            # write ../requirements.json
cargo run --release --bin extract -- --check # fail when it is stale
cargo run --release --bin report             # write ../REPORT.md and ../MISSING.md
cargo run --release --bin report -- --check  # also fail when either is stale
cargo run --release --bin classify           # counts per text and class
cargo run --release --bin classify -- oas-3.1.parameter-object      # each id, class, why
cargo run --release --bin classify -- --missing oas-3.1.schema-object  # in scope, no marker
cargo test --release --test ledger           # the ledger's own rules
cargo test --release --test readers          # the OpenAPI 2.0, RFC text and RFCXML readers
```

Updating a text: [`../specs/SOURCE.md`](../specs/SOURCE.md), "Updating a text".
