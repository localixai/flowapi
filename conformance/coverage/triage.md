# Coverage triage: what no marker answers yet

> 2026-11-24, after this pass: V1 (then/else without if), V2 and V6 (vocabularies: a dialect
> with validation off; an unknown required vocabulary and a missing core are errors), V3 (`$self`
> names the document), V4 (3.0 read-only and write-only properties are not required in the one
> type), V5 (the CLI normalizes reached paths) and V7 (a repeated `$id` is a warning) are fixed;
> each has a case in `crates/flowapi-ts/tests/cases/` or was checked by hand (V5, V7). OpenAPI
> 3.0's Schema Object now takes only its own keywords. Markers for them come with the next pass.


The first coverage pass over the in-scope `jsonschema-2020-12.*` ids and the OpenAPI Schema,
Discriminator, Reference and Components Objects, the dialect and the base-URI and reference
resolution sections (2026-11-24). Each id here is missing in `REPORT.md`; the markers of the ids
that are answered are in `suite.md`, `crates/flowapi-ts/tests/print.rs` and
`crates/flowapi/tests/repo.rs`. Written by hand: an id moves out of this file when a test or a fix
answers it. Not-applicable is a proposal for `scope.json`; the owner decides.

## Violated

Reproduced with `target/release/flowapi <file.json>`.

### V1. if/then/else: `then` and `else` are applied without an `if` (unsound, G3)

```
{"then":{"type":"string"},"else":{"type":"number"}}
```

Prints `Root = string | number`; `true` is valid (no `if`: both ignored).

```
{"allOf":[{"if":{"type":"string"}},{"then":{"const":1},"else":{"const":2}}]}
```

Prints `Root = 1 | 2`; `"x"` is valid (the second branch has no `if`).

graph/read.rs sets `then_else` without looking for `if`; build.rs combine() joins then|else whenever both are present.

Ids: `jsonschema-2020-12.core.conditional.must-not-else-interact-each-other-across-subschema`, `jsonschema-2020-12.core.conditional.must-not-other-words-one-branch-allof-have`, `jsonschema-2020-12.core.conditional.must-not-particular-they-treated-present-empty-schema`, `jsonschema-2020-12.core.else.must-not-implementations-evaluate-instance-against-keyword-either`, `jsonschema-2020-12.core.then.must-not-implementations-evaluate-instance-against-keyword-either`

### V2. `$vocabulary` of a meta-schema is never read: validation keywords apply without the validation vocabulary (unsound)

```
meta.json: {"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"meta.json","$vocabulary":{"https://json-schema.org/draft/2020-12/vocab/core":true,"https://json-schema.org/draft/2020-12/vocab/applicator":true}}
schema: {"$schema":"meta.json","type":"string","$defs":{"m":{"$ref":"meta.json"}}}
```

Prints `Root = string`; `1` is valid: `type` is not an assertion here.

vocabulary.json passes only because its sample keyword (`minimum`) is lossy anyway.

Ids: `jsonschema-2020-12.core.$vocabulary`, `jsonschema-2020-12.core.$vocabulary.must-any-vocabulary-understood-implementation-processed-manner`, `jsonschema-2020-12.core.detecting-a-meta-schema.must-implementations-recognize-schema-meta-schema-being`

### V3. OpenAPI 3.2 `$self`: same-document references stop resolving

```
{"openapi":"3.2.0","$self":"https://example.com/api/openapi.json","info":{"title":"t","version":"1"},"components":{"schemas":{"A":{"type":"object","properties":{"b":{"$ref":"#/components/schemas/B"}}},"B":{"type":"string"}}}}
```

Prints `error[schema.ref]: `$ref` names nothing among the documents given: https://example.com/api/openapi.json#/components/schemas/B`; B is in the same document.

spec.rs gives schemas the `$self` base; index.rs resolve() looks the resource up by the document's own URI (docs.find), which is not `$self`. spec/operations.rs follow() special-cases it; the schema index does not.

Ids: `oas-3.2.openapi-object.$self`, `oas-3.2.openapi-object.$self.must-implementations-support-identifying-targets-api-description`, `oas-3.2.openapi-object.establishing-the-base-uri.must-relative-uri-references-resolved-using-appropriate`

### V4. OpenAPI 3.0 `readOnly`/`writeOnly` with `required`: one type serves request and response

```
3.0 component P {"type":"object","required":["id","secret"],"properties":{"id":{"type":"string","readOnly":true},"secret":{"type":"string","writeOnly":true}}} used as requestBody and 200 response
```

Prints `P = { id: string; secret: string; ... } in both`; a request without `id` and a response without `secret` are valid ("required will take effect on the response only").

Type::read_only/write_only are kept and printed nowhere.

Ids: `oas-3.0.schema-object.readOnly`, `oas-3.0.schema-object.readOnly.should-not-means-sent-part-response-but-sent`, `oas-3.0.schema-object.writeOnly`, `oas-3.0.schema-object.writeOnly.should-not-therefore-sent-part-request-but-sent`

### V5. CLI: a document reached through `../` is named by its unnormalized path

```
main.json {"openapi":"3.1.0",...,"components":{"schemas":{"A":{"$ref":"a/b.json"}}}}; a/b.json {"$ref":"../c.json"}; c.json {"type":"string"}
```

Prints `error[schema.ref]: `$ref` names nothing among the documents given: c.json`; c.json was read, under the name a/../c.json.

flowapi-cli load.rs joins paths without removing dot segments; the core resolves per RFC 3986.

Ids: `oas-3.2.openapi-object.establishing-the-base-uri.should-therefore-all-implementations-allow-users-provide`

### V6. A required vocabulary flowapi does not know is not refused

```
meta2.json with "$vocabulary":{"...vocab/core":true,"https://example.com/vocab/unknown":true}; schema {"$schema":"meta2.json","type":"string",...}
```

Prints `Root = string, exit 0`; the schema MUST be refused.

Ids: `jsonschema-2020-12.core.$vocabulary.must-used-indicate-whether-each-vocabulary-required`, `jsonschema-2020-12.core.$vocabulary.must-value-true-implementations-do-not-recognize`, `jsonschema-2020-12.core.the-json-schema-core-vocabulary.recommended-however-implementations-detect-cases-raise-error`

### V7. Two schemas with one `$id`: no error (SHOULD)

```
{"$defs":{"a":{"$id":"https://e.com/x","type":"string"},"b":{"$id":"https://e.com/x","type":"number"}},"$ref":"https://e.com/x"}
```

Prints `Root = string, no diagnostic`; the first wins silently.

Ids: `jsonschema-2020-12.core.loading-a-referenced-schema.should-multiple-schemas-try-identify-same-uri`

Also seen, tied to no id: OpenAPI 3.0 reads 2020-12-only keywords (`const`, `prefixItems`,
`unevaluatedProperties`) as assertions, though 3.0's Schema Object has none of them; `type: integer`
and `patternProperties` are widened without being recorded in `Type::lossy`.

## Untested

flowapi does what the requirement asks (read in the code, often shown by a probe), but no test
or kept case shows it.

- `jsonschema-2020-12.core.$comment.must-not-implementations-present-string-end-users`: not printed (read nowhere), but no kept output shows it
- `jsonschema-2020-12.core.$schema`: chooses the dialect (index.rs scoped); no 2020-12 suite group switches dialect and no case does
- `jsonschema-2020-12.core.$vocabulary.must-vocabulary-keyword-ignored-schema-documents-not`: $vocabulary is read nowhere (so this holds); no group puts it in a schema that is not a meta-schema
- `jsonschema-2020-12.core.$vocabulary.should-per-extending-unrecognized-keywords-treated-annotations`: Keyword::Unknown is an annotation; no run suite group or case holds an unknown keyword
- `jsonschema-2020-12.core.differing-and-default-dialects.must-multiple-schema-resources-present-single-document`: index.rs scoped() inherits the enclosing dialect; every run group is 2020-12, so nothing tests an enclosing non-default dialect
- `jsonschema-2020-12.core.extending.should-implementations-treat-keywords-they-do-not`: Keyword::Unknown is an annotation; no run suite group or case holds an unknown keyword
- `jsonschema-2020-12.core.initial-base.recommended-implementations-document-any-default-base-uri`: the CLI uses the path as given; README does not say so
- `jsonschema-2020-12.core.regex.must-not-finally-implementations-take-regular-expressions-anchored`: flowapi-ts pattern.rs turns an unanchored literal x into `${string}x${string}` (probe); no test or case has such a pattern
- `jsonschema-2020-12.core.schema-document.json-schema-objects-and-keywords.may-json-schema-contain-properties-not-schema`: unknown properties are annotations (Keyword::Unknown); no run group or case holds one
- `jsonschema-2020-12.core.schema-document.json-schema-objects-and-keywords.should-unknown-keywords-treated-annotations-value-keyword`: Keyword::Unknown is an annotation; no run group or case holds one
- `jsonschema-2020-12.core.vocabulary.updates-to-meta-schema-and-vocabulary-uris.should-implementations-consider-uris-dated-after-specification`: Dialect::from_uri matches exact URIs only; untested
- `jsonschema-2020-12.validation.deprecated`: printed as @deprecated (flowapi-ts print.rs); no case holds it
- `jsonschema-2020-12.validation.deprecated.should-deprecated-value-boolean-true-indicates-applications`: the @deprecated tag passes it on; no case holds it
- `jsonschema-2020-12.validation.examples`: an annotation (no type effect); no suite file, no case
- `jsonschema-2020-12.validation.format.format-annotation-vocabulary.should-implementations-document-level-support-validation`: README does not say that formats are not validated
- `jsonschema-2020-12.validation.keywords-moved-from-validation-to-core.should-while-definitions-absent-single-vocabulary-meta`: Keyword::of reads definitions as $defs; no run group or case uses definitions
- `jsonschema-2020-12.validation.meta-schema.should-implementations-consider-uris-dated-after-specification`: Dialect::from_uri matches exact URIs only; untested
- `jsonschema-2020-12.validation.readonly-and-writeonly.readOnly`: kept on Type but printed nowhere; no case holds it
- `jsonschema-2020-12.validation.readonly-and-writeonly.writeOnly`: kept on Type but printed nowhere; no case holds it
- `jsonschema-2020-12.validation.title-and-description.description`: printed as JSDoc (flowapi-ts print.rs); no kept case holds one
- `jsonschema-2020-12.validation.title-and-description.title`: printed as JSDoc (flowapi-ts print.rs); no kept case holds one
- `oas-3.0.components-object.headers`: headers are not read yet (operations stage)
- `oas-3.0.components-object.parameters`: a parameter Reference Object is followed (spec/operations.rs); no case has one
- `oas-3.0.components-object.requestBodies`: no 3.0 description with operations is tested
- `oas-3.0.components-object.responses`: no 3.0 description with operations is tested
- `oas-3.0.schema-object.default`: an annotation (no type effect); no 3.0 case holds one
- `oas-3.0.schema-object.deprecated`: printed as @deprecated; no case holds it
- `oas-3.0.schema-object.deprecated.should-specifies-schema-deprecated-transitioned-out-usage`: the @deprecated tag passes it on; no case holds it
- `oas-3.0.schema-object.description`: printed as JSDoc; no kept case holds a description
- `oas-3.0.schema-object.title`: printed as JSDoc; no kept case holds a title
- `oas-3.1.components-object.headers`: headers are not read yet (operations stage)
- `oas-3.1.components-object.parameters`: a parameter Reference Object is followed (spec/operations.rs); no case has one
- `oas-3.1.components-object.pathItems`: a Path Item Reference Object is followed; no case has one
- `oas-3.1.discriminator-object.must-not-note-discriminator-change-validation-outcome-schema`: no 3.1 case holds a discriminator (oas30.json does, read by the same code)
- `oas-3.1.openapi-object.jsonSchemaDialect`: read (spec.rs default_dialect; probe P5 works); no case sets it
- `oas-3.1.reference-object.description`: not implemented: flowapi prints no documentation from a Reference Object
- `oas-3.1.reference-object.description.should-description-default-override-referenced-component`: not implemented (SHOULD): nothing from a Reference Object is printed
- `oas-3.1.reference-object.fixed-fields.shall-object-cannot-extended-additional-properties-any`: spec/operations.rs follow() ignores siblings; no case has one
- `oas-3.1.reference-object.summary`: not implemented: flowapi prints no documentation from a Reference Object
- `oas-3.1.reference-object.summary.should-short-summary-default-override-referenced-component`: not implemented (SHOULD): nothing from a Reference Object is printed
- `oas-3.1.schema-object.description`: printed as JSDoc; no kept case holds a description
- `oas-3.1.schema-object.discriminator`: no 3.1 case holds a discriminator
- `oas-3.1.schema-object.specifying-schema-dialects.must-schema-keyword-present-any-schema-object`: $schema in a Schema Object chooses its dialect (probe P6 works); no case
- `oas-3.2.components-object.headers`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.mediaTypes`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.parameters`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.pathItems`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.requestBodies`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.responses`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.components-object.schemas`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.discriminator-object.must-not-note-discriminator-change-validation-outcome-schema`: no 3.2 case holds a discriminator (oas30.json does, read by the same code)
- `oas-3.2.openapi-object.jsonSchemaDialect`: read (spec.rs default_dialect; probe P5 works); no case sets it
- `oas-3.2.openapi-object.openapi-description-structure.must-order-parsing-behavior-well-defined-all`: documents are read whole, as an OpenAPI Object or a schema (source/dialect.rs detect); untested for an OAD
- `oas-3.2.openapi-object.parsing-documents.must-each-document-oad-fully-parsed-order`: every document is read before the index (spec.rs); no OAD with several documents is tested
- `oas-3.2.openapi-object.parsing-documents.must-not-implementations-treat-reference-unresolvable-before-completely`: the index is built from all documents first; no OAD with several documents is tested
- `oas-3.2.openapi-object.relative-references-in-api-description-uris.may-unless-specified-otherwise-all-fields-uris`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.openapi-object.resolving-uri-fragments.should-representation-referenced-document-json-yaml-fragment`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.reference-object.$ref`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.reference-object.description`: not implemented: flowapi prints no documentation from a Reference Object
- `oas-3.2.reference-object.description.should-description-default-override-referenced-component`: not implemented (SHOULD): nothing from a Reference Object is printed
- `oas-3.2.reference-object.fixed-fields.shall-object-cannot-extended-additional-properties-any`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.reference-object.summary`: not implemented: flowapi prints no documentation from a Reference Object
- `oas-3.2.reference-object.summary.should-short-summary-default-override-referenced-component`: not implemented (SHOULD): nothing from a Reference Object is printed
- `oas-3.2.schema-object.description`: printed as JSDoc; no kept case holds a description
- `oas-3.2.schema-object.discriminator`: no 3.2 case holds a discriminator
- `oas-3.2.schema-object.non-json-data.must-due-limited-requirements-searching-schemas-serializers`: serialization of non-JSON and binary data: the SDK stage, not implemented
- `oas-3.2.schema-object.non-json-data.must-inspecting-schemas-given-starting-point-schema`: serialization of non-JSON and binary data: the SDK stage, not implemented
- `oas-3.2.schema-object.non-json-data.must-not-implementations-inspect-subschemas-possible-reference-targets`: serialization of non-JSON and binary data: the SDK stage, not implemented
- `oas-3.2.schema-object.schema-evaluation-and-binary-data.must-oas-implementations-do-not-have-access`: serialization of non-JSON and binary data: the SDK stage, not implemented
- `oas-3.2.schema-object.specifying-schema-dialects.must-default-not-set-oas-dialect-schema`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.schema-object.specifying-schema-dialects.must-schema-keyword-present-any-schema-object`: $schema in a Schema Object chooses its dialect (probe P6 works); no case
- `oas-3.2.schema-object.specifying-schema-dialects.must-tooling-support-oas-dialect-schema-id`: no OpenAPI 3.2 description is tested anywhere
- `oas-3.2.schema-object.working-with-binary-data.shall-however-contentmediatype-contradicts-relevant-media-type`: serialization of non-JSON and binary data: the SDK stage, not implemented

## Not applicable to a types generator (proposed)

### Constrains the schema or description author; flowapi does not validate documents against the meta-schema (145)

- `jsonschema-2020-12.core.$comment.must-value-keyword-string`
- `jsonschema-2020-12.core.$defs.must-each-member-value-object-valid-json`
- `jsonschema-2020-12.core.$defs.must-keyword-s-value-object`
- `jsonschema-2020-12.core.$dynamicRef.must-value-dynamicref-property-string-uri-reference`
- `jsonschema-2020-12.core.$id.identifying-the-root-schema.should-root-schema-json-schema-document-contain`
- `jsonschema-2020-12.core.$id.must-not-therefore-id-contain-non-empty-fragment`
- `jsonschema-2020-12.core.$id.must-present-value-keyword-string-represent-valid`
- `jsonschema-2020-12.core.$id.must-uri-reference-normalized-resolve-absolute-uri`
- `jsonschema-2020-12.core.$id.not-recommended-empty-fragment-form-retained-only-backwards`
- `jsonschema-2020-12.core.$ref.must-value-ref-keyword-string-uri-reference`
- `jsonschema-2020-12.core.$schema.must-current-schema-valid-against-meta-schema`
- `jsonschema-2020-12.core.$schema.must-not-appear-non-resource-root-schema-objects`
- `jsonschema-2020-12.core.$schema.must-value-keyword-uri-containing-scheme-uri`
- `jsonschema-2020-12.core.$schema.should-schema-keyword-used-document-root-schema`
- `jsonschema-2020-12.core.$vocabulary.must-not-appear-subschemas`
- `jsonschema-2020-12.core.$vocabulary.must-property-names-object-uris-containing-scheme`
- `jsonschema-2020-12.core.$vocabulary.must-value-keyword-object`
- `jsonschema-2020-12.core.$vocabulary.must-values-object-properties-booleans`
- `jsonschema-2020-12.core.$vocabulary.should-vocabulary-keyword-used-root-schema-any`
- `jsonschema-2020-12.core.additionalProperties.must-value-additionalproperties-valid-json-schema`
- `jsonschema-2020-12.core.allOf.must-each-item-array-valid-json-schema`
- `jsonschema-2020-12.core.allOf.must-keyword-s-value-non-empty-array`
- `jsonschema-2020-12.core.anchor.must-present-value-keyword-string-start-letter`
- `jsonschema-2020-12.core.anchor.recommended-therefore-anchor-used-create-plain-name`
- `jsonschema-2020-12.core.anyOf.must-each-item-array-valid-json-schema`
- `jsonschema-2020-12.core.anyOf.must-keyword-s-value-non-empty-array`
- `jsonschema-2020-12.core.contains.must-value-keyword-valid-json-schema`
- `jsonschema-2020-12.core.dependentSchemas.must-each-value-object-valid-json-schema`
- `jsonschema-2020-12.core.dependentSchemas.must-keyword-s-value-object`
- `jsonschema-2020-12.core.else.must-keyword-s-value-valid-json-schema`
- `jsonschema-2020-12.core.if.must-keyword-s-value-valid-json-schema`
- `jsonschema-2020-12.core.items.must-value-items-valid-json-schema`
- `jsonschema-2020-12.core.not.must-keyword-s-value-valid-json-schema`
- `jsonschema-2020-12.core.oneOf.must-each-item-array-valid-json-schema`
- `jsonschema-2020-12.core.oneOf.must-keyword-s-value-non-empty-array`
- `jsonschema-2020-12.core.patternProperties.must-each-property-value-object-valid-json`
- `jsonschema-2020-12.core.patternProperties.must-value-patternproperties-object`
- `jsonschema-2020-12.core.patternProperties.should-each-property-name-object-valid-regular`
- `jsonschema-2020-12.core.prefixItems.must-value-prefixitems-non-empty-array-valid`
- `jsonschema-2020-12.core.properties.must-each-value-object-valid-json-schema`
- `jsonschema-2020-12.core.properties.must-value-properties-object`
- `jsonschema-2020-12.core.propertyNames.must-value-propertynames-valid-json-schema`
- `jsonschema-2020-12.core.regex.should-furthermore-given-high-disparity-regular-expression`
- `jsonschema-2020-12.core.regex.should-regular-expressions-valid-according-regular-expression`
- `jsonschema-2020-12.core.scopes.must-note-some-keywords-schema-apply-lexical`
- `jsonschema-2020-12.core.then.must-keyword-s-value-valid-json-schema`
- `jsonschema-2020-12.core.unevaluatedItems.must-value-unevaluateditems-valid-json-schema`
- `jsonschema-2020-12.core.unevaluatedProperties.must-value-unevaluatedproperties-valid-json-schema`
- `jsonschema-2020-12.core.vocabulary.must-describing-valid-schema-syntax-schema-successfully`
- `jsonschema-2020-12.validation.contentEncoding.must-value-property-string`
- `jsonschema-2020-12.validation.contentMediaType.must-value-property-string-media-type-defined`
- `jsonschema-2020-12.validation.contentSchema.must-value-property-valid-json-schema`
- `jsonschema-2020-12.validation.default.recommended-default-value-valid-against-associated-schema`
- `jsonschema-2020-12.validation.dependentRequired.must-elements-each-array-any-strings-unique`
- `jsonschema-2020-12.validation.dependentRequired.must-properties-object-any-arrays`
- `jsonschema-2020-12.validation.dependentRequired.must-value-keyword-object`
- `jsonschema-2020-12.validation.deprecated.must-value-keyword-boolean`
- `jsonschema-2020-12.validation.enum.must-value-keyword-array`
- `jsonschema-2020-12.validation.enum.should-array-have-least-one-element`
- `jsonschema-2020-12.validation.enum.should-elements-array-unique`
- `jsonschema-2020-12.validation.examples.must-value-keyword-array`
- `jsonschema-2020-12.validation.examples.recommended-values-valid-against-associated-schema`
- `jsonschema-2020-12.validation.exclusiveMaximum.must-value-exclusivemaximum-number-representing-exclusive-upper`
- `jsonschema-2020-12.validation.exclusiveMinimum.must-value-exclusiveminimum-number-representing-exclusive-lower`
- `jsonschema-2020-12.validation.format.custom-format-attributes.recommended-due-limitation-historically-uneven-implementation-keyword`
- `jsonschema-2020-12.validation.format.custom-format-attributes.shall-not-save-agreement-between-parties-schema-authors`
- `jsonschema-2020-12.validation.format.foreword.must-string`
- `jsonschema-2020-12.validation.format.regex.should-regular-expression-valid-according-ecma-262`
- `jsonschema-2020-12.validation.maxContains.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.maxItems.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.maxLength.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.maxProperties.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.maximum.must-value-maximum-number-representing-inclusive-upper`
- `jsonschema-2020-12.validation.minContains.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.minItems.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.minLength.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.minProperties.must-value-keyword-non-negative-integer`
- `jsonschema-2020-12.validation.minimum.must-value-minimum-number-representing-inclusive-lower`
- `jsonschema-2020-12.validation.multipleOf.must-value-multipleof-number-strictly-greater-0`
- `jsonschema-2020-12.validation.pattern.must-value-keyword-string`
- `jsonschema-2020-12.validation.pattern.should-string-valid-regular-expression-according-ecma`
- `jsonschema-2020-12.validation.readonly-and-writeonly.must-value-keywords-boolean`
- `jsonschema-2020-12.validation.required.must-elements-array-any-strings-unique`
- `jsonschema-2020-12.validation.required.must-value-keyword-array`
- `jsonschema-2020-12.validation.title-and-description.must-value-both-keywords-string`
- `jsonschema-2020-12.validation.type.must-array-elements-array-strings-unique`
- `jsonschema-2020-12.validation.type.must-value-keyword-either-string-array`
- `jsonschema-2020-12.validation.uniqueItems.must-value-keyword-boolean`
- `oas-3.0.components-object.fixed-fields.must-all-fixed-fields-declared-above-objects`
- `oas-3.0.conditions-for-using-the-discriminator-object.must-both-oneof-anyof-use-cases-keywords`
- `oas-3.0.discriminator-object.options-for-mapping-values-to-schemas.must-ensure-ambiguous-value-e-g-foo`
- `oas-3.0.discriminator-object.options-for-mapping-values-to-schemas.must-mapping-keys-string-values-but-tooling`
- `oas-3.0.discriminator-object.propertyName.required`
- `oas-3.0.discriminator-object.propertyName.should-property-required-payload-schema-behavior-property`
- `oas-3.0.openapi-description-structure.recommended-entry-document-oad-named-openapi-json`
- `oas-3.0.reference-object.$ref.required`
- `oas-3.0.relative-references-in-urls.recommended-compatibility-future-versions-specification-ref-process`
- `oas-3.0.resolving-implicit-connections.recommended-some-cases-unambiguous-uri-based-alternative`
- `oas-3.0.schema-object.additionalProperties.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.allOf.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.anyOf.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.composition-and-inheritance-polymorphism.must-discriminator-field-required-field`
- `oas-3.0.schema-object.default.must-unlike-json-schema-value-conform-defined`
- `oas-3.0.schema-object.items.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.items.must-items-present-type-array`
- `oas-3.0.schema-object.items.must-value-object-not-array`
- `oas-3.0.schema-object.not.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.oneOf.must-inline-referenced-schema-schema-object-not`
- `oas-3.0.schema-object.pattern.should-string-valid-regular-expression-according-ecma`
- `oas-3.0.schema-object.properties.must-property-definitions-schema-object-not-standard`
- `oas-3.0.schema-object.readOnly.must-not-property-marked-both-readonly-writeonly-being`
- `oas-3.0.schema-object.type.must-value-string`
- `oas-3.0.schema-object.writeOnly.must-not-property-marked-both-readonly-writeonly-being`
- `oas-3.1.components-object.fixed-fields.must-all-fixed-fields-declared-above-objects`
- `oas-3.1.conditions-for-using-the-discriminator-object.must-both-oneof-anyof-use-cases-keywords`
- `oas-3.1.discriminator-object.options-for-mapping-values-to-schemas.must-ensure-ambiguous-value-e-g-foo`
- `oas-3.1.discriminator-object.options-for-mapping-values-to-schemas.must-mapping-keys-string-values-but-tooling`
- `oas-3.1.discriminator-object.propertyName.required`
- `oas-3.1.discriminator-object.propertyName.should-property-required-payload-schema-behavior-property`
- `oas-3.1.openapi-description-structure.recommended-entry-document-oad-named-openapi-json`
- `oas-3.1.openapi-object.jsonSchemaDialect.must-form-uri`
- `oas-3.1.parsing-documents.not-recommended-while-possible-structure-certain-openapi-descriptions`
- `oas-3.1.reference-object.$ref.must-form-uri`
- `oas-3.1.reference-object.$ref.required`
- `oas-3.1.resolving-implicit-connections.recommended-some-cases-unambiguous-uri-based-alternative`
- `oas-3.1.schema-object.composition-and-inheritance-polymorphism.must-discriminator-field-required-field`
- `oas-3.1.schema-object.specifying-schema-dialects.recommended-however-maximum-interoperability-openapi-description-authors`
- `oas-3.2.base-uri-within-content.must-guarantee-interoperability-schema-objects-containing-id`
- `oas-3.2.components-object.fixed-fields.must-all-fixed-fields-declared-above-objects`
- `oas-3.2.conditions-for-using-the-discriminator-object.must-both-oneof-anyof-use-cases-keywords`
- `oas-3.2.conflicts-between-field-types-and-reference-contexts.recommended-maximum-interoperability-openapi-description-authors-avoid`
- `oas-3.2.discriminator-object.optional-discriminating-property.must-discriminating-property-defined-optional-discriminator-object`
- `oas-3.2.discriminator-object.options-for-mapping-values-to-schemas.must-ensure-ambiguous-value-e-g-foo`
- `oas-3.2.discriminator-object.options-for-mapping-values-to-schemas.must-mapping-keys-string-values-but-tooling`
- `oas-3.2.discriminator-object.propertyName.must-discriminating-property-defined-required-optional-but`
- `oas-3.2.discriminator-object.propertyName.required`
- `oas-3.2.openapi-object.$self.must-string-form-uri-reference-defined-rfc3986`
- `oas-3.2.openapi-object.jsonSchemaDialect.must-form-uri`
- `oas-3.2.openapi-object.openapi-description-structure.recommended-entry-document-oad-named-openapi-json`
- `oas-3.2.openapi-object.resolving-implicit-connections.recommended-some-cases-unambiguous-uri-based-alternative`
- `oas-3.2.reference-object.$ref.must-form-uri`
- `oas-3.2.reference-object.$ref.required`
- `oas-3.2.schema-object.composition-and-inheritance-polymorphism.must-discriminating-property-defined-required-optional-but`
- `oas-3.2.schema-object.specifying-schema-dialects.recommended-however-maximum-interoperability-openapi-description-authors`
- `oas-3.2.warnings-regarding-fragmentary-parsing.not-recommended-while-possible-structure-certain-openapi-descriptions`

### A permission flowapi does not use; nothing to test (55)

- `jsonschema-2020-12.core.$comment.may-implementations-assume-comment-allowed-unless-vocabulary`
- `jsonschema-2020-12.core.$comment.may-implementations-strip-comment-values-any-point`
- `jsonschema-2020-12.core.$comment.may-tools-translate-other-media-types-programming`
- `jsonschema-2020-12.core.$comment.may-value-keyword-used-debug-error-output`
- `jsonschema-2020-12.core.$vocabulary.default-vocabularies.may-vocabulary-absent-implementation-determine-behavior-based`
- `jsonschema-2020-12.core.detecting-a-meta-schema.may-implementations-allow-schema-explicitly-passed-meta`
- `jsonschema-2020-12.core.embedded.may-implementation-choose-not-support-addressing-schema`
- `jsonschema-2020-12.core.extending.may-implementations-provide-ability-register-load-handlers`
- `jsonschema-2020-12.validation.content.implementation-requirements.may-implementations-offer-ability-decode-parse-validate`
- `jsonschema-2020-12.validation.examples.may-examples-absent-default-still-used-manner`
- `jsonschema-2020-12.validation.examples.may-implementations-use-value-s-default-present`
- `jsonschema-2020-12.validation.format.custom-format-attributes.may-implementations-support-custom-format-attributes`
- `oas-3.0.components-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.0.discriminator-object.examples.may-oas-3-0-response-payload-described`
- `oas-3.0.discriminator-object.examples.may-scenarios-value-discriminator-field-does-not`
- `oas-3.0.openapi-description-structure.may-openapi-description-oad-made-up-single`
- `oas-3.0.schema-object.description.may-commonmark-syntax-used-rich-text-representation`
- `oas-3.0.schema-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.0.schema-object.json-schema-keywords.may-other-json-schema-subset-fields-following`
- `oas-3.1.components-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.1.discriminator-object.examples.may-oas-3-x-response-payload-described`
- `oas-3.1.discriminator-object.examples.may-scenarios-value-discriminator-field-does-not`
- `oas-3.1.discriminator-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.1.openapi-description-structure.may-openapi-description-oad-made-up-single`
- `oas-3.1.parsing-documents.may-implementations-support-complete-document-parsing-any`
- `oas-3.1.reference-object.description.may-commonmark-syntax-used-rich-text-representation`
- `oas-3.1.relative-references-in-api-description-uris.may-practice-usually-retrieval-uri-document-determined`
- `oas-3.1.schema-object.annotated-enumerations.may-implementations-support-recognizing-oneof-anyof-each`
- `oas-3.1.schema-object.description.may-commonmark-syntax-used-rich-text-representation`
- `oas-3.1.schema-object.extended-validation-with-annotations.may-oas-implementations-use-annotations-including-extensions`
- `oas-3.1.schema-object.fixed-fields.may-object-extended-specification-extensions-though-noted`
- `oas-3.1.schema-object.json-schema-keywords.may-json-schema-implementations-choose-treat-keywords`
- `oas-3.1.schema-object.non-validating-constraint-keywords.may-extended-validation-one-way-constraints-enforced`
- `oas-3.1.schema-object.validating-readonly-and-writeonly.may-json-schema-validation-draft-2020-12`
- `oas-3.1.schema-object.validating-readonly-and-writeonly.may-validation-keywords-done-checking-annotation-read`
- `oas-3.2.appendix-g-parsing-and-resolution-guidance.may-implementations-support-complete-document-parsing-any`
- `oas-3.2.components-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.2.conflicts-between-field-types-and-reference-contexts.may-same-json-yaml-object-parsed-multiple`
- `oas-3.2.discriminator-object.examples.may-oas-3-x-response-payload-described`
- `oas-3.2.discriminator-object.fixed-fields.may-object-extended-specification-extensions`
- `oas-3.2.openapi-object.establishing-the-base-uri.may-implementations-support-document-retrieval-although-see`
- `oas-3.2.openapi-object.openapi-description-structure.may-documents-different-object-root-mix-oad`
- `oas-3.2.openapi-object.openapi-description-structure.may-oad-made-up-single-document-distributed`
- `oas-3.2.openapi-object.openapi-description-structure.may-openapi-object-embedded-another-format-called`
- `oas-3.2.reference-object.description.may-commonmark-syntax-used-rich-text-representation`
- `oas-3.2.schema-object.annotated-enumerations.may-implementations-support-recognizing-oneof-anyof-each`
- `oas-3.2.schema-object.data-type-format.optional-support-any-registered-format-strictly-support`
- `oas-3.2.schema-object.description.may-commonmark-syntax-used-rich-text-representation`
- `oas-3.2.schema-object.extended-validation-with-annotations.may-oas-implementations-use-annotations-including-extensions`
- `oas-3.2.schema-object.fixed-fields.may-object-extended-specification-extensions-though-noted`
- `oas-3.2.schema-object.json-schema-keywords.may-json-schema-implementations-choose-treat-keywords`
- `oas-3.2.schema-object.non-validating-constraint-keywords.may-extended-validation-one-way-constraints-enforced`
- `oas-3.2.schema-object.parsing-and-serializing.may-application-form-incorporates-any-additional-information`
- `oas-3.2.schema-object.validating-readonly-and-writeonly.may-json-schema-validation-draft-2020-12`
- `oas-3.2.schema-object.validating-readonly-and-writeonly.may-validation-keywords-done-checking-annotation-read`

### How a validator picks a schema by the discriminator; flowapi reads discriminator as an annotation (it MUST NOT change validation) and builds the type from oneOf/anyOf alone (20)

- `oas-3.0.conditions-for-using-the-discriminator-object.may-avoid-redundancy-discriminator-added-parent-schema`
- `oas-3.0.conditions-for-using-the-discriminator-object.recommended-however-search-all-named-schemas-components`
- `oas-3.0.discriminator-object.examples.may-discriminator-used-hint-improve-efficiency-selection`
- `oas-3.0.discriminator-object.examples.should-discriminating-value-does-not-match-implicit`
- `oas-3.0.discriminator-object.mapping`
- `oas-3.0.discriminator-object.options-for-mapping-values-to-schemas.recommended-behavior-mapping-value-both-valid-schema`
- `oas-3.0.discriminator-object.propertyName`
- `oas-3.1.conditions-for-using-the-discriminator-object.may-avoid-redundancy-discriminator-added-parent-schema`
- `oas-3.1.discriminator-object.examples.may-discriminator-used-hint-improve-efficiency-selection`
- `oas-3.1.discriminator-object.examples.should-discriminating-value-does-not-match-implicit`
- `oas-3.1.discriminator-object.mapping`
- `oas-3.1.discriminator-object.options-for-mapping-values-to-schemas.recommended-behavior-mapping-value-both-valid-schema`
- `oas-3.1.discriminator-object.propertyName`
- `oas-3.2.conditions-for-using-the-discriminator-object.may-avoid-redundancy-discriminator-added-parent-schema`
- `oas-3.2.discriminator-object.defaultMapping`
- `oas-3.2.discriminator-object.examples.should-discriminating-value-does-not-match-implicit`
- `oas-3.2.discriminator-object.mapping`
- `oas-3.2.discriminator-object.may-polymorphic-schema-include-discriminator-object-defines`
- `oas-3.2.discriminator-object.options-for-mapping-values-to-schemas.recommended-behavior-mapping-value-defaultmapping-value-both`
- `oas-3.2.discriminator-object.propertyName`

### Annotation collection or output; flowapi is not a validator and collects no annotations (19)

- `jsonschema-2020-12.core.$comment.must-not-particular-value-comment-collected-annotation-result`
- `jsonschema-2020-12.core.annotations.annotations-and-assertions.must-not-schema-objects-produce-false-assertion-result`
- `jsonschema-2020-12.core.annotations.collecting-annotations.may-omitted-same-schema-location-path-above`
- `jsonschema-2020-12.core.annotations.collecting-annotations.must-collected-annotation-include-following-information`
- `jsonschema-2020-12.core.annotations.distinguishing-among-multiple-values.may-applications-make-decisions-multiple-annotation-values`
- `jsonschema-2020-12.core.annotations.may-only-exception-subschemas-schema-object-failed`
- `jsonschema-2020-12.core.anyOf.must-note-annotations-being-collected-all-subschemas`
- `jsonschema-2020-12.core.contains.may-annotation-affects-behavior-unevaluateditems-unevaluated-vocabulary`
- `jsonschema-2020-12.core.contains.may-value-boolean-true-subschema-validates-successfully`
- `jsonschema-2020-12.core.contains.must-annotation-present-instance-array-keyword-s`
- `jsonschema-2020-12.core.contains.must-subschema-applied-every-array-element-even`
- `jsonschema-2020-12.core.default-behaviors.may-because-annotation-collection-add-significant-cost`
- `jsonschema-2020-12.core.default-behaviors.must-not-however-even-value-produces-default-behavior`
- `jsonschema-2020-12.core.prefixItems.may-value-boolean-true-subschema-applied-every`
- `jsonschema-2020-12.validation.default.should-multiple-occurrences-keyword-applicable-single-sub`
- `jsonschema-2020-12.validation.deprecated.should-multiple-occurrences-keyword-applicable-single-sub`
- `jsonschema-2020-12.validation.examples.must-multiple-occurrences-keyword-applicable-single-sub`
- `jsonschema-2020-12.validation.format.format-annotation-vocabulary.must-value-format-collected-annotation-implementation-supports`
- `jsonschema-2020-12.validation.readonly-and-writeonly.should-multiple-occurrences-keywords-applicable-single-sub`

### Applies to format validation or the OPTIONAL Format-Assertion vocabulary; flowapi asserts no format (15)

- `jsonschema-2020-12.validation.format.dates-times-and-duration.may-implementations-support-additional-attributes-using-other`
- `jsonschema-2020-12.validation.format.dates-times-and-duration.must-full-date-full-time-implemented-corresponding`
- `jsonschema-2020-12.validation.format.dates-times-and-duration.should-implementations-supporting-formats-implement-support-following`
- `jsonschema-2020-12.validation.format.dates-times-and-duration.should-not-implementations-define-extension-attributes-any-name`
- `jsonschema-2020-12.validation.format.foreword.optional-implementing-support-format-assertion-vocabulary`
- `jsonschema-2020-12.validation.format.format-annotation-vocabulary.may-implementations-still-treat-format-assertion-addition`
- `jsonschema-2020-12.validation.format.format-annotation-vocabulary.must-implementation-provide-options-enable-disable-evaluation`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.must-evaluate-format-assertion`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.must-format-assertion-vocabulary-declared-value-true`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.must-implement-syntactic-validation-all-format-attributes`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.must-implementations-cannot-provide-full-validation-support`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.must-still-collect-format-annotation-implementation-supports`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.recommended-implementations-use-common-parsing-library-each`
- `jsonschema-2020-12.validation.format.format-assertion-vocabulary.should-implementations-clearly-document-how-what-degree`
- `jsonschema-2020-12.validation.format.regex.must-implementations-validate-formats-accept-least-subset`

### Binds keyword or vocabulary definitions, not an implementation (11)

- `jsonschema-2020-12.core.keyword-behaviors.may-however-extension-keywords-define-other-behaviors`
- `jsonschema-2020-12.core.keyword-behaviors.should-extension-keywords-stay-within-categories-keeping`
- `jsonschema-2020-12.core.keyword-interactions.may-keyword-behavior-defined-terms-annotation-results`
- `jsonschema-2020-12.core.keyword-interactions.may-keywords-modify-behavior-based-presence-absence`
- `jsonschema-2020-12.core.keyword-interactions.must-not-keywords-result-circular-dependency`
- `jsonschema-2020-12.core.loading-instance-data.may-keywords-defined-use-json-pointers-relative`
- `jsonschema-2020-12.core.loading-instance-data.should-keywords-allow-adjusting-location-using-relative`
- `jsonschema-2020-12.core.non-schemas.must-not-however-behavior-implementation-specific-relied-upon`
- `jsonschema-2020-12.core.regex.may-keywords-use-regular-expressions-express-constraints`
- `jsonschema-2020-12.core.schema-document.json-schema-objects-and-keywords.should-keywords-may-fall-multiple-categories-although`
- `jsonschema-2020-12.core.scopes.may-keywords-defined-partial-value-uri-reference`

### Binds a bundler; flowapi bundles nothing (8)

- `jsonschema-2020-12.core.bundling.may-key-defs-now-embedded-schema-resource`
- `jsonschema-2020-12.core.bundling.may-schema-resource-embedded-location-other-defs`
- `jsonschema-2020-12.core.bundling.must-each-embedded-json-schema-resource-identify`
- `jsonschema-2020-12.core.bundling.must-not-bundled-schema-resource-bundled-replacing-schema`
- `jsonschema-2020-12.core.bundling.must-not-order-produce-identical-output-references-containing`
- `jsonschema-2020-12.core.bundling.recommended-schema-resource-referenced-reference-applicator-bundled`
- `jsonschema-2020-12.core.bundling.recommended-uri-identifier-value-id-absolute-uri`
- `jsonschema-2020-12.core.bundling.should-bundling-done-way-all-uris-used`

### Link operationId and tag or component names: flowapi reads no Link or Tag, and schemas connect only by URI (6)

- `oas-3.0.resolving-implicit-connections.recommended-consider-all-operation-objects-all-parsed`
- `oas-3.0.resolving-implicit-connections.recommended-resolving-component-tag-name-connections-referenced`
- `oas-3.1.resolving-implicit-connections.recommended-consider-all-operation-objects-all-parsed`
- `oas-3.1.resolving-implicit-connections.recommended-resolving-component-tag-name-connections-referenced`
- `oas-3.2.openapi-object.resolving-implicit-connections.recommended-resolving-components-object-tag-object-names`
- `oas-3.2.openapi-object.resolving-implicit-connections.recommended-resolving-operation-object-based-operationid-consider`

### Binds vocabulary authors (5)

- `jsonschema-2020-12.core.$comment.must-not-vocabularies-specify-any-effect-comment-beyond`
- `jsonschema-2020-12.core.$comment.should-schema-vocabularies-allow-comment-within-any`
- `jsonschema-2020-12.core.$vocabulary.may-vocabulary-authors-use-url-vocabulary-specification`
- `jsonschema-2020-12.core.instance-data-model.should-json-schema-vocabularies-wish-work-differences`
- `jsonschema-2020-12.validation.keywords-moved-from-validation-to-core.should-not-schema-vocabulary-authors-define-definitions-keyword`

### Binds authors (4)

- `jsonschema-2020-12.core.embedded.should-not-since-uris-involving-json-pointer-fragments`
- `jsonschema-2020-12.core.embedded.should-not-therefore-schema-authors-rely-uris-using`
- `jsonschema-2020-12.core.extending.shall-not-save-explicit-agreement-schema-authors-expect`
- `jsonschema-2020-12.core.guarding-against-infinite-recursion.should-not-schemas-make-use-infinite-recursive-nesting`

### Validating schema documents against meta-schemas, which flowapi does not do (4)

- `jsonschema-2020-12.core.validating.may-compound-schema-document-all-embedded-resources`
- `jsonschema-2020-12.core.validating.recommended-alternate-validation-process-provided-order-validate`
- `jsonschema-2020-12.core.validating.should-each-schema-resource-separately-validated-against`
- `jsonschema-2020-12.core.validating.should-not-given-compound-schema-document-may-have`

### A permission for authors (3)

- `jsonschema-2020-12.core.differing-and-default-dialects.may-since-any-schema-referenced-embedded-embedded`
- `jsonschema-2020-12.core.extending.may-additional-schema-keywords-schema-vocabularies-defined`
- `jsonschema-2020-12.validation.contentSchema.may-keyword-used-any-media-type-mapped`

### Binds extension authors (3)

- `jsonschema-2020-12.core.the-json-schema-core-vocabulary.recommended-while-prefix-not-formally-reserved-core`
- `jsonschema-2020-12.core.unevaluatedItems.must-not-authors-extension-keywords-define-place-applicator`
- `jsonschema-2020-12.core.unevaluatedProperties.must-not-authors-extension-keywords-define-place-applicator`

### Binds the owning authority or a user interface (3)

- `jsonschema-2020-12.validation.readonly-and-writeonly.may-instance-document-marked-readonly-entire-document`
- `jsonschema-2020-12.validation.readonly-and-writeonly.may-instance-document-marked-writeonly-entire-document`
- `jsonschema-2020-12.validation.readonly-and-writeonly.may-particular-application-choose-use-widget-hides`

### Describes the specification itself (2)

- `jsonschema-2020-12.core.a-vocabulary-for-applying-subschemas.recommended-section-defines-vocabulary-applicator-keywords-use`
- `jsonschema-2020-12.core.overview.recommended-additionally-document-defines-vocabulary-keywords-applying`

### Binds keyword definitions (2)

- `jsonschema-2020-12.core.default-behaviors.should-keywords-specified-terms-collected-annotations-describe`
- `jsonschema-2020-12.core.default-behaviors.should-some-cases-missing-keyword-assertion-behavior`

### Binds meta-schema authors (2)

- `jsonschema-2020-12.core.detecting-a-meta-schema.must-not-meta-schema-authors-expect-features-interoperable`
- `jsonschema-2020-12.core.the-json-schema-core-vocabulary.must-meta-schemas-use-vocabulary-keyword-declare`

### A statement about publishing (2)

- `jsonschema-2020-12.core.vocabulary.updates-to-meta-schema-and-vocabulary-uris.may-updated-vocabulary-meta-schema-uris-published`
- `jsonschema-2020-12.validation.meta-schema.may-updated-vocabulary-meta-schema-uris-published`

### Applies only to decoding (2)

- `jsonschema-2020-12.validation.contentEncoding.should-base64-defined-both-rfcs-definition-rfc`
- `jsonschema-2020-12.validation.contentEncoding.should-instance-value-string-property-defines-string`

### A statement of meaning (2)

- `jsonschema-2020-12.validation.deprecated.may-mean-property-going-removed-future`
- `jsonschema-2020-12.validation.deprecated.may-root-schema-containing-deprecated-value-true`

### Binds schema editors; flowapi edits no schema (1)

- `jsonschema-2020-12.core.$comment.should-tools-editing-schemas-support-displaying-editing`

### Binds whoever serves meta-schemas; flowapi retrieves nothing (1)

- `jsonschema-2020-12.core.$schema.should-uri-identifies-retrievable-resource-resource-media`

### A statement about vocabulary URIs (1)

- `jsonschema-2020-12.core.$vocabulary.may-uri-url-but-nature-retrievable-resource`

### A permission flowapi does not use; nothing to test (duplicate anchors: the first wins, no error) (1)

- `jsonschema-2020-12.core.anchor.may-implementations-raise-error-usage-detected`

### Binds the definition of applicators (vocabulary authors) (1)

- `jsonschema-2020-12.core.applicators.must-not-applicators-may-apply-any-boolean-logic`

### A permission flowapi does not use; nothing to test (the host names each document) (1)

- `jsonschema-2020-12.core.initial-base.may-no-source-known-no-uri-scheme`

### Binds producers of instances (1)

- `jsonschema-2020-12.core.integers.should-not-consistency-integer-json-numbers-encoded-fractional`

### A statement about schemas (1)

- `jsonschema-2020-12.core.loading-a-referenced-schema.may-schema-likely-will-have-multiple-uris`

### Flowapi evaluates no regular expression (1)

- `jsonschema-2020-12.core.regex.should-regular-expressions-built-u-flag-equivalent`

### A permission about identifiers (1)

- `jsonschema-2020-12.core.root.may-schema-resources-identified-uris-including-uris`

### A media type statement (1)

- `jsonschema-2020-12.core.schema-document.should-schema-itself-interpreted-instance-but-always`

### Binds servers that host schemas (1)

- `jsonschema-2020-12.core.security.must-servers-ensure-malicious-parties-cannot-change`

### Part of an example (1)

- `jsonschema-2020-12.validation.content.example.must-per-section-8-1-rfc-8259`

### Applies only to an implementation that decodes content (1)

- `jsonschema-2020-12.validation.content.implementation-requirements.should-process-equivalent-fully-evaluating-instance-against`

### Flowapi implements no Format-Assertion vocabulary (OPTIONAL); refusing a required one is V6 (1)

- `jsonschema-2020-12.validation.format.custom-format-attributes.must-format-assertion-vocabulary-specified-implementations-fail`
