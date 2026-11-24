# Conformance

How flowapi is held to the specifications: the official JSON Schema Test Suite through the TypeScript compiler, random schemas and the OAI's examples.

## The JSON Schema Test Suite

Every schema in the official [JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite) is turned into a type, and each of its test values is written as a literal of that type and given to the TypeScript compiler with `strict` and `exactOptionalPropertyTypes`. When the compiler refuses a valid value, the type is wrong. When it refuses an invalid one, the type caught it.

| Draft 2020-12 | Result |
| --- | --- |
| Valid values accepted (soundness) | **100.0%** (779 of 779) |
| Invalid values rejected (precision) | 69.0% (373 of 540) |
| Groups that failed to generate, or printed invalid TypeScript | 0 of 409 |

Soundness has to stay at 100%, and precision may never fall: the repository's checks fail otherwise. The table per keyword is in [conformance/results/suite.md](https://github.com/localixai/flowapi/blob/main/conformance/results/suite.md).

## Beside other generators

The same measurement, with each schema wrapped as an OpenAPI 3.1 component and each tool run with its defaults:

| Generator | Valid accepted | Invalid rejected | Failed to generate | Invalid TypeScript |
| --- | ---: | ---: | ---: | ---: |
| flowapi 0.1.0 | **100.0%** | **69.0%** | 0 | 0 |
| openapi-typescript 7.13.0 | 90.6% | 51.4% | 7 | 49 |
| @hey-api/openapi-ts 0.99.0 | 94.6% | 42.2% | 7 | 0 |

Per keyword, and on the groups every tool could generate: [conformance/results/compare.md](https://github.com/localixai/flowapi/blob/main/conformance/results/compare.md).

## The Rust target

For Rust, a valid case counts as accepted when serde reads it into the printed type and writes it back unchanged, and an invalid case counts as rejected when serde refuses it. Soundness is 100.0% here too (787 of 787 valid cases), and precision is 60.7% (330 of 543). Our own cases run beside the suite's, for integers up to 2^64 and past it ([conformance/results/suite-rust.md](https://github.com/localixai/flowapi/blob/main/conformance/results/suite-rust.md)).

## Past the suite

The suite has a few hundred schemas, so three more checks look further.

- Random schemas: 600 schemas from a fixed seed, 12 values each. flowschema, our own validator, which passes the whole suite, decides which values are valid, and every valid one must type-check against flowapi's type: 4,319 of 4,319 do ([conformance/results/props.md](https://github.com/localixai/flowapi/blob/main/conformance/results/props.md)).
- The OAI's examples: every valid example description the OpenAPI Initiative keeps beside its schemas must generate and type-check: 78 of 78 do ([conformance/results/oai.md](https://github.com/localixai/flowapi/blob/main/conformance/results/oai.md)).
- A requirements ledger: every MUST, SHOULD and MAY of OpenAPI 3.0.4, 3.1.2, 3.2.1 and JSON Schema 2020-12 that a types generator has to answer to is listed, and each one is either held by a test or named as something a type cannot state ([conformance/REPORT.md](https://github.com/localixai/flowapi/blob/main/conformance/REPORT.md)).

The JSON reader is flowapi's own and passes [JSONTestSuite](https://github.com/nst/JSONTestSuite). The YAML reader passes the YAML test suite.

## No keyword dropped in silence

Every keyword of every dialect is handled in one of three ways: it becomes part of the type, it is listed in the comment above the type as not checked, or it is read as an annotation. A keyword a later draft brought (`if` in draft 7, `prefixItems` in 2020-12) means nothing under an earlier draft's `$schema`.

## One limit of TypeScript

An object literal with a key named like a member every object inherits (`constructor`, `toString`) is checked against `Object`'s member when the type does not declare that key. So `{ constructor: [0] }` can be refused by a type that allows it. This is the compiler's behaviour, and it shows only in rare cases.
