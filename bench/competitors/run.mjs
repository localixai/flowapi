// One generator run on one description, types only, as each tool's documentation drives it from
// Node: `node run.mjs <tool> <input.json> <out.ts>`, the tool `openapi-typescript` or `hey-api`.
// `speed` runs it as a fresh process and measures the whole process, Node's start included, as a
// user who runs the tool pays it. `node run.mjs versions` prints each tool's version as JSON.
//
// - openapi-typescript: `openapiTS(url)` then `astToString(ast)`, its documented Node API; the
//   options are its defaults.
// - hey-api: `createClient` with the `@hey-api/typescript` plugin alone (no client, no SDK), into
//   a scratch folder; its `types.gen.ts` is the output. No formatter or linter runs after it.
import { mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))

function version(name) {
  const file = join(here, 'node_modules', name, 'package.json')
  return JSON.parse(readFileSync(file, 'utf8')).version
}

async function openapiTypescript(input, out) {
  const { default: openapiTS, astToString } = await import('openapi-typescript')
  const ast = await openapiTS(pathToFileURL(resolve(input)))
  writeFileSync(out, astToString(ast))
}

async function heyApi(input, out) {
  const { createClient } = await import('@hey-api/openapi-ts')
  const scratch = mkdtempSync(join(tmpdir(), 'flowapi-bench-hey-api-'))
  try {
    await createClient({
      input: resolve(input),
      output: { path: scratch },
      plugins: ['@hey-api/typescript'],
      logs: { level: 'silent' },
    })
    renameSync(join(scratch, 'types.gen.ts'), out)
  } finally {
    rmSync(scratch, { recursive: true, force: true })
  }
}

const tools = { 'openapi-typescript': openapiTypescript, 'hey-api': heyApi }
const [tool, input, out] = process.argv.slice(2)

if (tool === 'versions') {
  const names = ['openapi-typescript', '@hey-api/openapi-ts', 'typescript']
  console.log(JSON.stringify(Object.fromEntries(names.map((name) => [name, version(name)]))))
} else if (!tools[tool] || !input || !out) {
  console.error('usage: node run.mjs openapi-typescript|hey-api <input.json> <out.ts> | versions')
  process.exit(2)
} else {
  await tools[tool](input, out)
}
