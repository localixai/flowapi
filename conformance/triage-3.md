# Triage, third pass: the SDK stage (2026-11-26)

Why (the owner, 2026-11-26): «мы же должны детерминированно проверять, что мы следуем
спецификации официальной». The ledger (`ledger/`, O1, rule G13) held the texts a types generator
follows. flowapi now also prints an SDK: the SDK layer's table (`crates/flowapi-sdk`), its client
runtime (`crates/flowapi-sdk/src/client/core`) and the request writer O8 runs (`sdk/`). This pass
pins the texts that SDK follows, reads them, and decides the scope again for it. It adds no test:
markers were added only where an existing test already checks the id. Like `triage-2.md` it sits
beside `coverage/`, not in it (the workspace's settings deny reading `**/coverage/**`), and holds
no markers.

Counts are `REPORT.md`'s; the ids no test names yet are in `MISSING.md`. Both are written by
`cargo run --release --bin report` in `ledger/`.

## 1. The texts pinned

`specs/SOURCE.md` has each file's source, date and SHA-256, and the licences.

| Text | File | Read by | Ids |
| --- | --- | --- | ---: |
| OpenAPI 2.0 (Swagger), OAI commit `4f86429` | `openapi/2.0.md` | `markdown.rs` (tables without a leading `\|` now read) | 231 |
| RFC 6749, OAuth 2.0 | `rfc/rfc6749.txt` | `rfc_text.rs` (new) | 234 |
| RFC 6750, bearer tokens | `rfc/rfc6750.txt` | `rfc_text.rs` | 56 |
| RFC 7578, `multipart/form-data` | `rfc/rfc7578.txt` | `rfc_text.rs` | 19 |
| RFC 7617, Basic | `rfc/rfc7617.txt` | `rfc_text.rs` | 10 |
| RFC 9110, HTTP Semantics | `rfc/rfc9110.xml` (RFCXML v3) | `rfc_xml.rs` (new) | 412 |

The four older RFCs have no XML at the RFC Editor, so their text is the copy. RFC 8414 is not
pinned: the client reads no authorization server metadata. The 3.x texts gave the same ids as
before, byte for byte (`extract`: 0 added, 0 removed). `tests/readers.rs` holds each reader to a
fixture. 18 RFC ids carry a `-2` suffix: the same six words open two sentences of one section.

## 2. The scope, text by text

`scope.json`'s `status` is now "decided 2026-11-24, the SDK stage 2026-11-26". Every rule whose
reason said "no data type", "run time", "Revisit at the SDK stage" or "the SDK stage's" was decided
again; an in-scope rule's `note` names the layer. No 3.x id moved out of scope.

| Text | In scope before | In scope now | Out of scope | Covered | Lossy | Missing |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| OpenAPI 2.0 | (not pinned) | 128 | 103 | 59 | 0 | 69 |
| OpenAPI 3.0.4 | 131 | 186 | 243 | 120 | 17 | 49 |
| OpenAPI 3.1.2 | 146 | 184 | 252 | 148 | 2 | 34 |
| OpenAPI 3.2.1 | 141 | 225 | 297 | 141 | 6 | 78 |
| JSON Schema 2020-12 | 137 | 137 | 231 | 96 | 41 | 0 |
| RFC 6749 | — | 19 | 215 | 0 | 0 | 19 |
| RFC 6750 | — | 12 | 44 | 2 | 0 | 10 |
| RFC 7578 | — | 13 | 6 | 12 | 0 | 1 |
| RFC 7617 | — | 3 | 7 | 1 | 0 | 2 |
| RFC 9110 | — | 24 | 388 | 4 | 0 | 20 |
| **All** | 555 | 931 | 1786 | 583 | 66 | 282 |

### OpenAPI 2.0

Explicit `oas-2.0.` rules, so that a 3.x rule decides no 2.0 id by accident; each id was read.

| Section | Class | Layer or reason |
| --- | --- | --- |
| `swagger-object` (`swagger`, `paths`, `definitions`, `parameters`, `responses`, `consumes`, `produces`), the definitions, paths, path item, operation, parameter, items, header(s), response(s), schema and reference objects | in | The types: Swagger 2.0 read into the 3.x model (`spec/swagger.rs`), the `swagger20-*` cases of `flowapi-ts` |
| `host`, `basePath`, `schemes`, an operation's `schemes`, `securityDefinitions`, `security`, the security scheme and requirement objects | in | The SDK layer (`servers.rs`, `security.rs`), the cases `swagger20`, `swagger-host`, `swagger-nohost` |
| `tokenUrl`, `scopes`, the scopes object | in | The client runtime, which requests no token yet (section 3) |
| `authorizationUrl` | out | A grant that needs the resource owner: the caller runs it and gives the token |
| `info`, contact, license, tag, xml, external docs, example objects; `examples`; descriptions, summaries | out | Metadata, documentation and examples: no type or request depends on them |
| `allowEmptyValue` | out | A permission the client does not take |
| Each `x-` name, `swagger` being `"2.0"`, MIME types, path names, `type` values, `default` conforming, `operationId` unique, `readOnly` not required, `host` without a scheme | out | Rules for the author of a description |
| A Header Object's `collectionFormat` (decided 2026-11-26, the encoding pass) | out | A 2.0 Header Object is a response header's: the client parses no response header by its schema, as for 3.x's Header `style` and `explode` |
| A discriminator's value naming this schema or one that inherits it (same pass) | out | A rule for the author of a description |

### OpenAPI 3.x

| Section | Class | Layer or reason |
| --- | --- | --- |
| Servers at three levels, the Server and Server Variable Objects, 3.1's relative references in API URLs | in | The SDK layer (`servers.rs`); 3.0 and 3.2 now in whole, 3.1 was in by id |
| `components.securitySchemes`, `security`, the Security Scheme and Security Requirement Objects, 3.2's security schemes section, Appendix F | in | The SDK layer (`security.rs`) |
| `variables` default substitution, 3.2's `deprecated` scheme | in | The client runtime |
| OAuth Flows `clientCredentials`; OAuth Flow `tokenUrl`, `scopes` | in | The client runtime, which requests no token yet (section 3) |
| `implicit`, `password`, `authorizationCode`, `deviceAuthorization`, `authorizationUrl`, `deviceAuthorizationUrl` | out | A grant that needs the resource owner or a user agent: the caller runs it and gives the token, sent as a bearer token |
| `refreshUrl` | out | Refreshing is for the flows the caller runs; a client credentials grant has no refresh token (RFC 6749 4.4.3) |
| 3.2's `oauth2MetadataUrl` | out | The client reads no metadata (RFC 8414, not pinned) |
| The Encoding Object; `encoding`, `prefixEncoding`, `itemEncoding` and their sections; 3.2's non-JSON data | in | The multipart writer (`multipart.ts`, `encoding.rs`), O8 |
| Within them: keys that must exist, fields that exclude each other, `Content-Transfer-Encoding`, `schema` required | out | Rules for the author |
| Reading a multipart body by name; the additional encoding approaches; "not currently possible to correlate" | out | The client writes these bodies and reads none; permissions not taken |
| Header Object `style`, `explode`, its schema serialization, link headers, `Set-Cookie` | out | The client parses no response header by schema; a part's headers are the caller's strings |
| Header values passed through unchanged | in | The client runtime |
| Parameter `allowEmptyValue` | out | A deprecated permission not taken |
| `content` for cookies and headers (RECOMMENDED), schema serialization NOT RECOMMENDED | out | Rules for the author |
| Parsing parameters | out | The client writes parameters and parses none |
| Appendices C, D, the non-RFC 6570 combinations, the delimiters' and cookies' percent-encoding guidance | out | Guidance for a serializer built on an RFC 6570 library; the SDK writes each style by its own code (the parts O8 runs stay in by id) |
| Appendix E's statement, percent-encoding in form data, RFC 1738 interoperability, 3.2's cookie storage | out | Statements that ask nothing; the caller's data; the client stores no cookie |
| Interoperability with web browsers (WHATWG form rules) | in | O8 |
| Links, callbacks, webhooks | out | The client follows no links and only sends the described operations |
| Discriminator selection, publishing and run-time handling | out | Reworded: the client returns a body as parsed and checks no value |

### The RFCs

| Text | In scope | Out of scope, and why |
| --- | --- | --- |
| RFC 6749 | The client credentials grant (2.3 one method, 2.3.1 Basic and no body credentials in the URI, 3.2 POST and parameters once, 3.2.1, 4.4.2, 5.1 unknown names ignored, Appendix B's encoding, 10.3 minimal scope), and using a token (7.1, 10.3, 10.8) | The authorization and resource servers; the grants a user agent runs, refresh, native apps; registration, extensions, IANA; TLS (10.9) is the platform's fetch |
| RFC 6750 | Sending a bearer token: the Authorization header, one method, never a query, a page URL or a cookie, safeguarded, over TLS; 3.1's retry with a new token | The resource server's challenges and errors; the form and query methods, which the client never uses; certificate validation |
| RFC 7617 | No control characters in user-id or password; sent up front; not without TLS | The recipient, the challenge's parameters, protection spaces, proxies |
| RFC 7578 | The boundary, Content-Disposition and its name and filename, no `filename*`, one part per file, Content-Type, no Content-Transfer-Encoding, no other header fields, UTF-8, no coalescing, order kept | Receivers, form creators, intermediaries; two permissions not taken (percent-encoded names, `charset`) |
| RFC 9110 | No userinfo in a URI, a Content-Type for content, CRLF in a multipart body, retries (9.2.2), no content in GET, HEAD, DELETE, a Content-Type with OPTIONS content, the status classes, redirects (15.4, 301 to 308, the Location fragment), 401 and 408 | What the platform's fetch does (URIs, framing, fields, connections, TLS, 1xx), servers, intermediaries and caches; fields the client never sends of its own (Expect, User-Agent, Accept-*, If-*, Range); challenges; CONNECT and TRACE, which fetch refuses; 300 and 304; extending HTTP |

## 3. What the decisions found

- **OAuth client credentials.** `clientCredentials`, `tokenUrl`, `scopes` and RFC 6749's 4.4 rows
  are in scope, and the client does not do them: `auth.ts` sends the token the caller gives as
  a bearer token and requests none. They are the work list of the client's OAuth step (19 of RFC
  6749's 19 missing ids, 7 per 3.x version under `oauth-flow-object`).
- **RFC 7578 4.8 against OpenAPI.** "Other header fields MUST NOT be included" in a part, while
  the Encoding Object's `headers` describe exactly such fields and the writer sends them. The
  owner's decision (2026-11-26): flowapi follows OpenAPI, and the id is lossy with that reason
  (the O8 case on a part's header values written as given).
- **The encoding pass (2026-11-26).** The Encoding Object, `prefixEncoding`, `itemEncoding`,
  nesting, 3.2's non-JSON data and Appendix E's browser interoperability are covered by O8 cases
  and the table's cases (`encoding*.json`, `swagger-forms.json`). A form body written from its
  content (no `style`) is percent-encoded by the WHATWG rules 3.2 asks for (space as `+`, `!'()~`
  encoded); one written by `style` keeps RFC 3986's. Swagger 2.0's `formData` `collectionFormat`
  is read into the table as a style (`tsv` as the SDK's own `tabDelimited`). Left missing: a
  parameter's `collectionFormat` in query, path and header beyond csv, ssv, pipes and multi, and
  the Items Object's, which need a tab-delimited style and 2.0's delimited styles outside the
  query in the core's `spec/operations/style.rs`, at its line budget (G10). Both closed the same
  day: room made in the core by merging duplicated code, then `tabDelimited` and the delimited
  styles in a path and a header, and `Serialization::nested` for the Items Object (O8 cases,
  `crates/flowapi/tests/collection_format.rs`).
- **TLS and the mock.** RFC 6749 10.3, RFC 6750 5.3 and RFC 7617 4 ask that credentials go only
  over TLS. The client sends them to whatever base URL it is given, including the mock at
  `http://localhost`. A refusal of `http:` except for a loopback host would answer them; until a
  test holds that, they stay missing.
- **Retries.** RFC 9110 9.2.2: a client SHOULD NOT retry a non-idempotent method on its own, nor
  retry a failed retry. `retry.ts` retries any method on 408, 409, 429 and 5xx, up to
  `maxRetries` (2 by default). Both ids are missing, and as written the client goes against
  the first unless the server asks (`x-should-retry`). Next: a test that states the policy, then
  either the change or a lossy marker with the reason.
- **Retry-After.** RFC 9110 10.2.3 states no BCP 14 sentence, so it has no id; the client's
  reading of it is held by its own tests (R4) only.
- **3.0 and 3.2's servers and security.** The second pass had put only the 3.1 ids the table's
  cases named in scope. The whole objects are in now, so 3.0's and 3.2's servers and security ids
  are missing until their cases name them (the table reads all three versions alike).
- **The client's tests are markers now.** `.mjs` files are scanned, so `tests/client/*.mjs` can
  name what they hold; this pass marked only what a test already asserts (R5's 408, R34's 303 and
  its cross-origin refusal, the bearer, basic and server-variable tests in `auth.mjs`).

## 4. Next

A test for each missing id, or a lossy marker with its reason, in the order: the OAuth client
credentials step, the TLS rule with the mock's exception, the retry policy, then 3.0 and 3.2's
cases of the SDK table and Swagger 2.0's validation keywords. `MISSING.md` lists them by section.

## 5. Servers and HTTP, the tests (2026-11-26)

Every in-scope id on servers and on RFC 9110 outside section 11 now has a test that checks it:
56 ids, 14 of OpenAPI 3.0.4, 5 of 3.1.2, 17 of 3.2.1 and 20 of RFC 9110. The SDK table's cases
`servers30.json` and `servers32.json` read 3.0 and 3.2's servers as `servers.json` reads 3.1's (an
`x-` member of a Server or Server Variable Object read by no one), and `tests/table.rs` holds a
Server Object of another document, its relative URL written from the first. Two client cases,
`hosts` (3.2) and `edge` (3.1), hold the runtime in `tests/client/servers.mjs` (R43 to R45: a
URL with a user, a variable's value against its `enum`, a relative URL resolved against
`descriptionURL`) and `tests/client/http.mjs` (R46 to R55: retries, idempotency keys, status
classes, redirects, content by method).

What the section 3 findings became, by the owner's decisions (delegated):

- **Retries** (9.2.2). A request whose method is not idempotent is not retried on its own
  unless the server answers `x-should-retry: true` or it carries an idempotency key (the config's
  `defaults.idempotency_key` names the header; each call sends one random key, the same on its
  retries). The default `retries` is 1, so a failed retry is not retried; a larger value is the
  config's or the caller's explicit choice. A 403 is never retried (15.5.4).
- **Redirects** (15.4). Every 3xx with a `Location` is followed but 304, 305 and 306: 303 as a GET
  with no content, 301 and 302 turn a POST into a GET as browsers do (the permission of 15.4.2
  and 15.4.3 taken), the rest keep the method and the content (15.4.8 holds 307 to it). A loop is
  refused at once, a chain after 20, and a `Location` without a fragment keeps the request's
  (10.2.2).

One scope change: `rfc9110.15.4.1.may-user-agent-use-location-field-value` (a 300's `Location`
used for automatic redirection) is in scope now, as the client follows a 300 with a `Location`
like a 3xx no one defines (15.4 allows both); the other ids of 15.4.1 stay out, their reason
corrected (the client makes no choice from a 300's list). No lossy marker was added.
