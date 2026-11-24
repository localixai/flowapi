// What writing a request costs in the SDK flowapi generates (`flowapi --sdk`) against
// openapi-fetch's serializers (pinned in package.json), on the same operation shapes, in one
// Node process, one after the other: 100,000 requests a round, 5 rounds after a warm-up, the
// median kept. openapi-fetch is measured on what it does to make a URL and a body:
// `createFinalURL` with a query serializer made once and its default path serializer, and `defaultBodySerializer`; flowapi's on the
// whole `<op>Request` call (path, query, headers, body). Each pair is first checked to mean the
// same request: the same path and body, the same query once decoded.
// Run from the repository root after `cargo build --release`: node bench/competitors/sdk.mjs
// Writes bench/results/sdk.md.
import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { cpus, totalmem } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const root = resolve(here, '../..')
const out = join(root, 'bench/data/sdk')
mkdirSync(out, { recursive: true })
const N = 100_000
const ROUNDS = 5

// The shapes: a path parameter and a query flag; arrays and an object in the query; a path, a
// query, a header and a JSON body. Each in the style openapi-fetch writes by default (form with
// explode for arrays, deepObject for objects), so both write the same request.
const description = {
  openapi: '3.1.0',
  info: { title: 'bench', version: '1' },
  paths: {
    '/repos/{owner}/{repo}': {
      get: {
        operationId: 'getRepo',
        parameters: [
          { name: 'owner', in: 'path', required: true, schema: { type: 'string' } },
          { name: 'repo', in: 'path', required: true, schema: { type: 'string' } },
          { name: 'verbose', in: 'query', schema: { type: 'boolean' } },
        ],
        responses: { 200: { description: 'ok' } },
      },
    },
    '/search': {
      get: {
        operationId: 'search',
        parameters: [
          { name: 'tag', in: 'query', schema: { type: 'array', items: { type: 'string' } } },
          { name: 'filter', in: 'query', style: 'deepObject', schema: { type: 'object' } },
          { name: 'page', in: 'query', schema: { type: 'integer' } },
        ],
        responses: { 200: { description: 'ok' } },
      },
    },
    '/pets/{id}': {
      put: {
        operationId: 'putPet',
        parameters: [
          { name: 'id', in: 'path', required: true, schema: { type: 'integer' } },
          { name: 'dry', in: 'query', schema: { type: 'boolean' } },
          { name: 'X-Request-Id', in: 'header', schema: { type: 'string' } },
        ],
        requestBody: { content: { 'application/json': { schema: { type: 'object' } } } },
        responses: { 200: { description: 'ok' } },
      },
    },
  },
}
writeFileSync(join(out, 'bench.json'), JSON.stringify(description))
execFileSync(join(root, 'target/release/flowapi'), [join(out, 'bench.json'), '--sdk', '-q', '-o', join(out, 'bench.ts')])
execFileSync(join(root, 'conformance/ts/node_modules/.bin/tsc'), ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--target', 'es2022', '--module', 'es2022', '--skipLibCheck', '--outDir', out, join(out, 'bench.ts')])
const ours = await import(pathToFileURL(join(out, 'bench.js')).href)
const of = await import('openapi-fetch')
const query = of.createQuerySerializer()
const pet = { name: 'Rex', tags: ['a b', 'c'], age: 3 }

const shapes = [
  {
    name: 'path and a flag',
    ours: () => ours.getRepoRequest({ path: { owner: 'octo cat', repo: 'hello/world' }, query: { verbose: true } }),
    theirs: () => ({ url: of.createFinalURL('/repos/{owner}/{repo}', { baseUrl: '', params: { path: { owner: 'octo cat', repo: 'hello/world' }, query: { verbose: true } }, querySerializer: query, pathSerializer: of.defaultPathSerializer }) }),
  },
  {
    name: 'arrays and an object in the query',
    ours: () => ours.searchRequest({ query: { tag: ['red', 'big dog', 'x&y'], filter: { color: 'red', size: 'L' }, page: 2 } }),
    theirs: () => ({ url: of.createFinalURL('/search', { baseUrl: '', params: { query: { tag: ['red', 'big dog', 'x&y'], filter: { color: 'red', size: 'L' }, page: 2 } }, querySerializer: query, pathSerializer: of.defaultPathSerializer }) }),
  },
  {
    name: 'path, query, header and a JSON body',
    ours: () => ours.putPetRequest({ path: { id: 7 }, query: { dry: false }, header: { 'X-Request-Id': 'r-1' }, body: { contentType: 'application/json', value: pet } }),
    theirs: () => ({
      url: of.createFinalURL('/pets/{id}', { baseUrl: '', params: { path: { id: 7 }, query: { dry: false } }, querySerializer: query, pathSerializer: of.defaultPathSerializer }),
      headers: { 'X-Request-Id': 'r-1', 'content-type': 'application/json' },
      body: of.defaultBodySerializer(pet, {}),
    }),
  },
]

// The same request: path and body equal, the query equal once WHATWG's parser has decoded it.
for (const s of shapes) {
  const a = s.ours()
  const b = s.theirs()
  const [path, q = ''] = b.url.split('?')
  const pairs = (x) => JSON.stringify([...new URLSearchParams(x)])
  if (decodeURIComponent(a.path) !== decodeURIComponent(path) || pairs(a.query) !== pairs(q) || (a.body ?? '') !== (b.body ?? '')) {
    throw new Error(`${s.name}: not the same request\n  ours ${JSON.stringify(a)}\n  theirs ${JSON.stringify(b)}`)
  }
}

let sink = 0
function time(f) {
  const start = process.hrtime.bigint()
  for (let i = 0; i < N; i++) sink += f() === undefined ? 0 : 1
  return Number(process.hrtime.bigint() - start) / 1e6
}
const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)]
const rows = []
for (const s of shapes) {
  for (let i = 0; i < 3; i++) {
    time(s.ours)
    time(s.theirs)
  }
  const a = []
  const b = []
  for (let r = 0; r < ROUNDS; r++) {
    a.push(time(s.ours))
    b.push(time(s.theirs))
  }
  rows.push({ name: s.name, ours: median(a), theirs: median(b) })
}
if (sink !== shapes.length * N * (2 * 3 + 2 * ROUNDS)) throw new Error('a call returned nothing')

const version = JSON.parse(readFileSync(join(here, 'node_modules/openapi-fetch/package.json'), 'utf8')).version
const fmt = (ms) => `${ms.toFixed(1)} ms`
const per = (ms) => `${((ms * 1e6) / N).toFixed(0)} ns`
const lines = [
  '# Writing a request: the flowapi SDK against openapi-fetch',
  '',
  `Written by \`node bench/competitors/sdk.mjs\`; do not edit. ${new Date().toISOString().slice(0, 10)}, Node.js ${process.version}, ${cpus()[0]?.model ?? 'unknown CPU'}, ${cpus().length} cores, ${(totalmem() / 1e9).toFixed(1)} GB; openapi-fetch ${version}.`,
  '',
  `${N.toLocaleString('en')} requests a round, one process, one shape after another; the median of ${ROUNDS} rounds after 3 warm-up rounds. flowapi: the whole \`<op>Request\` call (path, query, headers, body). openapi-fetch: \`createFinalURL\` with a query serializer made once and \`defaultPathSerializer\`, and \`defaultBodySerializer\` for the body; its \`Request\` and \`Headers\` are not made. Each pair is checked first to write the same request.`,
  '',
  '| Shape | flowapi | per request | openapi-fetch | per request | flowapi faster by |',
  '| --- | ---: | ---: | ---: | ---: | ---: |',
  ...rows.map((r) => `| ${r.name} | ${fmt(r.ours)} | ${per(r.ours)} | ${fmt(r.theirs)} | ${per(r.theirs)} | ${(r.theirs / r.ours).toFixed(1)}x |`),
  '',
]
mkdirSync(join(root, 'bench/results'), { recursive: true })
writeFileSync(join(root, 'bench/results/sdk.md'), lines.join('\n'))
console.log(lines.join('\n'))
