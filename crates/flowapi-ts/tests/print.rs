//! The TypeScript printed for each description in `tests/cases/`, against the `.ts` kept beside
//! it. `FLOWAPI_BLESS=1 cargo test -p flowapi-ts --test print` writes them anew; a change to one
//! is read in review like a change to code.
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use std::fmt::Write;
use std::path::Path;

fn generate(uri: &str, text: &str) -> String {
    let mut docs = Documents::new();
    docs.add(uri, text).expect("JSON");
    let target = flowapi_ts::TypeScript::new("out.ts");
    let (output, diags) = flowapi::generate(&docs, &target);
    let mut text = output
        .map(|o| o.files.into_iter().map(|f| f.text).collect::<String>())
        .unwrap_or_default();
    for d in diags.all() {
        let _ = writeln!(text, "// {d}");
    }
    text
}

// The requirements each kept case covers (the ledger, conformance/ledger): the case's file, the
// type in it, and what its kept TypeScript shows.
//
// every case
// req: oas-3.2.openapi-object.establishing-the-base-uri.should-therefore-all-implementations-allow-users-provide — generate(): the host gives each document its URI (Documents::add), and references resolve against it
//
// components.json
// req: oas-3.1.components-object.schemas — every type: each component schema is a named type
// req: oas-3.1.relative-references-in-api-description-uris.may-unless-specified-otherwise-all-fields-uris — every $ref: fragment-only relative references resolve against the document
// req: oas-3.1.relative-references-in-api-description-uris.must-relative-uri-references-other-objects-schema — Pet: same-document refs resolve against the referring document's URI (multi-document: V5)
// req: oas-3.1.relative-references-in-api-description-uris.should-representation-referenced-document-json-yaml-fragment — Owner: #/components/schemas/... read as a JSON Pointer
// req: oas-3.1.schema-object.specifying-schema-dialects.must-default-not-set-oas-dialect-schema — Tuple: without jsonSchemaDialect, prefixItems and type arrays read as 2020-12
// req: oas-3.1.schema-object.specifying-schema-dialects.must-tooling-support-oas-dialect-schema-id — every type: the OAS dialect (2020-12 with OAS annotations) is the default
//
// oas30.json
// req: oas-3.0.components-object.schemas — every type: each component schema is a named type
// req: oas-3.0.discriminator-object.must-not-note-discriminator-change-validation-outcome-schema — Choice: the type is the oneOf union; the discriminator changes nothing
// req: oas-3.0.reference-object.$ref — Overridden: a schema Reference Object resolves to its target
// req: oas-3.0.reference-object.fixed-fields.shall-object-cannot-extended-additional-properties-any — Overridden: $ref with type: integer beside it prints as Maybe: the sibling is ignored
// req: oas-3.0.relative-references-in-urls.may-unless-specified-otherwise-all-fields-urls — every $ref: fragment-only relative references resolve against the document
// req: oas-3.0.schema-object.discriminator — Choice: read as an annotation beside oneOf
// req: oas-3.0.schema-object.nullable — Maybe, NoType: type plus nullable adds null; nullable without type changes nothing
// lossy: oas-3.0.schema-object.oneOf — Choice: a union stands for "exactly one"; recorded lossy (printed: Not checked by this type: oneOf)
// lossy: oas-3.0.schema-object.type — Maybe: string|null exact; integer prints as number (TS has no integer)
//
// operations.json
// req: oas-3.1.components-object.requestBodies — PutPetsId: components/requestBodies/PetBody reached by a Reference Object
// req: oas-3.1.components-object.responses — PutPetsId: components/responses/Err reached by a Reference Object
// req: oas-3.1.reference-object.$ref — PutPetsId: requestBody and response Reference Objects resolve to components
// req: oas-3.1.request-body-object.required — PutPetsId: PetBody required: true makes requestBody required
//
// annotations.json
// req: jsonschema-2020-12.core.$comment.may-implementations-strip-comment-values-any-point — Root: its $comment is in no output
// req: jsonschema-2020-12.core.$comment.must-not-implementations-present-string-end-users — Root: its $comment is in no output
// req: jsonschema-2020-12.core.$schema — emb: an embedded resource naming draft-07 reads items as a tuple
// req: jsonschema-2020-12.core.$vocabulary.default-vocabularies.may-vocabulary-absent-implementation-determine-behavior-based — emb: an embedded resource naming draft-07 reads items as a tuple
// req: jsonschema-2020-12.core.$vocabulary.must-vocabulary-keyword-ignored-schema-documents-not — Root: a $vocabulary naming an unknown required vocabulary in a schema that is no meta-schema is ignored (no error)
// req: jsonschema-2020-12.core.$vocabulary.should-per-extending-unrecognized-keywords-treated-annotations — Root: x-unknown-keyword and made-up change nothing
// req: jsonschema-2020-12.core.conditional.must-not-else-interact-each-other-across-subschema — split: an if in one allOf branch leaves the then/else of another without meaning: unknown
// req: jsonschema-2020-12.core.conditional.must-not-other-words-one-branch-allof-have — split: an if in one allOf branch leaves the then/else of another without meaning: unknown
// req: jsonschema-2020-12.core.differing-and-default-dialects.may-since-any-schema-referenced-embedded-embedded — emb: an embedded resource naming draft-07 reads items as a tuple
// req: jsonschema-2020-12.core.differing-and-default-dialects.must-multiple-schema-resources-present-single-document — inner: an embedded resource without $schema reads as 2020-12 (prefixItems)
// req: jsonschema-2020-12.core.extending.should-implementations-treat-keywords-they-do-not — Root: x-unknown-keyword and made-up change nothing
// req: jsonschema-2020-12.core.regex.must-not-finally-implementations-take-regular-expressions-anchored — patternProperties "id" matches anywhere: `${string}id${string}`
// lossy: jsonschema-2020-12.core.regex.should-regular-expressions-built-u-flag-equivalent — flowapi evaluates no regular expression; only a literal pattern narrows a key, other patterns leave it string
// req: jsonschema-2020-12.core.schema-document.json-schema-objects-and-keywords.should-unknown-keywords-treated-annotations-value-keyword — Root: x-unknown-keyword and made-up change nothing
// req: jsonschema-2020-12.validation.deprecated — old: @deprecated in its doc comment
// req: jsonschema-2020-12.validation.examples — old: examples and default change no type
// req: jsonschema-2020-12.validation.keywords-moved-from-validation-to-core.should-while-definitions-absent-single-vocabulary-meta — def: an $id under definitions is found like one under $defs
// req: jsonschema-2020-12.validation.readonly-and-writeonly.readOnly — ro, wo: annotations in 2020-12; both stay required
// req: jsonschema-2020-12.validation.readonly-and-writeonly.writeOnly — ro, wo: annotations in 2020-12; both stay required
// req: jsonschema-2020-12.validation.title-and-description.description — Root, old: title and description become the doc comment
// req: jsonschema-2020-12.validation.title-and-description.title — Root, old: title and description become the doc comment
//
// conditional.json
// req: jsonschema-2020-12.core.conditional.must-not-particular-they-treated-present-empty-schema — noIf: then and else without if mean nothing: unknown
// req: jsonschema-2020-12.core.else.must-not-implementations-evaluate-instance-against-keyword-either — noIf: then and else without if mean nothing: unknown
// req: jsonschema-2020-12.core.then.must-not-implementations-evaluate-instance-against-keyword-either — noIf: then and else without if mean nothing: unknown
//
// duplicate-id.json
// req: jsonschema-2020-12.core.loading-a-referenced-schema.should-multiple-schemas-try-identify-same-uri — two $defs claim one $id: a diagnostic names the second (a warning; the first keeps the URI)
//
// oas30-operations.json
// req: oas-3.0.components-object.fixed-fields.may-object-extended-specification-extensions — x-c/x-comp in components is ignored
// req: oas-3.0.components-object.parameters — a parameter reached by a Reference Object to components/parameters
// req: oas-3.0.components-object.requestBodies — a body and a response reached by Reference Objects
// req: oas-3.0.components-object.responses — a body and a response reached by Reference Objects
// req: oas-3.0.data-type-format.may-tools-do-not-recognize-specific-format — an unknown format leaves the type alone
// req: oas-3.0.media-type-object.fixed-fields.may-object-extended-specification-extensions — x-m in a media type is ignored
// req: oas-3.0.media-type-object.schema — each media type maps to its schema's type
// req: oas-3.0.openapi-object.components — every component schema is a named type
// req: oas-3.0.openapi-object.fixed-fields.may-object-extended-specification-extensions — x-top is ignored
// req: oas-3.0.openapi-object.openapi — the version selects how the document is read
// req: oas-3.0.openapi-object.openapi.should-openapi-field-used-tooling-interpret-openapi — the version selects how the document is read
// req: oas-3.0.openapi-object.paths — each path item gives operations and Paths
// req: oas-3.0.operation-object.fixed-fields.may-object-extended-specification-extensions — x-o is ignored
// req: oas-3.0.operation-object.operationId — operationId names the operation's type
// req: oas-3.0.operation-object.parameters — parameters by location, then name
// req: oas-3.0.operation-object.requestBody — requestBody by media type
// req: oas-3.0.operation-object.responses — responses by status, then media type
// req: oas-3.0.parameter-object.common-fixed-fields.may-object-extended-specification-extensions — x-param is ignored
// req: oas-3.0.parameter-object.content — a parameter with content takes its one media type's schema
// req: oas-3.0.parameter-object.in — in picks the parameter's group
// req: oas-3.0.parameter-object.name — name keys the parameter
// req: oas-3.0.parameter-object.required — required: true makes the member required
// req: oas-3.0.parameter-object.required.may-otherwise-field-included-default-value-false — without required the member is optional
// req: oas-3.0.parameter-object.required.required-parameter-location-path-field-value-true — path parameters are required
// req: oas-3.0.parameter-object.schema — a parameter's schema gives its type
// req: oas-3.0.path-item-object.$ref — a Path Item reached by $ref alone gives its operations (fields beside the $ref are dropped: conformance/triage-2.md F3)
// req: oas-3.0.path-item-object.delete — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.fixed-fields.may-object-extended-specification-extensions — x- fields of a path item are ignored
// req: oas-3.0.path-item-object.get — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.head — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.may-path-item-empty-due-acl-constraints — /empty gives no operation
// req: oas-3.0.path-item-object.options — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.parameters — path item parameters apply to each operation; the operation's own win
// req: oas-3.0.path-item-object.patch — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.post — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.put — each method of a path item is an operation in Paths
// req: oas-3.0.path-item-object.trace — each method of a path item is an operation in Paths
// req: oas-3.0.paths-object.pattern-path — each path keys Paths
// req: oas-3.0.paths-object.patterned-fields.may-object-extended-specification-extensions — an x- key of paths is not a path
// req: oas-3.0.request-body-object.content — the body by media type
// req: oas-3.0.request-body-object.fixed-fields.may-object-extended-specification-extensions — x-b is ignored
// req: oas-3.0.request-body-object.required — required: true makes requestBody required, else optional
// req: oas-3.0.response-object.content — the response by media type; none is never
// req: oas-3.0.response-object.fixed-fields.may-object-extended-specification-extensions — x-resp/x-r in a response is ignored
// req: oas-3.0.responses-object.default — default is a member like a status code
// req: oas-3.0.responses-object.may-default-used-default-response-object-all — default is a member like a status code
// req: oas-3.0.responses-object.pattern-http-status-code — each status code is a member
// req: oas-3.0.responses-object.pattern-http-status-code.may-define-range-response-codes-field-contain — a range (4XX, 5XX) is a member
// req: oas-3.0.responses-object.patterned-fields.may-object-extended-specification-extensions — x-r in responses is not a status
// req: oas-3.0.schema-object.default — User.name: default changes no type
// req: oas-3.0.schema-object.deprecated — User.name: @deprecated
// req: oas-3.0.schema-object.description — a schema's description is its doc comment
// req: oas-3.0.schema-object.fixed-fields.may-object-extended-specification-extensions — x-internal changes nothing
// req: oas-3.0.schema-object.readOnly — User: a required read-only or write-only property is optional in the one type
// lossy: oas-3.0.schema-object.readOnly.should-not-means-sent-part-response-but-sent — one type for requests and responses: a read-only property is optional there, not absent from requests
// req: oas-3.0.schema-object.title — User: title in its doc comment
// req: oas-3.0.schema-object.writeOnly — User: a required read-only or write-only property is optional in the one type
// lossy: oas-3.0.schema-object.writeOnly.should-not-therefore-sent-part-request-but-sent — one type for requests and responses: a write-only property is optional there, not absent from responses
// req: oas-3.0.specification-extensions.pattern-x — x- fields throughout change no type
// req: oas-3.0.versions.shall-major-minor-portion-version-string-example — 3.0.4 here and 3.0.3 in oas30.json read alike
// req: oas-3.0.versions.should-not-patch-version-considered-tooling-making-no — 3.0.4 here and 3.0.3 in oas30.json read alike
// req: oas-3.0.versions.should-tooling-supports-oas-3-1-compatible — 3.0.4 here and 3.0.3 in oas30.json read alike
//
// oas31-operations.json
// req: oas-3.1.components-object.fixed-fields.may-object-extended-specification-extensions — x-c/x-comp in components is ignored
// req: oas-3.1.components-object.parameters — a parameter reached by a Reference Object to components/parameters
// req: oas-3.1.components-object.pathItems — /pets: a Path Item from components/pathItems
// req: oas-3.1.data-type-format.may-tools-do-not-recognize-specific-format — an unknown format leaves the type alone
// req: oas-3.1.discriminator-object.fixed-fields.may-object-extended-specification-extensions — Pet: x-d in the discriminator is ignored
// req: oas-3.1.discriminator-object.must-not-note-discriminator-change-validation-outcome-schema — Pet: the oneOf union; the discriminator is an annotation
// req: oas-3.1.media-type-object.fixed-fields.may-object-extended-specification-extensions — x-m in a media type is ignored
// req: oas-3.1.media-type-object.schema — each media type maps to its schema's type
// req: oas-3.1.openapi-object.components — every component schema is a named type
// req: oas-3.1.openapi-object.fixed-fields.may-object-extended-specification-extensions — x-top is ignored
// req: oas-3.1.openapi-object.openapi — the version selects how the document is read
// req: oas-3.1.openapi-object.openapi.should-openapi-field-used-tooling-interpret-openapi — the version selects how the document is read
// req: oas-3.1.openapi-object.paths — each path item gives operations and Paths
// req: oas-3.1.operation-object.fixed-fields.may-object-extended-specification-extensions — x-o is ignored
// req: oas-3.1.operation-object.operationId — operationId names the operation's type
// req: oas-3.1.operation-object.parameters — parameters by location, then name
// req: oas-3.1.operation-object.requestBody — requestBody by media type
// req: oas-3.1.operation-object.responses — responses by status, then media type
// req: oas-3.1.parameter-object.common-fixed-fields.may-object-extended-specification-extensions — x-param is ignored
// req: oas-3.1.parameter-object.content — a parameter with content takes its one media type's schema
// req: oas-3.1.parameter-object.in — in picks the parameter's group
// req: oas-3.1.parameter-object.name — name keys the parameter
// req: oas-3.1.parameter-object.required — required: true makes the member required
// req: oas-3.1.parameter-object.required.may-otherwise-field-included-default-value-false — without required the member is optional
// req: oas-3.1.parameter-object.required.required-parameter-location-path-field-value-true — path parameters are required
// req: oas-3.1.parameter-object.schema — a parameter's schema gives its type
// req: oas-3.1.path-item-object.$ref — a Path Item reached by $ref alone gives its operations (fields beside the $ref are dropped: conformance/triage-2.md F3)
// req: oas-3.1.path-item-object.delete — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.fixed-fields.may-object-extended-specification-extensions — x- fields of a path item are ignored
// req: oas-3.1.path-item-object.get — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.head — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.may-path-item-empty-due-acl-constraints — /empty gives no operation
// req: oas-3.1.path-item-object.options — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.parameters — path item parameters apply to each operation; the operation's own win
// req: oas-3.1.path-item-object.patch — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.post — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.put — each method of a path item is an operation in Paths
// req: oas-3.1.path-item-object.trace — each method of a path item is an operation in Paths
// req: oas-3.1.paths-object.pattern-path — each path keys Paths
// req: oas-3.1.paths-object.patterned-fields.may-object-extended-specification-extensions — an x- key of paths is not a path
// req: oas-3.1.reference-object.fixed-fields.shall-object-cannot-extended-additional-properties-any — 404: x-extra beside $ref is ignored
// req: oas-3.1.request-body-object.content — the body by media type
// req: oas-3.1.request-body-object.fixed-fields.may-object-extended-specification-extensions — x-b is ignored
// req: oas-3.1.request-body-object.required — required: true makes requestBody required, else optional
// req: oas-3.1.response-object.content — the response by media type; none is never
// req: oas-3.1.response-object.fixed-fields.may-object-extended-specification-extensions — x-resp/x-r in a response is ignored
// req: oas-3.1.responses-object.default — default is a member like a status code
// req: oas-3.1.responses-object.may-default-used-default-response-object-all — default is a member like a status code
// req: oas-3.1.responses-object.pattern-http-status-code — each status code is a member
// req: oas-3.1.responses-object.pattern-http-status-code.may-define-range-response-codes-field-contain — a range (4XX, 5XX) is a member
// req: oas-3.1.responses-object.patterned-fields.may-object-extended-specification-extensions — x-r in responses is not a status
// req: oas-3.1.schema-object.description — a schema's description is its doc comment
// req: oas-3.1.schema-object.discriminator — Pet: the oneOf union; the discriminator is an annotation
// req: oas-3.1.schema-object.fixed-fields.may-object-extended-specification-extensions-though-noted — Cat: x-internal changes nothing
// req: oas-3.1.specification-extensions.pattern-x — x- fields throughout change no type
// req: oas-3.1.versions.shall-major-minor-portion-version-string-example — 3.1.2 here and 3.1.0 in operations.json read alike
// req: oas-3.1.versions.should-not-patch-version-considered-tooling-making-no — 3.1.2 here and 3.1.0 in operations.json read alike
// req: oas-3.1.versions.should-tooling-supports-oas-3-1-compatible — 3.1.2 here and 3.1.0 in operations.json read alike
//
// oas32.json
// req: oas-3.2.components-object.fixed-fields.may-object-extended-specification-extensions — x-c/x-comp in components is ignored
// req: oas-3.2.components-object.parameters — a parameter reached by a Reference Object to components/parameters
// req: oas-3.2.components-object.pathItems — /pets: a Path Item from components/pathItems
// req: oas-3.2.components-object.requestBodies — SearchItems body and GetItem 4XX reached by Reference Objects
// req: oas-3.2.components-object.responses — SearchItems body and GetItem 4XX reached by Reference Objects
// req: oas-3.2.components-object.schemas — every component schema is a named type
// req: oas-3.2.discriminator-object.fixed-fields.may-object-extended-specification-extensions — Pet: x-d in the discriminator is ignored
// req: oas-3.2.discriminator-object.must-not-note-discriminator-change-validation-outcome-schema — Pet: the oneOf union; the discriminator is an annotation
// req: oas-3.2.media-type-object.complete-vs-streaming-content.must-schema-field-applied-complete-content-defined — GetItem 200 application/jsonl: schema is the whole stream as an array: Item[]
// req: oas-3.2.media-type-object.fixed-fields.may-object-extended-specification-extensions — x-m in a media type is ignored
// lossy: oas-3.2.media-type-object.itemSchema — itemSchema is not read: a media type with only itemSchema is unknown (GetItem 200 text/event-stream)
// req: oas-3.2.media-type-object.schema — each media type maps to its schema's type
// req: oas-3.2.media-type-object.sequential-media-types.must-implementations-support-mapping-sequential-media-types — GetItem 200 application/jsonl: schema is the whole stream as an array: Item[]
// lossy: oas-3.2.media-type-object.special-considerations-for-server-sent-events.must-field-value-types-handled-specified-text — itemSchema is not read: a media type with only itemSchema is unknown (GetItem 200 text/event-stream)
// lossy: oas-3.2.media-type-object.special-considerations-for-server-sent-events.must-text-event-stream-implementations-work-event — itemSchema is not read: a media type with only itemSchema is unknown (GetItem 200 text/event-stream)
// lossy: oas-3.2.media-type-object.streaming-sequential-media-types.must-unlike-schema-applied-complete-content-treated — itemSchema is not read: a media type with only itemSchema is unknown (GetItem 200 text/event-stream)
// req: oas-3.2.openapi-object.$self — SearchItems: a $ref by the $self URI resolves to Item
// req: oas-3.2.openapi-object.$self.must-implementations-support-identifying-targets-api-description — SearchItems: a $ref by the $self URI resolves to Item
// req: oas-3.2.openapi-object.components — every component schema is a named type
// req: oas-3.2.openapi-object.establishing-the-base-uri.must-relative-uri-references-resolved-using-appropriate — SearchItems: a $ref by the $self URI resolves to Item
// req: oas-3.2.openapi-object.fixed-fields.may-object-extended-specification-extensions — x-top is ignored
// req: oas-3.2.openapi-object.openapi — the version selects how the document is read
// req: oas-3.2.openapi-object.openapi.should-openapi-field-used-tooling-interpret-openapi — the version selects how the document is read
// req: oas-3.2.openapi-object.paths — each path item gives operations and Paths
// req: oas-3.2.openapi-object.relative-references-in-api-description-uris.may-unless-specified-otherwise-all-fields-uris — #/components/... read as JSON Pointers against $self
// req: oas-3.2.openapi-object.resolving-uri-fragments.should-representation-referenced-document-json-yaml-fragment — #/components/... read as JSON Pointers against $self
// req: oas-3.2.operation-object.fixed-fields.may-object-extended-specification-extensions — x-o is ignored
// req: oas-3.2.operation-object.operationId — operationId names the operation's type
// req: oas-3.2.operation-object.parameters — parameters by location, then name
// req: oas-3.2.operation-object.requestBody — requestBody by media type
// req: oas-3.2.operation-object.responses — responses by status, then media type
// req: oas-3.2.parameter-object.common-fixed-fields.may-object-extended-specification-extensions — x-param is ignored
// req: oas-3.2.parameter-object.content — a parameter with content takes its one media type's schema
// req: oas-3.2.parameter-object.in — in picks the parameter's group
// req: oas-3.2.parameter-object.name — name keys the parameter
// req: oas-3.2.parameter-object.required — required: true makes the member required
// req: oas-3.2.parameter-object.required.may-otherwise-field-included-default-value-false — without required the member is optional
// req: oas-3.2.parameter-object.required.required-parameter-location-path-field-value-true — path parameters are required
// req: oas-3.2.parameter-object.schema — a parameter's schema gives its type
// req: oas-3.2.path-item-object.$ref — a Path Item reached by $ref alone gives its operations (fields beside the $ref are dropped: conformance/triage-2.md F3)
// req: oas-3.2.path-item-object.delete — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.fixed-fields.may-object-extended-specification-extensions — x- fields of a path item are ignored
// req: oas-3.2.path-item-object.get — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.head — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.may-path-item-empty-due-acl-constraints — /empty gives no operation
// req: oas-3.2.path-item-object.options — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.parameters — path item parameters apply to each operation; the operation's own win
// req: oas-3.2.path-item-object.patch — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.post — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.put — each method of a path item is an operation in Paths
// req: oas-3.2.path-item-object.query — /items/{id} query: SearchItems
// req: oas-3.2.path-item-object.trace — each method of a path item is an operation in Paths
// req: oas-3.2.paths-object.pattern-path — each path keys Paths
// req: oas-3.2.paths-object.patterned-fields.may-object-extended-specification-extensions — an x- key of paths is not a path
// req: oas-3.2.reference-object.$ref — SearchItems body and GetItem 4XX reached by Reference Objects
// req: oas-3.2.reference-object.fixed-fields.shall-object-cannot-extended-additional-properties-any — limit: description and x-sibling beside $ref are ignored
// req: oas-3.2.request-body-object.content — the body by media type
// req: oas-3.2.request-body-object.fixed-fields.may-object-extended-specification-extensions — x-b is ignored
// req: oas-3.2.request-body-object.required — required: true makes requestBody required, else optional
// req: oas-3.2.response-object.content — the response by media type; none is never
// req: oas-3.2.response-object.fixed-fields.may-object-extended-specification-extensions — x-resp/x-r in a response is ignored
// req: oas-3.2.responses-object.default — default is a member like a status code
// req: oas-3.2.responses-object.may-default-used-default-response-object-all — default is a member like a status code
// req: oas-3.2.responses-object.pattern-http-status-code — each status code is a member
// req: oas-3.2.responses-object.pattern-http-status-code.may-define-range-response-codes-field-contain — a range (4XX, 5XX) is a member
// req: oas-3.2.responses-object.patterned-fields.may-object-extended-specification-extensions — x-r in responses is not a status
// req: oas-3.2.schema-object.description — a schema's description is its doc comment
// req: oas-3.2.schema-object.discriminator — Pet: the oneOf union; the discriminator is an annotation
// req: oas-3.2.schema-object.fixed-fields.may-object-extended-specification-extensions-though-noted — Item: x-internal changes nothing
// req: oas-3.2.schema-object.specifying-schema-dialects.must-default-not-set-oas-dialect-schema — Pair: without jsonSchemaDialect, prefixItems read as 2020-12
// req: oas-3.2.schema-object.specifying-schema-dialects.must-schema-keyword-present-any-schema-object — Old: its $schema (draft-07) reads items as a tuple
// req: oas-3.2.schema-object.specifying-schema-dialects.must-tooling-support-oas-dialect-schema-id — Pair: without jsonSchemaDialect, prefixItems read as 2020-12
// req: oas-3.2.specification-extensions.pattern-x — x- fields throughout change no type
// req: oas-3.2.versions-and-deprecation.shall-major-minor-portion-version-string-example — 3.2.1 here and 3.2.0 in oas32-self.json read alike
// req: oas-3.2.versions-and-deprecation.should-not-patch-version-considered-tooling-making-no — 3.2.1 here and 3.2.0 in oas32-self.json read alike
// req: oas-3.2.versions-and-deprecation.should-tooling-supports-oas-3-1-compatible — 3.2.1 here and 3.2.0 in oas32-self.json read alike
//
// oas31-dialect.json
// req: oas-3.1.openapi-object.jsonSchemaDialect — Old: jsonSchemaDialect draft-07 reads items as a tuple
// req: oas-3.1.paths-object.may-paths-object-empty-due-access-control — empty paths: no Paths type
// req: oas-3.1.schema-object.specifying-schema-dialects.must-schema-keyword-present-any-schema-object — New: its own $schema (2020-12) wins over jsonSchemaDialect
//
// oas32-dialect.json
// req: oas-3.2.openapi-object.jsonSchemaDialect — Old: jsonSchemaDialect draft-07 reads items as a tuple
// req: oas-3.2.paths-object.may-paths-object-empty-due-access-control — empty paths: no Paths type
//
// oas30-read-only.json
// req: oas-3.0.schema-object.readOnly — A: a required read-only id is optional in the one type
// req: oas-3.0.paths-object.may-paths-object-empty-due-access-control — empty paths: no Paths type
//
// oas32-self.json
// req: oas-3.2.openapi-object.$self — A: $self names the document; #/components refs resolve against it
// The fixes of triage-2.md §4, each held by its case:
// req: oas-3.0.reference-object.$ref — violation-f1.json: a reference that names nothing is an error, never a part dropped (the same reader serves 3.0)
// req: oas-3.1.reference-object.$ref — violation-f1.json: three dangling references, three errors naming them
// req: oas-3.2.reference-object.$ref — violation-f1.json's reader is the one 3.2 documents use; the rule id follows the version
// req: oas-3.0.parameter-object.name.shall-header-name-field-accept-content-type — violation-f2.json's reader: Accept, Content-Type and Authorization headers are ignored in every version
// req: oas-3.1.parameter-object.name.shall-header-name-field-accept-content-type — violation-f2.json (3.2 document, same reader)
// req: oas-3.2.parameter-object.name.shall-header-name-field-accept-content-type — violation-f2.json: `Authorization` and `accept` are left out
// req: oas-3.0.path-item-object.$ref — violation-f3.json's reader: fields beside a Path Item's $ref are kept
// req: oas-3.1.path-item-object.$ref — violation-f3.json: `post` beside the $ref and `get` from its target are both there
// req: oas-3.2.path-item-object.$ref — violation-f3.json's reader serves 3.2 the same way
// req: oas-3.2.path-item-object.additionalOperations — violation-f4.json: LINK is read
// req: oas-3.2.components-object.mediaTypes — violation-f5.json: a Media Type given by reference is followed
// req: oas-3.0.operation-object.requestBody.shall-other-cases-http-spec-vague-get — violation-f7.json: a body on GET is ignored in 3.0
// req: oas-3.1.response-object.headers — headers.json: a response's headers by name, required ones required, Content-Type ignored
// req: oas-3.0.response-object.headers — headers.json's reader serves 3.0 the same way
// req: oas-3.2.response-object.headers — headers.json's reader serves 3.2 the same way
// req: oas-3.1.operation-object.deprecated — headers.json: @deprecated on the operation's type
// req: oas-3.0.operation-object.deprecated — headers.json's reader serves 3.0 the same way
// req: oas-3.2.operation-object.deprecated — headers.json's reader serves 3.2 the same way
// req: oas-3.1.parameter-object.deprecated — headers.json: @deprecated on the parameter
// req: oas-3.0.parameter-object.deprecated — headers.json's reader serves 3.0 the same way
// req: oas-3.2.parameter-object.deprecated — headers.json's reader serves 3.2 the same way
// req: oas-3.1.header-object.deprecated — headers.json: @deprecated on the header X-Old
// req: oas-3.0.header-object.deprecated — headers.json's reader serves 3.0 the same way
// req: oas-3.2.header-object.deprecated — headers.json's reader serves 3.2 the same way
// swagger20-*.json: Swagger 2.0 (the ledger's oas-2.0 ids).
// req: oas-2.0.swagger-object.swagger — swagger20-petstore.json: the document read as Swagger 2.0
// req: oas-2.0.swagger-object.definitions — swagger20-petstore.json: Category, Tag, Pet, ApiResponse as named types
// req: oas-2.0.definitions-object.pattern-name — swagger20-petstore.json: each definition's name names its type
// req: oas-2.0.swagger-object.consumes — swagger20-petstore.json: AddPet: the body under each of the global consumes
// req: oas-2.0.swagger-object.produces — swagger20-petstore.json: GetPetById: the response under the global produces
// req: oas-2.0.swagger-object.paths — swagger20-petstore.json: every operation's type
// req: oas-2.0.paths-object.pattern-path — swagger20-petstore.json: /pet/{petId}: path parameters from the template
// req: oas-2.0.swagger-object.parameters — swagger20-petstore.json: FindPetsByStatus: limit from #/parameters/Limit
// req: oas-2.0.parameters-definitions-object.pattern-name — swagger20-petstore.json: #/parameters/Limit found by its name
// req: oas-2.0.swagger-object.responses — swagger20-petstore.json: #/responses/BadRequest followed
// req: oas-2.0.responses-definitions-object.pattern-name — swagger20-petstore.json: #/responses/BadRequest found by its name
// req: oas-2.0.reference-object.$ref — swagger20-petstore.json: #/definitions, #/parameters and #/responses references followed
// req: oas-2.0.path-item-object.get — swagger20-petstore.json: GetPetById
// req: oas-2.0.path-item-object.put — swagger20-petstore.json: UpdatePet
// req: oas-2.0.path-item-object.post — swagger20-petstore.json: AddPet
// req: oas-2.0.path-item-object.delete — swagger20-petstore.json: DeletePet
// req: oas-2.0.path-item-object.parameters — swagger20-petstore.json: petId from the Path Item in each of its operations
// req: oas-2.0.operation-object.operationId — swagger20-petstore.json: each type named by its operationId
// req: oas-2.0.operation-object.operationId.may-tools-libraries-use-operationid-uniquely-identify — swagger20-petstore.json: each type named by its operationId
// req: oas-2.0.operation-object.consumes — swagger20-petstore.json: UpdatePet: its own consumes over the global ones
// req: oas-2.0.operation-object.produces — swagger20-petstore.json: FindPetsByStatus: its own produces over the global ones
// req: oas-2.0.operation-object.parameters — swagger20-petstore.json: FindPetsByStatus: query parameters
// req: oas-2.0.operation-object.responses — swagger20-petstore.json: each response by status
// req: oas-2.0.operation-object.deprecated — swagger20-petstore.json: DeletePet: @deprecated
// req: oas-2.0.parameter-object.name — swagger20-petstore.json: each parameter by its name
// req: oas-2.0.parameter-object.in — swagger20-petstore.json: query, path, header and body parameters apart
// req: oas-2.0.parameter-object.required — swagger20-petstore.json: FindPetsByStatus: status required
// req: oas-2.0.parameter-object.required.may-otherwise-property-included-default-value-false — swagger20-petstore.json: FindPetsByStatus: limit, without required, optional
// req: oas-2.0.parameter-object.schema — swagger20-petstore.json: AddPet: the body's schema
// req: oas-2.0.parameter-object.type — swagger20-petstore.json: petId: integer as number
// req: oas-2.0.parameter-object.items — swagger20-petstore.json: FindPetsByStatus: status an array of its items
// req: oas-2.0.items-object.type — swagger20-petstore.json: FindPetsByStatus: status items strings
// req: oas-2.0.items-object.enum — swagger20-petstore.json: FindPetsByStatus: status items one of available, pending, sold
// req: oas-2.0.responses-object.pattern-http-status-code — swagger20-petstore.json: each response by its status
// req: oas-2.0.responses-object.default — swagger20-petstore.json: GetPetById: the default response
// req: oas-2.0.response-object.schema — swagger20-petstore.json: FindPetsByStatus: Pet[] under each produced type
// req: oas-2.0.response-object.headers — swagger20-petstore.json: FindPetsByStatus: X-Rate-Limit and X-Expires-After
// req: oas-2.0.headers-object.pattern-name — swagger20-petstore.json: X-Rate-Limit by its name
// req: oas-2.0.header-object.type — swagger20-petstore.json: X-Rate-Limit: integer as number
// req: oas-2.0.parameter-object.enum — swagger20-forms.json: UpdatePetWithForm: status one of available, sold
// req: oas-2.0.schema-object.discriminator — swagger20-schema.json: Pet: discriminator read as a property's name, the type unchanged
// req: oas-2.0.schema-object.readOnly — swagger20-schema.json: Pet: a required readOnly id optional
// req: oas-2.0.schema-object.pattern-x — swagger20-schema.json: Ext: x-internal read and kept, never an error
// lossy: oas-2.0.schema-object.readOnly.must-not-means-sent-part-response-but-sent — swagger20-schema.json: Pet: one type for requests and responses: a read-only property is optional there, not absent from requests
// req: oas-2.0.path-item-object.$ref — swagger20-fields.json: PutAlias: /alias's operations read from the Path Item its $ref names
// req: oas-2.0.path-item-object.options — swagger20-fields.json: OptionsBounds
// req: oas-2.0.path-item-object.head — swagger20-fields.json: HeadBounds
// req: oas-2.0.path-item-object.patch — swagger20-fields.json: PatchBounds
// req: oas-2.0.operation-object.consumes.may-empty-value-used-clear-global-definition — swagger20-fields.json: PatchBounds: consumes [] clears the global application/xml (the body under application/json, the default)
// req: oas-2.0.operation-object.produces.may-empty-value-used-clear-global-definition — swagger20-fields.json: PatchBounds: produces [] clears the global application/xml (the response under application/json); PutGlobal keeps the global ones
// lossy: oas-2.0.parameter-object.format — swagger20-fields.json: no TS type checks a format; the type of `type` alone (n int64 -> number, s date -> string)
// req: oas-2.0.parameter-object.default — swagger20-fields.json: X-Limit, n: a default leaves the parameter optional and its type unnarrowed (the server's value, not a literal)
// lossy: oas-2.0.parameter-object.maximum — swagger20-fields.json: TypeScript's number has no bound (n stays number); read: an enum is narrowed by it (nAt 1 | 2)
// lossy: oas-2.0.parameter-object.exclusiveMaximum — swagger20-fields.json: TypeScript's number has no bound (n stays number); read in 2.0's boolean form: nEnum (4) drops 6
// lossy: oas-2.0.parameter-object.minimum — swagger20-fields.json: TypeScript's number has no bound (n stays number); read: an enum is narrowed by it (nAt 1 | 2)
// lossy: oas-2.0.parameter-object.exclusiveMinimum — swagger20-fields.json: TypeScript's number has no bound (n stays number); read in 2.0's boolean form: nEnum (4) drops 2
// lossy: oas-2.0.parameter-object.multipleOf — swagger20-fields.json: TypeScript's number has no step (n stays number); read: nEnum (4) drops 3 and 5
// lossy: oas-2.0.parameter-object.maxLength — swagger20-fields.json: TypeScript's string has no length (s stays string); read: sEnum drops dddd
// lossy: oas-2.0.parameter-object.minLength — swagger20-fields.json: TypeScript's string has no length (s stays string); read: sEnum drops a
// lossy: oas-2.0.parameter-object.pattern — swagger20-fields.json: flowapi evaluates no regular expression for a plain string (s stays string); an enum is narrowed by it (sEnum "bb" | "bc")
// lossy: oas-2.0.parameter-object.maxItems — swagger20-fields.json: a short bound is a tuple (a: one to three), a long one is not (big: two or more, maxItems 1000 not held)
// req: oas-2.0.parameter-object.minItems — swagger20-fields.json: a and big: tuples of at least one and two items
// lossy: oas-2.0.parameter-object.uniqueItems — swagger20-fields.json: no TS type checks uniqueness (a)
// lossy: oas-2.0.items-object.format — swagger20-fields.json: no TS type checks a format (a's items byte -> string, m's float -> number)
// req: oas-2.0.items-object.items — swagger20-fields.json: m: an array of arrays of numbers by the nested Items Object
// req: oas-2.0.items-object.default — swagger20-fields.json: a's items: a default leaves them string, no literal
// lossy: oas-2.0.items-object.maximum — swagger20-fields.json: TypeScript's number has no bound (m's items stay number); read: mEnum's items 3 | 6
// lossy: oas-2.0.items-object.exclusiveMaximum — swagger20-fields.json: TypeScript's number has no bound; read in 2.0's boolean form: mEnum's items drop 9
// lossy: oas-2.0.items-object.minimum — swagger20-fields.json: TypeScript's number has no bound; read: mEnum's items 3 | 6
// lossy: oas-2.0.items-object.exclusiveMinimum — swagger20-fields.json: TypeScript's number has no bound; read in 2.0's boolean form: mEnum's items drop 0
// lossy: oas-2.0.items-object.multipleOf — swagger20-fields.json: TypeScript's number has no step; read: mEnum's items drop 4
// lossy: oas-2.0.items-object.maxLength — swagger20-fields.json: TypeScript's string has no length (a's items stay string); read: aEnum's items drop xyzw
// lossy: oas-2.0.items-object.minLength — swagger20-fields.json: TypeScript's string has no length (a's items stay string); read: aEnum's items drop x
// lossy: oas-2.0.items-object.pattern — swagger20-fields.json: no regular expression evaluated for a plain string (a's items); an enum is narrowed by it (aEnum's items "xy")
// req: oas-2.0.items-object.maxItems — swagger20-fields.json: m's inner arrays: at most two, a tuple
// req: oas-2.0.items-object.minItems — swagger20-fields.json: m's inner arrays: at least one
// lossy: oas-2.0.items-object.uniqueItems — swagger20-fields.json: no TS type checks uniqueness (m's inner arrays)
// lossy: oas-2.0.header-object.format — swagger20-fields.json: no TS type checks a format (X-Count int32 -> number, X-Grid int64 -> number)
// req: oas-2.0.header-object.items — swagger20-fields.json: X-Grid: an array of arrays of numbers
// req: oas-2.0.header-object.default — swagger20-fields.json: X-Count, X-Name: a default leaves the header's type unnarrowed
// lossy: oas-2.0.header-object.maximum — swagger20-fields.json: TypeScript's number has no bound (X-Count stays number); read: X-Step 5 | 10
// lossy: oas-2.0.header-object.exclusiveMaximum — swagger20-fields.json: TypeScript's number has no bound; read in 2.0's boolean form: X-Step drops 15
// lossy: oas-2.0.header-object.minimum — swagger20-fields.json: TypeScript's number has no bound; read: X-Step 5 | 10
// lossy: oas-2.0.header-object.exclusiveMinimum — swagger20-fields.json: TypeScript's number has no bound; read in 2.0's boolean form: X-Step drops 0
// lossy: oas-2.0.header-object.multipleOf — swagger20-fields.json: TypeScript's number has no step; read: X-Step drops 7
// lossy: oas-2.0.header-object.maxLength — swagger20-fields.json: TypeScript's string has no length (X-Name stays string); read: X-Code drops Bcdef
// lossy: oas-2.0.header-object.minLength — swagger20-fields.json: TypeScript's string has no length (X-Name stays string); read: X-Code drops a
// lossy: oas-2.0.header-object.pattern — swagger20-fields.json: no regular expression evaluated for a plain string (X-Name); an enum is narrowed by it (X-Code drops Cc)
// req: oas-2.0.header-object.maxItems — swagger20-fields.json: X-Tags: at most four, a tuple
// req: oas-2.0.header-object.minItems — swagger20-fields.json: X-Tags: at least one
// lossy: oas-2.0.header-object.uniqueItems — swagger20-fields.json: no TS type checks uniqueness (X-Tags)
// req: oas-2.0.header-object.enum — swagger20-fields.json: X-Mode: "fast" | "slow"
// Petstore:
// `definitions` as named types, `body` under each type of `consumes`, a response's `schema` under
// each of `produces`, non-body parameters and headers as their own schemas, `#/parameters/…`
// and `#/responses/…` followed, `trace` not a method. Forms: `formData` as a closed form under
// the form types of `consumes`, the operation's fields winning over the Path Item's, `file` as
// any value, a body beside a form warned about. Schema: `x-nullable`, `discriminator` as a name,
// `$ref` overriding its siblings, 3.x-only keywords (`anyOf`, `nullable`, `writeOnly`, `const`)
// meaning nothing, a required `readOnly` property optional. Fields: the Parameter, Items and
// Header Objects' validation keywords (2.0's boolean `exclusiveMinimum` and `exclusiveMaximum`
// too) narrowing an `enum` and leaving a plain value as wide as its type, `minItems` and
// `maxItems` as tuples, `default` and `format` changing no type, nested Items Objects, `options`,
// `head` and `patch`, a Path Item behind `$ref`, and empty `consumes` and `produces` clearing the
// global ones.
#[test]
fn cases_print_as_kept() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
    let bless = std::env::var_os("FLOWAPI_BLESS").is_some();
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    let mut changed = Vec::new();
    for case in names {
        let text = std::fs::read_to_string(&case).unwrap();
        let printed = generate(&case.file_name().unwrap().to_string_lossy(), &text);
        let kept = case.with_extension("ts");
        if bless {
            std::fs::write(&kept, &printed).unwrap();
        } else if std::fs::read_to_string(&kept).ok().as_deref() != Some(printed.as_str()) {
            changed.push(format!("{}:\n{printed}", kept.display()));
        }
    }
    assert!(
        changed.is_empty(),
        "printed otherwise than kept:\n{}",
        changed.join("\n")
    );
}

// req: jsonschema-2020-12.core.security.should-implementations-guard-against-excessive-consumption-system — a 20,000-ref chain and 500 levels of nesting end in output (G1)
// req: jsonschema-2020-12.core.guarding-against-infinite-recursion.must-not-schema-run-infinite-loop-against-instance — the chain and the nesting end in output
// req: jsonschema-2020-12.core.security.must-not-validators-fall-infinite-loop — the chain and the nesting end in output
/// G1: a chain of references thousands long, and nesting as deep as the reader takes, end in
/// output, not in an overflowed stack.
#[test]
fn deep_inputs_do_not_overflow() {
    let mut defs = String::new();
    for i in 0..20_000 {
        let _ = write!(
            defs,
            "\"d{i}\":{{\"type\":\"object\",\"properties\":{{\"next\":{{\"$ref\":\"#/$defs/d{}\"}}}}}},",
            i + 1
        );
    }
    defs.push_str("\"d20000\":{\"type\":\"string\"}");
    let chain = format!("{{\"$ref\":\"#/$defs/d0\",\"$defs\":{{{defs}}}}}");
    assert!(generate("chain.json", &chain).contains("export type Root"));
    let nested = format!(
        "{}{{\"type\":\"string\"}}{}",
        "{\"items\":".repeat(500),
        "}".repeat(500)
    );
    assert!(generate("nested.json", &nested).contains("export type Root"));
}

// req: oas-2.0.swagger-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of the Swagger Object changes neither the types nor the SDK
// req: oas-2.0.paths-object.pattern-x — swagger_extensions_change_nothing: an `x-` key of the Paths Object is no path
// req: oas-2.0.path-item-object.pattern-x — swagger_extensions_change_nothing: an `x-` key of a Path Item is no operation
// req: oas-2.0.operation-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of an Operation Object changes nothing
// req: oas-2.0.parameter-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of a query, header and body parameter changes nothing
// req: oas-2.0.items-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of an Items Object, nested too, changes nothing
// req: oas-2.0.responses-object.pattern-x — swagger_extensions_change_nothing: an `x-` key of the Responses Object is no status
// req: oas-2.0.response-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of a Response Object changes nothing
// req: oas-2.0.header-object.pattern-x — swagger_extensions_change_nothing: an `x-` field of a response's Header Object changes nothing
/// A Swagger 2.0 description with an `x-` field on every object that takes one prints the same
/// types and the same SDK as without them: a reader ignores extensions (2.0, "Vendor
/// Extensions"). Their values are each kind 2.0 allows: `null`, a primitive, an array, an object.
#[test]
fn swagger_extensions_change_nothing() {
    let doc = |x: &[&str; 9]| {
        format!(
            r#"{{"swagger":"2.0","info":{{"title":"x","version":"1"}}{},
 "paths":{{{}"/p/{{id}}":{{{}"get":{{"operationId":"getP"{},
   "parameters":[
    {{"name":"id","in":"path","required":true,"type":"string"{}}},
    {{"name":"q","in":"query","type":"array","items":{{"type":"array","items":{{"type":"integer"{}}}}}}},
    {{"name":"body","in":"body","schema":{{"type":"object"}}}}],
   "responses":{{{}"200":{{"description":"ok","schema":{{"type":"string"}}{},
    "headers":{{"X-N":{{"type":"integer"{}}}}}}}}}}}}}}}}}"#,
            x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7], x[8]
        )
    };
    let with = doc(&[
        r#","x-api":{"a":[1,null]}"#,
        r#""x-paths":"not a path","#,
        r#""x-item":{"get":{"responses":{}}},"#,
        r#","x-op":true"#,
        r#","x-param":null"#,
        r#","x-items":[1,"two"]"#,
        r#""x-status":{"description":"not a status"},"#,
        r#","x-response":3.5"#,
        r#","x-header":"h""#,
    ]);
    let without = doc(&[""; 9]);
    assert_ne!(with, without);
    for sdk in [false, true] {
        let print = |text: &str| {
            let mut docs = Documents::new();
            docs.add("x.json", text).expect("JSON");
            let target = flowapi_ts::TypeScript::new("out.ts");
            let target = if sdk { target.with_sdk() } else { target };
            let (output, diags) = flowapi::generate(&docs, &target);
            let files = output.map(|o| o.files.into_iter().map(|f| f.text).collect::<String>());
            (files.expect("output"), diags.all().len())
        };
        let (printed, warnings) = print(&with);
        assert_eq!((printed.clone(), warnings), print(&without), "sdk {sdk}");
        assert!(printed.contains("GetP"), "sdk {sdk}: {printed}");
    }
}

/// G4: the same input prints the same bytes, every time.
#[test]
fn output_is_deterministic() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/schema.json"),
    )
    .unwrap();
    let first = generate("schema.json", &text);
    for _ in 0..5 {
        assert_eq!(generate("schema.json", &text), first);
    }
}
