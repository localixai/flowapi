
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "https://api.example.com:8443/v2" },
  { url: "http://api.example.com:8443/v2" },
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
  "basic": { type: "http", scheme: "basic" },
  "key": { type: "apiKey", in: "query", name: "key" },
  "headerKey": { type: "apiKey", in: "header", name: "X-Key" },
  "code": { type: "oauth2", flows: ["authorizationCode"] },
  "app": { type: "oauth2", flows: ["clientCredentials"], clientCredentials: { tokenUrl: "https://example.com/t", scopes: [] } },
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
  listPets: {
    method: "GET",
    path: "/pets",
    security: [{ "code": ["pets"] }, { "key": [], "headerKey": [] }],
    responses: { "200": { content: { "application/json": "json", "application/octet-stream": "bytes" } } },
    request: listPetsRequest,
    response: listPetsResponse,
  },
  putPets: {
    method: "PUT",
    path: "/pets",
    security: [{ "basic": [] }],
    responses: { "204": { content: {} } },
    request: putPetsRequest,
    response: putPetsResponse,
  },
  watchPets: {
    method: "POST",
    path: "/pets",
    servers: [{ url: "wss://api.example.com:8443/v2" }],
    security: [],
    responses: { "101": { content: {} } },
    request: watchPetsRequest,
    response: watchPetsResponse,
  },
} as const;
// warning[swagger.server]: a scheme is none of http, https, ws, wss: left out
// warning[swagger.server]: a scheme is none of http, https, ws, wss: left out
// warning[swagger.security]: security scheme `app`: the client credentials flow has no `scopes` map: read as none
// warning[swagger.security]: security scheme `noFlow` has no `flows` it can be used by: left out
// warning[swagger.security]: security scheme `cookie` has no `in` it can be used by: left out
// warning[swagger.security]: security scheme `bearer` has a `type` "http" this version does not define: left out
