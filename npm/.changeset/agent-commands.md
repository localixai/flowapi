---
"@localix-ai/flowapi": minor
---

The command runs the Rust command's commands for an agent that writes an SDK's config,
`flowapi-sdk.yml`: `flowapi summary <description> [--tag T] [--path P] [--max-bytes N]`,
`flowapi config draft <description> [-o file]`, `flowapi config schema` and
`flowapi config check <config> [--spec D] [--json]`, with the same arguments, output and exit
codes, byte for byte. The API gives them as `summary`, `configDraft`, `configSchema` and
`configCheck` (every issue as data, as the text `config check` prints and as its `--json`), and
`loadSdk()` gives them synchronous with `configSpec`, the `spec` a config names, for the host to
read before the check. They are a second module, `flowapi-sdk.wasm` (about 314 KB gzipped),
loaded only when one of them is first used: `generate` and `references` load `flowapi.wasm` alone,
as before.
