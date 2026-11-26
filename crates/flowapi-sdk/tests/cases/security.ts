
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "/" },
] as const;

/**
 * The security schemes, by name: `apiKey` a key sent `in` a header, a query parameter or a
 * cookie under `name`; `http` as `Authorization: <scheme> <credentials>`; `oauth2` and
 * `openIdConnect` a token the caller obtained, sent as a bearer token. `mutualTLS` is a client
 * certificate, which no request carries: a client cannot send it, only its transport can.
 */
export const flowapiSecuritySchemes = {
  "headerKey": { type: "apiKey", in: "header", name: "X-API-Key", description: "your key" },
  "queryKey": { type: "apiKey", in: "query", name: "api_key" },
  "cookieKey": { type: "apiKey", in: "cookie", name: "session" },
  "bearer": { type: "http", scheme: "bearer", bearerFormat: "JWT" },
  "basic": { type: "http", scheme: "basic" },
  "oauth": { type: "oauth2", flows: ["authorizationCode", "clientCredentials"] },
  "oidc": { type: "openIdConnect", openIdConnectUrl: "https://example.com/.well-known/openid-configuration" },
  "cert": { type: "mutualTLS" },
  "alias": { type: "http", scheme: "bearer", bearerFormat: "JWT" },
} as const;

/**
 * Each operation, by the name of its functions: its method and path; the servers that serve it
 * instead of `flowapiServers`, where it has its own; its security, alternatives of which any one
 * is enough, each naming the schemes it needs with their scopes (`[]`: none); how each response's
 * body is read, by media type (`json`, `text`, `bytes`, or `sent`: by the `Content-Type` the
 * answer sends, as `flowapiReadsBytes` says); by `multipart/form-data` media type, the type
 * each part is sent as (`encoding`, the description's Encoding Objects: give it as
 * `body.encoding`); and its `<name>Request` and `<name>Response`.
 */
export const flowapiOperations = {
  inherits: {
    method: "GET",
    path: "/a",
    security: [{ "headerKey": [] }, { "bearer": [], "basic": [] }],
    responses: { "200": { content: {} } },
    request: inheritsRequest,
    response: inheritsResponse,
  },
  anonymous: {
    method: "PUT",
    path: "/a",
    security: [],
    responses: { "200": { content: {} } },
    request: anonymousRequest,
    response: anonymousResponse,
  },
  scoped: {
    method: "POST",
    path: "/a",
    security: [{ "oauth": ["read", "write"] }, { "oidc": ["openid"] }, {}],
    responses: { "200": { content: {} } },
    request: scopedRequest,
    response: scopedResponse,
  },
  malformed: {
    method: "DELETE",
    path: "/a",
    security: [{ "cookieKey": [] }],
    responses: { "200": { content: {} } },
    request: malformedRequest,
    response: malformedResponse,
  },
  notAList: {
    method: "OPTIONS",
    path: "/a",
    security: [{ "headerKey": [] }, { "bearer": [], "basic": [] }],
    responses: { "200": { content: {} } },
    request: notAListRequest,
    response: notAListResponse,
  },
  unknown: {
    method: "PATCH",
    path: "/a",
    security: [{ "nope": [] }, { "cert": [], "queryKey": ["admin"] }],
    responses: { "200": { content: {} } },
    request: unknownRequest,
    response: unknownResponse,
  },
} as const;
// warning[oas-3.1.reference-object.$ref]: security scheme `dangling` is a reference that names nothing: #/components/securitySchemes/nowhere
// warning[oas-3.1.security-scheme-object.name.required]: security scheme `noName` has no `name` it can be used by: left out
// warning[oas-3.1.security-scheme-object.in.required]: security scheme `badIn` has no `in` it can be used by: left out
// warning[oas-3.1.security-scheme-object.scheme.required]: security scheme `noScheme` has no `scheme` it can be used by: left out
// warning[oas-3.1.security-scheme-object.flows.required]: security scheme `noFlows` has no `flows` it can be used by: left out
// warning[oas-3.1.security-scheme-object.type]: security scheme `magic` has a `type` "magic" this version does not define: left out
// warning[oas-3.1.security-scheme-object.type.required]: security scheme `untyped` has no `type` it can be used by: left out
// warning[oas-3.1.security-scheme-object.openIdConnectUrl.required]: security scheme `noUrl` has no `openIdConnectUrl` it can be used by: left out
// warning[oas-3.1.security-requirement-object.pattern-name]: a security requirement is not an object: left out
// warning[oas-3.1.security-requirement-object.pattern-name]: a requirement's scopes are not a list of strings: read as the strings it holds
// warning[oas-3.1.operation-object.security]: `security` is not a list: left out
// warning[oas-3.1.security-requirement-object.pattern-name.must-each-name-correspond-security-scheme-declared]: a security requirement names `nope`, which no security scheme is declared under
