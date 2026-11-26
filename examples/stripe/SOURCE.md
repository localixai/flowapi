# Stripe's API: where the example comes from

Only `flowapi-sdk.yml` and this file are committed. The description is megabytes, too large for
git, and so is the repository generated from it (its files and bytes are measured by
`bench/sdk-repo.mjs`, with what generating and judging it costs:
[`bench/results/sdk-repo.md`](../../bench/results/sdk-repo.md)). Both are made here by the commands
of [`../README.md`](../README.md).

- **The description**, `openapi.json` once fetched: `stripe/openapi`, `openapi/spec3.json` at
  commit `5237fa08d785bc435df80fe40c2416e2848e49b7`, the `stripe` row of
  [`bench/corpus/corpus.tsv`](../../bench/corpus/corpus.tsv), which pins its sha256 and size.
  `../fetch.sh` fetches the corpus and links the file here.
- **Its licence**: MIT (the repository's `LICENSE`).
- **The config**: `flowapi config draft openapi.json`, then refined by hand; its first comment
  says what changed and why. The description declares HTTP bearer and basic itself, so the config
  gives each its variable (`STRIPE_API_KEY` for the secret key as a bearer token) and
  `flowapi config check` finds no issue.
