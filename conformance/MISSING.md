# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**155 missing.**

## openapi 2.0 — `openapi/2.0.md`: 9 missing

### Specification > Schema > Operation Object > Fixed Fields (`#operationSchemes`, line 422)

- `oas-2.0.operation-object.schemes.must-values-list-http-https-ws-wss` (MUST, by `oas-*.operation-object`): Values MUST be from the list: `"http"`, `"https"`, `"ws"`, `"wss"`.

### Specification > Schema > Parameter Object > Fixed Fields (`#parameterCollectionFormat`, line 598)

- `oas-2.0.parameter-object.collectionFormat` (field, by `oas-*.parameter-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. `multi` - corresponds to multiple parameter instances instead of multiple values for a single instance `foo=bar&foo=baz`. This is valid only for parameters `in` "query" or "formData". Default value is `csv`.

### Specification > Schema > Items Object > Fixed Fields (`#itemsCollectionFormat`, line 778)

- `oas-2.0.items-object.collectionFormat` (field, by `oas-2.0.items-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. Default value is `csv`.

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

## openapi 3.0.4 — `openapi/3.0.4.md`: 35 missing

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

### Appendix F: Resolving Security Requirements in a Referenced Document (`#appendix-f-resolving-security-requirements-in-a-referenced-document`, line 4360)

- `oas-3.0.appendix-f-resolving-security-requirements-in-a-referenced-document.recommended-however-documented-section-tools-resolve-component` (SHOULD, by `oas-*.appendix-f-resolving-security-requirements-in-a-referenced-document`): However, documented in that section, it is RECOMMENDED that tools resolve component names from the entry document.

## openapi 3.1.2 — `openapi/3.1.2.md`: 16 missing

### Specification > Relative References in API URLs (`#relative-references-in-api-urls`, line 305)

- `oas-3.1.relative-references-in-api-urls.may-note-themselves-relative-referring-document` (MAY, by `oas-3.1.relative-references-in-api-urls`): Note that these themselves MAY be relative to the referring document.
- `oas-3.1.relative-references-in-api-urls.may-unless-specified-otherwise-all-fields-urls` (MAY, by `oas-3.1.relative-references-in-api-urls`): Unless specified otherwise, all fields that are URLs MAY be relative references as defined by RFC3986.

### Specification > Schema > Server Object > Fixed Fields (`#fixed-fields-3`, line 463)

- `oas-3.1.server-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-object`): This object MAY be extended with Specification Extensions.

### Specification > Schema > Server Variable Object > Fixed Fields (`#server-variable-default`, line 566)

- `oas-3.1.server-variable-object.default.shall-default-value-use-substitution-sent-alternate` (MUST, by `oas-3.1.server-variable-object.default.shall-default-value-use-substitution-sent-alternate`): The default value to use for substitution, which SHALL be sent if an alternate value is _not_ supplied.
- `oas-3.1.server-variable-object.fixed-fields.may-object-extended-specification-extensions` (MAY, by `oas-*.server-variable-object`): This object MAY be extended with Specification Extensions.

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

### Appendix F: Resolving Security Requirements in a Referenced Document (`#appendix-f-resolving-security-requirements-in-a-referenced-document`, line 4786)

- `oas-3.1.appendix-f-resolving-security-requirements-in-a-referenced-document.recommended-however-documented-section-tools-resolve-component` (SHOULD, by `oas-*.appendix-f-resolving-security-requirements-in-a-referenced-document`): However, documented in that section, it is RECOMMENDED that tools resolve component names from the entry document.

## openapi 3.2.1 — `openapi/3.2.1.md`: 44 missing

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
