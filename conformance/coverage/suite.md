# Coverage by O3: the JSON-Schema-Test-Suite

Each group of an official suite file for keyword K is generated and its cases checked by the
TypeScript compiler (`conformance/README.md`). Every valid case of every run group is accepted
(soundness 100%, `results/suite.md`), so a file for K genuinely tests that the type is sound for
K's validation requirements. A `req:` line below counts that as coverage of the type's soundness
for the requirement; its precision is the file's row in `results/suite.md`. A `lossy:` line says
the type is wider than the schema there and why: the file shows it is still sound.

OpenAPI 3.0 Schema Object keywords are listed under the file of the JSON Schema keyword when 3.0
reads them with the same code (`schema/keyword.rs` maps them in every dialect) and gives them the
meaning JSON Schema does. Read by the ledger (`conformance/ledger`, `report`).

## `additionalProperties.json`

- req: jsonschema-2020-12.core.additionalProperties — soundness 100%; the index signature is exact without patternProperties
- req: jsonschema-2020-12.core.additionalProperties.may-implementations-choose-implement-optimize-keyword-another — done by names in properties, as the MAY describes
- req: jsonschema-2020-12.core.additionalProperties.must-implementations-do-not-support-annotation-collection — no annotation collection; the equivalent check is made
- req: oas-3.0.schema-object.additionalProperties — defaults to true: oas30.json Cat (3.0 reads it with the 2020-12 code; same meaning)

## `allOf.json`

- req: jsonschema-2020-12.core.a-vocabulary-for-applying-subschemas.should-meta-schemas-do-not-use-vocabulary — applicators apply under the standard meta-schema
- req: jsonschema-2020-12.core.allOf — an intersection; soundness 100%
- req: jsonschema-2020-12.core.in-place.must-not-subschemas-keywords-evaluate-instance-completely-independently — each branch is its own type; an intersection or union of them
- req: oas-3.0.schema-object.allOf — an intersection (3.0 reads it with the 2020-12 code; same meaning)

## `anchor.json`

- req: jsonschema-2020-12.core.anchor.$anchor — plain-name fragments resolve
- req: jsonschema-2020-12.core.fragments.must-all-fragment-identifiers-do-not-match — non-pointer fragments resolve as plain names

## `anyOf.json`

- req: jsonschema-2020-12.core.anyOf — a union; soundness 100%
- req: oas-3.0.schema-object.anyOf — a union (3.0 reads it with the 2020-12 code; same meaning)

## `boolean_schema.json`

- req: jsonschema-2020-12.core.schema-document.must-json-schema-object-boolean — boolean schemas read as any and never; other shapes are the schema.shape error (untested)
- req: oas-3.1.schema-object.may-empty-schema-allows-any-instance-validate — true is any, false is never (schema.json `never`)
- req: oas-3.2.schema-object.may-empty-schema-allows-any-instance-validate — true is any, false is never (schema.json `never`)

## `const.json`

- req: jsonschema-2020-12.core.$comment — $comment inside schemas changes no case
- req: jsonschema-2020-12.core.$comment.must-not-implementations-take-any-other-action-based — schemas carrying $comment decide every case as without it
- req: jsonschema-2020-12.core.security.must-not-implementations-parse-otherwise-take-action-based — schemas carrying $comment decide every case as without it
- req: jsonschema-2020-12.validation.const — a literal type
- req: jsonschema-2020-12.validation.const.may-value-keyword-any-type-including-null — const null, objects, arrays

## `contains.json`

- lossy: jsonschema-2020-12.core.contains — no TS type says "some item matches"; recorded lossy
- lossy: jsonschema-2020-12.core.contains.must-array-instance-valid-against-contains-least — recorded lossy; soundness 100%
- lossy: jsonschema-2020-12.validation.maxContains — no TS type expresses it: the type stays wide; recorded lossy
- lossy: jsonschema-2020-12.validation.minContains — no TS type expresses it: the type stays wide; recorded lossy

## `content.json`

- req: jsonschema-2020-12.validation.content.foreword.must-not-they-do-not-function-validation-assertions — malformed encoded content is accepted
- req: jsonschema-2020-12.validation.content.foreword.should-meta-schemas-do-not-use-vocabulary — content keywords read under the standard meta-schema
- req: jsonschema-2020-12.validation.content.implementation-requirements.must-not-due-security-performance-concerns-well-open — content is never decoded: every valid case accepted
- req: jsonschema-2020-12.validation.content.implementation-requirements.must-not-however-perform-operations-default-provide-validation — never decoded by default
- req: jsonschema-2020-12.validation.contentEncoding — an annotation: no type effect, every valid case accepted
- req: jsonschema-2020-12.validation.contentMediaType — an annotation: no type effect, every valid case accepted
- req: jsonschema-2020-12.validation.contentSchema — an annotation: no type effect, every valid case accepted
- req: jsonschema-2020-12.validation.contentSchema.should-ignored-contentmediatype-not-present — contentSchema is never applied

## `default.json`

- req: jsonschema-2020-12.validation.a-vocabulary-for-basic-meta-data-annotations.should-meta-schemas-do-not-use-vocabulary — meta-data keywords read as annotations under the standard meta-schema
- req: jsonschema-2020-12.validation.default — an annotation: the default keyword does not do anything

## `dependentRequired.json`

- lossy: jsonschema-2020-12.validation.dependentRequired — which properties are required depends on which are present; recorded lossy

## `dependentSchemas.json`

- lossy: jsonschema-2020-12.core.dependentSchemas — conditional on a property being present; recorded lossy

## `dynamicRef.json`

- lossy: jsonschema-2020-12.core.$dynamicRef — a $dynamicRef follows the dynamic scope, which a type cannot; the type is any and $dynamicRef is recorded lossy
- lossy: jsonschema-2020-12.core.$dynamicRef.must-initially-resolved-starting-point-uri-includes — a $dynamicRef follows the dynamic scope, which a type cannot; the type is any and $dynamicRef is recorded lossy
- lossy: jsonschema-2020-12.core.anchor.$dynamicAnchor — indexed as a plain anchor ($ref to it resolves); its dynamic use through $dynamicRef is widened to any
- lossy: oas-3.1.schema-object.generic-template-data-structures.may-implementations-support-defining-generic-template-data — a $dynamicRef follows the dynamic scope, which a type cannot; the type is any and $dynamicRef is recorded lossy
- lossy: oas-3.2.schema-object.generic-template-data-structures.should-implementations-support-defining-generic-template-data — a $dynamicRef follows the dynamic scope, which a type cannot; the type is any and $dynamicRef is recorded lossy

## `enum.json`

- req: jsonschema-2020-12.validation.enum — a union of literals
- req: oas-3.0.schema-object.enum — a union of literals; oas30.json Cat.kind (3.0 reads it with the 2020-12 code; same meaning)

## `format-date.json`

- req: jsonschema-2020-12.validation.format.foreword.required-implementing-support-vocabulary — the Format-Annotation vocabulary: format read, never asserted
- req: jsonschema-2020-12.validation.format.foreword.should-type-instance-validate-not-set-validation — non-strings pass every format
- req: jsonschema-2020-12.validation.format.format-annotation-vocabulary.may-choose-implement-validation-any-all-format — validation of every format is a no-op
- lossy: jsonschema-2020-12.validation.format.format-annotation-vocabulary.should-provide-implementation-specific-best-effort-validation — no TS type checks a format; the string stays string (graph keeps it in the string branch)

## `format-unknown.json`

- req: jsonschema-2020-12.validation.format — an annotation by default: every format-* file, soundness 100%
- req: jsonschema-2020-12.validation.format.custom-format-attributes.must-not-implementation-fail-collect-unknown-formats-annotations — an unknown format fails nothing
- lossy: oas-3.0.schema-object.format — no TS type checks a format; the type of `type` alone (components.json int64 -> number)
- lossy: oas-3.1.schema-object.format — no TS type checks a format; the type of `type` alone (components.json int64 -> number)
- req: oas-3.2.schema-object.data-type-format.may-tools-do-not-recognize-specific-format — an unknown format leaves the type of `type` alone
- lossy: oas-3.2.schema-object.format — no TS type checks a format; the type of `type` alone (components.json int64 -> number)

## `id.json`

- req: jsonschema-2020-12.core.$id — base URI changes, nested and relative $id, URN and other schemes
- req: jsonschema-2020-12.core.$id.must-absolute-uri-form-considered-canonical-uri — $id resolved and split from its fragment; refs by the absolute URI find the resource
- req: jsonschema-2020-12.core.compound-documents.must-each-embedded-schema-resource-treated-individual — embedded resources keep their own base (base URI change in subschema); per-resource vocabulary is V2
- req: jsonschema-2020-12.core.identifiers.must-not-canonical-schema-uris-change-while-processing — bases are fixed once in the index (schema/index.rs), before any type is built
- req: jsonschema-2020-12.core.initial-base.should-unless-id-keyword-described-earlier-section — retrieved nested refs resolve relative to their retrieval URI, not $id

## `if-then-else.json`

- lossy: jsonschema-2020-12.core.else — joined with then into then|else; recorded as if lossy
- lossy: jsonschema-2020-12.core.if — then|else stands for the conditional; recorded lossy
- lossy: jsonschema-2020-12.core.if.must-instances-fail-validate-against-keyword-s — the type is then|else, wider; recorded lossy
- lossy: jsonschema-2020-12.core.if.must-instances-successfully-validate-against-keyword-s — the type is then|else, wider; recorded lossy
- lossy: jsonschema-2020-12.core.then — joined with else into then|else; recorded as if lossy

## `infinite-loop-detection.json`

- req: jsonschema-2020-12.core.guarding-against-infinite-recursion.must-not-schema-run-infinite-loop-against-instance — and print.rs deep_inputs_do_not_overflow
- req: jsonschema-2020-12.core.security.must-not-validators-fall-infinite-loop — and print.rs deep_inputs_do_not_overflow (G1)

## `items.json`

- req: jsonschema-2020-12.core.default-behaviors.must-not-missing-keyword-produce-false-assertion-result — additional items are allowed by default; absent keywords narrow nothing
- req: jsonschema-2020-12.core.items — the rest of an array after prefixItems
- req: jsonschema-2020-12.core.items.may-implementations-choose-implement-optimize-keyword-another — done from the size of prefixItems (graph/own.rs array)
- req: jsonschema-2020-12.core.items.must-implementations-do-not-support-annotation-collection — no annotation collection; the equivalent check is made
- req: jsonschema-2020-12.core.prefixItems — a tuple; components.json Tuple
- req: oas-3.0.schema-object.items — an array of the items type (3.0 reads it with the 2020-12 code; same meaning)

## `maxItems.json`

- req: jsonschema-2020-12.validation.maxItems — a tuple with optional and rest items
- req: oas-3.0.schema-object.maxItems — a tuple with optional items (3.0 reads it with the 2020-12 code; same meaning)

## `maxLength.json`

- lossy: jsonschema-2020-12.validation.maxLength — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.maxLength — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `maxProperties.json`

- lossy: jsonschema-2020-12.validation.maxProperties — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.maxProperties — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `maximum.json`

- lossy: jsonschema-2020-12.validation.exclusiveMaximum — no TS type expresses it: the type stays wide; recorded lossy
- lossy: jsonschema-2020-12.validation.maximum — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.exclusiveMaximum — a boolean modifier in 3.0; no TS type expresses a bound; recorded lossy
- lossy: oas-3.0.schema-object.maximum — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `minItems.json`

- req: jsonschema-2020-12.validation.minItems — a tuple with optional and rest items
- req: oas-3.0.schema-object.minItems — a tuple with optional items (3.0 reads it with the 2020-12 code; same meaning)

## `minLength.json`

- lossy: jsonschema-2020-12.validation.minLength — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.minLength — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `minProperties.json`

- lossy: jsonschema-2020-12.validation.minProperties — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.minProperties — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `minimum.json`

- lossy: jsonschema-2020-12.validation.exclusiveMinimum — no TS type expresses it: the type stays wide; recorded lossy
- lossy: jsonschema-2020-12.validation.minimum — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.exclusiveMinimum — a boolean modifier in 3.0; no TS type expresses a bound; recorded lossy
- lossy: oas-3.0.schema-object.minimum — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `multipleOf.json`

- lossy: jsonschema-2020-12.validation.multipleOf — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.multipleOf — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `not.json`

- lossy: jsonschema-2020-12.core.not — TS has no negation: only a not of type names or a boolean narrows; any other not is recorded lossy
- lossy: oas-3.0.schema-object.not — TS has no negation; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `oneOf.json`

- lossy: jsonschema-2020-12.core.oneOf — a union stands for "exactly one"; recorded lossy

## `pattern.json`

- lossy: jsonschema-2020-12.validation.pattern — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.pattern — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `patternProperties.json`

- lossy: jsonschema-2020-12.core.patternProperties — only a literal text with ^ or $ becomes a template-literal key (flowapi-ts pattern.rs); any other pattern leaves string keys and joins the index signature. Not recorded in Type::lossy

## `properties.json`

- req: jsonschema-2020-12.core.properties — object members
- req: oas-3.0.schema-object.properties — object members; oas30.json Cat (3.0 reads it with the 2020-12 code; same meaning)
- req: oas-3.2.schema-object.non-json-data.must-schema-objects-do-not-contain-type — without type, properties says nothing of non-objects (properties.json; components.json Loose)

## `propertyNames.json`

- lossy: jsonschema-2020-12.core.propertyNames — no TS type constrains keys by a schema; recorded lossy

## `ref.json`

- req: jsonschema-2020-12.core.$defs — references into $defs resolve (ref to defs, remote ref with ref to defs)
- req: jsonschema-2020-12.core.$ref — every 2020-12 group: pointers, escapes, siblings, remote refs
- req: jsonschema-2020-12.core.$vocabulary.default-vocabularies.must-implementation-proceeds-processing-schema-assume-use — core keywords ($ref, $id, $defs) always apply
- req: jsonschema-2020-12.core.dereferencing.should-resulting-uri-identifies-schema-within-current — remote refs to documents given beside the schema resolve without retrieval
- req: jsonschema-2020-12.core.fragments.must-application-schema-json-supports-two-fragment — root pointer ref "#", pointer fragments, empty tokens
- req: jsonschema-2020-12.core.fragments.should-accordance-section-3-1-rfc-6839 — JSON Pointer fragments as for application/json
- req: jsonschema-2020-12.core.loading-a-referenced-schema.should-implementations-able-associate-arbitrary-uris-arbitrary — documents are added under any URI (externalSchemas) and $id URIs are indexed
- req: jsonschema-2020-12.core.loading-a-referenced-schema.should-use-uris-identify-remote-schemas-does — remote refs resolve among documents given ahead; nothing is fetched (G5, tests/repo.rs)
- req: jsonschema-2020-12.core.overview.must-document-defines-core-vocabulary-supported-any — core keywords always apply
- req: jsonschema-2020-12.core.the-json-schema-core-vocabulary.must-core-vocabulary-considered-mandatory-all-times — core keywords always apply
- req: jsonschema-2020-12.core.the-json-schema-core-vocabulary.must-meta-schemas-do-not-use-vocabulary — core keywords apply under any meta-schema
- req: oas-3.1.schema-object.specifying-schema-dialects.should-standalone-json-schema-documents-do-not — a document without $schema reads as 2020-12, the OAS dialect without its annotation vocabulary
- req: oas-3.2.schema-object.specifying-schema-dialects.should-standalone-json-schema-documents-do-not — a document without $schema reads as 2020-12, the OAS dialect without its annotation vocabulary

## `required.json`

- req: jsonschema-2020-12.validation.required — required members are not optional; components.json Pet
- req: oas-3.0.schema-object.required — oas30.json Cat.kind is required (3.0 reads it with the 2020-12 code; same meaning)

## `type.json`

- req: jsonschema-2020-12.core.$vocabulary.default-vocabularies.should-example-implementation-validator-assume-use-all — without $vocabulary every validation keyword applies
- req: jsonschema-2020-12.core.$vocabulary.default-vocabularies.should-implementation-built-specific-purpose-assume-use — without $vocabulary every applicator and validation keyword applies
- req: jsonschema-2020-12.validation.a-vocabulary-for-structural-validation.should-meta-schemas-do-not-use-vocabulary — validation keywords apply under the standard meta-schema
- lossy: jsonschema-2020-12.validation.type — integer prints as number (TS has no integer); every other type exact integer is not recorded in Type::lossy
- lossy: jsonschema-2020-12.validation.type.must-string-values-one-six-primitive-types — the six types exact; integer widened to number

## `unevaluatedItems.json`

- lossy: jsonschema-2020-12.core.unevaluatedItems — only unevaluatedItems: false narrows (closes the shape over what applicators evaluate); a schema value is recorded lossy
- lossy: jsonschema-2020-12.core.unevaluatedItems.must-boolean-true-value-present-any-relevant — the evaluated prefix is computed statically for false; otherwise recorded lossy
- lossy: jsonschema-2020-12.core.unevaluatedItems.must-means-prefixitems-items-contains-all-place — the evaluated prefix is computed statically for false; otherwise recorded lossy
- lossy: jsonschema-2020-12.core.unevaluatedItems.must-no-relevant-annotations-present-unevaluateditems-subschema — the evaluated prefix is computed statically for false; otherwise recorded lossy
- lossy: jsonschema-2020-12.core.unevaluatedItems.must-otherwise-subschema-applied-any-index-greater — the evaluated prefix is computed statically for false; otherwise recorded lossy

## `unevaluatedProperties.json`

- req: jsonschema-2020-12.core.a-vocabulary-for-unevaluated-locations.should-meta-schemas-do-not-use-vocabulary — unevaluated* apply under the standard meta-schema
- lossy: jsonschema-2020-12.core.unevaluatedProperties — only unevaluatedProperties: false narrows (closes the shape over what applicators evaluate); a schema value is recorded lossy
- lossy: jsonschema-2020-12.core.unevaluatedProperties.must-means-properties-patternproperties-additionalproperties-all-place — evaluated properties computed statically for false; otherwise recorded lossy

## `uniqueItems.json`

- lossy: jsonschema-2020-12.validation.uniqueItems — no TS type expresses it: the type stays wide; recorded lossy
- lossy: oas-3.0.schema-object.uniqueItems — no TS type expresses it; recorded lossy (3.0 reads it with the 2020-12 code; same meaning)

## `vocabulary.json`

- req: jsonschema-2020-12.core.$vocabulary.should-value-false-implementations-do-not-recognize — ignore unrecognized optional vocabulary: processing goes on
