
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
  postForm: {
    method: "POST",
    path: "/forms",
    security: [],
    responses: { "200": { content: { "multipart/form-data": "bytes" } } },
    encoding: { "multipart/form-data": { encoding: { "id": { contentType: "text/plain" }, "file": { contentType: "application/octet-stream" }, "b64": { contentType: "application/octet-stream" }, "files": { contentType: "application/octet-stream" } } }, "application/x-www-form-urlencoded": { encoding: { "tags": { style: "spaceDelimited", explode: false }, "meta": { contentType: "application/json" } } } },
    request: postFormRequest,
    response: postFormResponse,
  },
} as const;
