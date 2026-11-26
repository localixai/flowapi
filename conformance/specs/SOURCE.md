# The pinned specification texts

The requirements ledger (`../ledger/`, oracle O1) reads these files and nothing else: no network,
no other copy. They are copies, byte for byte, of the texts the specifications' authors publish.
Never edit one by hand; take a new release as below.

| File | Specification | Repository | Ref | Commit | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `openapi/2.0.md` | OpenAPI 2.0 (Swagger 2.0) | https://github.com/OAI/OpenAPI-Specification | `main`, `versions/2.0.md` | `4f86429f134f7e48b6351563549b4502fc69de52` (2026-10-18) | `57bf00dfabdb79634e2886e022f3c6195296682b237ed4a56c7fc631c85b3c91` |
| `openapi/3.0.4.md` | OpenAPI 3.0.4 | https://github.com/OAI/OpenAPI-Specification | `main`, `versions/3.0.4.md` | `4f86429f134f7e48b6351563549b4502fc69de52` (2026-10-18) | `77af0760571a228e61ca2126d6f4de9efcf053d5dcfb7f0ad2f84d7ecf622ff6` |
| `openapi/3.1.2.md` | OpenAPI 3.1.2 | same | `main`, `versions/3.1.2.md` | same | `7e30126d65acad53523f420e522d34edb3717af225aae727e73a522abc227da7` |
| `openapi/3.2.1.md` | OpenAPI 3.2.1 | same | `main`, `versions/3.2.1.md` | same | `01fadb4ce8253fa70b962e1a04cb73c549f2fe98c6311dff2b323cac6e322fad` |
| `openapi/LICENSE` | The OAI repository's licence | same | `main`, `LICENSE` | same | `4948367c65e1ce06690e2cadc6e86fce1a6a6db55ef874ce4b78c0f472ce5f13` |
| `json-schema-2020-12/jsonschema-core.xml` | JSON Schema 2020-12 Core (`draft-bhutton-json-schema-01`) | https://github.com/json-schema-org/json-schema-spec | tag `draft-bhutton-json-schema-01` | `add836e705c9a07434c467b6b90946ba45258a73` (2022-06-10) | `94fe780e41c5230db71549786f57db9e24544828e23a4a94714545d9b41f1431` |
| `json-schema-2020-12/jsonschema-validation.xml` | JSON Schema 2020-12 Validation (`draft-bhutton-json-schema-validation-01`) | same | same tag (also `draft-bhutton-json-schema-validation-01`) | same | `dcba094c316c17922cb95cc36affc53b9d9521139bfd5a3afdd1cc810ab39773` |
| `rfc/rfc6749.txt` | RFC 6749, The OAuth 2.0 Authorization Framework (October 2012) | https://www.rfc-editor.org/rfc/rfc6749.txt | the RFC Editor's text | (an RFC does not change) | `f204fc8661d6c92d2ec6e0b54808f961a9ad26e792f57f312d9528335519bd71` |
| `rfc/rfc6750.txt` | RFC 6750, OAuth 2.0 Bearer Token Usage (October 2012) | https://www.rfc-editor.org/rfc/rfc6750.txt | same | same | `9dc385cf4ecdd85024e5a95e447ede9230e11700bf35251906b25b37557d604b` |
| `rfc/rfc7578.txt` | RFC 7578, Returning Values from Forms: multipart/form-data (July 2015) | https://www.rfc-editor.org/rfc/rfc7578.txt | same | same | `813277f91c6c65c9047da9e9a8f1f0c19c1fb4bb1aaeea944bcf3bbdcc2eef49` |
| `rfc/rfc7617.txt` | RFC 7617, The 'Basic' HTTP Authentication Scheme (September 2015) | https://www.rfc-editor.org/rfc/rfc7617.txt | same | same | `bc941321da9912fbb1bc92ad1820848ddfd3283dbfbc30f26ea2800bdaf541db` |
| `rfc/rfc9110.xml` | RFC 9110, HTTP Semantics (June 2022, STD 97) | https://www.rfc-editor.org/rfc/rfc9110.xml | the RFC Editor's RFCXML v3 source, as published (prepped) | same | `09ec9ad4d1664623adcb298577e3fb9cd5c285f1bbe6b24d8c715b29f218e721` |

The OpenAPI files under `versions/` are the published releases; `main` only gains new ones.
3.0.4, 3.1.2 and 3.2.1 are the latest patch releases of their minor versions on that commit
(3.2.1 is dated 2026-09-10 in its revision history). 3.2.1 replaced 3.2.0 here on 2026-11-24,
decided by the maintainers (owner delegated); the ids it changed are listed in
`../triage-2.md`. OpenAPI 2.0 (`versions/2.0.md`, Swagger 2.0) is pinned from the same commit since 2026-11-26:
flowapi reads Swagger 2.0 into the 3.x model, and the SDK lifts its `host`, `basePath`, `schemes`
and `securityDefinitions`.

The RFCs are the ones the SDK's client and its multipart writer follow (`../triage-3.md`). An RFC
is never revised once published, so the RFC Editor's file is the pin and its SHA-256 the check.
RFC 9110 is published as RFCXML v3, and its XML is the copy here: the ledger reads its numbered
sections. RFCs 6749, 6750, 7578 and 7617 came before RFCXML v3 and the RFC Editor publishes no XML
for them, so their plain text is the copy (`../ledger/src/rfc_text.rs` reads it). RFC 8414 (OAuth
authorization server metadata) is not pinned: the client reads no metadata; OpenAPI 3.2's
`oauth2MetadataUrl` is out of scope for that reason.

The JSON Schema files are the sources of the 2020-12 publication as Internet-Drafts
(xml2rfc v2), not the rendered HTML: the ledger reads their sections, titles and anchors.

## Licences

- **OpenAPI**: Apache License 2.0. Each text says so itself ("This document is licensed under The
  Apache License, Version 2.0"); `openapi/LICENSE` is the repository's copy of it, kept beside
  the texts as the licence asks.
- **JSON Schema 2020-12**: the two XML sources at the pinned tag carry `ipr="trust200902"`, so
  as Internet-Drafts they fall under the IETF Trust's Legal Provisions relating to IETF Documents
  (BCP 78), which allow them to be copied and redistributed. The tag has no `LICENSE` file. On
  `main` (since 2026-11-19) the repository licenses its contents under BSD-3-Clause or the
  Academic Free License 3.0. The copies here are kept unmodified, so each keeps its attribution:
  the editors and authors in its front matter, and its `ipr` statement (from which xml2rfc writes
  the IETF Trust copyright notice); this table names where each came from. Decided 2026-11-24 by the
  maintainers (owner delegated).
- **The RFCs**: IETF Stream documents under the IETF Trust's Legal Provisions relating to IETF
  Documents (BCP 78), which allow them to be copied and redistributed whole and unmodified. Each
  copy keeps its own notice: RFCs 6749 and 6750 "Copyright (c) 2012 IETF Trust and the persons
  identified as the document authors", RFCs 7578 and 7617 the same for 2015, and RFC 9110 its
  `ipr="pre5378Trust200902"`, from which xml2rfc writes its notice (it holds material from before
  November 10, 2008). The copies are unmodified and this table names where each came from.

## Updating a text

1. Take the new file from the same repository at a tag or a commit, never from a rendered page:
   `git clone --depth 1 https://github.com/OAI/OpenAPI-Specification` and copy
   `versions/<version>.md`; for JSON Schema, check out the release's tag and copy the XML; for an
   RFC, `https://www.rfc-editor.org/rfc/rfc<N>.xml` where it exists, else `rfc<N>.txt`.
2. Replace the file here (a new version is a new file, and a new entry in `SOURCES` in
   `../ledger/src/sources.rs`), and write its commit and `sha256sum` into the table above.
3. In `../ledger/`: `cargo run --release --bin extract`. It prints how many ids were added and
   which were removed, and fails when two requirements share an id or a 2020-12 keyword has no
   section.
4. `cargo run --release --bin report`: fails on markers whose ids are gone, on new ids no rule of
   `../scope.json` classifies, and on rules that no longer fit anything. Classify the new ids and
   re-read each changed section against the tests that mark it.
5. Commit the text, this file, `requirements.json`, `scope.json` and `REPORT.md` together.
