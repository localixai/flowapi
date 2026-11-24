# flowapi against the other generators

2026-11-24, flowapi 0.1.0 (no commit yet, with changes not committed), rustc 1.100.0 (0ef1bc386 2026-11-09), on Intel(R) N150, 4 cores, 16.5 GB, Ubuntu 24.04.5 LTS, Linux 7.0.0-43-generic.
Against openapi-typescript 7.13.0 and @hey-api/openapi-ts 0.99.0 (its `@hey-api/typescript` plugin alone) with TypeScript 5.9.3, on Node.js v22.23.5.
Each tool is run as a fresh process, the whole process timed; the time is the median of 5 rounds. Memory is the process's peak resident set. Outputs are type-checked by TypeScript 7.0.2 as `tsc --ignoreConfig --strict --exactOptionalPropertyTypes --noEmit --target es2022 --skipLibCheck`.

## Time

| Description | Size | `flowapi` | `openapi-typescript` | `hey-api` | flowapi faster than the fastest other by |
| --- | ---: | ---: | ---: | ---: | ---: |
| slack | 999 KB | 14 ms | 892 ms | 992 ms | 65.2x |
| twilio | 1.4 MB | 17 ms | 923 ms | 1.13 s | 55.2x |
| digitalocean | 1.9 MB | 50 ms | failed | failed | - |
| stripe | 8.3 MB | 114 ms | 2.44 s | 10.57 s | 21.5x |
| github | 14.3 MB | 185 ms | 3.73 s | 4.49 s | 20.2x |
| cloudflare | 29.8 MB | 435 ms | 7.96 s | 12.15 s | 18.3x |

## Each description

| Description | Tool | Time | Peak memory | Output | Type-checks |
| --- | --- | ---: | ---: | ---: | --- |
| slack | `flowapi` | 14 ms | 7 MB | 261 KB | yes |
| slack | `openapi-typescript` | 892 ms | 181 MB | 464 KB | yes |
| slack | `hey-api` | 992 ms | 193 MB | 308 KB | yes |
| twilio | `flowapi` | 17 ms | 8 MB | 456 KB | yes |
| twilio | `openapi-typescript` | 923 ms | 178 MB | 823 KB | yes |
| twilio | `hey-api` | 1.13 s | 198 MB | 621 KB | yes |
| digitalocean | `flowapi` | 50 ms | 12 MB | 707 KB | yes |
| digitalocean | `openapi-typescript` | failed | | | |
| digitalocean | `hey-api` | failed | | | |
| stripe | `flowapi` | 114 ms | 42 MB | 2.9 MB | yes |
| stripe | `openapi-typescript` | 2.44 s | 330 MB | 4.6 MB | yes |
| stripe | `hey-api` | 10.57 s | 642 MB | 3.5 MB | yes |
| github | `flowapi` | 185 ms | 62 MB | 3.2 MB | yes |
| github | `openapi-typescript` | 3.73 s | 470 MB | 7.4 MB | yes |
| github | `hey-api` | 4.49 s | 484 MB | 4.5 MB | yes |
| cloudflare | `flowapi` | 435 ms | 140 MB | 8.1 MB | yes |
| cloudflare | `openapi-typescript` | 7.96 s | 937 MB | 16.2 MB | no (5 errors) |
| cloudflare | `hey-api` | 12.15 s | 1027 MB | 10.3 MB | yes |

## What failed

- digitalocean, `openapi-typescript`: exited with 1: ✘  Can't resolve $ref: ENOENT: no such file or directory 'bench/data/models/domain_record_types.yml' at #/paths/~1v2~1domains~1{domain_name}~1records/post/requestBody/content/application~1json/schema/discriminator/mapping/A
✘  Can't resolve $ref: ENOENT: no such file or directory 'bench/data/models/domain_record_types.yml' at #/paths/~1v2~1domains~1{domain_name}~1records/post/requestBody/content/application~1json/schema/discriminator/mapping/AAAA
✘  Can't resolve $ref: ENOENT: no such file or directory 'bench/data/models/domain_record_types.yml' at #/paths/~1v2~1domains~1{domain_name}~1records/post/requestBody/content/application~1json/schema/discriminator/mapping/CAA
✘  Can't resolve $ref: ENOENT: no such file or directory 'bench/data/models/domain_record_types.yml' at #/paths/~1v2~1domains~1{domain_name}~1records/post/requestBody/content/application~1json/schema/discriminator/mapping/CNAME
- digitalocean, `hey-api`: exited with 1: if (error instanceof Error) throw new JobError("", {
JobError
originalError: {
error: Error: Symbol finalName has not been resolved yet for ~ Symbol#54 ""
- cloudflare, `openapi-typescript`, type errors: data/out/openapi-typescript/cloudflare.ts(70987,9): error TS2502: 'brapi_CrawlJsonValue' is referenced directly or indirectly in its own type annotation.
