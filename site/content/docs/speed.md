# Speed

How long flowapi takes on real API descriptions, and how much memory it uses, beside openapi-typescript and @hey-api/openapi-ts.

## How it was measured

Each tool ran as a fresh process, the whole process timed, five rounds each, on an Intel N150 with 4 cores under Ubuntu 24.04. A time is the median of the five rounds; memory is the process's peak resident set. Every output was type-checked by TypeScript 7.0.2 with `--strict` and `--exactOptionalPropertyTypes`. The versions were openapi-typescript 7.13.0 and @hey-api/openapi-ts 0.99.0 (its `@hey-api/typescript` plugin alone), on Node.js 22.

## Time

| Description | Size | flowapi | openapi-typescript | @hey-api/openapi-ts |
| --- | ---: | ---: | ---: | ---: |
| Slack | 999 KB | **14 ms** | 892 ms | 992 ms |
| Twilio | 1.4 MB | **17 ms** | 923 ms | 1.13 s |
| DigitalOcean | 1.9 MB | **50 ms** | failed | failed |
| Stripe | 8.3 MB | **114 ms** | 2.44 s | 10.57 s |
| GitHub | 14.3 MB | **185 ms** | 3.73 s | 4.49 s |
| Cloudflare | 29.8 MB | **435 ms** | 7.96 s | 12.15 s |

## Memory and output

| Description | flowapi | openapi-typescript | @hey-api/openapi-ts |
| --- | ---: | ---: | ---: |
| Stripe | **42 MB** | 330 MB | 642 MB |
| GitHub | **62 MB** | 470 MB | 484 MB |
| Cloudflare | **140 MB** | 937 MB | 1027 MB |

flowapi's output type-checks for all six descriptions. openapi-typescript's output for Cloudflare's description has 5 type errors, and both other tools fail on DigitalOcean's.

## Where the numbers live

Every table here is copied from [bench/results/speed.md](https://github.com/localixai/flowapi/blob/main/bench/results/speed.md), which `bench/run.sh` writes. The limits flowapi's own times are held to are in [bench/budgets.json](https://github.com/localixai/flowapi/blob/main/bench/budgets.json). The other tools are pinned in `bench/competitors/`, so the run can be repeated.
