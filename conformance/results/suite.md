# O3: JSON-Schema-Test-Suite through the TypeScript compiler

Written by `cd conformance/suite && cargo run --release`; do not edit.

- Soundness: 100.0% (779 of 779 valid cases accepted)
- Precision: 63.3% (342 of 540 invalid cases rejected)
- Groups: 613 in the suite, 409 run, 409 generated
- Generation errors: 0; invalid TypeScript: 0

| File | Groups | Run | Gen. errors | Invalid TS | Valid | Accepted | Sound % | Invalid | Rejected | Precision % |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| additionalProperties | 10 | 9 | 0 | 0 | 12 | 12 | 100.0 | 9 | 8 | 88.8 |
| allOf | 12 | 12 | 0 | 0 | 10 | 10 | 100.0 | 20 | 12 | 60.0 |
| anchor-legacy | 14 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| anchor | 9 | 9 | 0 | 0 | 9 | 9 | 100.0 | 9 | 9 | 100.0 |
| anyOf | 8 | 8 | 0 | 0 | 12 | 12 | 100.0 | 6 | 4 | 66.6 |
| boolean_schema | 2 | 2 | 0 | 0 | 9 | 9 | 100.0 | 9 | 9 | 100.0 |
| const | 17 | 17 | 0 | 0 | 22 | 22 | 100.0 | 32 | 32 | 100.0 |
| contains | 21 | 20 | 0 | 0 | 37 | 37 | 100.0 | 31 | 0 | 0.0 |
| content | 4 | 4 | 0 | 0 | 18 | 18 | 100.0 | 0 | 0 | - |
| default | 3 | 3 | 0 | 0 | 6 | 6 | 100.0 | 1 | 0 | 0.0 |
| dependentRequired | 9 | 4 | 0 | 0 | 14 | 14 | 100.0 | 6 | 6 | 100.0 |
| dependentSchemas | 8 | 4 | 0 | 0 | 10 | 10 | 100.0 | 10 | 9 | 90.0 |
| disallow | 3 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| dynamicRef | 33 | 21 | 0 | 0 | 22 | 22 | 100.0 | 22 | 4 | 18.1 |
| enum | 18 | 17 | 0 | 0 | 24 | 24 | 100.0 | 31 | 31 | 100.0 |
| extends | 3 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| format-color | 3 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| format-date-time | 4 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-date | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-duration | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-email | 4 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-host-name | 3 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| format-hostname | 5 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-idn-email | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-idn-hostname | 4 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-ip-address | 3 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| format-ipv4 | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-ipv6 | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-iri-reference | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-iri | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-json-pointer | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-regex | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-relative-json-pointer | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-time | 7 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-unknown | 1 | 1 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-uri-reference | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-uri-template | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-uri | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| format-uuid | 3 | 2 | 0 | 0 | 7 | 7 | 100.0 | 0 | 0 | - |
| id-legacy | 40 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| id | 25 | 25 | 0 | 0 | 25 | 25 | 100.0 | 27 | 22 | 81.4 |
| if-then-else | 12 | 12 | 0 | 0 | 20 | 20 | 100.0 | 10 | 2 | 20.0 |
| infinite-loop-detection-legacy | 2 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| infinite-loop-detection | 1 | 1 | 0 | 0 | 1 | 1 | 100.0 | 1 | 1 | 100.0 |
| items | 28 | 14 | 0 | 0 | 26 | 26 | 100.0 | 14 | 13 | 92.8 |
| maxItems | 2 | 2 | 0 | 0 | 4 | 4 | 100.0 | 2 | 2 | 100.0 |
| maxLength | 2 | 2 | 0 | 0 | 5 | 5 | 100.0 | 2 | 0 | 0.0 |
| maxProperties | 3 | 3 | 0 | 0 | 7 | 7 | 100.0 | 3 | 0 | 0.0 |
| maximum | 5 | 3 | 0 | 0 | 8 | 8 | 100.0 | 4 | 0 | 0.0 |
| minItems | 2 | 2 | 0 | 0 | 4 | 4 | 100.0 | 2 | 2 | 100.0 |
| minLength | 2 | 2 | 0 | 0 | 4 | 4 | 100.0 | 3 | 0 | 0.0 |
| minProperties | 2 | 2 | 0 | 0 | 8 | 8 | 100.0 | 2 | 0 | 0.0 |
| minimum | 5 | 3 | 0 | 0 | 10 | 10 | 100.0 | 5 | 0 | 0.0 |
| multipleOf | 12 | 6 | 0 | 0 | 8 | 8 | 100.0 | 5 | 0 | 0.0 |
| not | 9 | 9 | 0 | 0 | 16 | 16 | 100.0 | 24 | 20 | 83.3 |
| oneOf | 11 | 11 | 0 | 0 | 12 | 12 | 100.0 | 15 | 6 | 40.0 |
| pattern | 4 | 4 | 0 | 0 | 12 | 12 | 100.0 | 3 | 0 | 0.0 |
| patternProperties | 6 | 6 | 0 | 0 | 15 | 15 | 100.0 | 11 | 0 | 0.0 |
| properties | 6 | 6 | 0 | 0 | 16 | 16 | 100.0 | 12 | 9 | 75.0 |
| propertyNames | 6 | 6 | 0 | 0 | 17 | 17 | 100.0 | 5 | 0 | 0.0 |
| recursiveRef | 10 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| ref-legacy | 14 | 0 | 0 | 0 | 0 | 0 | - | 0 | 0 | - |
| ref | 23 | 23 | 0 | 0 | 25 | 25 | 100.0 | 27 | 27 | 100.0 |
| required | 7 | 5 | 0 | 0 | 12 | 12 | 100.0 | 6 | 5 | 83.3 |
| type | 16 | 12 | 0 | 0 | 21 | 21 | 100.0 | 59 | 57 | 96.6 |
| unevaluatedItems | 48 | 29 | 0 | 0 | 42 | 42 | 100.0 | 29 | 14 | 48.2 |
| unevaluatedProperties | 46 | 44 | 0 | 0 | 67 | 67 | 100.0 | 62 | 35 | 56.4 |
| uniqueItems | 10 | 6 | 0 | 0 | 46 | 46 | 100.0 | 19 | 2 | 10.5 |
| vocabulary | 4 | 2 | 0 | 0 | 3 | 3 | 100.0 | 2 | 1 | 50.0 |
| **total** | 613 | 409 | 0 | 0 | 779 | 779 | 100.0 | 540 | 342 | 63.3 |

## Unsound cases

A valid case the type rejects: each is a bug.


## Invalid TypeScript

The generated types themselves do not compile.


## Generation errors

