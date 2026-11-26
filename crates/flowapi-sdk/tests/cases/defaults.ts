
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
  "session": { type: "apiKey", in: "cookie", name: "sid" },
  "oauth": { type: "oauth2", flows: ["implicit"] },
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
  open: {
    method: "GET",
    path: "/open",
    security: [],
    responses: { "200": { content: { "text/plain": "text" } } },
    request: openRequest,
    response: openResponse,
  },
  mine: {
    method: "GET",
    path: "/mine",
    security: [{ "session": [] }, { "oauth": ["read"] }, { "ghost": [] }],
    responses: { "200": { content: { "application/json": "json", "image/*": "sent" } }, "default": { content: { "*/*": "sent" } } },
    request: mineRequest,
    response: mineResponse,
  },
} as const;
// warning[oas-3.0.security-scheme-object.type]: security scheme `cert` has a `type` "mutualTLS" this version does not define: left out
// warning[oas-3.0.security-scheme-object.type.required]: security scheme `untyped` has no `type` it can be used by: left out
// warning[oas-3.0.security-scheme-object.in.required]: security scheme `noIn` has no `in` it can be used by: left out
// warning[oas-3.0.security-scheme-object.name.required]: security scheme `noName` has no `name` it can be used by: left out
// warning[oas-3.0.security-requirement-object.pattern-name.must-each-name-correspond-security-scheme-declared]: a security requirement names `ghost`, which no security scheme is declared under
