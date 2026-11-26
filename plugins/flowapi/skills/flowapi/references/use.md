<!-- generated from README.md by plugins/flowapi/scripts/docs.mjs: edit the source, not this file -->

## Use

From npm, the same generator compiled to WebAssembly, reading JSON and YAML alike
([npm/README.md](https://github.com/localixai/flowapi/blob/main/npm/README.md) has its API):

```sh
npx @localix-ai/flowapi openapi.json -o types.ts
```

It has the commands for an SDK's config too (`summary`, `config draft|schema|check`, below), with
the Rust command's output byte for byte, and `generate` (below) the same files, lock included.

The Rust command, built from a release tag:

```sh
cargo install --locked --git https://github.com/localixai/flowapi --tag v0.2.0 flowapi-cli
flowapi openapi.json -o types.ts
```

The crates are not on crates.io. A program takes them from the repository at a tag:

```toml
[dependencies]
flowapi = { git = "https://github.com/localixai/flowapi", tag = "v0.2.0" }
flowapi-ts = { git = "https://github.com/localixai/flowapi", tag = "v0.2.0" }
```

In a checkout, `cargo run --release -p flowapi-cli -- openapi.json -o types.ts` does the same.

```ts
// One type for each component schema and each operation, and Paths:
export type Pet = {
  /** Unique within the store. */
  id: number;
  kind: "cat" | "dog";
  tags?: [string, ...string[]];
  [key: `x-${string}`]: string;
};
export type GetPet = {
  parameters: {
    path: {
      id: number;
    };
  };
  responses: {
    "200": {
      headers: {
        "X-Rate-Limit": number;
      };
      content: {
        "application/json": Pet;
      };
    };
  };
};
export type Paths = {
  "/pets/{id}": {
    get: GetPet;
  };
};
```

If the schema lets an object hold other properties, its type has an index signature for them.
Swagger 2.0 descriptions are read too, into the same types as 3.x: `definitions` are the named
types, a `body` or `formData` parameter is the request body under each media type the
operation consumes, and a response's `schema` is its content under each type it produces.
A relative `$ref` resolves against the document's own URI: the path you give the command, or
the URI the library is given with each document; flowapi assumes no other base.
Files that a `$ref` names by relative path are read too, and so are YAML descriptions (`.yaml`,
`.yml`, or any file that does not open with `{` or `[`). YAML is read as JSON's data model under
YAML 1.2's core schema: a key that is not a string becomes its JSON text (`200:` is `"200"`) and
anchors are expanded. What JSON cannot hold is an error with the YAML line and column: `.inf`
and `.nan`, a mapping as a key, a repeated key, `<<` merge keys, a second document, or aliases
that would copy more than a million nodes (code `yaml.syntax`). The YAML reader is one crate
(`crates/flowapi-yaml`) the command and the npm package share, and it passes the YAML test suite
(`crates/flowapi-yaml/tests/yaml_suite.rs`).

### An SDK, with no HTTP library

`--sdk` adds two functions per operation after the types: `<op>Request(input)` writes the request
and sends nothing, `<op>Response(status, headers, body)` reads the answer you got back. Use any
HTTP client, or none in a test.

```sh
cargo run --release -p flowapi-cli -- openapi.json --sdk -o api.ts
```

```ts
const r = getPetRequest({ path: { id: 7 }, query: { tags: ["a b", "c"] } })
// { method: "GET", path: "/pets/7", query: "tags=a%20b&tags=c", headers: {} }
const res = await fetch(base + r.path + (r.query && "?" + r.query), r)
const body = new Uint8Array(await res.arrayBuffer())
const out = getPetResponse(res.status, Object.fromEntries(res.headers), body)
if (out.response === "200") out.body // Pet, parsed from JSON

const up = uploadPhotoRequest({
  path: { id: 7 },
  body: { contentType: "multipart/form-data", value: { photo: await flowapiBlob(file), tags: ["a", "b"] } },
})
// up.headers["content-type"]: "multipart/form-data; boundary=flowapi-<32 hex digits>"
// up.body: a Uint8Array, a part `photo` with the file's name and type, then `tags` twice
```

Parameters are written as their `style`, `explode` and `allowReserved` say (`simple`, `label`,
`matrix`, `form`, `spaceDelimited`, `pipeDelimited`, `deepObject` and 3.2's `cookie`; Swagger
2.0's `collectionFormat` read as those), percent-encoded per RFC 3986 everywhere but headers and
`style: cookie`, which pass as given; a parameter with `content` as its media type's text; cookies
into one `cookie` header. The body is `{ contentType, value }` of one of the operation's media
types: JSON, a form, text as given, or bytes (a `Uint8Array`, for a type neither JSON, text nor a
form, sent as they are). A `multipart/form-data` body is written from its members as the
Encoding Object's defaults say: a `Uint8Array` or a file a file part (`filename` its last path
segment, `blob` when it has none; `application/octet-stream` when it has no type), a string,
number or boolean a text part, an object JSON, an array's items one part each under its name, in
order (RFC 7578, 4.3); `undefined` is left out, `null` refused. A file is
`flowapiFile(bytes or text, name, type)`, or `await flowapiBlob(blobOrFile)`, the one step that
waits: `<op>Request` takes bytes and stays synchronous. The description's `encoding` is not read
yet: give it as the body's `encoding` (`{ photo: { contentType, headers } }`); its `contentType`
replaces the default (a file's own type replaces it, and a list or a range needs one) and its
`headers` go into the part. The boundary is the content type's (`; boundary=`), else 128 random
bits from `globalThis.crypto`, drawn again while a part holds them and added to the request's
`content-type`; `body` is then bytes. 3.2's array of objects is written as its parts in order;
another multipart type is yours to write, as a string or bytes. A response is picked
by its status (the code, then `2XX`, then `default`) and its `Content-Type`, and its body, given as
text or as bytes, read by the type sent as the Rust SDK reads it: JSON parsed (a leading byte
order mark skipped), `text/*` a `string`, any other a `Uint8Array`, a media range any of those
its range allows. Bytes given for JSON or text are read as UTF-8; a string given for a binary type
is encoded as UTF-8, so give bytes (a string decoded from them has lost what was not UTF-8).
What the description does not name, and a JSON or text body that is not UTF-8 or not JSON, comes
back as `response: null` with the body as given. `flowapiReadsBytes(contentType)` (exported)
says which bodies are read as bytes, for a client that reads the others with `text()`.
`<op>Request` throws a `FlowapiRequestError` (exported, with `location`, `parameter` and
`reason`) rather than write a request that would mean something else: a CR, LF or NUL in a header
or a `style: cookie` value (`control-character`), a path parameter that leaves its segment empty
or makes it `.` or `..` (`empty-segment`, `dot-segment`; `%2E` is a dot too), NaN or an infinity
anywhere in a parameter or a JSON or form body (`non-finite`: JSON has no such number, and
`JSON.stringify` would write `null`). In a multipart body (`parameter` the part's name) it also
refuses a CR, LF or NUL in a part's name, file name, type or header, `null` (`null`), a `"` or `\`
in a name or a file name (`quote`: RFC 7578 and WHATWG escape them differently), a file name that
is empty, `.` or `..` (`file-name`), a part header's name that is not a token or is
`Content-Disposition` (`header-name`), a list or a range as a part's type (`media-range`), a
Blob not read yet (`unread-blob`), an item of 3.2's array that is not an object (`unnamed-part`),
and a boundary RFC 2046 does not allow or a part holds (`boundary`). A header's name that is not a token is left out of the
operation, with a warning. The output imports nothing: the runtime is a few functions
printed once. With `--sdk` the base URL and any authentication are still yours to add: the
command does not print `servers`, `securitySchemes` or `security`. The SDK layer
(`crates/flowapi-sdk`, the one `generate` writes a client with) reads them, Swagger 2.0's `host`, `basePath` and
`schemes` included, and prints a table to put after this output: the servers with their
variables, the security schemes, and each operation's method, path, own servers, security and
how each response body is read. Every row of the specification's
Style Examples table, the pitfalls other generators fixed, and the SDKs of GitHub's, Stripe's and
Cloudflare's descriptions are run or type-checked
([conformance/results/sdk.md](https://github.com/localixai/flowapi/blob/main/conformance/results/sdk.md)); writing a request against
openapi-fetch's serializers: [bench/results/sdk.md](https://github.com/localixai/flowapi/blob/main/bench/results/sdk.md).

For Rust, `--target rust --sdk` prints the same after the serde types: per operation an input
struct, `<op>_request(&input)`, which returns an `sdk::Request` (method, path, query, headers, and
the body's content type and bytes) or an `sdk::RequestError` for what the TypeScript SDK refuses,
with the same `reason` codes (`Reason::code()`), and `<op>_response(status, headers, body)`, which returns an
enum of the operation's responses or an `sdk::Error` holding the answer. No I/O, no async; the
runtime is a module printed once (`pub mod sdk`) and needs serde and serde_json, as the types do.

```sh
cargo run --release -p flowapi-cli -- openapi.json --target rust --sdk -o src/api.rs
```

```rust
let r = api::get_pet_request(&api::GetPetInput { parameters: params })?;
// sdk::Request { method: "GET", path: "/pets/7", query: "tags=a%20b&tags=c", headers: [], body: None }
let res = client.get(format!("{base}{}?{}", r.path, r.query)).send()?;
let headers: Vec<(String, String)> = /* the answer's headers as text */;
match api::get_pet_response(res.status().as_u16(), &headers, &res.bytes()?)? {
    api::GetPetResponse::Status200(pet) => { /* Pet, read by serde_json */ }
    api::GetPetResponse::Default(status, body) => { /* any other status, and its bytes */ }
}
```

Each parameter is written by the same rules, through a serde `Serializer` that writes straight
into the request; members of a struct go in the order its schema names them, a map's
(`additionalProperties`) in key order. The body is an enum of the media types: JSON and form
bodies take their type, a multipart body or a media range its content type and bytes, any other
bytes; parameters of a content type (`; charset=utf-8`) are yours to add to `body`. A response
is an enum variant per status and media type: JSON read into its type (an empty or broken body is
an error, never a default; a leading byte order mark is skipped), `text/*` a `String`, other
bodies bytes; a range or `default` keeps the status. A JSON integer is the printed `Integer`:
exact from `i64::MIN` to `u64::MAX`, past that the whole `f64` serde_json reads it as. The same table, the cases and Rust's own (numbers as JavaScript writes them, integers
past 2^53 exact, enums as their strings) are compiled into one crate and run
([conformance/results/sdk-rust.md](https://github.com/localixai/flowapi/blob/main/conformance/results/sdk-rust.md)); generating the SDK of
GitHub's, Stripe's and Cloudflare's descriptions:
[conformance/results/sdk-rust-corpus.md](https://github.com/localixai/flowapi/blob/main/conformance/results/sdk-rust-corpus.md); writing a
request against progenitor's way: [bench/results/sdk-rust.md](https://github.com/localixai/flowapi/blob/main/bench/results/sdk-rust.md).

### A config for a whole SDK

For an agent that writes an SDK's config, `flowapi-sdk.yml` (its resources, a name for each
operation, pagination, environments, authentication), four commands read the description so the
agent does not have to:

```sh
flowapi summary openapi.json --tag pets          # the operations in compact JSON, with the pagination each looks like it has
flowapi config draft openapi.json -o flowapi-sdk.yml   # a first config: every operation in a resource
flowapi config check flowapi-sdk.yml --json     # every issue at once: code, JSON Pointer, YAML line, hint
flowapi config schema > flowapi-sdk.schema.json # the format as JSON Schema
```

`summary` stays under `--max-bytes` (60 KB): over it, it gives the tags with their counts, to be
read one at a time with `--tag` or `--path`. `config check` holds the config to its format (a
member it does not name is an error) and to the description: every operation is in a resource
or in `skip` with a reason, every reference names an operation (a typo names the closest), names
are compared as words (`list_all` and `listAll` are one name), a method's pagination fits its
operation's parameters and response schema, `auth` names the description's security schemes.
A draft passes the check, and the same description written in another order gives the same
draft. A drafted method's name leaves out the words its resource already says
(`actions/get-actions-cache-retention-limit-for-enterprise` in `actions` is
`get_cache_retention_limit_for_enterprise`), keeps a word that qualifies it (`users`'
`list_followers_for_authenticated_user`), and where two names meet, both take a longer one. A description named `summary` or `config` is read by its path: `flowapi ./summary`.

From a config that passes the check, the SDK layer prints the client itself in TypeScript
(`flowapi_sdk::client_files`): `src/api.ts` (the `--sdk` output and the
operations table), a class for each resource (`client.pets.photos.list(params, options)`), the
client class with its options, and the runtime in `src/core/`, which needs no dependency and
type-checks with `lib: ["es2022"]` alone. A call goes to the call's `baseURL`, else the
client's, else the operation's own servers (an operation's over its Path Item's), else the
config's environment, else the description's first server: each `{name}` of a server's URL is the
caller's `serverVariables` value, checked against the variable's `enum`, or else its `default`,
and a relative server URL (`.`, `./v2`, `/v1`, one of another document written relative to the
first) is resolved against `descriptionURL`, where the description is served (OpenAPI 3.1 and
3.2, "Relative References in API URLs"); a URL with a user name or password in it is never sent.

The runtime retries by RFC 9110 9.2.2: a failed connection, or a 408, 409, 429 or 5xx, is retried
when the request is idempotent (GET, HEAD, OPTIONS, TRACE, PUT, DELETE), when the server answers
`x-should-retry: true` (`false` stops any retry), or when it carries an idempotency key: with
`defaults.idempotency_key: Idempotency-Key` in `flowapi-sdk.yml`, every call of another method
(POST, PATCH) sends a random key in that header, the same on each of its attempts, so that the
server can apply it once. A POST without one is never sent twice on the client's own. A 403 is
not retried; a status that is no status code is read as a 5xx. `retries` is 1 unless the config
says more, as a retry that failed is not retried again; a larger `maxRetries` is the caller's
choice. The wait is what `retry-after-ms` or `Retry-After` ask (a wait past `maxRetryDelay` is
refused), else a backoff. Redirects are followed by the runtime, not by `fetch`: every 3xx with a
`Location` but 304, 305 and 306; a 303 as a GET with no content, a POST redirected by 301 or 302
as a GET as browsers do, any other method and status with its method and content kept; a
`Location` with no fragment keeps the request's; a loop is refused at once and a chain after 20; a
redirect to another origin is refused unless `allowCrossOriginRedirects`, and then goes without
the credentials. A GET or HEAD with content, and an OPTIONS with content but no `Content-Type`,
are refused before anything is sent.

It also times each attempt out with its body read, removes its listener from the caller's
`AbortSignal`, sends the credentials of the first security alternative it can meet, masks them in
its logs, reads lists by each pagination scheme with a guard against a server that loops, and
reads server-sent events and JSON lines whatever the chunks. A call is awaited for its data;
`asResponse()` and `withResponse()` give the raw response.

`flowapi generate` writes the SDK's whole repository around that client:

```sh
flowapi generate flowapi-sdk.yml --out .         # the repository, and .flowapi/lock.json
flowapi generate flowapi-sdk.yml --out . --check # in CI: exits 1 when the output would differ
```

It is one package with no dependency at run time and `typescript` as its one development
dependency: built by `tsc` twice, ES modules and CommonJS, each with its types (`exports` sends
`import` and `require` each to its own); POSIX `scripts/` (`build`, `lint`, `test`, `check`,
`mock`); a mock server on `node:http` printed from the description, strict where a client goes
wrong (a path, a method, a required parameter, a body's media type) and answering each operation
with an example of its schema; a `node --test` file per resource calling every method against
it; `README.md` (its counts and examples written from the config, never typed, and each example
compiled against the client by flowapi's tests), `api.md` (every method's signature in the
package's types), the repository's `AGENTS.md`, the license (MIT or Apache-2.0), CI on Node 20, 22
and 24, and release-please's config. The README's heading is `readme.title`, else the
description's `info.title` with `TypeScript SDK` (`Twilio Studio API` is `Twilio Studio TypeScript
SDK`); its first paragraph is `readme.intro`, else `package.description`, which is also
`package.json`'s description and defaults to a sentence of `info`; `package.keywords` go to
`package.json`. Each method's doc comment is its operation's summary and description as plain
text, with a link to its `externalDocs` and `@deprecated` where the description says so. `.flowapi/lock.json` holds the hash of every file written, of the
description and of the config. A generated file changed by hand since is a conflict: nothing is
written and it exits 1; `src/lib/` and `examples/` are the user's, `CHANGELOG.md` is written once,
and a file the lock does not list is never touched. A file no longer generated is deleted unless
changed. Every write is staged first, so a refusal leaves the folder as it was.
