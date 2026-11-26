// One generator run on one description, as each tool's documentation drives it from Node: types
// only, `node run.mjs <tool> <input.json> <out.ts>`, the tool `openapi-typescript` or `hey-api`;
// or a client SDK, `node run.mjs sdk <tool> <input.json> <out-dir>` (below). `speed` and
// `sdk-compare.mjs` run it as a fresh process and measure the whole process, Node's start
// included, as a user who runs the tool pays it. `node run.mjs versions` prints each tool's
// version as JSON.
//
// - openapi-typescript: `openapiTS(url)` then `astToString(ast)`, its documented Node API; the
//   options are its defaults.
// - hey-api: `createClient` with the `@hey-api/typescript` plugin alone (no client, no SDK), into
//   a scratch folder; its `types.gen.ts` is the output. No formatter or linter runs after it.
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs'
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

// A client SDK, each tool in the mode its documentation gives for a fetch client, written into
// `out` (a new folder), with no formatter after it:
// - hey-api: `createClient` with `@hey-api/client-fetch`, `@hey-api/sdk` and `@hey-api/typescript`,
//   imports ending in `.js` (`module.extension`) so that Node's ES modules load them.
// - openapi-typescript: its types as `schema.d.ts`, for openapi-fetch's `createClient<paths>`: a
//   client by composition, the types generated and the runtime a library.
// - orval: `generate` with `client: 'fetch'`, one file, the base URL from the description.
// - swagger-typescript-api: `generateApi` with `httpClientType: 'fetch'`, one file.
const sdks = {
  'hey-api': async (input, out) => {
    const { createClient } = await import('@hey-api/openapi-ts')
    await createClient({
      input: resolve(input),
      output: { path: out, module: { extension: '.js' } },
      plugins: ['@hey-api/client-fetch', '@hey-api/sdk', '@hey-api/typescript'],
      logs: { level: 'silent' },
    })
  },
  'openapi-typescript': async (input, out) => {
    mkdirSync(out, { recursive: true })
    await openapiTypescript(input, join(out, 'schema.d.ts'))
  },
  orval: async (input, out) => {
    const { generate } = await import('orval')
    const output = { target: join(out, 'api.ts'), client: 'fetch', mode: 'single' }
    await generate({ input: { target: resolve(input) }, output: { ...output, baseUrl: { getBaseUrlFromSpecification: true } } })
  },
  'swagger-typescript-api': async (input, out) => {
    const { generateApi } = await import('swagger-typescript-api')
    await generateApi({ input: resolve(input), output: out, fileName: 'api.ts', httpClientType: 'fetch', silent: true })
  },
}

const tools = { 'openapi-typescript': openapiTypescript, 'hey-api': heyApi }
const [tool, input, out] = process.argv.slice(2)
const usage = 'usage: node run.mjs openapi-typescript|hey-api <input.json> <out.ts> | sdk <tool> <input.json> <out-dir> | versions'

if (tool === 'versions') {
  const names = ['openapi-typescript', 'openapi-fetch', '@hey-api/openapi-ts', 'orval', 'swagger-typescript-api', 'typescript']
  console.log(JSON.stringify(Object.fromEntries(names.map((name) => [name, version(name)]))))
} else if (tool === 'sdk') {
  const [, which, from, into] = process.argv.slice(2)
  if (!sdks[which] || !from || !into) {
    console.error(usage)
    process.exit(2)
  }
  await sdks[which](from, into)
} else if (!tools[tool] || !input || !out) {
  console.error(usage)
  process.exit(2)
} else {
  await tools[tool](input, out)
}
