// `flowapi generate <flowapi-sdk.yml> --out <dir> [--check]`: an SDK's repository written, as the
// Rust command writes it (crates/flowapi-cli/src/args.rs and generate.rs): the same arguments,
// files, lock, messages and exit codes. The config and its description are read here; the module
// prints the repository (`repoFiles`) and decides what it changes on disk (`repoPlan`), and
// `generate-disk.ts` reads the disk for it and makes the changes.

import { readFile } from 'node:fs/promises'
import { formatDiagnostic, LOCK_PATH, loadSdk, type FlowapiSdk } from './index.ts'
import { readDescription } from './node.ts'
import { apply, readDisk, reason, Refused } from './generate-disk.ts'
import { join, parent } from './paths.ts'
import { Usage } from './sdk-cli.ts'
import type { Diagnostic, Document, LockFile } from './types.ts'

/** `generate`, as `args.rs` reads it. */
export interface Generate {
  readonly kind: 'generate'
  readonly config: string
  readonly out: string
  readonly check: boolean
}

/** `generate <flowapi-sdk.yml> --out <dir> [--check]`; `'help'` for `-h`. */
export function parseGenerate(args: readonly string[]): Generate | 'help' {
  let config: string | undefined
  let out: string | undefined
  let check = false
  for (let i = 0; i < args.length; i++) {
    const arg = args[i] as string
    if (arg === '-h' || arg === '--help') return 'help'
    if (arg === '-o' || arg === '--out') {
      out = args[++i]
      if (out === undefined) throw new Usage('`--out` needs a directory')
    } else if (arg === '--check') check = true
    else if (arg.startsWith('-')) throw new Usage(`unknown flag \`${arg}\` for \`generate\``)
    else if (config !== undefined) throw new Usage('one config at a time')
    else config = arg
  }
  if (config === undefined) throw new Usage('`generate` needs a config')
  if (out === undefined) throw new Usage('`generate` needs `--out <dir>`')
  return { kind: 'generate', config, out, check }
}

const utf8 = new TextDecoder('utf-8', { fatal: true })

/** A file's text, read as Rust's `read_to_string` reads it: not UTF-8 is an error worded so. */
async function text(path: string): Promise<string> {
  const bytes = await readFile(path)
  try {
    return utf8.decode(bytes)
  } catch {
    throw new Error('stream did not contain valid UTF-8')
  }
}

/** The previous lock, absent, or the message of one that cannot be read. */
async function previous(path: string): Promise<LockFile> {
  try {
    return { path, text: await text(path) }
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return { path }
    return { path, error: `flowapi: cannot read ${path}: ${reason(error)}` }
  }
}

/** The first document or config that could not be read, as the command prints it. */
function unread(diagnostics: readonly Diagnostic[]): void {
  const first = diagnostics[0]
  if (first !== undefined) throw new Refused(formatDiagnostic(first))
}

/** Runs `generate`; its exit code. Every message goes to standard error, as the command's. */
export async function runGenerate(command: Generate): Promise<number> {
  try {
    return await generate(await loadSdk(), command)
  } catch (error) {
    if (!(error instanceof Refused)) throw error
    process.stderr.write(`${error.message}\n`)
    return 1
  }
}

async function generate(flowapi: FlowapiSdk, { config, out, check }: Generate): Promise<number> {
  let configText: string
  try {
    configText = await text(config)
  } catch (error) {
    throw new Refused(`flowapi: cannot read ${config}: ${reason(error)}`)
  }
  const doc: Document = { uri: config, text: configText }
  const named = flowapi.configSpec(doc)
  unread(named.diagnostics)
  const spec = named.text
  if (spec === null) {
    process.stderr.write(flowapi.configCheck(doc).text ?? '')
    throw new Refused(`flowapi: ${config} names no description (\`spec\`)`)
  }
  if (spec.includes('://')) {
    throw new Refused(
      `flowapi: the config's spec is a URL (${spec}), which generate does not fetch: download it and name the file`,
    )
  }
  let documents: Document[]
  try {
    documents = await readDescription(join(parent(config), spec))
  } catch (error) {
    throw new Refused(`flowapi: ${reason(error)}`)
  }
  const lock = await previous(join(out, LOCK_PATH))
  const repo = flowapi.repoFiles(doc, documents, lock)
  unread(repo.diagnostics)
  process.stderr.write(repo.text)
  if (repo.error !== null || repo.lock === null) throw new Refused(repo.error ?? '')
  const disk = readDisk(out, repo.read)
  const files = repo.files.map(({ path, owner, executable }) => ({ path, owner, executable }))
  const input = { out, check, files, lock: repo.lock, previous: lock.text ?? null, disk }
  const plan = flowapi.repoPlan(input)
  unread(plan.diagnostics)
  process.stderr.write(plan.text)
  if (plan.next === 'done') return 0
  if (plan.next === 'failed') return 1
  const texts = new Map(repo.files.map((f) => [f.path, f.text]))
  apply(out, plan.changes, (path) => texts.get(path) ?? plan.lock)
  process.stderr.write(`${plan.summary}\n`)
  return 0
}
