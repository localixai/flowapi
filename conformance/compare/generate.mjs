// One generator over every group, in this one process: `node generate.mjs <tool> <in> <out>`.
// `<in>` is a JSON list of { name, document }; `<out>` gets a list of { name, types } or
// { name, error }, in the same order. Each tool runs with its defaults, driven as
// bench/competitors/run.mjs drives it, but given the document as an object.
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const competitors = join(dirname(fileURLToPath(import.meta.url)), '../../bench/competitors')

// A package of bench/competitors, imported by its `exports` entry: this script sits outside
// that folder, so a bare import would not find it.
async function load(name) {
  const dir = join(competitors, 'node_modules', name)
  const pkg = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf8'))
  const dot = pkg.exports?.['.']
  const entry = (typeof dot === 'string' ? dot : (dot?.import ?? dot?.default)) ?? pkg.main
  return import(pathToFileURL(join(dir, entry)).href)
}

// The type the harness's `Root` is bound to: openapi-typescript keeps components in one
// interface.
async function openapiTypescript() {
  const { default: openapiTS, astToString } = await load('openapi-typescript')
  return async (document) => {
    const text = astToString(await openapiTS(document))
    return `${text}${text.endsWith('\n') ? '' : '\n'}type Root = components["schemas"]["Root"];\n`
  }
}

// hey-api exports one type per component, named from the component (`Root` for `Root`); when
// that name is taken it is renamed, so the export is looked for rather than assumed.
async function heyApi() {
  const { createClient } = await load('@hey-api/openapi-ts')
  const scratch = mkdtempSync(join(tmpdir(), 'flowapi-compare-hey-api-'))
  process.on('exit', () => rmSync(scratch, { recursive: true, force: true }))
  let n = 0
  return async (document) => {
    const out = join(scratch, String(n++))
    try {
      await createClient({
        input: document,
        output: { path: out },
        plugins: ['@hey-api/typescript'],
        logs: { level: 'silent', file: false },
      })
      const file = join(out, 'types.gen.ts')
      if (!existsSync(file)) throw new Error('no types.gen.ts written')
      const text = readFileSync(file, 'utf8')
      if (!/^export type Root\b/m.test(text)) throw new Error('no exported type `Root`')
      return text.endsWith('\n') ? text : `${text}\n`
    } finally {
      rmSync(out, { recursive: true, force: true })
    }
  }
}

const TIMEOUT_MS = 30_000

function timeout() {
  return new Promise((_, reject) => {
    const t = setTimeout(() => reject(new Error(`no result in ${TIMEOUT_MS / 1000} s`)), TIMEOUT_MS)
    t.unref()
  })
}

// The first line of an error's message; hey-api wraps a failure in a `JobError` with an empty
// message and the cause under `originalError.error`.
function message(e) {
  const inner = e?.originalError?.error ?? e
  const text = inner instanceof Error ? inner.message : String(inner)
  const first = text.split('\n')[0]
  return first || (inner instanceof Error ? inner.name : '') || 'error without a message'
}

const [tool, input, output] = process.argv.slice(2)
const tools = { 'openapi-typescript': openapiTypescript, 'hey-api': heyApi }
if (!tools[tool] || !input || !output) {
  console.error('usage: node generate.mjs openapi-typescript|hey-api <in.json> <out.json>')
  process.exit(2)
}
const generate = await tools[tool]()
const results = []
for (const { name, document } of JSON.parse(readFileSync(input, 'utf8'))) {
  try {
    results.push({ name, types: await Promise.race([generate(document), timeout()]) })
  } catch (e) {
    results.push({ name, error: message(e) })
  }
}
writeFileSync(output, JSON.stringify(results))
// A timed-out generation may still hold the event loop.
process.exit(0)
