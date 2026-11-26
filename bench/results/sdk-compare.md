# The whole SDK: flowapi against the other client generators

Written by `node bench/competitors/sdk-compare.mjs`; do not edit. 2026-11-26, commit c31c449 (with uncommitted changes), Intel(R) N150, 4 cores, 16.5 GB, Ubuntu 24.04.5 LTS, Linux 7.0.0-43-generic, Node.js v22.23.5.

Versions: flowapi 0.3.0, openapi-typescript 7.13.0, openapi-fetch 0.17.0, @hey-api/openapi-ts 0.99.0, orval 8.49.0, swagger-typescript-api 13.15.0, typescript 5.9.3, tsc (the check) 7.0.2.

**A partial run** (`--only shop,auth`): not every description was measured.

Each generation is a fresh process into an empty folder, the median of 5 with its peak resident memory; flowapi runs `generate` on the config `config draft` wrote (the whole repository: client, tests, mock, README), the others their client mode. Files and bytes are all a generator wrote. `tsc` errors are those of `tsc --ignoreConfig --strict --target es2022 --module nodenext --lib es2022,dom --skipLibCheck --noEmit` on the SDK's TypeScript (flowapi's `src/`; openapi-typescript's types with the five lines that compose them with openapi-fetch).

## shop

a small description of the flowapi tests: two security schemes, a cursor list (`crates/flowapi-sdk/tests/client/shop/spec.json`, 8 KB).

| Generator | Generate | Peak memory | Files | Bytes | TypeScript | tsc errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowapi | 6.8 ms | 4 MB | 55 | 229 KB | 180 KB | 0 |
| hey-api | 1.11 s | 153 MB | 16 | 68 KB | 68 KB | 0 |
| openapi-typescript | 1.21 s | 146 MB | 1 | 15 KB | 15 KB | 0 |
| orval | 921 ms | 105 MB | 1 | 18 KB | 18 KB | 0 |
| swagger-typescript-api | 1.85 s | 309 MB | 1 | 15 KB | 15 KB | 0 |

## auth

a small description of the flowapi tests: an API key in a query and in a cookie, HTTP basic and bearer (`crates/flowapi-sdk/tests/client/auth/spec.json`, 3 KB).

| Generator | Generate | Peak memory | Files | Bytes | TypeScript | tsc errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| flowapi | 7.3 ms | 4 MB | 63 | 225 KB | 173 KB | 0 |
| hey-api | 825 ms | 150 MB | 16 | 64 KB | 64 KB | 0 |
| openapi-typescript | 861 ms | 141 MB | 1 | 8 KB | 8 KB | 0 |
| orval | 746 ms | 108 MB | 1 | 10 KB | 10 KB | 0 |
| swagger-typescript-api | 2.18 s | 310 MB | 1 | 12 KB | 12 KB | 0 |

## Behavior

Each SDK called through its own surface against one mock server made from the description, which records the requests as they arrive. `pass (k of n)`: passed on every description that tests it; `fail (where)`: the generated code has the feature and the test failed; `n/a`: no surface for it in the generated code (an option, a method, a credential), the caller does it; `-`: no description here tests it. Every cell is an observation; none is read from documentation.

| Feature | flowapi | hey-api | openapi-typescript | orval | swagger-typescript-api |
| --- | --- | --- | --- | --- | --- |
| Path parameters encoded | pass (1 of 1) | fail (shop) | pass (1 of 1) | fail (shop) | fail (shop) |
| Query arrays | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) |
| JSON body | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) |
| Form body by its Encoding Objects | - | - | - | - | - |
| Credential: API key in a header | pass (1 of 1) | pass (1 of 1) | n/a | n/a | n/a |
| Credential: API key in a query | pass (1 of 1) | pass (1 of 1) | n/a | n/a | n/a |
| Credential: API key in a cookie | pass (1 of 1) | pass (1 of 1) | n/a | n/a | n/a |
| Credential: HTTP bearer | pass (2 of 2) | pass (2 of 2) | n/a | n/a | n/a |
| Credential: HTTP basic | pass (1 of 1) | pass (1 of 1) | n/a | n/a | n/a |
| Retry on 429 after Retry-After | pass (1 of 1) | n/a | n/a | n/a | n/a |
| Retry on 503 after Retry-After | pass (1 of 1) | n/a | n/a | n/a | n/a |
| Pagination to the last page | pass (1 of 1) | n/a | n/a | n/a | n/a |
| An error as a value with its status and body | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) | pass (1 of 1) |

### What each failure sent

- hey-api, shop, `path`: fetch failed (no request)
- orval, shop, `path`: path /v1/pets/a%20b/c
- swagger-typescript-api, shop, `path`: fetch failed (no request)

### The first tsc errors

None.

## Not measured

- openapi-generator (typescript-fetch): runs on Java (or Docker); this machine has neither.
- openapi-typescript-codegen: no longer developed; @hey-api/openapi-ts is the fork that continues it.
- kubb: not measured in this round: its client is a plugin (`@kubb/plugin-client`) set up in a configuration of its own, and it has the fewest weekly installs of the candidates.
