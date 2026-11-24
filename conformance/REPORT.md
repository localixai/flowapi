# Requirements report

Written by `cargo run --release --bin report` in `conformance/ledger/` from `requirements.json`,
`scope.json` and the `req:` and `lossy:` markers under `crates/` and `conformance/`. Do not edit by hand.

**Answered: 494 of 494 requirements in scope (100.0%): 428 covered by a test, 66 lossy with a reason; 0 missing.** Out of scope: 1261. Total: 1755.
Scope: decided 2026-11-24 by the maintainers, owner delegated.

| Text | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| openapi 3.0.4 (`openapi/3.0.4.md`) | 429 | 119 | 310 | 102 | 17 | 0 | 100.0% |
| openapi 3.1.2 (`openapi/3.1.2.md`) | 436 | 109 | 327 | 107 | 2 | 0 | 100.0% |
| openapi 3.2.1 (`openapi/3.2.1.md`) | 522 | 129 | 393 | 123 | 6 | 0 | 100.0% |
| json-schema 2020-12 (`json-schema-2020-12/jsonschema-core.xml`) | 241 | 90 | 151 | 66 | 24 | 0 | 100.0% |
| json-schema 2020-12 (`json-schema-2020-12/jsonschema-validation.xml`) | 127 | 47 | 80 | 30 | 17 | 0 | 100.0% |
| **All** | 1755 | 494 | 1261 | 428 | 66 | 0 | 100.0% |

## By level

| Text | MUST | SHOULD | MAY | field | keyword |
| --- | ---: | ---: | ---: | ---: | ---: |
| openapi 3.0.4 (`openapi/3.0.4.md`) | 115 | 53 | 84 | 150 | 27 |
| openapi 3.1.2 (`openapi/3.1.2.md`) | 127 | 53 | 101 | 153 | 2 |
| openapi 3.2.1 (`openapi/3.2.1.md`) | 185 | 54 | 105 | 176 | 2 |
| json-schema 2020-12 (`json-schema-2020-12/jsonschema-core.xml`) | 108 | 67 | 40 | 0 | 26 |
| json-schema 2020-12 (`json-schema-2020-12/jsonschema-validation.xml`) | 52 | 28 | 16 | 0 | 31 |

## openapi 3.0.4 — `openapi/3.0.4.md`

| Section (id after `oas-3.0.`) | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `appendix-c-using-rfc6570-based-serialization` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-d-serializing-headers-and-cookies` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-f-resolving-security-requirements-in-a-referenced-document` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `callback-object` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `components-object` | 11 | 6 | 5 | 6 | 0 | 0 | 100.0% |
| `conditions-for-using-the-discriminator-object` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `contact-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `data-type-format` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `discriminator-object` | 12 | 1 | 11 | 1 | 0 | 0 | 100.0% |
| `encoding-object` | 18 | 1 | 17 | 1 | 0 | 0 | 100.0% |
| `example-object` | 10 | 0 | 10 | 0 | 0 | 0 | — |
| `external-documentation-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `format` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `header-object` | 23 | 6 | 17 | 6 | 0 | 0 | 100.0% |
| `http-status-codes` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `info-object` | 12 | 0 | 12 | 0 | 0 | 0 | — |
| `interoperability-with-historical-specifications` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `interoperability-with-web-browser-environments` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `license-object` | 5 | 0 | 5 | 0 | 0 | 0 | — |
| `link-object` | 15 | 0 | 15 | 0 | 0 | 0 | — |
| `media-type-object` | 11 | 2 | 9 | 2 | 0 | 0 | 100.0% |
| `media-types` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `non-rfc6570-field-values-and-combinations` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `oauth-flow-object` | 12 | 0 | 12 | 0 | 0 | 0 | — |
| `oauth-flows-object` | 5 | 0 | 5 | 0 | 0 | 0 | — |
| `openapi-description-structure` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `openapi-object` | 17 | 5 | 12 | 5 | 0 | 0 | 100.0% |
| `operation-object` | 20 | 8 | 12 | 8 | 0 | 0 | 100.0% |
| `parameter-object` | 33 | 15 | 18 | 15 | 0 | 0 | 100.0% |
| `path-item-object` | 18 | 12 | 6 | 12 | 0 | 0 | 100.0% |
| `path-templating` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `paths-object` | 5 | 3 | 2 | 3 | 0 | 0 | 100.0% |
| `percent-encoding-and-illegal-or-reserved-delimiters` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `reference-object` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `relative-references-in-urls` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `request-body-object` | 6 | 3 | 3 | 3 | 0 | 0 | 100.0% |
| `resolving-implicit-connections` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `response-object` | 8 | 4 | 4 | 4 | 0 | 0 | 100.0% |
| `responses-object` | 7 | 5 | 2 | 5 | 0 | 0 | 100.0% |
| `rich-text-formatting` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `schema-object` | 57 | 35 | 22 | 18 | 17 | 0 | 100.0% |
| `security-filtering` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `security-requirement-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `security-scheme-object` | 17 | 0 | 17 | 0 | 0 | 0 | — |
| `server-object` | 7 | 0 | 7 | 0 | 0 | 0 | — |
| `server-variable-object` | 9 | 0 | 9 | 0 | 0 | 0 | — |
| `specification` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `specification-extensions` | 4 | 2 | 2 | 2 | 0 | 0 | 100.0% |
| `structural-interoperability` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `tag-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `undefined-and-implementation-defined-behavior` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `versions` | 3 | 3 | 0 | 3 | 0 | 0 | 100.0% |
| `working-with-binary-data` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `xml-object` | 9 | 0 | 9 | 0 | 0 | 0 | — |

## openapi 3.1.2 — `openapi/3.1.2.md`

| Section (id after `oas-3.1.`) | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `appendix-c-using-rfc6570-based-serialization` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-e-percent-encoding-and-form-media-types` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-f-resolving-security-requirements-in-a-referenced-document` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `callback-object` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `components-object` | 12 | 7 | 5 | 7 | 0 | 0 | 100.0% |
| `conditions-for-using-the-discriminator-object` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `contact-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `data-type-format` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `discriminator-object` | 13 | 2 | 11 | 2 | 0 | 0 | 100.0% |
| `encoding-object` | 22 | 1 | 21 | 1 | 0 | 0 | 100.0% |
| `example-object` | 10 | 0 | 10 | 0 | 0 | 0 | — |
| `external-documentation-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `format` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `header-object` | 23 | 6 | 17 | 6 | 0 | 0 | 100.0% |
| `http-status-codes` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `info-object` | 13 | 0 | 13 | 0 | 0 | 0 | — |
| `interoperability-with-historical-specifications` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `interoperability-with-web-browser-environments` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `license-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `link-object` | 16 | 0 | 16 | 0 | 0 | 0 | — |
| `media-type-object` | 12 | 2 | 10 | 2 | 0 | 0 | 100.0% |
| `media-types` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `non-rfc6570-field-values-and-combinations` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `oauth-flow-object` | 12 | 0 | 12 | 0 | 0 | 0 | — |
| `oauth-flows-object` | 5 | 0 | 5 | 0 | 0 | 0 | — |
| `openapi-description` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `openapi-description-structure` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `openapi-object` | 20 | 6 | 14 | 6 | 0 | 0 | 100.0% |
| `operation-object` | 19 | 7 | 12 | 7 | 0 | 0 | 100.0% |
| `parameter-object` | 41 | 23 | 18 | 23 | 0 | 0 | 100.0% |
| `parsing-documents` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `path-item-object` | 18 | 12 | 6 | 12 | 0 | 0 | 100.0% |
| `path-templating` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `paths-object` | 5 | 3 | 2 | 3 | 0 | 0 | 100.0% |
| `percent-encoding-and-cookies` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `percent-encoding-and-form-data` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `percent-encoding-and-illegal-or-reserved-delimiters` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `reference-object` | 9 | 2 | 7 | 2 | 0 | 0 | 100.0% |
| `relative-references-in-api-description-uris` | 4 | 3 | 1 | 3 | 0 | 0 | 100.0% |
| `relative-references-in-api-urls` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `request-body-object` | 7 | 3 | 4 | 3 | 0 | 0 | 100.0% |
| `resolving-implicit-connections` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `response-object` | 8 | 4 | 4 | 4 | 0 | 0 | 100.0% |
| `responses-object` | 7 | 5 | 2 | 5 | 0 | 0 | 100.0% |
| `rich-text-formatting` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `schema-object` | 23 | 10 | 13 | 8 | 2 | 0 | 100.0% |
| `security-filtering` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `security-requirement-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `security-scheme-object` | 17 | 0 | 17 | 0 | 0 | 0 | — |
| `server-object` | 8 | 0 | 8 | 0 | 0 | 0 | — |
| `server-variable-object` | 9 | 0 | 9 | 0 | 0 | 0 | — |
| `specification` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `specification-extensions` | 4 | 2 | 2 | 2 | 0 | 0 | 100.0% |
| `structural-interoperability` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `tag-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `undefined-and-implementation-defined-behavior` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `versions` | 3 | 3 | 0 | 3 | 0 | 0 | 100.0% |
| `working-with-binary-data` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `xml-object` | 12 | 0 | 12 | 0 | 0 | 0 | — |

## openapi 3.2.1 — `openapi/3.2.1.md`

| Section (id after `oas-3.2.`) | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `appendix-c-using-rfc6570-based-serialization` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-e-percent-encoding-and-form-media-types` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `appendix-g-parsing-and-resolution-guidance` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `base-uri-within-content` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `callback-object` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `components-object` | 13 | 8 | 5 | 8 | 0 | 0 | 100.0% |
| `conditions-for-using-the-discriminator-object` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `conflicts-between-field-types-and-reference-contexts` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `contact-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `discriminator-object` | 14 | 2 | 12 | 2 | 0 | 0 | 100.0% |
| `encoding-object` | 26 | 1 | 25 | 1 | 0 | 0 | 100.0% |
| `example-object` | 23 | 0 | 23 | 0 | 0 | 0 | — |
| `external-documentation-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `format` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `guidance-regarding-implicit-connections` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `header-object` | 25 | 6 | 19 | 6 | 0 | 0 | 100.0% |
| `info-object` | 13 | 0 | 13 | 0 | 0 | 0 | — |
| `json-and-yaml-compatibility` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `license-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `link-object` | 16 | 0 | 16 | 0 | 0 | 0 | — |
| `media-type-object` | 38 | 8 | 30 | 4 | 4 | 0 | 100.0% |
| `non-rfc6570-field-values-and-combinations` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `oauth-flow-object` | 15 | 0 | 15 | 0 | 0 | 0 | — |
| `oauth-flows-object` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `objects-and-fields` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `openapi-object` | 41 | 16 | 25 | 16 | 0 | 0 | 100.0% |
| `operation-object` | 19 | 7 | 12 | 7 | 0 | 0 | 100.0% |
| `parameter-object` | 44 | 26 | 18 | 26 | 0 | 0 | 100.0% |
| `path-item-object` | 21 | 14 | 7 | 14 | 0 | 0 | 100.0% |
| `paths-object` | 8 | 4 | 4 | 4 | 0 | 0 | 100.0% |
| `percent-encoding-and-cookies` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `percent-encoding-and-form-data` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `percent-encoding-and-illegal-or-reserved-delimiters` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `reference-object` | 9 | 2 | 7 | 2 | 0 | 0 | 100.0% |
| `request-body-object` | 7 | 3 | 4 | 3 | 0 | 0 | 100.0% |
| `response-object` | 8 | 4 | 4 | 4 | 0 | 0 | 100.0% |
| `responses-object` | 8 | 5 | 3 | 5 | 0 | 0 | 100.0% |
| `rich-text-formatting` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `schema-object` | 31 | 14 | 17 | 12 | 2 | 0 | 100.0% |
| `security-filtering` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `security-requirement-object` | 8 | 0 | 8 | 0 | 0 | 0 | — |
| `security-scheme-object` | 20 | 0 | 20 | 0 | 0 | 0 | — |
| `security-schemes` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `server-object` | 11 | 0 | 11 | 0 | 0 | 0 | — |
| `server-variable-object` | 10 | 0 | 10 | 0 | 0 | 0 | — |
| `specification-extensions` | 4 | 2 | 2 | 2 | 0 | 0 | 100.0% |
| `tag-object` | 10 | 0 | 10 | 0 | 0 | 0 | — |
| `undefined-and-implementation-defined-behavior` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `versions-and-deprecation` | 3 | 3 | 0 | 3 | 0 | 0 | 100.0% |
| `warnings-regarding-fragmentary-parsing` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `xml-object` | 17 | 0 | 17 | 0 | 0 | 0 | — |

## json-schema 2020-12 — `json-schema-2020-12/jsonschema-core.xml`

| Section (id after `jsonschema-2020-12.core.`) | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `$comment` | 12 | 4 | 8 | 4 | 0 | 0 | 100.0% |
| `$defs` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `$dynamicRef` | 3 | 2 | 1 | 0 | 2 | 0 | 100.0% |
| `$id` | 7 | 2 | 5 | 2 | 0 | 0 | 100.0% |
| `$ref` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `$schema` | 6 | 1 | 5 | 1 | 0 | 0 | 100.0% |
| `$vocabulary` | 18 | 11 | 7 | 10 | 1 | 0 | 100.0% |
| `a-vocabulary-for-applying-subschemas` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `a-vocabulary-for-unevaluated-locations` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `additionalProperties` | 4 | 3 | 1 | 3 | 0 | 0 | 100.0% |
| `allOf` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `anchor` | 5 | 2 | 3 | 1 | 1 | 0 | 100.0% |
| `annotations` | 5 | 0 | 5 | 0 | 0 | 0 | — |
| `anyOf` | 4 | 1 | 3 | 1 | 0 | 0 | 100.0% |
| `applicators` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `best-practices-for-vocabulary-and-meta-schema-authors` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `bundling` | 8 | 0 | 8 | 0 | 0 | 0 | — |
| `changelog` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `compound-documents` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `conditional` | 3 | 3 | 0 | 3 | 0 | 0 | 100.0% |
| `contains` | 7 | 2 | 5 | 0 | 2 | 0 | 100.0% |
| `default-behaviors` | 5 | 1 | 4 | 1 | 0 | 0 | 100.0% |
| `dependentSchemas` | 3 | 1 | 2 | 0 | 1 | 0 | 100.0% |
| `dereferencing` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `detecting-a-meta-schema` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `differing-and-default-dialects` | 2 | 2 | 0 | 2 | 0 | 0 | 100.0% |
| `else` | 3 | 2 | 1 | 1 | 1 | 0 | 100.0% |
| `embedded` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `extending` | 4 | 1 | 3 | 1 | 0 | 0 | 100.0% |
| `fragments` | 3 | 3 | 0 | 3 | 0 | 0 | 100.0% |
| `guarding-against-infinite-recursion` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `hypermedia` | 6 | 0 | 6 | 0 | 0 | 0 | — |
| `identifiers` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `if` | 4 | 3 | 1 | 0 | 3 | 0 | 100.0% |
| `in-place` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `initial-base` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `instance-data-model` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `integers` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `items` | 4 | 3 | 1 | 3 | 0 | 0 | 100.0% |
| `keyword-behaviors` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `keyword-interactions` | 3 | 0 | 3 | 0 | 0 | 0 | — |
| `loading-a-referenced-schema` | 4 | 3 | 1 | 3 | 0 | 0 | 100.0% |
| `loading-instance-data` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `non-schemas` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `not` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `oneOf` | 3 | 1 | 2 | 0 | 1 | 0 | 100.0% |
| `output` | 17 | 0 | 17 | 0 | 0 | 0 | — |
| `overview` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `patternProperties` | 4 | 1 | 3 | 0 | 1 | 0 | 100.0% |
| `prefixItems` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `properties` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `propertyNames` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `references` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `regex` | 5 | 2 | 3 | 1 | 1 | 0 | 100.0% |
| `root` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `schema-document` | 5 | 2 | 3 | 2 | 0 | 0 | 100.0% |
| `scopes` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `security` | 4 | 3 | 1 | 3 | 0 | 0 | 100.0% |
| `the-json-schema-core-vocabulary` | 5 | 3 | 2 | 3 | 0 | 0 | 100.0% |
| `then` | 3 | 2 | 1 | 1 | 1 | 0 | 100.0% |
| `unevaluatedItems` | 7 | 5 | 2 | 0 | 5 | 0 | 100.0% |
| `unevaluatedProperties` | 4 | 2 | 2 | 0 | 2 | 0 | 100.0% |
| `validating` | 4 | 0 | 4 | 0 | 0 | 0 | — |
| `vocabulary` | 3 | 0 | 3 | 0 | 0 | 0 | — |

## json-schema 2020-12 — `json-schema-2020-12/jsonschema-validation.xml`

| Section (id after `jsonschema-2020-12.validation.`) | Total | In scope | Out of scope | Covered | Lossy | Missing | Answered share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `a-vocabulary-for-basic-meta-data-annotations` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `a-vocabulary-for-structural-validation` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `changelog` | 1 | 0 | 1 | 0 | 0 | 0 | — |
| `const` | 2 | 2 | 0 | 2 | 0 | 0 | 100.0% |
| `content` | 7 | 4 | 3 | 4 | 0 | 0 | 100.0% |
| `contentEncoding` | 4 | 1 | 3 | 1 | 0 | 0 | 100.0% |
| `contentMediaType` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `contentSchema` | 4 | 2 | 2 | 2 | 0 | 0 | 100.0% |
| `default` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `dependentRequired` | 4 | 1 | 3 | 0 | 1 | 0 | 100.0% |
| `deprecated` | 6 | 1 | 5 | 1 | 0 | 0 | 100.0% |
| `enum` | 4 | 1 | 3 | 1 | 0 | 0 | 100.0% |
| `examples` | 6 | 1 | 5 | 1 | 0 | 0 | 100.0% |
| `exclusiveMaximum` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `exclusiveMinimum` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `format` | 29 | 6 | 23 | 5 | 1 | 0 | 100.0% |
| `keywords-moved-from-validation-to-core` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `maxContains` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `maxItems` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `maxLength` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `maxProperties` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `maximum` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `meta-schema` | 2 | 0 | 2 | 0 | 0 | 0 | — |
| `minContains` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `minItems` | 2 | 1 | 1 | 1 | 0 | 0 | 100.0% |
| `minLength` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `minProperties` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `minimum` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `multipleOf` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |
| `pattern` | 3 | 1 | 2 | 0 | 1 | 0 | 100.0% |
| `readonly-and-writeonly` | 7 | 2 | 5 | 2 | 0 | 0 | 100.0% |
| `required` | 3 | 1 | 2 | 1 | 0 | 0 | 100.0% |
| `security` | 1 | 1 | 0 | 1 | 0 | 0 | 100.0% |
| `title-and-description` | 3 | 2 | 1 | 2 | 0 | 0 | 100.0% |
| `type` | 4 | 2 | 2 | 0 | 2 | 0 | 100.0% |
| `uniqueItems` | 2 | 1 | 1 | 0 | 1 | 0 | 100.0% |

## Markers per file

| File | Requirements |
| --- | ---: |
| `conformance/coverage/suite.md` | 134 |
| `conformance/sdk/cases.json` | 46 |
| `crates/flowapi-cli/tests/documents_cli.rs` | 8 |
| `crates/flowapi-cli/tests/openapi_cli.rs` | 30 |
| `crates/flowapi-ts/tests/print.rs` | 267 |
| `crates/flowapi/tests/documents.rs` | 14 |
| `crates/flowapi/tests/repo.rs` | 2 |

## Markers on out-of-scope requirements

None.
