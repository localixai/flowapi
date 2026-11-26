
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "https://example.com", name: "prod", description: "production" },
] as const;

/**
 * The security schemes, by name: `apiKey` a key sent `in` a header, a query parameter or a
 * cookie under `name`; `http` as `Authorization: <scheme> <credentials>`; `oauth2` and
 * `openIdConnect` a token the caller obtained, sent as a bearer token. `mutualTLS` is a client
 * certificate, which no request carries: a client cannot send it, only its transport can.
 */
export const flowapiSecuritySchemes = {
  "old": { type: "apiKey", in: "header", name: "X-Old", deprecated: true },
  "token": { type: "http", scheme: "bearer" },
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
  sharedGet: {
    method: "GET",
    path: "/shared",
    servers: [{ url: "https://shared.example.com" }],
    security: [{ "old": [] }],
    responses: { "200": { content: {} } },
    request: sharedGetRequest,
    response: sharedGetResponse,
  },
  copyIt: {
    method: "COPY",
    path: "/shared",
    servers: [{ url: "https://shared.example.com" }],
    security: [{ "token": [] }, { "https://example.com/other.json#/components/securitySchemes/x": [] }],
    responses: { "200": { content: {} } },
    request: copyItRequest,
    response: copyItResponse,
  },
  queryFiles: {
    method: "QUERY",
    path: "/files/{id}",
    security: [{ "token": [] }],
    responses: { "200": { content: { "application/pdf": "bytes", "text/csv": "text", "application/problem+json": "json" } } },
    request: queryFilesRequest,
    response: queryFilesResponse,
  },
} as const;
// warning[oas-3.2.security-scheme-object.type.required]: security scheme `untyped` has no `type` it can be used by: left out
// warning[oas-3.2.security-scheme-object.scheme.required]: security scheme `noScheme` has no `scheme` it can be used by: left out
// warning[oas-3.2.security-requirement-object.pattern-name.must-each-name-uri-correspond-security-scheme]: a security requirement names `https://example.com/other.json#/components/securitySchemes/x`, which no security scheme is declared under
