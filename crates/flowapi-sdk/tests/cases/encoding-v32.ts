
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
  postBatch: {
    method: "POST",
    path: "/batch",
    security: [],
    responses: { "204": { content: {} } },
    encoding: { "multipart/mixed": { prefixEncoding: [{}, { contentType: "multipart/mixed", prefixEncoding: [{ contentType: "application/xml" }, {}, { contentType: "image/png" }] }, { contentType: "application/octet-stream", headers: { "Content-Transfer-Encoding": "base64" } }], itemEncoding: { contentType: "application/octet-stream", headers: { "Content-Transfer-Encoding": "base64url" } } }, "multipart/form-data": { encoding: { "meta": { contentType: "multipart/form-data", encoding: { "a": { contentType: "application/json" } } }, "shot": { contentType: "image/jpeg" }, "deep": { contentType: "multipart/mixed", itemEncoding: { contentType: "multipart/mixed", itemEncoding: { contentType: "multipart/mixed", itemEncoding: { contentType: "multipart/mixed" } } } }, "scan": { contentType: "application/octet-stream" } }, itemEncoding: { contentType: "text/plain" } }, "application/x-www-form-urlencoded": { encoding: { "n": { explode: true } } } },
    request: postBatchRequest,
    response: postBatchResponse,
  },
} as const;
// warning[oas-3.2.encoding-object.nested-encoding.must-implementations-support-one-level-nesting-support]: Encoding Objects nested deeper than 4 levels are not read: the part is written by its value's defaults
