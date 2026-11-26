#!/usr/bin/env node
// The `flowapi` command of the npm package: the Rust command's flags, output and exit codes
// (crates/flowapi-cli/src/main.rs), over the WebAssembly module; JSON and YAML, read alike. The
// commands about an SDK's config, `summary` and `config`, are in `sdk-cli.ts`, and `generate`, an
// SDK's whole repository, in `generate-cli.ts`.

import { readFileSync, writeFileSync } from 'node:fs'
import { formatDiagnostic, load } from './index.ts'
import { readDescription } from './node.ts'
import { parseGenerate, runGenerate, type Generate as Repo } from './generate-cli.ts'
import { parseConfig, parseSummary, run, Usage, type Sdk } from './sdk-cli.ts'

const HELP = `flowapi: types from an OpenAPI description or a JSON Schema

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
  config check    every issue of a config against its description (its \`spec\`, relative to the
                  config, or --spec): text, or JSON with --json; exits 1 on an error
  generate        the SDK's repository into --out: the client, package.json, scripts, tests
                  against a mock server, README.md, api.md, and .flowapi/lock.json, the hash of
                  each file written. A generated file changed by hand since (src/lib/ and
                  examples/ are yours) is a conflict: nothing is written and it exits 1. A file
                  no longer generated is deleted unless changed; one the lock does not list is
                  never touched. --check writes nothing and exits 1 when the output differs

A description named \`summary\`, \`config\` or \`generate\` is read by its path: \`flowapi ./summary\`.`

interface Generate {
  readonly input: string
  readonly output: string | undefined
  readonly quiet: boolean
  readonly sdk: boolean
}

type Command = 'help' | 'version' | Generate | Sdk | Repo

function parse(args: readonly string[]): Command {
  if (args[0] === 'summary') return parseSummary(args.slice(1))
  if (args[0] === 'config') return parseConfig(args.slice(1))
  if (args[0] === 'generate') return parseGenerate(args.slice(1))
  let input: string | undefined
  let output: string | undefined
  let quiet = false
  let sdk = false
  for (let i = 0; i < args.length; i++) {
    const arg = args[i] as string
    if (arg === '-h' || arg === '--help') return 'help'
    if (arg === '-V' || arg === '--version') return 'version'
    if (arg === '-q' || arg === '--quiet') quiet = true
    else if (arg === '--sdk') sdk = true
    else if (arg === '-o' || arg === '--output') {
      output = args[++i]
      if (output === undefined) throw new Usage('`-o` needs a file')
    } else if (arg.startsWith('-') && arg !== '-') throw new Usage(`unknown flag \`${arg}\``)
    else if (input !== undefined) throw new Usage('one description at a time')
    else input = arg
  }
  if (input === undefined) throw new Usage('a description to read')
  return { input, output, quiet, sdk }
}

function version(): string {
  const manifest = readFileSync(new URL('../package.json', import.meta.url), 'utf8')
  return (JSON.parse(manifest) as { version: string }).version
}

async function main(args: readonly string[]): Promise<number> {
  let command: Command
  try {
    command = parse(args)
  } catch (error) {
    if (!(error instanceof Usage)) throw error
    process.stderr.write(`flowapi: ${error.message}\n\n${HELP}\n`)
    return 2
  }
  if (command === 'help') {
    process.stdout.write(`${HELP}\n`)
    return 0
  }
  if (command === 'version') {
    process.stdout.write(`flowapi ${version()}\n`)
    return 0
  }
  if ('kind' in command) return command.kind === 'generate' ? runGenerate(command) : run(command)
  let documents
  try {
    documents = await readDescription(command.input)
  } catch (error) {
    process.stderr.write(`flowapi: ${error instanceof Error ? error.message : String(error)}\n`)
    return 1
  }
  const flowapi = await load()
  const result = flowapi.generate(documents, { output: command.output ?? '-', sdk: command.sdk })
  for (const d of result.diagnostics) {
    if (d.severity === 'error' || !command.quiet) process.stderr.write(`${formatDiagnostic(d)}\n`)
  }
  if (result.files.length === 0) return 1
  for (const file of result.files) {
    if (file.path === '-') {
      process.stdout.write(file.text)
      continue
    }
    try {
      writeFileSync(file.path, file.text)
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error)
      process.stderr.write(`flowapi: cannot write ${file.path}: ${reason}\n`)
      return 1
    }
  }
  return 0
}

process.exitCode = await main(process.argv.slice(2))
