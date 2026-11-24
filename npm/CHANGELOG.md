# @localix-ai/flowapi

## 0.1.0

- First version: flowapi's generator compiled to WebAssembly, with `generate`, `load`,
  `formatDiagnostic`, `@localix-ai/flowapi/node` (`generateFile`, `readDescription`) and the
  `flowapi` command. JSON only.
- The `sdk` option and the command's `--sdk`: each operation's request and response functions
  after the types, with no HTTP library, byte for byte what the Rust command prints. Swagger 2.0
  descriptions are read too.
