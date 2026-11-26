
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "https://{region}.api.example.com/{version}", name: "prod", description: "production", variables: { "region": { default: "eu", enum: ["eu", "us"], description: "where the data lives" }, "version": { default: "v1" } } },
  { url: ".", description: "where the description is served" },
  { url: "./test", description: "beside it" },
  { url: "https://{x}.example.com", variables: { "x": { default: "z", enum: ["a", "b"] } } },
  { url: "https://{y}.example.com", variables: { "y": { default: "y" } } },
  { url: "https://example.com/?debug=1" },
] as const;

/**
 * The security schemes, by name: `apiKey` a key sent `in` a header, a query parameter or a
 * cookie under `name`; `http` as `Authorization: <scheme> <credentials>`; `oauth2` and
 * `openIdConnect` a token the caller obtained, sent as a bearer token. `mutualTLS` is a client
 * certificate, which no request carries: a client cannot send it, only its transport can.
 */
export const flowapiSecuritySchemes = {
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
    servers: [{ url: "https://pets.example.com" }],
    security: [],
    responses: { "200": { content: { "application/json": "json" } } },
    request: listPetsRequest,
    response: listPetsResponse,
  },
  createPet: {
    method: "POST",
    path: "/pets",
    servers: [{ url: "https://write.example.com", description: "writes" }],
    security: [],
    responses: { "201": { content: {} } },
    request: createPetRequest,
    response: createPetResponse,
  },
  health: {
    method: "GET",
    path: "/health",
    security: [],
    responses: { "204": { content: {} } },
    request: healthRequest,
    response: healthResponse,
  },
} as const;
// warning[server.variable]: server URL "https://{missing}.example.com" names `{missing}`, which `variables` does not define: left out
// warning[oas-3.2.server-object.url.required]: a server has no `url` string: left out
// warning[oas-3.2.server-variable-object.default.required]: variable `host` of server "https://{host}" has no `default` string: the server is left out
// warning[oas-3.2.server-variable-object.default.must-enum-defined-value-exist-enum-s]: the `default` "z" of variable `x` is not in its `enum`
// warning[oas-3.2.server-variable-object.enum.must-not-array-empty]: variable `y` has an `enum` of no string: any value is allowed
// warning[oas-3.2.server-object.url.must-not-query-fragment-part-url]: server URL "https://example.com/?debug=1" has a query or a fragment, which it MUST NOT
