
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
 * `openIdConnect` a token sent as a bearer token, which the caller obtained or, for OAuth 2.0's
 * `clientCredentials` flow, the client requests at its `tokenUrl` (relative to the server's URL
 * where it is relative) for the scopes it needs. `mutualTLS` is a client certificate, which no
 * request carries: a client cannot send it, only its transport can.
 */
export const flowapiSecuritySchemes = {
  "headerKey": { type: "apiKey", in: "header", name: "X-API-Key", description: "Your *key*" },
  "bearer": { type: "http", scheme: "bearer", bearerFormat: "JWT" },
  "basic": { type: "http", scheme: "basic" },
  "client": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "/token?tenant=a", scopes: ["read", "x-not"] } },
  "empty": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "https://example.com/token", scopes: [] } },
  "noTokenUrl": { type: "oauth2", flows: ["clientCredentials"] },
  "badTokenUrl": { type: "oauth2", flows: ["clientCredentials"] },
  "noScopes": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "https://example.com/token", scopes: [] } },
  "oidc": { type: "openIdConnect", openIdConnectUrl: "https://example.com/.well-known/openid-configuration" },
  "cert": { type: "mutualTLS" },
} as const;

/**
 * Each operation, by the name of its functions: its method and path; the servers that serve it
 * instead of `flowapiServers`, where it has its own; its security, alternatives of which any one
 * is enough, each naming the schemes it needs with their scopes (`[]`: none); how each response's
 * body is read, by media type (`json`, `text`, `bytes`, or `sent`: by the `Content-Type` the
 * answer sends, as `flowapiReadsBytes` says); by form and `multipart` media type, how the body's
 * values are written (`encoding`: the description's Encoding Objects by name and by position,
 * given as the body's `encoding`, `prefixEncoding` and `itemEncoding`); and its `<name>Request`
 * and `<name>Response`.
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
  token: {
    method: "POST",
    path: "/a",
    security: [{ "client": ["read"] }, { "empty": [] }, { "bearer": [], "headerKey": [] }],
    responses: { "200": { content: {} } },
    request: tokenRequest,
    response: tokenResponse,
  },
  others: {
    method: "PATCH",
    path: "/a",
    security: [{ "oidc": ["openid"] }, { "headerKey": ["admin"] }],
    responses: { "200": { content: {} } },
    request: othersRequest,
    response: othersResponse,
  },
} as const;
// warning[oas-3.1.oauth-flow-object.tokenUrl.required]: security scheme `noTokenUrl`: the client credentials flow has no `tokenUrl`: the client requests no token for it
// warning[oas-3.1.oauth-flow-object.tokenUrl.must-form-url]: security scheme `badTokenUrl`: the `tokenUrl` "not a url" is not in the form of a URL: the client requests no token for it
// warning[oas-3.1.oauth-flow-object.scopes.required]: security scheme `noScopes`: the client credentials flow has no `scopes` map: read as none
