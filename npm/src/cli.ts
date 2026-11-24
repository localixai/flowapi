#!/usr/bin/env node
// The `flowapi` command of the npm package: the Rust command's flags, output and exit codes
// (crates/flowapi-cli/src/main.rs), over the WebAssembly module. JSON only in this release.

import { readFileSync, writeFileSync } from 'node:fs'
import { formatDiagnostic, load } from './index.ts'
import { readDescription } from './node.ts'

const HELP = `flowapi: types from an OpenAPI description or a JSON Schema

Usage: flowapi <description> [-o <types.ts>] [--sdk] [--quiet]

  -o, --output <file>   where to write the types (standard output by default)
      --sdk             also a function per operation that writes its request and one that
                        reads its response, with no HTTP library and no I/O
  -q, --quiet           print errors only, not warnings
  -h, --help            this help
  -V, --version         the version

The description is Swagger 2.0, OpenAPI 3.0, 3.1 or 3.2, or a JSON Schema (2020-12 when it names no
$schema), in JSON. Files its $refs name by a relative path are read too. YAML is read by the
flowapi command built from Rust; this package reads JSON only.`

interface Generate {
  readonly input: string
  readonly output: string | undefined
  readonly quiet: boolean
  readonly sdk: boolean
}

type Command = 'help' | 'version' | Generate

function parse(args: readonly string[]): Command {
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

class Usage extends Error {}

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
