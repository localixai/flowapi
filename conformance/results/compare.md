# O3 for every generator: JSON-Schema-Test-Suite through the TypeScript compiler

Written by `node conformance/compare/compare.mjs`; do not edit. How it works and its caveats: `conformance/compare/README.md`.

TypeScript 7.0.2; flowapi 0.1.0, openapi-typescript 7.13.0, hey-api 0.99.0.

| Tool | Version | Groups run | Gen. errors | Invalid TS | Valid | Accepted | Sound % | Invalid | Rejected | Precision % |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| flowapi | 0.1.0 | 409 | 0 | 0 | 779 | 779 | 100.0 | 540 | 336 | 62.2 |
| openapi-typescript | 7.13.0 | 409 | 7 | 49 | 714 | 647 | 90.6 | 470 | 242 | 51.4 |
| hey-api | 0.99.0 | 409 | 7 | 0 | 773 | 732 | 94.6 | 530 | 224 | 42.2 |

On the 352 groups every tool generated into compiling TypeScript:

| Tool | Valid | Accepted | Sound % |
| --- | ---: | ---: | ---: |
| flowapi | 713 | 713 | 100.0 |
| openapi-typescript | 713 | 646 | 90.6 |
| hey-api | 713 | 680 | 95.3 |

## Soundness per keyword file

Accepted of valid cases, then the share; files with no group run are left out.

| File | flowapi | openapi-typescript | hey-api |
| --- | ---: | ---: | ---: |
| additionalProperties | 12/12 (100.0) | 6/12 (50.0) | 8/12 (66.6) |
| allOf | 10/10 (100.0) | 9/9 (100.0) | 9/9 (100.0) |
| anchor | 9/9 (100.0) | 3/4 (75.0) | 9/9 (100.0) |
| anyOf | 12/12 (100.0) | 12/12 (100.0) | 12/12 (100.0) |
| boolean_schema | 9/9 (100.0) | 9/9 (100.0) | 9/9 (100.0) |
| const | 22/22 (100.0) | 22/22 (100.0) | 22/22 (100.0) |
| contains | 37/37 (100.0) | 37/37 (100.0) | 37/37 (100.0) |
| content | 18/18 (100.0) | 18/18 (100.0) | 18/18 (100.0) |
| default | 6/6 (100.0) | 3/6 (50.0) | 6/6 (100.0) |
| dependentRequired | 14/14 (100.0) | 14/14 (100.0) | 14/14 (100.0) |
| dependentSchemas | 10/10 (100.0) | 8/10 (80.0) | 8/10 (80.0) |
| dynamicRef | 22/22 (100.0) | 4/10 (40.0) | 20/22 (90.9) |
| enum | 24/24 (100.0) | 24/24 (100.0) | 22/24 (91.6) |
| format-date-time | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-date | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-duration | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-email | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-hostname | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-idn-email | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-idn-hostname | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-ipv4 | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-ipv6 | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-iri-reference | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-iri | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-json-pointer | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-regex | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-relative-json-pointer | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-time | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-unknown | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-uri-reference | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-uri-template | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-uri | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| format-uuid | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| id | 25/25 (100.0) | 2/12 (16.6) | 20/25 (80.0) |
| if-then-else | 20/20 (100.0) | 20/20 (100.0) | 20/20 (100.0) |
| infinite-loop-detection | 1/1 (100.0) | 0/1 (0.0) | 1/1 (100.0) |
| items | 26/26 (100.0) | 24/24 (100.0) | 24/24 (100.0) |
| maxItems | 4/4 (100.0) | 4/4 (100.0) | 4/4 (100.0) |
| maxLength | 5/5 (100.0) | 5/5 (100.0) | 5/5 (100.0) |
| maxProperties | 7/7 (100.0) | 7/7 (100.0) | 7/7 (100.0) |
| maximum | 8/8 (100.0) | 8/8 (100.0) | 8/8 (100.0) |
| minItems | 4/4 (100.0) | 4/4 (100.0) | 4/4 (100.0) |
| minLength | 4/4 (100.0) | 4/4 (100.0) | 4/4 (100.0) |
| minProperties | 8/8 (100.0) | 8/8 (100.0) | 8/8 (100.0) |
| minimum | 10/10 (100.0) | 10/10 (100.0) | 10/10 (100.0) |
| multipleOf | 8/8 (100.0) | 8/8 (100.0) | 8/8 (100.0) |
| not | 16/16 (100.0) | 15/16 (93.7) | 15/16 (93.7) |
| oneOf | 12/12 (100.0) | 12/12 (100.0) | 12/12 (100.0) |
| pattern | 12/12 (100.0) | 12/12 (100.0) | 12/12 (100.0) |
| patternProperties | 15/15 (100.0) | 11/15 (73.3) | 15/15 (100.0) |
| properties | 16/16 (100.0) | 10/16 (62.5) | 9/16 (56.2) |
| propertyNames | 17/17 (100.0) | 17/17 (100.0) | 17/17 (100.0) |
| ref | 25/25 (100.0) | 8/16 (50.0) | 22/24 (91.6) |
| required | 12/12 (100.0) | 7/12 (58.3) | 7/12 (58.3) |
| type | 21/21 (100.0) | 19/21 (90.4) | 21/21 (100.0) |
| unevaluatedItems | 42/42 (100.0) | 38/38 (100.0) | 40/41 (97.5) |
| unevaluatedProperties | 67/67 (100.0) | 39/51 (76.4) | 56/66 (84.8) |
| uniqueItems | 46/46 (100.0) | 46/46 (100.0) | 46/46 (100.0) |
| vocabulary | 3/3 (100.0) | - | 3/3 (100.0) |

## flowapi: unsound cases (0)

None.

## openapi-typescript: unsound cases (67, the first 50)

- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores arrays: `[1,2,3]`  
  error TS2322: Type 'number[]' is not assignable to type '{ foo?: unknown; bar?: unknown; } & { [key: string]: unknown; }'.
- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores strings: `"foobarbaz"`  
  error TS2322: Type 'string' is not assignable to type '{ foo?: unknown; bar?: unknown; } & { [key: string]: unknown; }'.
- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type '{ foo?: unknown; bar?: unknown; } & { [key: string]: unknown; }'.
- `additionalProperties` #2 additionalProperties with schema / no additional properties is valid: `{"foo":1}`  
  error TS2322: Type '{ foo: number; }' is not assignable to type '{ foo?: unknown; bar?: unknown; } & { [key: string]: boolean; }'.
- `additionalProperties` #2 additionalProperties with schema / an additional valid property is valid: `{"foo":1,"bar":2,"quux":true}`  
  error TS2322: Type '{ foo: number; bar: number; quux: true; }' is not assignable to type '{ foo?: unknown; bar?: unknown; } & { [key: string]: boolean; }'.
- `additionalProperties` #4 additionalProperties are allowed by default / additional properties are allowed: `{"foo":1,"bar":2,"quux":true}`  
  error TS2353: Object literal may only specify known properties, and '"quux"' does not exist in type '{ foo?: unknown; bar?: unknown; }'.
- `anchor` #5 URN base URI with URN and anchor ref / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `default` #0 invalid type for default / still valid when the invalid default is used: `{}`  
  error TS2741: Property 'foo' is missing in type '{}' but required in type '{ foo: number; }'.
- `default` #1 invalid string value for default / still valid when the invalid default is used: `{}`  
  error TS2741: Property 'bar' is missing in type '{}' but required in type '{ bar: string; }'.
- `default` #2 the default keyword does not do anything if the property is missing / missing properties are not filled in with the default: `{}`  
  error TS2741: Property 'alpha' is missing in type '{}' but required in type '{ alpha: number; }'.
- `dependentSchemas` #6 dependent subschema incompatible with root / matches dependency: `{"bar":1}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: unknown; }'.
- `dependentSchemas` #6 dependent subschema incompatible with root / no dependency: `{"baz":1}`  
  error TS2353: Object literal may only specify known properties, and '"baz"' does not exist in type '{ foo?: unknown; }'.
- `dynamicRef` #9 after leaving a dynamic scope, it is not used by a $dynamicRef (2020) / /then/$defs/thingy is the final stop for the $dynamicRef: `null`  
  error TS2322: Type 'null' is not assignable to type '{ $defs: { start: unknown; thingy: string; }; }'.
- `dynamicRef` #13 tests for implementation dynamic anchor and reference link (2020) / correct extended schema: `{"elements":[{"a":1}]}`  
  error TS2741: Property '$defs' is missing in type '{ elements: { a: number; }[]; }' but required in type '{ elements: unknown[]; $defs: { elements: unknown; }; }'.
- `dynamicRef` #15 $ref and $dynamicAnchor are independent of order - $defs first (2020) / correct extended schema: `{"elements":[{"a":1}]}`  
  error TS2322: Type '{ elements: { a: number; }[]; }' is not assignable to type '{ elements: unknown[]; $defs: { elements: unknown; }; } & { $defs: { elements: { a: unknown; }; }; }'.
- `dynamicRef` #17 $ref and $dynamicAnchor are independent of order - $ref first (2020) / correct extended schema: `{"elements":[{"a":1}]}`  
  error TS2322: Type '{ elements: { a: number; }[]; }' is not assignable to type '{ $defs: { elements: { a: unknown; }; }; } & { elements: unknown[]; $defs: { elements: unknown; }; }'.
- `dynamicRef` #31 multiple dynamic paths to the $dynamicRef keyword (2020) / number list with number values: `{"kindOfList":"numbers","list":[1.1]}`  
  error TS2353: Object literal may only specify known properties, and '"kindOfList"' does not exist in type '{ $defs: { genericList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; numberList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; stringList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; }; }'.
- `dynamicRef` #31 multiple dynamic paths to the $dynamicRef keyword (2020) / string list with string values: `{"kindOfList":"strings","list":["foo"]}`  
  error TS2353: Object literal may only specify known properties, and '"kindOfList"' does not exist in type '{ $defs: { genericList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; numberList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; stringList: { list?: unknown; $defs: { defaultItemType: unknown; }; }; }; }'.
- `id` #1 base URI change - change folder / number is valid: `{"list":[1]}`  
  error TS2741: Property '$defs' is missing in type '{ list: number[]; }' but required in type '{ list?: number[]; $defs: { baz: number[]; }; }'.
- `id` #2 base URI change - change folder in subschema / number is valid: `{"list":[1]}`  
  error TS2741: Property '$defs' is missing in type '{ list: number[]; }' but required in type '{ list?: number[]; $defs: { baz: { $defs: { bar: number[]; }; }; }; }'.
- `id` #9 simple URN base URI with $ref via the URN / valid under the URN IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type '{ foo?: ...; }'.
- `id` #10 simple tag URI base URI with $ref via the tag URI / valid under the tag URI IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type '{ foo?: ...; }'.
- `id` #11 simple base URI using an arbitrary URI scheme with $ref to that URI / valid under the arbitrary scheme IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type '{ foo?: ...; }'.
- `id` #12 simple URN base URI with JSON pointer / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `id` #13 URN base URI with NSS / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `id` #14 URN base URI with r-component / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `id` #15 URN base URI with q-component / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `id` #16 URN base URI with URN and JSON pointer ref / a string is valid: `{"foo":"bar"}`  
  error TS2741: Property '$defs' is missing in type '{ foo: string; }' but required in type '{ foo?: string; $defs: { bar: string; }; }'.
- `infinite-loop-detection` #0 evaluating the same schema location against the same data location twice is not a sign of an infinite loop / passing case: `{"foo":1}`  
  error TS2322: Type '{ foo: number; }' is not assignable to type '{ $defs: { int: number; }; } & { foo?: number; } & { [key: string]: number; }'.
- `not` #3 forbidden property / property absent: `{"bar":1,"baz":2}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: unknown; }'.
- `patternProperties` #0 patternProperties validates properties matching a regex / ignores arrays: `["foo"]`  
  error TS2322: Type 'string' is not assignable to type 'number'.
- `patternProperties` #0 patternProperties validates properties matching a regex / ignores strings: `"foo"`  
  error TS2322: Type 'string' is not assignable to type '{ [key: string]: number; }'.
- `patternProperties` #0 patternProperties validates properties matching a regex / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type '{ [key: string]: number; }'.
- `patternProperties` #2 regexes are not anchored by default and are case sensitive / regexes are case sensitive: `{"a_x_3":3}`  
  error TS2322: Type 'number' is not assignable to type 'string \| boolean'.
- `properties` #0 object properties validation / doesn't invalidate other properties: `{"quux":[]}`  
  error TS2353: Object literal may only specify known properties, and '"quux"' does not exist in type '{ foo?: number; bar?: string; }'.
- `properties` #0 object properties validation / ignores arrays: `[]`  
  error TS2559: Type 'never[]' has no properties in common with type '{ foo?: number; bar?: string; }'.
- `properties` #0 object properties validation / ignores other non-objects: `12`  
  error TS2559: Type '12' has no properties in common with type '{ foo?: number; bar?: string; }'.
- `properties` #5 properties whose names are Javascript object property names / ignores arrays: `[]`  
  error TS2322: Type 'never[]' is not assignable to type '{ __proto__?: number; toString?: { length?: string; }; constructor?: number; }'.
- `properties` #5 properties whose names are Javascript object property names / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type '{ __proto__?: number; toString?: { length?: string; }; constructor?: number; }'.
- `properties` #5 properties whose names are Javascript object property names / none of the properties mentioned: `{}`  
  error TS2322: Type '{}' is not assignable to type '{ __proto__?: number; toString?: { length?: string; }; constructor?: number; }'.
- `ref` #0 root pointer ref / match: `{"foo":false}`  
  error TS2559: Type 'false' has no properties in common with type '{ foo?: ...; }'.
- `ref` #0 root pointer ref / recursive match: `{"foo":{"foo":false}}`  
  error TS2559: Type 'false' has no properties in common with type '{ foo?: ...; }'.
- `ref` #3 escaped pointer ref / slash valid: `{"slash":123}`  
  error TS2741: Property '$defs' is missing in type '{ slash: number; }' but required in type '{ tilde?: number; slash?: number; percent?: number; $defs: { "tilde~field": number; "slash/field": number; "percent%field": number; }; }'.
- `ref` #3 escaped pointer ref / tilde valid: `{"tilde":123}`  
  error TS2741: Property '$defs' is missing in type '{ tilde: number; }' but required in type '{ tilde?: number; slash?: number; percent?: number; $defs: { "tilde~field": number; "slash/field": number; "percent%field": number; }; }'.
- `ref` #3 escaped pointer ref / percent valid: `{"percent":123}`  
  error TS2741: Property '$defs' is missing in type '{ percent: number; }' but required in type '{ tilde?: number; slash?: number; percent?: number; $defs: { "tilde~field": number; "slash/field": number; "percent%field": number; }; }'.
- `ref` #5 ref applies alongside sibling keywords / ref valid, maxItems valid: `{"foo":[]}`  
  error TS2741: Property '$defs' is missing in type '{ foo: never[]; }' but required in type '{ foo?: unknown[]; $defs: { reffed: unknown[]; }; }'.
- `ref` #7 property named $ref, containing an actual $ref / property named $ref valid: `{"$ref":"a"}`  
  error TS2741: Property '$defs' is missing in type '{ $ref: string; }' but required in type '{ $ref?: string; $defs: { "is-string": string; }; }'.
- `ref` #12 refs with quote / object with numbers is valid: `{"foo\"bar":1}`  
  error TS2741: Property '$defs' is missing in type '{ "foo\"bar": number; }' but required in type '{ "foo\"bar"?: number; $defs: { "foo\"bar": number; }; }'.
- `required` #1 required validation / ignores arrays: `[]`  
  error TS2741: Property 'foo' is missing in type 'never[]' but required in type '{ foo: unknown; bar?: unknown; }'.
- `required` #1 required validation / ignores strings: `""`  
  error TS2322: Type 'string' is not assignable to type '{ foo: unknown; bar?: unknown; }'.

### openapi-typescript: generation errors (7)

- `allOf` #3 allOf with boolean schemas, all true: Cannot use 'in' operator to search for '$ref' in true
- `allOf` #4 allOf with boolean schemas, some false: Cannot use 'in' operator to search for '$ref' in true
- `allOf` #5 allOf with boolean schemas, all false: Cannot use 'in' operator to search for '$ref' in false
- `dynamicRef` #32 $dynamicRef points to a boolean schema: Cannot use 'in' operator to search for 'readOnly' in true
- `ref` #17 empty tokens in $ref json-pointer: Can't resolve $ref at #/components/schemas/Root/allOf/0
- `unevaluatedItems` #26 unevaluatedItems with boolean schemas: Cannot use 'in' operator to search for '$ref' in true
- `unevaluatedProperties` #18 unevaluatedProperties with boolean schemas: Cannot use 'in' operator to search for '$ref' in true

### openapi-typescript: invalid TypeScript (49)

- `anchor` #0 Location-independent identifier: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `anchor` #1 Location-independent identifier with absolute URI: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `anchor` #2 Location-independent identifier with base URI change in subschema: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `anchor` #3 same $anchor with different base uri: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `anchor` #4 Reference an anchor with a non-relative URI: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #3 A $dynamicRef resolves to the first $dynamicAnchor still in scope that is encountered when the schema is evaluated (2020): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #5 A $dynamicRef with intermediate scopes that don't include a matching $dynamicAnchor does not affect dynamic scope resolution (2020): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #7 An $anchor with the same name as a $dynamicAnchor is not used for dynamic scope resolution (2020): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #21 $dynamicRef skips over intermediate resources - direct reference (2020): line 8: error TS2502: 'bar' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #23 $dynamicRef avoids the root of each schema, but scopes are still registered (2020): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #25 A $ref to a $dynamicAnchor in the same schema resource behaves like a normal $ref to an $anchor: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #26 A $dynamicRef without anchor in fragment behaves identical to $ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #27 A $dynamicRef without a matching $dynamicAnchor in the same schema resource behaves like a normal $ref to $anchor: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #28 A $dynamicRef with a non-matching $dynamicAnchor in the same schema resource behaves like a normal $ref to $anchor: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #29 A $dynamicRef that initially resolves to a schema with a matching $dynamicAnchor resolves to the first $dynamicAnchor in the dynamic scope: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `dynamicRef` #30 A $dynamicRef that initially resolves to a schema without a matching $dynamicAnchor behaves like a normal $ref to $anchor: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #3 refs with relative uris and defs: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #4 relative refs with absolute uris and defs: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #5 $id must be resolved against nearest parent, not just immediate parent: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #6 order of evaluation: $id and $ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #7 order of evaluation: $id and $anchor and $ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #8 order of evaluation: $id and $ref on nested schema: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #17 ref with absolute-path-reference: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #18 $id with file URI still resolves pointers - *nix: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #19 $id with file URI still resolves pointers - windows: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #20 URN ref with nested pointer ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `id` #22 remote HTTP ref with different $id: line 6: error TS2502: 'Remote0' is referenced directly or indirectly in its own type annotation.
- `id` #23 remote HTTP ref with different URN $id: line 6: error TS2502: 'Remote0' is referenced directly or indirectly in its own type annotation.
- `id` #24 remote HTTP ref with nested absolute ref: line 6: error TS2502: 'Remote0' is referenced directly or indirectly in its own type annotation.
- `items` #7 items and subitems: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #4 nested refs: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #9 $ref to boolean schema true: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #10 $ref to boolean schema false: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #13 ref creates new scope when adjacent to keywords: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #14 ref to if: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #15 ref to then: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #16 ref to else: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `ref` #21 root ref in remote ref: line 6: error TS2339: Property '$defs' does not exist on type 'String'.
- `ref` #22 remote ref with ref to defs: line 6: error TS2502: 'Remote0' is referenced directly or indirectly in its own type annotation.
- `unevaluatedItems` #27 unevaluatedItems with $ref (prefixItems): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedItems` #29 unevaluatedItems before $ref (prefixItems): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedItems` #32 unevaluatedItems with $dynamicRef (2020-12): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedProperties` #19 unevaluatedProperties with $ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedProperties` #20 unevaluatedProperties before $ref: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedProperties` #22 unevaluatedProperties with $dynamicRef (2020): line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedProperties` #36 unevaluatedProperties + ref inside allOf / oneOf: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `unevaluatedProperties` #37 dynamic evaluation inside nested refs: line 5: error TS2502: 'Root' is referenced directly or indirectly in its own type annotation.
- `vocabulary` #2 schema that uses custom metaschema with no validation vocabulary: line 58: error TS2300: Duplicate identifier '$defs'.
- `vocabulary` #3 ignore unrecognized optional vocabulary (dynamic anchor): line 66: error TS2300: Duplicate identifier '$defs'.

## hey-api: unsound cases (41)

- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores arrays: `[1,2,3]`  
  error TS2322: Type 'number[]' is not assignable to type 'Root'.
- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores strings: `"foobarbaz"`  
  error TS2322: Type 'string' is not assignable to type 'Root'.
- `additionalProperties` #0 additionalProperties being false does not allow other properties / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type 'Root'.
- `additionalProperties` #4 additionalProperties are allowed by default / additional properties are allowed: `{"foo":1,"bar":2,"quux":true}`  
  error TS2353: Object literal may only specify known properties, and '"quux"' does not exist in type 'Root'.
- `dependentSchemas` #6 dependent subschema incompatible with root / matches dependency: `{"bar":1}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.
- `dependentSchemas` #6 dependent subschema incompatible with root / no dependency: `{"baz":1}`  
  error TS2353: Object literal may only specify known properties, and '"baz"' does not exist in type 'Root'.
- `dynamicRef` #29 A $dynamicRef that initially resolves to a schema with a matching $dynamicAnchor resolves to the first $dynamicAnchor in the dynamic scope / The recursive part is valid against the root: `{"foo":"pass","bar":{"baz":{"foo":"pass"}}}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `dynamicRef` #30 A $dynamicRef that initially resolves to a schema without a matching $dynamicAnchor behaves like a normal $ref to $anchor / The recursive part doesn't need to validate against the root: `{"foo":"pass","bar":{"baz":{"foo":"fail"}}}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `enum` #1 heterogeneous enum validation / valid object matches: `{"foo":12}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type '[]'.
- `enum` #2 heterogeneous enum-with-null validation / null is valid: `null`  
  error TS2322: Type 'null' is not assignable to type '6'.
- `id` #3 refs with relative uris and defs / valid on both fields: `{"foo":{"bar":"a"},"bar":"a"}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `id` #4 relative refs with absolute uris and defs / valid on both fields: `{"foo":{"bar":"a"},"bar":"a"}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `id` #9 simple URN base URI with $ref via the URN / valid under the URN IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type 'Root'.
- `id` #10 simple tag URI base URI with $ref via the tag URI / valid under the tag URI IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type 'Root'.
- `id` #11 simple base URI using an arbitrary URI scheme with $ref to that URI / valid under the arbitrary scheme IDed schema: `{"foo":37}`  
  error TS2559: Type 'number' has no properties in common with type 'Root'.
- `not` #3 forbidden property / property absent: `{"bar":1,"baz":2}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.
- `properties` #0 object properties validation / doesn't invalidate other properties: `{"quux":[]}`  
  error TS2353: Object literal may only specify known properties, and '"quux"' does not exist in type 'Root'.
- `properties` #0 object properties validation / ignores arrays: `[]`  
  error TS2559: Type 'never[]' has no properties in common with type 'Root'.
- `properties` #0 object properties validation / ignores other non-objects: `12`  
  error TS2559: Type '12' has no properties in common with type 'Root'.
- `properties` #5 properties whose names are Javascript object property names / ignores arrays: `[]`  
  error TS2322: Type 'never[]' is not assignable to type 'Root'.
- `properties` #5 properties whose names are Javascript object property names / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type 'Root'.
- `properties` #5 properties whose names are Javascript object property names / none of the properties mentioned: `{}`  
  error TS2322: Type '{}' is not assignable to type 'Root'.
- `properties` #5 properties whose names are Javascript object property names / all present and valid: `{"__proto__":12,"toString":{"length":"foo"},"constructor":37}`  
  error TS2353: Object literal may only specify known properties, and '"__proto__"' does not exist in type 'Root'.
- `ref` #0 root pointer ref / match: `{"foo":false}`  
  error TS2559: Type 'false' has no properties in common with type 'Root'.
- `ref` #0 root pointer ref / recursive match: `{"foo":{"foo":false}}`  
  error TS2559: Type 'false' has no properties in common with type 'Root'.
- `required` #1 required validation / ignores arrays: `[]`  
  error TS2741: Property 'foo' is missing in type 'never[]' but required in type 'Root'.
- `required` #1 required validation / ignores strings: `""`  
  error TS2322: Type 'string' is not assignable to type 'Root'.
- `required` #1 required validation / ignores other non-objects: `12`  
  error TS2322: Type 'number' is not assignable to type 'Root'.
- `required` #1 required validation / ignores null: `null`  
  error TS2322: Type 'null' is not assignable to type 'Root'.
- `required` #1 required validation / ignores boolean: `true`  
  error TS2322: Type 'boolean' is not assignable to type 'Root'.
- `unevaluatedItems` #32 unevaluatedItems with $dynamicRef (2020-12) / with no unevaluated items: `["foo","bar"]`  
  error TS2322: Type '[string, string]' is not assignable to type 'Root'.
- `unevaluatedProperties` #8 unevaluatedProperties with nested patternProperties / with no additional properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: string; }'.
- `unevaluatedProperties` #9 unevaluatedProperties with nested additionalProperties / with additional properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: string; }'.
- `unevaluatedProperties` #10 unevaluatedProperties with nested unevaluatedProperties / with nested unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: string; }'.
- `unevaluatedProperties` #17 unevaluatedProperties with dependentSchemas / with no unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.
- `unevaluatedProperties` #19 unevaluatedProperties with $ref / with no unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `unevaluatedProperties` #20 unevaluatedProperties before $ref / with no unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"foo"' does not exist in type 'Root'.
- `unevaluatedProperties` #22 unevaluatedProperties with $dynamicRef (2020) / with no unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.
- `unevaluatedProperties` #26 nested unevaluatedProperties, outer false, inner true, properties outside / with nested unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type '{ foo?: string; }'.
- `unevaluatedProperties` #27 nested unevaluatedProperties, outer false, inner true, properties inside / with nested unevaluated properties: `{"foo":"foo","bar":"bar"}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.
- `unevaluatedProperties` #42 dependentSchemas with unevaluatedProperties / unevaluatedProperties sees bar when foo2 is present: `{"foo2":"","bar":""}`  
  error TS2353: Object literal may only specify known properties, and '"bar"' does not exist in type 'Root'.

### hey-api: generation errors (7)

- `allOf` #3 allOf with boolean schemas, all true: Cannot use 'in' operator to search for '$ref' in true
- `allOf` #4 allOf with boolean schemas, some false: Cannot use 'in' operator to search for '$ref' in true
- `allOf` #5 allOf with boolean schemas, all false: Cannot use 'in' operator to search for '$ref' in false
- `items` #7 items and subitems: Symbol finalName has not been resolved yet for ~ Symbol#1 ""
- `ref` #8 naive replacement of $ref with its destination is not correct: Cannot read properties of null (reading 'a_string')
- `unevaluatedItems` #26 unevaluatedItems with boolean schemas: Cannot use 'in' operator to search for '$ref' in true
- `unevaluatedProperties` #18 unevaluatedProperties with boolean schemas: Cannot use 'in' operator to search for '$ref' in true
