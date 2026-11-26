// The command's commands about an SDK's config, `summary` and `config draft|schema|check`: the
// Rust command's arguments, output and exit codes (crates/flowapi-cli/src/args.rs and sdk.rs),
// over the module. Paths are joined and made relative as Rust's `Path` does, so the URIs a
// diagnostic names and the `spec` a draft writes are the Rust command's.

import { realpathSync, writeFileSync } from 'node:fs'
import { readFile } from 'node:fs/promises'
import { formatDiagnostic, loadSdk, type FlowapiSdk } from './index.ts'
import { readDescription } from './node.ts'
import type { Diagnostic, Document, Output, SummaryFilter } from './types.ts'

/** A command line the command cannot run: exit 2, with the help. */
export class Usage extends Error {}

/** A command about an SDK's config, as `args.rs` reads it. */
export type Sdk =
  | { readonly kind: 'summary'; readonly input: string; readonly filter: SummaryFilter }
  | { readonly kind: 'draft'; readonly input: string; readonly output: string | undefined }
  | { readonly kind: 'schema' }
  | {
      readonly kind: 'check'
      readonly config: string
      readonly spec: string | undefined
      readonly json: boolean
    }

/** `summary <description> [--tag T] [--path P] [--max-bytes N]`; `'help'` for `-h`. */
export function parseSummary(args: readonly string[]): Sdk | 'help' {
  let input: string | undefined
  const filter: { tag?: string; path?: string; maxBytes?: number } = {}
  for (let i = 0; i < args.length; i++) {
    const arg = args[i] as string
    if (arg === '-h' || arg === '--help') return 'help'
    if (arg === '--tag') filter.tag = value(args[++i], '`--tag` needs a tag')
    else if (arg === '--path') filter.path = value(args[++i], "`--path` needs a path's start")
    else if (arg === '--max-bytes') {
      filter.maxBytes = bytes(value(args[++i], '`--max-bytes` needs a number'))
    } else if (arg.startsWith('-')) throw new Usage(`unknown flag \`${arg}\``)
    else if (input !== undefined) throw new Usage('one description at a time')
    else input = arg
  }
  if (input === undefined) throw new Usage('a description to summarize')
  return { kind: 'summary', input, filter }
}

/** `config draft|schema|check …`; `'help'` for `-h`. */
export function parseConfig(args: readonly string[]): Sdk | 'help' {
  const what = value(args[0], '`config` is followed by `draft`, `schema` or `check`')
  let path: string | undefined
  let output: string | undefined
  let spec: string | undefined
  let json = false
  for (let i = 1; i < args.length; i++) {
    const arg = args[i] as string
    if (arg === '-h' || arg === '--help') return 'help'
    if ((arg === '-o' || arg === '--output') && what === 'draft') {
      output = value(args[++i], '`-o` needs a file')
    } else if (arg === '--spec' && what === 'check') {
      spec = value(args[++i], '`--spec` needs a description')
    } else if (arg === '--json' && what === 'check') json = true
    else if (arg.startsWith('-')) {
      throw new Usage(`unknown flag \`${arg}\` for \`config ${what}\``)
    }
    else if (path !== undefined) throw new Usage('one file at a time')
    else path = arg
  }
  if (what === 'schema' && path === undefined) return { kind: 'schema' }
  if (what === 'draft' && path !== undefined) return { kind: 'draft', input: path, output }
  if (what === 'check' && path !== undefined) return { kind: 'check', config: path, spec, json }
  if (what === 'schema') throw new Usage('`config schema` reads no file')
  if (what === 'draft') throw new Usage('`config draft` needs a description')
  if (what === 'check') throw new Usage('`config check` needs a config')
  throw new Usage(`\`config ${what}\`: the commands are \`draft\`, \`schema\` and \`check\``)
}

function value(arg: string | undefined, missing: string): string {
  if (arg === undefined) throw new Usage(missing)
  return arg
}

/** A `usize` as Rust parses one, its errors worded so; past 2^53 no bound binds anyway. */
function bytes(n: string): number {
  const digits = n.startsWith('+') ? n.slice(1) : n
  if (n === '') throw new Usage(`\`--max-bytes ${n}\`: cannot parse integer from empty string`)
  if (!/^[0-9]+$/.test(digits)) {
    throw new Usage(`\`--max-bytes ${n}\`: invalid digit found in string`)
  }
  const big = BigInt(digits)
  if (big > 0xffff_ffff_ffff_ffffn) {
    throw new Usage(`\`--max-bytes ${n}\`: number too large to fit in target type`)
  }
  return big > BigInt(Number.MAX_SAFE_INTEGER) ? Number.MAX_SAFE_INTEGER : Number(big)
}

/** A failure the command prints on standard error before it exits with 1. */
class Failed extends Error {}

/** Runs one command; its exit code. */
export async function run(command: Sdk): Promise<number> {
  try {
    // The second module, only for these commands; `readDescription` reads with the first.
    const flowapi = await loadSdk()
    switch (command.kind) {
      case 'summary': {
        const docs = await documents(command.input)
        write(described(flowapi.summary(docs, command.filter), command.input), undefined)
        return 0
      }
      case 'draft': {
        const docs = await documents(command.input)
        const { input, output } = command
        const spec = output === undefined ? input : relative(input, output)
        write(described(flowapi.configDraft(docs, spec), command.input), command.output)
        return 0
      }
      case 'schema':
        write(flowapi.configSchema(), undefined)
        return 0
      case 'check':
        return await check(flowapi, command.config, command.spec, command.json)
    }
  } catch (error) {
    if (!(error instanceof Failed)) throw error
    process.stderr.write(`${error.message}\n`)
    return 1
  }
}

/** `config check`: the config read first, its description found (`--spec`, else its `spec`
 * relative to the config), every issue printed; 1 when one is an error. */
async function check(
  flowapi: FlowapiSdk,
  path: string,
  given: string | undefined,
  json: boolean,
): Promise<number> {
  const config: Document = { uri: path, text: await read(path) }
  const named = flowapi.configSpec(config)
  unread(named.diagnostics)
  const spec = given ?? named.text ?? undefined
  if (spec?.includes('://')) {
    throw new Failed(
      `flowapi: the config's spec is a URL (${spec}), which this command does not fetch: name a file with --spec`,
    )
  }
  // No spec, or an empty one, is a config checked against nothing, as the Rust command checks it.
  const at = spec === undefined ? '' : (given ?? join(parent(path), spec))
  const docs = at === '' ? undefined : await documents(at)
  const found = flowapi.configCheck(config, docs)
  unread(found.diagnostics)
  if (json) process.stdout.write(found.json ?? '')
  else if (found.issues.length === 0) process.stdout.write(`${path}: no issues\n`)
  else process.stdout.write(found.text ?? '')
  return found.ok ? 0 : 1
}

/** A description and the files its references reach; a file that cannot be read fails. */
async function documents(path: string): Promise<Document[]> {
  try {
    return await readDescription(path)
  } catch (error) {
    throw new Failed(`flowapi: ${reason(error)}`)
  }
}

async function read(path: string): Promise<string> {
  try {
    return await readFile(path, 'utf8')
  } catch (error) {
    throw new Failed(`flowapi: cannot read ${path}: ${reason(error)}`)
  }
}

/** The first document or config that could not be read, as the command prints it. */
function unread(diagnostics: readonly Diagnostic[]): void {
  const first = diagnostics[0]
  if (first !== undefined) throw new Failed(formatDiagnostic(first))
}

function reason(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** A summary's or a draft's text; a description that is not one fails. */
function described(output: Output, input: string): string {
  unread(output.diagnostics)
  if (output.text === null) {
    throw new Failed(`flowapi: ${input} is not an OpenAPI or Swagger description`)
  }
  return output.text
}

/** The text written to a file, or printed. */
function write(text: string, output: string | undefined): void {
  if (output === undefined || output === '-') {
    process.stdout.write(text)
    return
  }
  try {
    writeFileSync(output, text)
  } catch (error) {
    throw new Failed(`flowapi: cannot write ${output}: ${reason(error)}`)
  }
}

/** `Path::parent` as text: `''` for a bare name, `'/'` for a file at the root. */
function parent(path: string): string {
  const trimmed = path.replace(/(?<=.)\/+$/, '')
  const slash = trimmed.lastIndexOf('/')
  if (slash === -1) return ''
  const head = trimmed.slice(0, slash).replace(/\/+$/, '')
  return head === '' ? '/' : head
}

/** `Path::join`: an absolute path replaces, an empty base adds nothing. */
function join(base: string, path: string): string {
  if (path.startsWith('/') || base === '') return path
  return base.endsWith('/') ? `${base}${path}` : `${base}/${path}`
}

/** The description's path as the config written to `output` names it: relative to the
 * config's folder; absolute when they share only the root; as given when either cannot be
 * found (`relative` in sdk.rs). */
function relative(input: string, output: string): string {
  const folder = parent(output) || '.'
  let spec: string
  let dir: string
  try {
    spec = realpathSync(input)
    dir = realpathSync(folder)
  } catch {
    // Not there: the path as given, as the Rust command falls back to it.
    return input
  }
  const a = components(spec)
  const b = components(dir)
  let common = 0
  while (common < a.length && common < b.length && a[common] === b[common]) common++
  if (common <= 1) return spec
  return [...b.slice(common).map(() => '..'), ...a.slice(common)].join('/')
}

/** An absolute path's components, the root first. */
function components(path: string): string[] {
  return ['/', ...path.split('/').filter((part) => part !== '')]
}
