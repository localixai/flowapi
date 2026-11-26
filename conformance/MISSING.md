# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**185 missing.**

## openapi 2.0 — `openapi/2.0.md`: 63 missing

### Specification > Schema > Swagger Object > Patterned Objects (`#swaggerExtensions`, line 129)

- `oas-2.0.swagger-object.pattern-x` (field, by `oas-2.0.swagger-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Paths Object > Patterned Fields (`#pathsExtensions`, line 261)

- `oas-2.0.paths-object.pattern-x` (field, by `oas-*.paths-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Path Item Object > Fixed Fields (`#pathItemRef`, line 313)

- `oas-2.0.path-item-object.$ref` (field, by `oas-*.path-item-object`): Allows for an external definition of this path item. The referenced structure MUST be in the format of a Path Item Object. If there are conflicts between the referenced definition and this Path Item's definition, the behavior is *undefined*.
- `oas-2.0.path-item-object.options` (field, by `oas-*.path-item-object`): A definition of a OPTIONS operation on this path.
- `oas-2.0.path-item-object.head` (field, by `oas-*.path-item-object`): A definition of a HEAD operation on this path.
- `oas-2.0.path-item-object.patch` (field, by `oas-*.path-item-object`): A definition of a PATCH operation on this path.

### Specification > Schema > Path Item Object > Patterned Fields (`#pathItemExtensions`, line 327)

- `oas-2.0.path-item-object.pattern-x` (field, by `oas-*.path-item-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Operation Object > Fixed Fields (`#operationConsumes`, line 418)

- `oas-2.0.operation-object.consumes.may-empty-value-used-clear-global-definition` (MAY, by `oas-*.operation-object`): An empty value MAY be used to clear the global definition.
- `oas-2.0.operation-object.produces.may-empty-value-used-clear-global-definition` (MAY, by `oas-*.operation-object`): An empty value MAY be used to clear the global definition.
- `oas-2.0.operation-object.schemes.must-values-list-http-https-ws-wss` (MUST, by `oas-*.operation-object`): Values MUST be from the list: `"http"`, `"https"`, `"ws"`, `"wss"`.

### Specification > Schema > Operation Object > Patterned Objects (`#operationExtensions`, line 430)

- `oas-2.0.operation-object.pattern-x` (field, by `oas-*.operation-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Parameter Object > Fixed Fields (`#parameterFormat`, line 595)

- `oas-2.0.parameter-object.format` (field, by `oas-*.parameter-object`): The extending format for the previously mentioned `type`. See Data Type Formats for further details.
- `oas-2.0.parameter-object.collectionFormat` (field, by `oas-*.parameter-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. `multi` - corresponds to multiple parameter instances instead of multiple values for a single instance `foo=bar&foo=baz`. This is valid only for parameters `in` "query" or "formData". Default value is `csv`.
- `oas-2.0.parameter-object.default` (field, by `oas-*.parameter-object`): Declares the value of the parameter that the server will use if none is provided, for example a "count" to control the number of results per page might default to 100 if not supplied by the client in the request. (Note: "default" has no meaning for required parameters.) See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-6.2. Unlike JSON Schema this value MUST conform to the defined `type` for this parameter.
- `oas-2.0.parameter-object.maximum` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.parameter-object.exclusiveMaximum` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.parameter-object.minimum` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.parameter-object.exclusiveMinimum` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.parameter-object.maxLength` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.1.
- `oas-2.0.parameter-object.minLength` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.2.
- `oas-2.0.parameter-object.pattern` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.3.
- `oas-2.0.parameter-object.maxItems` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.2.
- `oas-2.0.parameter-object.minItems` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.3.
- `oas-2.0.parameter-object.uniqueItems` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.4.
- `oas-2.0.parameter-object.multipleOf` (field, by `oas-*.parameter-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.1.

### Specification > Schema > Parameter Object > Patterned Fields (`#parameterExtensions`, line 617)

- `oas-2.0.parameter-object.pattern-x` (field, by `oas-*.parameter-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Items Object > Fixed Fields (`#itemsFormat`, line 776)

- `oas-2.0.items-object.format` (field, by `oas-2.0.items-object`): The extending format for the previously mentioned `type`. See Data Type Formats for further details.
- `oas-2.0.items-object.items` (field, by `oas-2.0.items-object`): Required if `type` is "array". Describes the type of items in the array.
- `oas-2.0.items-object.collectionFormat` (field, by `oas-2.0.items-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. Default value is `csv`.
- `oas-2.0.items-object.default` (field, by `oas-2.0.items-object`): Declares the value of the item that the server will use if none is provided. (Note: "default" has no meaning for required items.) See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-6.2. Unlike JSON Schema this value MUST conform to the defined `type` for the data type.
- `oas-2.0.items-object.maximum` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.items-object.exclusiveMaximum` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.items-object.minimum` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.items-object.exclusiveMinimum` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.items-object.maxLength` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.1.
- `oas-2.0.items-object.minLength` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.2.
- `oas-2.0.items-object.pattern` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.3.
- `oas-2.0.items-object.maxItems` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.2.
- `oas-2.0.items-object.minItems` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.3.
- `oas-2.0.items-object.uniqueItems` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.4.
- `oas-2.0.items-object.multipleOf` (field, by `oas-2.0.items-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.1.

### Specification > Schema > Items Object > Patterned Objects (`#itemsExtensions`, line 797)

- `oas-2.0.items-object.pattern-x` (field, by `oas-2.0.items-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Responses Object > Patterned Fields (`#responsesExtensions`, line 853)

- `oas-2.0.responses-object.pattern-x` (field, by `oas-*.responses-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Response Object > Patterned Objects (`#responseExtensions`, line 903)

- `oas-2.0.response-object.pattern-x` (field, by `oas-*.response-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Header Object (`#headerFormat`, line 1080)

- `oas-2.0.header-object.format` (field, by `oas-*.header-object`): The extending format for the previously mentioned `type`. See Data Type Formats for further details.
- `oas-2.0.header-object.items` (field, by `oas-*.header-object`): Required if `type` is "array". Describes the type of items in the array.
- `oas-2.0.header-object.collectionFormat` (field, by `oas-*.header-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. Default value is `csv`.
- `oas-2.0.header-object.default` (field, by `oas-*.header-object`): Declares the value of the header that the server will use if none is provided. (Note: "default" has no meaning for required headers.) See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-6.2. Unlike JSON Schema this value MUST conform to the defined `type` for the header.
- `oas-2.0.header-object.maximum` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.header-object.exclusiveMaximum` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.2.
- `oas-2.0.header-object.minimum` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.header-object.exclusiveMinimum` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.3.
- `oas-2.0.header-object.maxLength` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.1.
- `oas-2.0.header-object.minLength` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.2.
- `oas-2.0.header-object.pattern` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.2.3.
- `oas-2.0.header-object.maxItems` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.2.
- `oas-2.0.header-object.minItems` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.3.
- `oas-2.0.header-object.uniqueItems` (field, by `oas-*.header-object`): https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.3.4.
- `oas-2.0.header-object.enum` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.5.1.
- `oas-2.0.header-object.multipleOf` (field, by `oas-*.header-object`): See https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.1.1.

### Specification > Schema > Header Object > Patterned Objects (`#headerExtensions`, line 1101)

- `oas-2.0.header-object.pattern-x` (field, by `oas-*.header-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Schema Object > Fixed Fields (`#schemaDiscriminator`, line 1234)

- `oas-2.0.schema-object.discriminator.must-used-value-name-schema-any-schema` (MUST, by `oas-*.schema-object`): When used, the value MUST be the name of this schema or any schema that inherits it.
- `oas-2.0.schema-object.readOnly.must-not-means-sent-part-response-but-sent` (MUST, by `oas-*.schema-object`): This means that it MAY be sent as part of a response but MUST NOT be sent as part of the request.

## openapi 3.0.4 — `openapi/3.0.4.md`: 28 missing

### Specification > Schema > Server Object > Fixed Fields (`#server-url`, line 388)

- `oas-3.0.server-object.url` (field, by `oas-*.server-object`): REQUIRED. A URL to the target host. This URL supports Server Variables and MAY be relative, to indicate that the host location is relative to the location where the document containing the Server Object is being served. Variable substitutions will be made when a variable is named in `{`braces`}`.
- `oas-3.0.server-object.url.may-url-supports-server-variables-relative-indicate` (MAY, by `oas-*.server-object`): This URL supports Server Variables and MAY be relative, to indicate that the host location is relative to the location where the document containing the Server Object is being served.
- `oas-3.0.server-object.url.required` (MUST, by `oas-*.server-object`): REQUIRED.
- `oas-3.0.server-object.description` (field, by `oas-*.server-object`): An optional string describing the host designated by the URL. CommonMark syntax MAY be used for rich text representation.
- `oas-3.0.server-object.variables` (field, by `oas-*.server-object`): A map between a variable name and its value. The value is used for substitution in the server's URL template.
- `oas-3.0.server-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Server Variable Object > Fixed Fields (`#server-variable-enum`, line 494)

- `oas-3.0.server-variable-object.enum` (field, by `oas-*.server-variable-object`): An enumeration of string values to be used if the substitution options are from a limited set. The array SHOULD NOT be empty.
- `oas-3.0.server-variable-object.enum.should-not-array-empty` (SHOULD, by `oas-*.server-variable-object`): The array SHOULD NOT be empty.
- `oas-3.0.server-variable-object.default.required` (MUST, by `oas-*.server-variable-object`): REQUIRED.
- `oas-3.0.server-variable-object.default.should-enum-defined-value-exist-enum-s` (SHOULD, by `oas-*.server-variable-object`): If the `enum` is defined, the value SHOULD exist in the enum's values.
- `oas-3.0.server-variable-object.description` (field, by `oas-*.server-variable-object`): An optional description for the server variable. CommonMark syntax MAY be used for rich text representation.
- `oas-3.0.server-variable-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-variable-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Path Item Object > Fixed Fields (`#path-item-servers`, line 806)

- `oas-3.0.path-item-object.servers` (field, by `oas-*.path-item-object.servers`): An alternative `servers` array to service all operations in this path. If a `servers` array is specified at the OpenAPI Object level, it will be overridden by this value.

### Specification > Schema > Operation Object > Fixed Fields (`#operation-servers`, line 914)

- `oas-3.0.operation-object.servers` (field, by `oas-*.operation-object.servers`): An alternative `servers` array to service this operation. If a `servers` array is specified at the Path Item Object or OpenAPI Object level, it will be overridden by this value.

### Specification > Schema > Media Type Object > Fixed Fields (`#media-type-encoding`, line 1435)

- `oas-3.0.media-type-object.encoding.shall-encoding-field-only-apply-request-body` (MUST, by `oas-*.media-type-object.encoding`): The `encoding` field SHALL only apply to Request Body Objects, and only when the media type is `multipart` or `application/x-www-form-urlencoded`.

### Specification > Schema > Encoding Object > Fixed Fields > Common Fixed Fields (`#encoding-headers`, line 1572)

- `oas-3.0.encoding-object.headers.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not a `multipart`.
- `oas-3.0.encoding-object.common-fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.encoding-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Encoding Object > Fixed Fields > Fixed Fields for RFC6570-style Serialization (`#encoding-style`, line 1594)

- `oas-3.0.encoding-object.style` (field, by `oas-*.encoding-object`): Describes how a specific property value will be serialized depending on its type. See Parameter Object for details on the `style` field. The behavior follows the same values as `query` parameters, including default values. Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually). This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.style.must-note-initial-used-query-strings-not` (MUST, by `oas-*.encoding-object`): Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually).
- `oas-3.0.encoding-object.style.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.explode` (field, by `oas-*.encoding-object`): When this is true, property values of type `array` or `object` generate separate parameters for each value of the array, or key-value-pair of the map. For other types of properties this field has no effect. When `style` is `"form"`, the default value is `true`. For all other styles, the default value is `false`. Note that despite `false` being the default for `deepObject`, the combination of `false` with `deepObject` is undefined. This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.explode.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.allowReserved` (field, by `oas-*.encoding-object`): When this is true, parameter values are serialized using reserved expansion, as defined by RFC6570, which allows RFC3986's reserved character set, as well as percent-encoded triples, to pass through unchanged, while still percent-encoding all other disallowed characters (including `%` outside of percent-encoded triples). Applications are still responsible for percent-encoding reserved characters that are not allowed in the query string (`[`, `]`, `#`), or have a special meaning in `application/x-www-form-urlencoded` (`-`, `&`, `+`); see Appendices C and E for details. The default value is `false`. This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.allowReserved.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded`.
- `oas-3.0.encoding-object.fixed-fields-for-rfc6570-style-serialization.recommended-match-intent-fields-compatible-version-3` (SHOULD, by `oas-*.encoding-object`): To match the intent of these fields and be compatible with version 3.1 of this specification, it is RECOMMENDED that whenever any of `style`, `explode`, or `allowReserved` are present with an explicit value:
- `oas-3.0.encoding-object.fixed-fields-for-rfc6570-style-serialization.recommended-however-all-three-style-explode-allowreserved` (SHOULD, by `oas-*.encoding-object`): However, if all three of `style`, `explode`, and `allowReserved` fields are absent, it is RECOMMENDED that:

### Appendix E: Percent-Encoding and Form Media Types > Generating and Validating URIs and `form-urlencoded` Strings > Interoperability with Web Browser Environments (`#interoperability-with-web-browser-environments`, line 4240)

- `oas-3.0.interoperability-with-web-browser-environments.should-implementations-needing-maximum-compatibility-web-browsers` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): Implementations needing maximum compatibility with web browsers SHOULD use WHATWG's `form-urlencoded` percent-encoding rules.
- `oas-3.0.interoperability-with-web-browser-environments.should-not-however-they-rely-whatwg-s-less` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): However, they SHOULD NOT rely on WHATWG's less stringent generic query string rules, as the resulting URLs would fail RFC3986 validation, including JSON Schema's `format: uri` and `format: uri-reference`.

## openapi 3.1.2 — `openapi/3.1.2.md`: 23 missing

### Specification > Relative References in API URLs (`#relative-references-in-api-urls`, line 305)

- `oas-3.1.relative-references-in-api-urls.may-note-themselves-relative-referring-document` (MAY, by `oas-3.1.relative-references-in-api-urls`): Note that these themselves MAY be relative to the referring document.
- `oas-3.1.relative-references-in-api-urls.may-unless-specified-otherwise-all-fields-urls` (MAY, by `oas-3.1.relative-references-in-api-urls`): Unless specified otherwise, all fields that are URLs MAY be relative references as defined by RFC3986.

### Specification > Schema > Server Object > Fixed Fields (`#fixed-fields-3`, line 463)

- `oas-3.1.server-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Server Variable Object > Fixed Fields (`#server-variable-default`, line 566)

- `oas-3.1.server-variable-object.default.shall-default-value-use-substitution-sent-alternate` (MUST, by `oas-3.1.server-variable-object.default.shall-default-value-use-substitution-sent-alternate`): The default value to use for substitution, which SHALL be sent if an alternate value is _not_ supplied.
- `oas-3.1.server-variable-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-variable-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Media Type Object > Fixed Fields (`#media-type-encoding`, line 1551)

- `oas-3.1.media-type-object.encoding.shall-encoding-field-only-apply-request-body` (MUST, by `oas-*.media-type-object.encoding`): The `encoding` field SHALL only apply to Request Body Objects, and only when the media type is `multipart` or `application/x-www-form-urlencoded`.

### Specification > Schema > Encoding Object > Fixed Fields > Common Fixed Fields (`#encoding-headers`, line 1695)

- `oas-3.1.encoding-object.headers.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not a `multipart`.
- `oas-3.1.encoding-object.common-fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.encoding-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Encoding Object > Fixed Fields > Fixed Fields for RFC6570-style Serialization (`#encoding-style`, line 1718)

- `oas-3.1.encoding-object.style` (field, by `oas-*.encoding-object`): Describes how a specific property value will be serialized depending on its type. See Parameter Object for details on the `style` field. The behavior follows the same values as `query` parameters, including the default value of `"form"` which applies only when `contentType` is _not_ being used due to one or both of `explode` or `allowReserved` being explicitly specified. Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually). This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.style.must-note-initial-used-query-strings-not` (MUST, by `oas-*.encoding-object`): Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually).
- `oas-3.1.encoding-object.style.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.1.encoding-object.style.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.explode` (field, by `oas-*.encoding-object`): When this is true, property values of type `array` or `object` generate separate parameters for each value of the array, or key-value-pair of the map. For other types of properties this field has no effect. When `style` is `"form"`, the default value is `true`. For all other styles, the default value is `false`. Note that despite `false` being the default for `deepObject`, the combination of `false` with `deepObject` is undefined. This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.explode.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.1.encoding-object.explode.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.allowReserved` (field, by `oas-*.encoding-object`): When this is true, parameter values are serialized using reserved expansion, as defined by RFC6570, which allows RFC3986's reserved character set, as well as percent-encoded triples, to pass through unchanged, while still percent-encoding all other disallowed characters (including `%` outside of percent-encoded triples). Applications are still responsible for percent-encoding reserved characters that are not allowed in the query string (`[`, `]`, `#`), or have a special meaning in `application/x-www-form-urlencoded` (`-`, `&`, `+`); see URL Percent-Encoding for details. The default value is `false`. This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.allowReserved.shall-field-ignored-request-body-media-type` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the request body media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.1.encoding-object.allowReserved.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.1.encoding-object.fixed-fields-for-rfc6570-style-serialization.must-not-using-rfc6570-style-serialization-multipart-form` (MUST, by `oas-*.encoding-object`): When using RFC6570-style serialization for `multipart/form-data`, URI percent-encoding MUST NOT be applied, and the value of `allowReserved` has no effect.

### Specification > Schema > Encoding Object > Encoding `multipart` Media Types (`#encoding-multipart-media-types`, line 1834)

- `oas-3.1.encoding-object.encoding-multipart-media-types.shall-note-stated-working-binary-data-encoding` (MUST, by `oas-*.encoding-object`): Note that as stated in Working with Binary Data, if the Encoding Object's `contentType`, whether set explicitly or implicitly through its default value rules, disagrees with the `contentMediaType` in a Schema Object, the `contentMediaType` SHALL be ignored.

### Specification > Schema > Header Object > Fixed Fields > Fixed Fields for use with `schema` (`#fixed-fields-for-use-with-schema-0`, line 2615)

- `oas-3.1.header-object.fixed-fields-for-use-with-schema.must-implementations-pass-header-values-through-unchanged` (MUST, by `oas-*.header-object.fixed-fields-for-use-with-schema.must-implementations-pass-header-values-through-unchanged`): Implementations MUST pass header values through unchanged rather than attempting to automatically quote header values, as the quoting rules vary too widely among different headers; see Appendix D for guidance on quoting and escaping.

### Appendix E: Percent-Encoding and Form Media Types > Generating and Validating URIs and `form-urlencoded` Strings > Interoperability with Web Browser Environments (`#interoperability-with-web-browser-environments`, line 4664)

- `oas-3.1.interoperability-with-web-browser-environments.should-implementations-needing-maximum-compatibility-web-browsers` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): Implementations needing maximum compatibility with web browsers SHOULD use WHATWG's `form-urlencoded` percent-encoding rules.
- `oas-3.1.interoperability-with-web-browser-environments.should-not-however-they-rely-whatwg-s-less` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): However, they SHOULD NOT rely on WHATWG's less stringent generic query string rules, as the resulting URLs would fail RFC3986 validation, including JSON Schema's `format: uri` and `format: uri-reference` (when `format` validation is endabled).

## openapi 3.2.1 — `openapi/3.2.1.md`: 51 missing

### Objects and Fields > Server Object > Fixed Fields (`#server-url`, line 270)

- `oas-3.2.server-object.url` (field, by `oas-*.server-object`): REQUIRED. A URL to the target host. This URL supports Server Variables and MAY be relative, to indicate that the host location is relative to the location where the document containing the Server Object is being served. Query and fragment MUST NOT be part of this URL. Variable substitutions will be made when a variable is named in `{`braces`}`.
- `oas-3.2.server-object.url.may-url-supports-server-variables-relative-indicate` (MAY, by `oas-*.server-object`): This URL supports Server Variables and MAY be relative, to indicate that the host location is relative to the location where the document containing the Server Object is being served.
- `oas-3.2.server-object.url.must-not-query-fragment-part-url` (MUST, by `oas-*.server-object.url.must-not-query-fragment-part-url`): Query and fragment MUST NOT be part of this URL.
- `oas-3.2.server-object.url.required` (MUST, by `oas-*.server-object`): REQUIRED.
- `oas-3.2.server-object.variables` (field, by `oas-*.server-object`): A map between a variable name and its value. The value is used for substitution in the server's URL template.
- `oas-3.2.server-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > Server Object > Relative References in API URLs (`#relative-references-in-api-urls`, line 281)

- `oas-3.2.server-object.relative-references-in-api-urls.may-unless-specified-otherwise-all-fields-urls` (MAY, by `oas-*.server-object`): Unless specified otherwise, all fields that are URLs MAY be relative references as defined by RFC3986.
- `oas-3.2.server-object.relative-references-in-api-urls.may-note-themselves-relative-referring-document` (MAY, by `oas-*.server-object`): Note that these themselves MAY be relative to the referring document.

### Objects and Fields > Server Variable Object > Fixed Fields (`#server-variable-enum`, line 389)

- `oas-3.2.server-variable-object.enum` (field, by `oas-*.server-variable-object`): An enumeration of string values to be used if the substitution options are from a limited set. The array MUST NOT be empty.
- `oas-3.2.server-variable-object.enum.must-not-array-empty` (MUST, by `oas-*.server-variable-object`): The array MUST NOT be empty.
- `oas-3.2.server-variable-object.default` (field, by `oas-*.server-variable-object`): REQUIRED. The default value to use for substitution, which SHALL be sent if an alternate value is _not_ supplied. If the `enum` is defined, the value MUST exist in the enum's values. Note that this behavior is different from the Schema Object's `default` keyword, which documents the receiver's behavior rather than inserting the value into the data.
- `oas-3.2.server-variable-object.default.must-enum-defined-value-exist-enum-s` (MUST, by `oas-*.server-variable-object`): If the `enum` is defined, the value MUST exist in the enum's values.
- `oas-3.2.server-variable-object.default.required` (MUST, by `oas-*.server-variable-object`): REQUIRED.
- `oas-3.2.server-variable-object.default.shall-default-value-use-substitution-sent-alternate` (MUST, by `oas-*.server-variable-object`): The default value to use for substitution, which SHALL be sent if an alternate value is _not_ supplied.
- `oas-3.2.server-variable-object.description` (field, by `oas-*.server-variable-object`): An optional description for the server variable. CommonMark syntax MAY be used for rich text representation.
- `oas-3.2.server-variable-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-variable-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > Operation Object > Fixed Fields (`#operation-servers`, line 690)

- `oas-3.2.operation-object.servers` (field, by `oas-*.operation-object.servers`): An alternative `servers` array to service this operation. If a `servers` array is specified at the Path Item Object or OpenAPI Object level, it will be overridden by this value.

### Objects and Fields > Media Type Object > Fixed Fields (`#media-type-encoding`, line 1274)

- `oas-3.2.media-type-object.encoding.shall-encoding-field-only-apply-media-type` (MUST, by `oas-*.media-type-object.encoding`): The `encoding` field SHALL only apply when the media type is `multipart` or `application/x-www-form-urlencoded`.
- `oas-3.2.media-type-object.prefixEncoding` (field, by `oas-*.media-type-object.prefixEncoding`): An array of positional encoding information, as defined under Encoding By Position. The `prefixEncoding` field SHALL only apply when the media type is `multipart`. If no Encoding Object is provided for an item, the behavior is determined by the default values documented for the Encoding Object. This field MUST NOT be present if `encoding` is present.
- `oas-3.2.media-type-object.prefixEncoding.shall-prefixencoding-field-only-apply-media-type` (MUST, by `oas-*.media-type-object.prefixEncoding`): The `prefixEncoding` field SHALL only apply when the media type is `multipart`.
- `oas-3.2.media-type-object.itemEncoding` (field, by `oas-*.media-type-object.itemEncoding`): A single Encoding Object that provides encoding information for multiple array items, as defined under Encoding By Position. The `itemEncoding` field SHALL only apply when the media type is `multipart`. If no Encoding Object is provided for an item, the behavior is determined by the default values documented for the Encoding Object. This field MUST NOT be present if `encoding` is present.
- `oas-3.2.media-type-object.itemEncoding.shall-itemencoding-field-only-apply-media-type` (MUST, by `oas-*.media-type-object.itemEncoding`): The `itemEncoding` field SHALL only apply when the media type is `multipart`.

### Objects and Fields > Media Type Object > Encoding Usage and Restrictions (`#encoding-usage-and-restrictions`, line 1370)

- `oas-3.2.media-type-object.encoding-usage-and-restrictions.shall-each-field-own-set-media-types` (MUST, by `oas-*.media-type-object.encoding-usage-and-restrictions`): Each field has its own set of media types with which it can be used; for all other media types all three fields SHALL be ignored.

### Objects and Fields > Media Type Object > Encoding Usage and Restrictions > Encoding By Name (`#encoding-by-name`, line 1377)

- `oas-3.2.media-type-object.encoding-by-name.must-all-other-value-types-both-top` (MUST, by `oas-*.media-type-object.encoding-by-name`): For all other value types for both top-level non-array property values and for other values, including items of the array type within a top-level array, the Encoding Object MUST be applied to the entire value.
- `oas-3.2.media-type-object.encoding-by-name.must-application-x-www-form-urlencoded-encoding` (MUST, by `oas-*.media-type-object.encoding-by-name`): For `application/x-www-form-urlencoded`, the `encoding` keys MUST map to parameter names, with the serialized values produced according to the rules of the Encoding Object.
- `oas-3.2.media-type-object.encoding-by-name.must-multipart-types-decode-object-multipart-form` (MUST, by `oas-*.media-type-object.encoding-by-name`): For `multipart` types that decode to an object, such as `multipart/form-data`, the `encoding` keys MUST map to the `name` parameter of the `Content-Disposition: form-data` header of each part, as is defined for `multipart/form-data` in [RFC7578].

### Objects and Fields > Media Type Object > Encoding Usage and Restrictions > Encoding By Position (`#encoding-by-position`, line 1401)

- `oas-3.2.media-type-object.encoding-by-position.shall-prefixitems-not-error-instance-array-shorter` (MUST, by `oas-*.media-type-object.encoding-by-position`): As with `prefixItems`, it is _not_ an error if the instance array is shorter than the `prefixEncoding` array; the additional Encoding Objects SHALL be ignored.

### Objects and Fields > Encoding Object > Fixed Fields > Common Fixed Fields (`#encoding-headers`, line 1724)

- `oas-3.2.encoding-object.headers.shall-field-ignored-media-type-not-multipart` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the media type is not a `multipart`.
- `oas-3.2.encoding-object.encoding` (field, by `oas-*.encoding-object`): Applies nested Encoding Objects in the same manner as the Media Type Object's `encoding` field.
- `oas-3.2.encoding-object.prefixEncoding` (field, by `oas-*.encoding-object`): Applies nested Encoding Objects in the same manner as the Media Type Object's `prefixEncoding` field.
- `oas-3.2.encoding-object.itemEncoding` (field, by `oas-*.encoding-object`): Applies nested Encoding Objects in the same manner as the Media Type Object's `itemEncoding` field.
- `oas-3.2.encoding-object.common-fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.encoding-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > Encoding Object > Fixed Fields > Fixed Fields for RFC6570-style Serialization (`#encoding-style`, line 1761)

- `oas-3.2.encoding-object.style` (field, by `oas-*.encoding-object`): Describes how a specific value will be serialized depending on its type. See Parameter Object for details on the `style` field. The behavior follows the same values as `query` parameters, including the default value of `"form"` which applies only when `contentType` is _not_ being used due to one or both of `explode` or `allowReserved` being explicitly specified. Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually). This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.style.must-note-initial-used-query-strings-not` (MUST, by `oas-*.encoding-object`): Note that the initial `?` used in query strings is not used in `application/x-www-form-urlencoded` message bodies, and MUST be removed (if using an RFC6570 implementation) or simply not added (if constructing the string manually).
- `oas-3.2.encoding-object.style.shall-field-ignored-media-type-not-application` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.2.encoding-object.style.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.explode` (field, by `oas-*.encoding-object`): When this is true, values of type `array` or `object` generate separate parameters for each value of the array, or key-value-pair of the map. For other types of properties, or when `style` is `"deepObject"`, this field has no effect. When `style` is `"form"`, the default value is `true`. For all other styles, the default value is `false`. This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.explode.shall-field-ignored-media-type-not-application` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.2.encoding-object.explode.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.allowReserved` (field, by `oas-*.encoding-object`): When this is true, values are serialized using reserved expansion, as defined by RFC6570, which allows RFC3986's reserved character set, as well as percent-encoded triples, to pass through unchanged, while still percent-encoding all other disallowed characters (including `%` outside of percent-encoded triples). Applications are still responsible for percent-encoding reserved characters that are not allowed in the target media type; see URL Percent-Encoding for details. The default value is `false`. This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`. If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.allowReserved.shall-field-ignored-media-type-not-application` (MUST, by `oas-*.encoding-object`): This field SHALL be ignored if the media type is not `application/x-www-form-urlencoded` or `multipart/form-data`.
- `oas-3.2.encoding-object.allowReserved.shall-value-explicitly-defined-value-contenttype-implicit` (MUST, by `oas-*.encoding-object`): If a value is explicitly defined, then the value of `contentType` (implicit or explicit) SHALL be ignored.
- `oas-3.2.encoding-object.fixed-fields-for-rfc6570-style-serialization.must-not-using-rfc6570-style-serialization-multipart-form` (MUST, by `oas-*.encoding-object`): When using RFC6570-style serialization for `multipart/form-data`, URI percent-encoding MUST NOT be applied, and the value of `allowReserved` has no effect.

### Objects and Fields > Encoding Object > Nested Encoding (`#nested-encoding`, line 1773)

- `oas-3.2.encoding-object.nested-encoding.must-implementations-support-one-level-nesting-support` (MUST, by `oas-*.encoding-object`): Implementations MUST support one level of nesting, and MAY support additional levels.

### Objects and Fields > Encoding Object > Encoding `multipart` Media Types > Handling Multiple `contentType` Values (`#handling-multiple-contenttype-values`, line 1873)

- `oas-3.2.encoding-object.handling-multiple-contenttype-values.must-encoding-serialization-implementations-provide-mechanism-applications` (MUST, by `oas-*.encoding-object`): For encoding and serialization, implementations MUST provide a mechanism for applications to indicate which media type is intended.
- `oas-3.2.encoding-object.handling-multiple-contenttype-values.must-not-implementations-choose-offer-media-type-sniffing` (MUST, by `oas-*.encoding-object`): Implementations MAY choose to offer media type sniffing ([SNIFF]) as an alternative, but this MUST NOT be the default behavior due to the security risks inherent in the process.

### Objects and Fields > Encoding Object > Encoding `multipart` Media Types > `Content-Transfer-Encoding` and `contentEncoding` (`#content-transfer-encoding-and-contentencoding`, line 1881)

- `oas-3.2.encoding-object.content-transfer-encoding-and-contentencoding.shall-note-stated-working-binary-data-encoding` (MUST, by `oas-*.encoding-object`): Note that as stated in Working with Binary Data, if the Encoding Object's `contentType`, whether set explicitly or implicitly through its default value rules, disagrees with the `contentMediaType` in a Schema Object, the `contentMediaType` SHALL be ignored.

### Objects and Fields > Header Object > Fixed Fields > Fixed Fields for use with `schema` (`#fixed-fields-for-use-with-schema-0`, line 2713)

- `oas-3.2.header-object.fixed-fields-for-use-with-schema.must-implementations-pass-header-values-through-unchanged` (MUST, by `oas-*.header-object.fixed-fields-for-use-with-schema.must-implementations-pass-header-values-through-unchanged`): Implementations MUST pass header values through unchanged rather than attempting to automatically quote header values, as the quoting rules vary too widely among different headers; see Appendix D for guidance on quoting and escaping.

### Objects and Fields > Schema Object > Parsing and Serializing > Non-JSON Data (`#non-json-data`, line 3094)

- `oas-3.2.schema-object.non-json-data.must-inspecting-schemas-given-starting-point-schema` (MUST, by `oas-*.schema-object.non-json-data.must-inspecting-schemas-given-starting-point-schema`): When inspecting schemas, given a starting point schema, implementations MUST examine that schema and all schemas that can be reached from it by following only `$ref` and `allOf` keywords.
- `oas-3.2.schema-object.non-json-data.must-not-implementations-inspect-subschemas-possible-reference-targets` (MUST, by `oas-*.schema-object.non-json-data.must-not-implementations-inspect-subschemas-possible-reference-targets`): Implementations MAY inspect subschemas or possible reference targets of other keywords such as `oneOf` or `$dynamicRef`, but MUST NOT attempt to resolve ambiguities.
- `oas-3.2.schema-object.non-json-data.must-due-limited-requirements-searching-schemas-serializers` (MUST, by `oas-*.schema-object.non-json-data.must-due-limited-requirements-searching-schemas-serializers`): Due to these limited requirements for searching schemas, serializers that have access to validated data MUST inspect the data if possible; implementations that either do not work with runtime data (such as code generators) or cannot access validated data for some reason MUST fall back to schema inspection.

## rfc 7578 — `rfc/rfc7578.txt`: 1 missing

### Definition of multipart/form-data > Other "Content-" Header Fields (`#section-4.8`, line 345)

- `rfc7578.4.8.must-not-other-header-fields-included-ignored` (MUST, by `rfc7578`): Other header fields MUST NOT be included and MUST be ignored.

## rfc 9110 — `rfc/rfc9110.xml`: 19 missing

### Identifiers in HTTP > HTTP-Related URI Schemes > Deprecation of userinfo in http(s) URIs (`#section-4.2.4`, line 2289)

- `rfc9110.4.2.4.must-not-sender-generate-userinfo-subcomponent-delimiter-http` (MUST, by `rfc9110.4.2.4.must-not-sender-generate-userinfo-subcomponent-delimiter-http`): A sender MUST NOT generate the userinfo subcomponent (and its "@" delimiter) when an "http" or "https" URI reference is generated within a message as a target URI or field value.

### Methods > Common Method Properties > Idempotent Methods (`#section-9.2.2`, line 5681)

- `rfc9110.9.2.2.should-not-client-automatically-retry-request-non-idempotent` (SHOULD, by `rfc9110.9.2.2.should-not-client-automatically-retry-request-non-idempotent`): A client SHOULD NOT automatically retry a request with a non-idempotent method unless it has some means to know that the request semantics are actually idempotent, regardless of the method, or some means to detect that the original request was never applied.
- `rfc9110.9.2.2.should-not-client-automatically-retry-failed-automatic-retry` (SHOULD, by `rfc9110.9.2.2.should-not-client-automatically-retry-failed-automatic-retry`): A client SHOULD NOT automatically retry a failed automatic retry.

### Methods > Method Definitions > GET (`#section-9.3.1`, line 5770)

- `rfc9110.9.3.1.should-not-client-generate-content-get-request-unless` (SHOULD, by `rfc9110.9.3.1.should-not-client-generate-content-get-request-unless`): A client SHOULD NOT generate content in a GET request unless it is made directly to an origin server that has previously indicated, in or out of band, that such a request has a purpose and will be adequately supported.

### Methods > Method Definitions > HEAD (`#section-9.3.2`, line 5827)

- `rfc9110.9.3.2.should-not-client-generate-content-head-request-unless` (SHOULD, by `rfc9110.9.3.2.should-not-client-generate-content-head-request-unless`): A client SHOULD NOT generate content in a HEAD request unless it is made directly to an origin server that has previously indicated, in or out of band, that such a request has a purpose and will be adequately supported.

### Methods > Method Definitions > DELETE (`#section-9.3.5`, line 6084)

- `rfc9110.9.3.5.should-not-client-generate-content-delete-request-unless` (SHOULD, by `rfc9110.9.3.5.should-not-client-generate-content-delete-request-unless`): A client SHOULD NOT generate content in a DELETE request unless it is made directly to an origin server that has previously indicated, in or out of band, that such a request has a purpose and will be adequately supported.

### Methods > Method Definitions > OPTIONS (`#section-9.3.7`, line 6238)

- `rfc9110.9.3.7.must-client-generates-options-request-containing-content` (MUST, by `rfc9110.9.3.7.must-client-generates-options-request-containing-content`): A client that generates an OPTIONS request containing content MUST send a valid Content-Type header field describing the representation media type.

### Message Context > Response Context Fields > Location (`#section-10.2.2`, line 6728)

- `rfc9110.10.2.2.must-location-value-provided-3xx-redirection-response` (MUST, by `rfc9110.10.2.2.must-location-value-provided-3xx-redirection-response`): If the Location value provided in a 3xx (Redirection) response does not have a fragment component, a user agent MUST process the redirection as if the value inherits the fragment component of the URI reference used to generate the target URI (i.e., the redirection inherits the original reference's fragment, if any).

### Status Codes (`#section-15`, line 9386)

- `rfc9110.15.must-however-client-understand-class-any-status` (MUST, by `rfc9110.15.must-however-client-understand-class-any-status`): However, a client MUST understand the class of any status code, as indicated by the first digit, and treat an unrecognized status code as being equivalent to the x00 status code of that class.
- `rfc9110.15.should-client-receives-response-invalid-status-code` (SHOULD, by `rfc9110.15.should-client-receives-response-invalid-status-code`): A client that receives a response with an invalid status code SHOULD process the response as if it had a 5xx (Server Error) status code.

### Status Codes > Redirection 3xx (`#section-15.4`, line 9957)

- `rfc9110.15.4.may-location-header-field-section-10-2` (MAY, by `rfc9110.15.4.may-location-header-field-section-10-2`): If a Location header field (Section 10.2.2) is provided, the user agent MAY automatically redirect its request to the URI referenced by the Location field value, even if the specific status code is not understood.
- `rfc9110.15.4.should-client-detect-intervene-cyclical-redirections-i` (SHOULD, by `rfc9110.15.4.should-client-detect-intervene-cyclical-redirections-i`): A client SHOULD detect and intervene in cyclical redirections (i.e., "infinite" redirection loops).

### Status Codes > Redirection 3xx > 301 Moved Permanently (`#section-15.4.2`, line 10097)

- `rfc9110.15.4.2.may-user-agent-use-location-field-value` (MAY, by `rfc9110.15.4.2.may-user-agent-use-location-field-value`): The user agent MAY use the Location field value for automatic redirection.
- `rfc9110.15.4.2.may-note-historical-reasons-user-agent-change` (MAY, by `rfc9110.15.4.2.may-note-historical-reasons-user-agent-change`): Note: For historical reasons, a user agent MAY change the request method from POST to GET for the subsequent request.

### Status Codes > Redirection 3xx > 302 Found (`#section-15.4.3`, line 10126)

- `rfc9110.15.4.3.may-user-agent-use-location-field-value` (MAY, by `rfc9110.15.4.3.may-user-agent-use-location-field-value`): The user agent MAY use the Location field value for automatic redirection.
- `rfc9110.15.4.3.may-note-historical-reasons-user-agent-change` (MAY, by `rfc9110.15.4.3.may-note-historical-reasons-user-agent-change`): Note: For historical reasons, a user agent MAY change the request method from POST to GET for the subsequent request.

### Status Codes > Redirection 3xx > 307 Temporary Redirect (`#section-15.4.8`, line 10249)

- `rfc9110.15.4.8.must-not-307-temporary-redirect-status-code-indicates` (MUST, by `rfc9110.15.4.8.must-not-307-temporary-redirect-status-code-indicates`): The 307 (Temporary Redirect) status code indicates that the target resource resides temporarily under a different URI and the user agent MUST NOT change the request method if it performs an automatic redirection to that URI.
- `rfc9110.15.4.8.may-user-agent-use-location-field-value` (MAY, by `rfc9110.15.4.8.may-user-agent-use-location-field-value`): The user agent MAY use the Location field value for automatic redirection.

### Status Codes > Redirection 3xx > 308 Permanent Redirect (`#section-15.4.9`, line 10279)

- `rfc9110.15.4.9.may-user-agent-use-location-field-value` (MAY, by `rfc9110.15.4.9.may-user-agent-use-location-field-value`): The user agent MAY use the Location field value for automatic redirection.
