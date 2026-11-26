<!-- generated from crates/flowapi-cli/src/args.rs and npm/src/cli.ts by plugins/flowapi/scripts/docs.mjs: edit the source, not this file -->

# The `flowapi` command

## The Rust command

```text
flowapi: types from an OpenAPI description or a JSON Schema

Usage: flowapi <description> [-o <types.ts>] [--target ts|rust] [--sdk] [--quiet]
       flowapi summary <description> [--tag <tag>] [--path <prefix>] [--max-bytes <n>]
       flowapi config draft <description> [-o <flowapi-sdk.yml>]
       flowapi config schema
       flowapi config check <flowapi-sdk.yml> [--spec <description>] [--json]
       flowapi generate <flowapi-sdk.yml> --out <dir> [--check]

  -o, --output <file>   where to write the types (standard output by default)
  -t, --target <lang>   the language of the types: ts (the default) or rust (serde)
      --sdk             also a function per operation that writes its request and one that
                        reads its response, with no HTTP library and no I/O (Rust: serde and
                        serde_json only, as the types)
  -q, --quiet           print errors only, not warnings
  -h, --help            this help
  -V, --version         the version

The description is Swagger 2.0, OpenAPI 3.0, 3.1 or 3.2, or a JSON Schema (2020-12 when it names no
$schema), in JSON or YAML (.yaml, .yml, or any text not opening with { or [). Files its
$refs name by a relative path are read too.

For an SDK's config, flowapi-sdk.yml, which an agent writes:
  summary         the description in compact JSON: operations with their parameters, media
                  types and the pagination they look like they have; over --max-bytes (60000)
                  the tags alone, to be read on by --tag or --path
  config draft    a first config, every operation in a resource, as YAML
  config schema   the config's format, as JSON Schema
  config check    every issue of a config against its description (its `spec`, relative to the
                  config, or --spec): text, or JSON with --json; exits 1 on an error
  generate        the SDK's repository into --out: the client, package.json, scripts, tests
                  against a mock server, README.md, api.md, and .flowapi/lock.json, the hash of
                  each file written. A generated file changed by hand since (src/lib/ and
                  examples/ are yours) is a conflict: nothing is written and it exits 1. A file
                  no longer generated is deleted unless changed; one the lock does not list is
                  never touched. --check writes nothing and exits 1 when the output differs

A description named `summary`, `config` or `generate` is read by its path: `flowapi ./summary`.
```

## The npm command

```text
flowapi: types from an OpenAPI description or a JSON Schema

Usage: flowapi <description> [-o <types.ts>] [--sdk] [--quiet]
       flowapi summary <description> [--tag <tag>] [--path <prefix>] [--max-bytes <n>]
       flowapi config draft <description> [-o <flowapi-sdk.yml>]
       flowapi config schema
       flowapi config check <flowapi-sdk.yml> [--spec <description>] [--json]
       flowapi generate <flowapi-sdk.yml> --out <dir> [--check]

  -o, --output <file>   where to write the types (standard output by default)
      --sdk             also a function per operation that writes its request and one that
                        reads its response, with no HTTP library and no I/O
  -q, --quiet           print errors only, not warnings
  -h, --help            this help
  -V, --version         the version

The description is Swagger 2.0, OpenAPI 3.0, 3.1 or 3.2, or a JSON Schema (2020-12 when it names no
$schema), in JSON or YAML (.yaml, .yml, or any text not opening with { or [). Files its
$refs name by a relative path are read too.

For an SDK's config, flowapi-sdk.yml, which an agent writes:
  summary         the description in compact JSON: operations with their parameters, media
                  types and the pagination they look like they have; over --max-bytes (60000)
                  the tags alone, to be read on by --tag or --path
  config draft    a first config, every operation in a resource, as YAML
  config schema   the config's format, as JSON Schema
  config check    every issue of a config against its description (its `spec`, relative to the
                  config, or --spec): text, or JSON with --json; exits 1 on an error
  generate        the SDK's repository into --out: the client, package.json, scripts, tests
                  against a mock server, README.md, api.md, and .flowapi/lock.json, the hash of
                  each file written. A generated file changed by hand since (src/lib/ and
                  examples/ are yours) is a conflict: nothing is written and it exits 1. A file
                  no longer generated is deleted unless changed; one the lock does not list is
                  never touched. --check writes nothing and exits 1 when the output differs

A description named `summary`, `config` or `generate` is read by its path: `flowapi ./summary`.
```
