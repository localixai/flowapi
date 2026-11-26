# The SDK

With `--sdk`, flowapi prints two functions per operation after the types: one writes the request and sends nothing, the other reads the answer you got back.

## TypeScript

```sh
flowapi openapi.json --sdk -o api.ts
```

`<op>Request(input)` returns the method, the path, the query, the headers and the body. `<op>Response(status, headers, body)` picks the response the description names for that status and `Content-Type`. Any HTTP client can send the request, or none at all in a test.

```ts
const r = getPetRequest({ path: { id: 7 }, query: { tags: ["a b", "c"] } })
// { method: "GET", path: "/pets/7", query: "tags=a%20b&tags=c", headers: {} }
const res = await fetch(base + r.path + (r.query && "?" + r.query), r)
const out = getPetResponse(res.status, Object.fromEntries(res.headers), await res.text())
if (out.response === "200") out.body // Pet, parsed from JSON
```

The output imports nothing: the runtime is a few functions printed once. With `--sdk` the base URL and authentication are yours to add; `flowapi generate` writes a whole client that reads `servers`, `securitySchemes` and `security` (see the README).

## How parameters are written

Parameters follow their `style`, `explode` and `allowReserved`: `simple`, `label`, `matrix`, `form`, `spaceDelimited`, `pipeDelimited`, `deepObject` and 3.2's `cookie`. Swagger 2.0's `collectionFormat` is read as one of those. Values are percent-encoded per RFC 3986 everywhere except headers and `style: cookie`, which pass as given, and cookies go into one `cookie` header.

The body is `{ contentType, value }` for one of the operation's media types: JSON, a form, text as given, or bytes. A `multipart/form-data` body is written from its members, a file part from `flowapiFile` or `flowapiBlob`.

A response is matched by its status (the exact code, then `2XX`, then `default`) and its `Content-Type`. A JSON body is parsed, a text body is a string, and any other body is bytes (`Uint8Array`); the body may be given as a string or as bytes. An answer the description does not name comes back as `response: null` with the text.

## What it refuses to write

`<op>Request` throws a `FlowapiRequestError`, with `location`, `parameter` and `reason`, instead of writing a request that would mean something else:

- a CR, LF or NUL in a header or a `style: cookie` value (`control-character`);
- a path parameter that leaves its segment empty, or makes it `.` or `..` (`empty-segment`, `dot-segment`; `%2E` counts as a dot);
- NaN or an infinity in a parameter or a JSON or form body (`non-finite`), which JSON has no way to write.

A header name that is not a token is left out of the operation, with a warning.

## Rust

```sh
flowapi openapi.json --target rust --sdk -o src/api.rs
```

For each operation there is an input struct, `<op>_request(&input)`, which returns an `sdk::Request` or an `sdk::RequestError` with the same `reason` codes, and `<op>_response(status, headers, body)`, which returns an enum of the operation's responses or an `sdk::Error` that keeps the answer.

```rust
let r = api::get_pet_request(&api::GetPetInput { parameters: params })?;
let res = client.get(format!("{base}{}?{}", r.path, r.query)).send()?;
match api::get_pet_response(res.status().as_u16(), &headers, &res.bytes()?)? {
    api::GetPetResponse::Status200(pet) => { /* Pet, read by serde_json */ }
    api::GetPetResponse::Default(status, body) => { /* any other status, and its bytes */ }
}
```

There is no I/O and no async. The runtime is a module printed once (`pub mod sdk`), and it needs only serde and serde_json, as the types do. A JSON integer is the printed `Integer`, exact from `i64::MIN` to `u64::MAX`.

## How it is checked

Every row of the specification's Style Examples table and the pitfalls other generators had to fix are generated, compiled and called under Node: 51 style examples and 234 cases, none of them different from what is expected ([conformance/results/sdk.md](https://github.com/localixai/flowapi/blob/main/conformance/results/sdk.md)). The Rust SDK runs the same table and cases compiled into one crate ([conformance/results/sdk-rust.md](https://github.com/localixai/flowapi/blob/main/conformance/results/sdk-rust.md)), and the SDKs of GitHub's, Stripe's and Cloudflare's descriptions are generated and type-checked.
