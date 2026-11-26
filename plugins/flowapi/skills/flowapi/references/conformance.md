<!-- generated from README.md by plugins/flowapi/scripts/docs.mjs: edit the source, not this file -->

## How it holds to the specification

For every schema in the official
[JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite), flowapi prints
a type, and each test value is written as a literal of that type and handed to the TypeScript
compiler (`strict`, `exactOptionalPropertyTypes`). If the compiler refuses a valid value, the type
is wrong. If it refuses an invalid one, the type caught it.

| Draft 2020-12 | Result |
| --- | --- |
| Valid values accepted (soundness) | **100.0%** (779 of 779) |
| Invalid values rejected (precision) | 69.0% (373 of 540) |
| Groups that failed to generate, or printed invalid TypeScript | 0 of 409 |

From [conformance/results/suite.md](https://github.com/localixai/flowapi/blob/main/conformance/results/suite.md). The same measurement, each
schema wrapped as an OpenAPI 3.1 component and each tool run with its defaults
([conformance/results/compare.md](https://github.com/localixai/flowapi/blob/main/conformance/results/compare.md)):

| Generator | Valid accepted | Invalid rejected | Failed to generate | Invalid TypeScript |
| --- | ---: | ---: | ---: | ---: |
| flowapi 0.1.0 | **100.0%** | **69.0%** | 0 | 0 |
| openapi-typescript 7.13.0 | 90.6% | 51.4% | 7 | 49 |
| @hey-api/openapi-ts 0.99.0 | 94.6% | 42.2% | 7 | 0 |

TypeScript has no type for some keywords (`pattern`, `minimum`, `uniqueItems`). There the type
accepts more than the schema does, and a comment above it names the keywords it does not check.
Every keyword of every dialect is handled one of three ways: it becomes part of the type, it is
listed in that comment, or it is read as an annotation. The JSON reader is our own and passes
[JSONTestSuite](https://github.com/nst/JSONTestSuite).

The suite has a few hundred schemas, so two more checks look past it. Thousands of random
schemas and values are generated from fixed seeds; our own validator, flowschema (which passes
the whole suite), says which values are valid, and every valid one must type-check against
flowapi's type ([conformance/results/props.md](https://github.com/localixai/flowapi/blob/main/conformance/results/props.md)). And every valid
example description the OAI keeps beside its schemas generates and type-checks
([conformance/results/oai.md](https://github.com/localixai/flowapi/blob/main/conformance/results/oai.md)). A requirements ledger lists every
MUST, SHOULD and MAY of OpenAPI 3.0.4, 3.1.2, 3.2.1 and JSON Schema 2020-12 that a types
generator answers to, each held by a test or named as something a type cannot state
([conformance/REPORT.md](https://github.com/localixai/flowapi/blob/main/conformance/REPORT.md)).

One TypeScript limit shows in rare cases: an object literal with a key named like a member
every object inherits (`constructor`, `toString`) is checked against `Object`'s member when the
key is not declared, so `{ constructor: [0] }` can be refused by a type that allows it.
