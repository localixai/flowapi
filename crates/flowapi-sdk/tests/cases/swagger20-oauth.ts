
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "https://api.example.com/" },
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
  "app": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "https://example.com/token", scopes: ["pets:read", "pets:write"] }, description: "The *client's* own" },
  "relative": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "/token", scopes: [] } },
  "noTokenUrl": { type: "oauth2", flows: ["clientCredentials"] },
  "noScopes": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "https://example.com/token", scopes: [] } },
  "key": { type: "apiKey", in: "header", name: "X-Key", description: "A key" },
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
  listPets: {
    method: "GET",
    path: "/pets",
    security: [{ "app": ["pets:read"] }],
    responses: { "200": { content: {} } },
    request: listPetsRequest,
    response: listPetsResponse,
  },
  addPet: {
    method: "POST",
    path: "/pets",
    security: [{ "app": ["pets:write"] }, { "key": [] }],
    responses: { "200": { content: {} } },
    request: addPetRequest,
    response: addPetResponse,
  },
} as const;
// warning[swagger.security]: security scheme `noTokenUrl`: the client credentials flow has no `tokenUrl`: the client requests no token for it
// warning[swagger.security]: security scheme `noScopes`: the client credentials flow has no `scopes` map: read as none
