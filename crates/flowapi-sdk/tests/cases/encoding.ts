
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
  { url: "https://files.example" },
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
  uploadPhoto: {
    method: "POST",
    path: "/photos",
    security: [],
    responses: { "201": { content: {} } },
    encoding: { "multipart/form-data": { "photo": { contentType: "image/png" }, "meta": { contentType: "application/json; charset=utf-8" } } },
    request: uploadPhotoRequest,
    response: uploadPhotoResponse,
  },
  putDoc: {
    method: "PUT",
    path: "/docs",
    security: [],
    responses: { "204": { content: {} } },
    encoding: { "multipart/form-data; charset=utf-8": { "file": { contentType: "application/pdf" } } },
    request: putDocRequest,
    response: putDocResponse,
  },
  plain: {
    method: "POST",
    path: "/plain",
    security: [],
    responses: { "204": { content: {} } },
    request: plainRequest,
    response: plainResponse,
  },
} as const;
// warning[oas-3.1.encoding-object.contentType]: the part `thumb` is encoded as `image/png, image/jpeg`, a list or a range of types: no one type to send it as, so it is sent as its value's default type
