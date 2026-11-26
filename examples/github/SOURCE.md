# GitHub's REST API: where the example comes from

Only `flowapi-sdk.yml` and this file are committed. The description is megabytes, too large for
git, and so is the repository generated from it (its files and bytes are measured by
`bench/sdk-repo.mjs`, with what generating and judging it costs:
[`bench/results/sdk-repo.md`](../../bench/results/sdk-repo.md)). Both are made here by the commands
of [`../README.md`](../README.md).

- **The description**, `openapi.json` once fetched: `github/rest-api-description`,
  `descriptions/api.github.com/api.github.com.json` at commit
  `c9c95fe34ced05fe076001f5c55704742bc8e7f8`, the `github` row of
  [`bench/corpus/corpus.tsv`](../../bench/corpus/corpus.tsv), which pins its sha256 and size.
  `../fetch.sh` fetches the corpus and links the file here.
- **Its licence**: MIT (the repository's `LICENSE.md`, and the description's `info.license`).
- **The config**: `flowapi config draft openapi.json`, then refined by hand; its first comment
  says what changed and why. The description declares no security scheme, so the draft warns
  `config.auth.none`; the config declares the bearer token GitHub's documentation describes
  (`GITHUB_TOKEN`), optional for every operation, and `flowapi config check` then finds no issue.
  That scheme is the config's, not the description's.
