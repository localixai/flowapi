# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**282 missing.**

## openapi 2.0 — `openapi/2.0.md`: 69 missing

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

### Specification > Schema > Security Scheme Object > Fixed Fields (`#securitySchemeDescription`, line 2141)

- `oas-2.0.security-scheme-object.description` (field, by `oas-*.security-scheme-object`): A short description for security scheme.
- `oas-2.0.security-scheme-object.tokenUrl` (field, by `oas-2.0.security-scheme-object.tokenUrl`): Required. The token URL to be used for this flow. This SHOULD be in the form of a URL.
- `oas-2.0.security-scheme-object.scopes` (field, by `oas-2.0.security-scheme-object.scopes`): Required. The available scopes for the OAuth2 security scheme.

### Specification > Schema > Security Scheme Object > Patterned Fields (`#securitySchemeExtensions`, line 2153)

- `oas-2.0.security-scheme-object.x` (field, by `oas-*.security-scheme-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

### Specification > Schema > Scopes Object > Patterned Fields (`#scopesName`, line 2216)

- `oas-2.0.scopes-object.pattern-name` (field, by `oas-2.0.scopes-object`): Maps between a name of a scope to a short description of it (as the value of the property).

### Specification > Schema > Scopes Object > Patterned Objects (`#scopesExtensions`, line 2222)

- `oas-2.0.scopes-object.pattern-x` (field, by `oas-2.0.scopes-object`): Allows extensions to the Swagger Schema. The field name MUST begin with `x-`, for example, `x-internal-id`. The value can be `null`, a primitive, an array or an object. See Vendor Extensions for further details.

## openapi 3.0.4 — `openapi/3.0.4.md`: 49 missing

### Specification > Schema > OpenAPI Object > Fixed Fields (`#oas-security`, line 265)

- `oas-3.0.openapi-object.security` (field, by `oas-*.openapi-object.security`): A declaration of which security mechanisms can be used across the API. The list of values includes alternative Security Requirement Objects that can be used. Only one of the Security Requirement Objects need to be satisfied to authorize a request. Individual operations can override this definition. The list can be incomplete, up to being empty or absent. To make security explicitly optional, an empty security requirement (`{}`) can be included in the array.

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

### Specification > Schema > Security Scheme Object > Fixed Fields (`#security-scheme-description`, line 3570)

- `oas-3.0.security-scheme-object.description` (field, by `oas-*.security-scheme-object`): A description for security scheme. CommonMark syntax MAY be used for rich text representation.
- `oas-3.0.security-scheme-object.scheme` (field, by `oas-*.security-scheme-object`): REQUIRED. The name of the HTTP Authentication scheme to be used in the Authorization header as defined in RFC7235. The values used SHOULD be registered in the IANA Authentication Scheme registry. The value is case-insensitive, as defined in RFC7235.
- `oas-3.0.security-scheme-object.scheme.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.0.security-scheme-object.bearerFormat` (field, by `oas-*.security-scheme-object`): A hint to the client to identify how the bearer token is formatted. Bearer tokens are usually generated by an authorization server, so this information is primarily for documentation purposes.
- `oas-3.0.security-scheme-object.flows` (field, by `oas-*.security-scheme-object`): REQUIRED. An object containing configuration information for the flow types supported.
- `oas-3.0.security-scheme-object.flows.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.0.security-scheme-object.openIdConnectUrl` (field, by `oas-*.security-scheme-object`): REQUIRED. Well-known URL to discover the [OpenID-Connect-Discovery] provider metadata.
- `oas-3.0.security-scheme-object.openIdConnectUrl.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.0.security-scheme-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.security-scheme-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > OAuth Flows Object > Fixed Fields (`#oauth-flows-client-credentials`, line 3665)

- `oas-3.0.oauth-flows-object.clientCredentials` (field, by `oas-*.oauth-flows-object`): Configuration for the OAuth Client Credentials flow. Previously called `application` in OpenAPI 2.0.
- `oas-3.0.oauth-flows-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flows-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > OAuth Flow Object > Fixed Fields (`#oauth-flow-token-url`, line 3679)

- `oas-3.0.oauth-flow-object.tokenUrl` (field, by `oas-*.oauth-flow-object`): REQUIRED. The token URL to be used for this flow. This MUST be in the form of a URL. The OAuth2 standard requires the use of TLS.
- `oas-3.0.oauth-flow-object.tokenUrl.must-form-url` (MUST, by `oas-*.oauth-flow-object`): This MUST be in the form of a URL.
- `oas-3.0.oauth-flow-object.tokenUrl.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.0.oauth-flow-object.scopes` (field, by `oas-*.oauth-flow-object`): REQUIRED. The available scopes for the OAuth2 security scheme. A map between the scope name and a short description for it. The map MAY be empty.
- `oas-3.0.oauth-flow-object.scopes.may-map-empty` (MAY, by `oas-*.oauth-flow-object`): The map MAY be empty.
- `oas-3.0.oauth-flow-object.scopes.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.0.oauth-flow-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flow-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Security Requirement Object (`#security-requirement-object`, line 3731)

- `oas-3.0.security-requirement-object.must-security-requirement-object-refer-multiple-security` (MUST, by `oas-*.security-requirement-object`): A Security Requirement Object MAY refer to multiple security schemes in which case all schemes MUST be satisfied for a request to be authorized.

### Appendix E: Percent-Encoding and Form Media Types > Generating and Validating URIs and `form-urlencoded` Strings > Interoperability with Web Browser Environments (`#interoperability-with-web-browser-environments`, line 4240)

- `oas-3.0.interoperability-with-web-browser-environments.should-implementations-needing-maximum-compatibility-web-browsers` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): Implementations needing maximum compatibility with web browsers SHOULD use WHATWG's `form-urlencoded` percent-encoding rules.
- `oas-3.0.interoperability-with-web-browser-environments.should-not-however-they-rely-whatwg-s-less` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): However, they SHOULD NOT rely on WHATWG's less stringent generic query string rules, as the resulting URLs would fail RFC3986 validation, including JSON Schema's `format: uri` and `format: uri-reference`.

### Appendix F: Resolving Security Requirements in a Referenced Document (`#appendix-f-resolving-security-requirements-in-a-referenced-document`, line 4360)

- `oas-3.0.appendix-f-resolving-security-requirements-in-a-referenced-document.recommended-however-documented-section-tools-resolve-component` (SHOULD, by `oas-*.appendix-f-resolving-security-requirements-in-a-referenced-document`): However, documented in that section, it is RECOMMENDED that tools resolve component names from the entry document.

## openapi 3.1.2 — `openapi/3.1.2.md`: 34 missing

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

### Specification > Schema > Security Scheme Object > Fixed Fields (`#fixed-fields-23`, line 3964)

- `oas-3.1.security-scheme-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.security-scheme-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > OAuth Flows Object > Fixed Fields (`#oauth-flows-client-credentials`, line 4065)

- `oas-3.1.oauth-flows-object.clientCredentials` (field, by `oas-*.oauth-flows-object`): Configuration for the OAuth Client Credentials flow. Previously called `application` in OpenAPI 2.0.
- `oas-3.1.oauth-flows-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flows-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > OAuth Flow Object > Fixed Fields (`#oauth-flow-token-url`, line 4079)

- `oas-3.1.oauth-flow-object.tokenUrl` (field, by `oas-*.oauth-flow-object`): REQUIRED. The token URL to be used for this flow. This MUST be in the form of a URL. The OAuth2 standard requires the use of TLS.
- `oas-3.1.oauth-flow-object.tokenUrl.must-form-url` (MUST, by `oas-*.oauth-flow-object`): This MUST be in the form of a URL.
- `oas-3.1.oauth-flow-object.tokenUrl.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.1.oauth-flow-object.scopes` (field, by `oas-*.oauth-flow-object`): REQUIRED. The available scopes for the OAuth2 security scheme. A map between the scope name and a short description for it. The map MAY be empty.
- `oas-3.1.oauth-flow-object.scopes.may-map-empty` (MAY, by `oas-*.oauth-flow-object`): The map MAY be empty.
- `oas-3.1.oauth-flow-object.scopes.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.1.oauth-flow-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flow-object`): This object MAY be extended with Specification Extensions.

### Appendix E: Percent-Encoding and Form Media Types > Generating and Validating URIs and `form-urlencoded` Strings > Interoperability with Web Browser Environments (`#interoperability-with-web-browser-environments`, line 4664)

- `oas-3.1.interoperability-with-web-browser-environments.should-implementations-needing-maximum-compatibility-web-browsers` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): Implementations needing maximum compatibility with web browsers SHOULD use WHATWG's `form-urlencoded` percent-encoding rules.
- `oas-3.1.interoperability-with-web-browser-environments.should-not-however-they-rely-whatwg-s-less` (SHOULD, by `oas-*.interoperability-with-web-browser-environments`): However, they SHOULD NOT rely on WHATWG's less stringent generic query string rules, as the resulting URLs would fail RFC3986 validation, including JSON Schema's `format: uri` and `format: uri-reference` (when `format` validation is endabled).

### Appendix F: Resolving Security Requirements in a Referenced Document (`#appendix-f-resolving-security-requirements-in-a-referenced-document`, line 4786)

- `oas-3.1.appendix-f-resolving-security-requirements-in-a-referenced-document.recommended-however-documented-section-tools-resolve-component` (SHOULD, by `oas-*.appendix-f-resolving-security-requirements-in-a-referenced-document`): However, documented in that section, it is RECOMMENDED that tools resolve component names from the entry document.

## openapi 3.2.1 — `openapi/3.2.1.md`: 78 missing

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

### Objects and Fields > Components Object > Fixed Fields (`#components-security-schemes`, line 410)

- `oas-3.2.components-object.securitySchemes` (field, by `oas-*.components-object.securitySchemes`): An object to hold reusable Security Scheme Objects.

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

### Objects and Fields > Security Scheme Object > Fixed Fields (`#security-scheme-description`, line 4601)

- `oas-3.2.security-scheme-object.description` (field, by `oas-*.security-scheme-object`): A description for security scheme. CommonMark syntax MAY be used for rich text representation.
- `oas-3.2.security-scheme-object.name` (field, by `oas-*.security-scheme-object`): REQUIRED. The name of the header, query or cookie parameter to be used.
- `oas-3.2.security-scheme-object.name.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.2.security-scheme-object.in` (field, by `oas-*.security-scheme-object`): REQUIRED. The location of the API key. Valid values are `"query"`, `"header"`, or `"cookie"`.
- `oas-3.2.security-scheme-object.in.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.2.security-scheme-object.bearerFormat` (field, by `oas-*.security-scheme-object`): A hint to the client to identify how the bearer token is formatted. Bearer tokens are usually generated by an authorization server, so this information is primarily for documentation purposes.
- `oas-3.2.security-scheme-object.flows` (field, by `oas-*.security-scheme-object`): REQUIRED. An object containing configuration information for the flow types supported.
- `oas-3.2.security-scheme-object.flows.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.2.security-scheme-object.openIdConnectUrl` (field, by `oas-*.security-scheme-object`): REQUIRED. Well-known URL to discover the [OpenID-Connect-Discovery] provider metadata.
- `oas-3.2.security-scheme-object.openIdConnectUrl.required` (MUST, by `oas-*.security-scheme-object`): REQUIRED.
- `oas-3.2.security-scheme-object.deprecated.should-consumers-refrain-usage-declared-scheme` (SHOULD, by `oas-3.2.security-scheme-object.deprecated.should-consumers-refrain-usage-declared-scheme`): Consumers SHOULD refrain from usage of the declared scheme.
- `oas-3.2.security-scheme-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.security-scheme-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > OAuth Flows Object > Fixed Fields (`#oauth-flows-client-credentials`, line 4667)

- `oas-3.2.oauth-flows-object.clientCredentials` (field, by `oas-*.oauth-flows-object`): Configuration for the OAuth Client Credentials flow. Previously called `application` in OpenAPI 2.0.
- `oas-3.2.oauth-flows-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flows-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > OAuth Flow Object > Fixed Fields (`#oauth-flow-token-url`, line 4683)

- `oas-3.2.oauth-flow-object.tokenUrl` (field, by `oas-*.oauth-flow-object`): REQUIRED. The token URL to be used for this flow. This MUST be in the form of a URL. The OAuth2 standard requires the use of TLS.
- `oas-3.2.oauth-flow-object.tokenUrl.must-form-url` (MUST, by `oas-*.oauth-flow-object`): This MUST be in the form of a URL.
- `oas-3.2.oauth-flow-object.tokenUrl.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.2.oauth-flow-object.scopes` (field, by `oas-*.oauth-flow-object`): REQUIRED. The available scopes for the OAuth2 security scheme. A map between the scope name and a short description for it. The map MAY be empty.
- `oas-3.2.oauth-flow-object.scopes.may-map-empty` (MAY, by `oas-*.oauth-flow-object`): The map MAY be empty.
- `oas-3.2.oauth-flow-object.scopes.required` (MUST, by `oas-*.oauth-flow-object`): REQUIRED.
- `oas-3.2.oauth-flow-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.oauth-flow-object`): This object MAY be extended with Specification Extensions.

### Objects and Fields > Security Requirement Object (`#security-requirement-object`, line 4717)

- `oas-3.2.security-requirement-object.must-security-requirement-object-refer-multiple-security` (MUST, by `oas-*.security-requirement-object`): A Security Requirement Object MAY refer to multiple security schemes in which case all schemes MUST be satisfied for a request to be authorized.

### Objects and Fields > Security Requirement Object > Patterned Fields (`#security-requirements-name`, line 4729)

- `oas-3.2.security-requirement-object.pattern-name` (field, by `oas-*.security-requirement-object`): Each name or URI MUST correspond to a security scheme as described above. If the security scheme is of type `"oauth2"` or `"openIdConnect"`, then the value is a list of scope names required for the execution, and the list MAY be empty if authorization does not require a specified scope. For other security scheme types, the array MAY contain a list of role names which are required for the execution, but are not otherwise defined or exchanged in-band.
- `oas-3.2.security-requirement-object.pattern-name.may-other-security-scheme-types-array-contain` (MAY, by `oas-*.security-requirement-object`): For other security scheme types, the array MAY contain a list of role names which are required for the execution, but are not otherwise defined or exchanged in-band.
- `oas-3.2.security-requirement-object.pattern-name.may-security-scheme-type-oauth2-openidconnect-value` (MAY, by `oas-*.security-requirement-object`): If the security scheme is of type `"oauth2"` or `"openIdConnect"`, then the value is a list of scope names required for the execution, and the list MAY be empty if authorization does not require a specified scope.

### Security Considerations > Security Schemes (`#security-schemes`, line 4803)

- `oas-3.2.security-schemes.recommended-implementation-defined-whether-component-name-used` (SHOULD, by `oas-3.2.security-schemes`): It is implementation-defined whether a component name used by a Security Requirement Object in a referenced document is resolved from the entry document (RECOMMENDED) or the referenced document.

## rfc 6749 — `rfc/rfc6749.txt`: 19 missing

### Client Registration > Client Authentication (`#section-2.3`, line 864)

- `rfc6749.2.3.must-not-client-use-more-one-authentication-method` (MUST, by `rfc6749.2.3.must-not-client-use-more-one-authentication-method`): The client MUST NOT use more than one authentication method in each request.

### Client Registration > Client Authentication > Client Password (`#section-2.3.1`, line 869)

- `rfc6749.2.3.1.may-clients-possession-client-password-use-http` (MAY, by `rfc6749.2.3.1.may-clients-possession-client-password-use-http`): Clients in possession of a client password MAY use the HTTP Basic authentication scheme as defined in [RFC2617] to authenticate with the authorization server.
- `rfc6749.2.3.1.must-not-parameters-only-transmitted-request-body-included` (MUST, by `rfc6749.2.3.1.must-not-parameters-only-transmitted-request-body-included`): The parameters can only be transmitted in the request-body and MUST NOT be included in the request URI.
- `rfc6749.2.3.1.not-recommended-including-client-credentials-request-body-using` (SHOULD, by `rfc6749.2.3.1.not-recommended-including-client-credentials-request-body-using`): Including the client credentials in the request-body using the two parameters is NOT RECOMMENDED and SHOULD be limited to clients unable to directly utilize the HTTP Basic authentication scheme (or other password-based HTTP authentication schemes).

### Protocol Endpoints > Token Endpoint (`#section-3.2`, line 1187)

- `rfc6749.3.2.must-endpoint-uri-include-application-x-www` (MUST, by `rfc6749.3.2.must-endpoint-uri-include-application-x-www`): The endpoint URI MAY include an "application/x-www-form-urlencoded" formatted (per Appendix B) query component ([RFC3986] Section 3.4), which MUST be retained when adding additional query parameters.
- `rfc6749.3.2.must-client-use-http-post-method-making` (MUST, by `rfc6749.3.2.must-client-use-http-post-method-making`): The client MUST use the HTTP "POST" method when making access token requests.
- `rfc6749.3.2.must-not-request-response-parameters-included-more-once` (MUST, by `rfc6749.3.2.must-not-request-response-parameters-included-more-once`): Request and response parameters MUST NOT be included more than once.

### Protocol Endpoints > Token Endpoint > Client Authentication (`#section-3.2.1`, line 1207)

- `rfc6749.3.2.1.must-confidential-clients-other-clients-issued-client` (MUST, by `rfc6749.3.2.1.must-confidential-clients-other-clients-issued-client`): Confidential clients or other clients issued client credentials MUST authenticate with the authorization server as described in Section 2.3 when making requests to the token endpoint.

### Obtaining Authorization > Client Credentials Grant > Access Token Request (`#section-4.4.2`, line 2280)

- `rfc6749.4.4.2.must-value-set-client-credentials` (MUST, by `rfc6749.4.4.2.must-value-set-client-credentials`): Value MUST be set to "client_credentials".
- `rfc6749.4.4.2.required-grant-type` (MUST, by `rfc6749.4.4.2.required-grant-type`): grant_type REQUIRED.
- `rfc6749.4.4.2.optional-scope` (MAY, by `rfc6749.4.4.2.optional-scope`): scope OPTIONAL.
- `rfc6749.4.4.2.must-client-authenticate-authorization-server-described-section` (MUST, by `rfc6749.4.4.2.must-client-authenticate-authorization-server-described-section`): The client MUST authenticate with the authorization server as described in Section 3.2.1.

### Issuing an Access Token > Successful Response (`#section-5.1`, line 2454)

- `rfc6749.5.1.must-client-ignore-unrecognized-value-names-response` (MUST, by `rfc6749.5.1.must-client-ignore-unrecognized-value-names-response`): The client MUST ignore unrecognized value names in the response.

### Accessing Protected Resources > Access Token Types (`#section-7.1`, line 2697)

- `rfc6749.7.1.must-not-client-use-access-token-does-not` (MUST, by `rfc6749.7.1.must-not-client-use-access-token-does-not`): The client MUST NOT use an access token if it does not understand the token type.

### Security Considerations > Access Tokens (`#section-10.3`, line 3033)

- `rfc6749.10.3.must-access-token-credentials-only-transmitted-using` (MUST, by `rfc6749.10.3.must-access-token-credentials-only-transmitted-using`): Access token credentials MUST only be transmitted using TLS as described in Section 1.6 with server authentication as defined by [RFC2818].
- `rfc6749.10.3.must-access-token-credentials-well-any-confidential` (MUST, by `rfc6749.10.3.must-access-token-credentials-well-any-confidential`): Access token credentials (as well as any confidential access token attributes) MUST be kept confidential in transit and storage, and only shared among the authorization server, the resource servers the access token is valid for, and the client to whom the access token is issued.
- `rfc6749.10.3.should-client-request-access-tokens-minimal-scope` (SHOULD, by `rfc6749.10.3.should-client-request-access-tokens-minimal-scope`): The client SHOULD request access tokens with the minimal scope necessary.

### Security Considerations > Request Confidentiality (`#section-10.8`, line 3201)

- `rfc6749.10.8.must-not-access-tokens-refresh-tokens-resource-owner` (MUST, by `rfc6749.10.8.must-not-access-tokens-refresh-tokens-resource-owner`): Access tokens, refresh tokens, resource owner passwords, and client credentials MUST NOT be transmitted in the clear.

### Use of application/x-www-form-urlencoded Media Type (`#appendix-B`, line 4118)

- `rfc6749.B.must-address-shortcoming-generating-payloads-using-media` (MUST, by `rfc6749.B.must-address-shortcoming-generating-payloads-using-media`): To address this shortcoming when generating payloads using this media type, names and values MUST be encoded using the UTF-8 character encoding scheme [RFC3629] first; the resulting octet sequence then needs to be further encoded using the escaping rules defined in [W3C.REC-html401-19991224].

## rfc 6750 — `rfc/rfc6750.txt`: 10 missing

### Authenticated Requests (`#section-2`, line 218)

- `rfc6750.2.must-not-clients-use-more-one-method-transmit` (MUST, by `rfc6750.2.must-not-clients-use-more-one-method-transmit`): Clients MUST NOT use more than one method to transmit the token in each request.

### Authenticated Requests > Form-Encoded Body Parameter (`#section-2.2`, line 262)

- `rfc6750.2.2.must-not-client-use-method-unless-all-following` (MUST, by `rfc6750.2.2.must-not-client-use-method-unless-all-following`): The client MUST NOT use this method unless all of the following conditions are met:

### Authenticated Requests > URI Query Parameter (`#section-2.3`, line 352)

- `rfc6750.2.3.should-not-because-security-weaknesses-associated-uri-method` (SHOULD, by `rfc6750.2.3.should-not-because-security-weaknesses-associated-uri-method`): Because of the security weaknesses associated with the URI method (see Section 5), including the high likelihood that the URL containing the access token will be logged, it SHOULD NOT be used unless it is impossible to transport the access token in the "Authorization" request header field or the HTTP request entity-body.

### The WWW-Authenticate Response Header Field > Error Codes (`#section-3.1`, line 476)

- `rfc6750.3.1.may-client-request-new-access-token-retry` (MAY, by `rfc6750.3.1.may-client-request-new-access-token-retry`): The client MAY request a new access token and retry the protected resource request.

### Security Considerations > Threat Mitigation (`#section-5.2`, line 631)

- `rfc6750.5.2.must-not-therefore-bearer-tokens-stored-cookies-sent` (MUST, by `rfc6750.5.2.must-not-therefore-bearer-tokens-stored-cookies-sent`): Therefore, bearer tokens MUST NOT be stored in cookies that can be sent in the clear.
- `rfc6750.5.2.must-second-confidentiality-protection-exchanges-between-client` (MUST, by `rfc6750.5.2.must-second-confidentiality-protection-exchanges-between-client`): Second, confidentiality protection of the exchanges between the client and the authorization server and between the client and the resource server MUST be applied.

### Security Considerations > Summary of Recommendations (`#section-5.3`, line 692)

- `rfc6750.5.3.must-always-use-tls-https-clients-always` (MUST, by `rfc6750.5.3.must-always-use-tls-https-clients-always`): Always use TLS (https): Clients MUST always use TLS [RFC5246] (https) or equivalent transport security when making requests with bearer tokens.
- `rfc6750.5.3.must-not-don-t-store-bearer-tokens-cookies` (MUST, by `rfc6750.5.3.must-not-don-t-store-bearer-tokens-cookies`): Don't store bearer tokens in cookies: Implementations MUST NOT store bearer tokens within cookies that can be sent in the clear (which is the default transmission mode for cookies).
- `rfc6750.5.3.should-instead-bearer-tokens-passed-http-message` (SHOULD, by `rfc6750.5.3.should-instead-bearer-tokens-passed-http-message`): Instead, bearer tokens SHOULD be passed in HTTP message headers or message bodies for which confidentiality measures are taken.
- `rfc6750.5.3.should-not-don-t-pass-bearer-tokens-page` (SHOULD, by `rfc6750.5.3.should-not-don-t-pass-bearer-tokens-page`): Don't pass bearer tokens in page URLs: Bearer tokens SHOULD NOT be passed in page URLs (for example, as query string parameters).

## rfc 7578 — `rfc/rfc7578.txt`: 1 missing

### Definition of multipart/form-data > Other "Content-" Header Fields (`#section-4.8`, line 345)

- `rfc7578.4.8.must-not-other-header-fields-included-ignored` (MUST, by `rfc7578`): Other header fields MUST NOT be included and MUST be ignored.

## rfc 7617 — `rfc/rfc7617.txt`: 2 missing

### The 'Basic' Authentication Scheme (`#section-2`, line 254)

- `rfc7617.2.must-not-user-id-password-contain-any-control` (MUST, by `rfc7617.2.must-not-user-id-password-contain-any-control`): The user-id and password MUST NOT contain any control characters (see "CTL" in Appendix B.1 of [RFC5234]).

### Security Considerations (`#section-4`, line 425)

- `rfc7617.4.should-not-because-basic-authentication-involves-cleartext-transmission` (SHOULD, by `rfc7617.4.should-not-because-basic-authentication-involves-cleartext-transmission`): Because Basic authentication involves the cleartext transmission of passwords, it SHOULD NOT be used (without enhancements such as HTTPS [RFC2818]) to protect sensitive or valuable information.

## rfc 9110 — `rfc/rfc9110.xml`: 20 missing

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

### Status Codes > Client Error 4xx > 403 Forbidden (`#section-15.5.4`, line 10361)

- `rfc9110.15.5.4.should-not-client-automatically-repeat-request-same-credentials` (SHOULD, by `rfc9110.15.5.4.should-not-client-automatically-repeat-request-same-credentials`): The client SHOULD NOT automatically repeat the request with the same credentials.
