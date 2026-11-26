// An SDK generated, measured and built the same way for every generator: each generation a fresh
// process into an empty folder, timed with its peak memory by the bench's `measure` (the kernel's
// account, as `speed` takes it); what it wrote counted; its TypeScript checked by the conformance
// harness's compiler under `--strict`; then emitted as ES modules for the behavior tests.
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, symlinkSync, writeFileSync } from 'node:fs'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
export const root = resolve(here, '../../..')
export const flowapi = join(root, 'target/release/flowapi')
export const measureBin = join(root, 'bench/target/release/measure')
export const tsc = join(root, 'conformance/ts/node_modules/.bin/tsc')
const run = join(here, '../run.mjs')

/** The flags every SDK is checked with; `lib` has the DOM's for `fetch`, `Request` and `Response`. */
export const TSC_FLAGS = ['--ignoreConfig', '--strict', '--target', 'es2022', '--module', 'nodenext', '--lib', 'es2022,dom', '--skipLibCheck']

/** One fresh process measured: `{ ok, ms, peakRssKb, failure }`. */
export function measure(log, timeout, program, args) {
  const line = execFileSync(measureBin, [log, String(timeout), '--', program, ...args], { encoding: 'utf8' })
  return JSON.parse(line)
}

/** The command that generates `tool`'s SDK of the case into `out`. */
export function command(tool, work, out) {
  if (tool === 'flowapi') return [flowapi, ['generate', join(work, 'flowapi-sdk.yml'), '--out', out]]
  return ['node', [run, 'sdk', tool, join(work, 'spec.json'), out]]
}

/** Every file under `dir`, as paths relative to it. */
export function walk(dir, at = dir) {
  if (!existsSync(at)) return []
  return readdirSync(at, { withFileTypes: true }).flatMap((entry) => {
    const path = join(at, entry.name)
    if (entry.isSymbolicLink()) return []
    return entry.isDirectory() ? walk(dir, path) : [relative(dir, path)]
  })
}

/** What a generator wrote: files and bytes, and those of its TypeScript. */
export function count(dir) {
  const files = walk(dir)
  const size = (list) => list.reduce((sum, file) => sum + statSync(join(dir, file)).size, 0)
  const ts = files.filter((file) => /\.ts$/.test(file))
  return { files: files.length, bytes: size(files), tsFiles: ts.length, tsBytes: size(ts) }
}

/** The TypeScript the SDK is made of, the folder it is rooted at, and where it is emitted. */
export function sources(tool, dir) {
  const base = tool === 'flowapi' ? join(dir, 'src') : dir
  const files = walk(base)
    .filter((file) => /\.ts$/.test(file) && !file.startsWith('js/'))
    .map((file) => join(base, file))
  return { base, files, out: join(dir, 'js') }
}

/**
 * Made ready to load: an ES module package where the generator wrote none, the competitors'
 * node_modules reachable, and for openapi-typescript the few lines a user writes to compose its
 * types with openapi-fetch (checked with them, not counted as generated).
 */
export function prepare(tool, dir) {
  if (tool !== 'flowapi') writeFileSync(join(dir, 'package.json'), '{ "type": "module" }\n')
  const modules = join(dir, 'node_modules')
  if (!existsSync(modules)) symlinkSync(join(here, '../node_modules'), modules)
  if (tool === 'openapi-typescript') {
    const client = [
      "import createClient from 'openapi-fetch'",
      "import type { paths } from './schema.js'",
      '',
      'export const client = (baseUrl: string) => createClient<paths>({ baseUrl })',
      '',
    ]
    writeFileSync(join(dir, 'client.ts'), client.join('\n'))
  }
}

/** `tsc --strict --noEmit` on the SDK: the errors counted and the first few kept. */
export function typeCheck(tool, dir, timeout) {
  const { files } = sources(tool, dir)
  const log = join(dir, '..', `${tool}.tsc.log`)
  const run = measure(log, timeout, tsc, [...TSC_FLAGS, '--noEmit', ...files])
  const text = existsSync(log) ? readFileSync(log, 'utf8') : ''
  const errors = text.split('\n').filter((line) => line.includes('error TS'))
  const sample = errors.slice(0, 3).map((line) => line.replace(`${dir}/`, '').slice(0, 200))
  return { ms: run.ms, peakRssKb: run.peakRssKb, errors: errors.length, sample, finished: run.failure === '' || /exited/.test(run.failure) }
}

/** The SDK emitted as JavaScript, types not checked again (`--noCheck`). */
export function emit(tool, dir, timeout) {
  const { base, files, out } = sources(tool, dir)
  rmSync(out, { recursive: true, force: true })
  mkdirSync(out, { recursive: true })
  const log = join(dir, '..', `${tool}.emit.log`)
  return measure(log, timeout, tsc, [...TSC_FLAGS, '--noCheck', '--rootDir', base, '--outDir', out, ...files])
}
