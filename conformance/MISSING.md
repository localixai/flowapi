# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**99 missing.**

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

## openapi 3.0.4 — `openapi/3.0.4.md`: 21 missing

### Specification > Schema > OpenAPI Object > Fixed Fields (`#oas-security`, line 265)

- `oas-3.0.openapi-object.security` (field, by `oas-*.openapi-object.security`): A declaration of which security mechanisms can be used across the API. The list of values includes alternative Security Requirement Objects that can be used. Only one of the Security Requirement Objects need to be satisfied to authorize a request. Individual operations can override this definition. The list can be incomplete, up to being empty or absent. To make security explicitly optional, an empty security requirement (`{}`) can be included in the array.

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

## openapi 3.1.2 — `openapi/3.1.2.md`: 11 missing

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

## openapi 3.2.1 — `openapi/3.2.1.md`: 27 missing

### Objects and Fields > Components Object > Fixed Fields (`#components-security-schemes`, line 410)

- `oas-3.2.components-object.securitySchemes` (field, by `oas-*.components-object.securitySchemes`): An object to hold reusable Security Scheme Objects.

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
