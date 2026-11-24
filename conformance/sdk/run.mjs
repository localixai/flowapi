// Oracle O8: the TypeScript SDK (`flowapi --sdk`) run, not read. Each row of the OpenAPI 3.2.1
// "Style Examples" table (conformance/suites/style-examples.json, from extract.mjs) and each case
// of cases.json (the pitfalls other generators fixed, the spec's own examples) becomes an
// operation of a description; flowapi generates its SDK, the TypeScript compiler checks it under
// strict and exactOptionalPropertyTypes and emits JavaScript, and Node calls every request and
// response function, comparing the strings exactly. `--corpus` also generates the SDK of GitHub's,
// Stripe's and Cloudflare's descriptions (bench/data, bench/corpus/fetch.sh) and type-checks it.
// Run from the repository root after `cargo build --release`:
//   node conformance/sdk/run.mjs [--corpus] [path/to/flowapi]
// Writes conformance/results/sdk.md; exits 1 when anything differs.
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const args = process.argv.slice(2)
const corpus = args.includes('--corpus')
const flowapi = args.find((a) => !a.startsWith('--')) ?? 'target/release/flowapi'
const tsc = 'conformance/ts/node_modules/.bin/tsc'
const out = 'conformance/out/sdk'
const strict = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--target', 'es2022', '--skipLibCheck', '--pretty', 'false']
rmSync(out, { recursive: true, force: true })
mkdirSync(out, { recursive: true })

const table = JSON.parse(readFileSync('conformance/suites/style-examples.json', 'utf8'))
const cases = JSON.parse(readFileSync('conformance/sdk/cases.json', 'utf8')).cases

// One description per version; each case adds an operation `c<i>`, each table row `s<line><where>`.
const docs = {}
function doc(version) {
  docs[version] ??= version === '2.0'
    ? { swagger: '2.0', info: { title: 'sdk', version: '1' }, paths: {} }
    : { openapi: version, info: { title: 'sdk', version: '1' }, paths: {} }
  return docs[version]
}
function add(version, id, path, method, operation) {
  const item = (doc(version).paths[path] ??= {})
  const op = { operationId: id, responses: { default: { description: 'any' } }, ...operation }
  if (/^[a-z]+$/.test(method)) item[method] = op
  else (item.additionalOperations ??= {})[method] = op
}

// The table: where each style goes, and what of the request is its serialization.
const where = { matrix: ['path'], label: ['path'], simple: ['path', 'header'], form: ['query'], spaceDelimited: ['query'], pipeDelimited: ['query'], deepObject: ['query'], cookie: ['cookie'] }
const rows = []
for (const row of table.rows) {
  for (const location of where[row.style]) {
    const id = `s${row.line}${location[0]}`
    const param = { name: 'color', in: location, style: row.style, schema: {}, ...(location === 'path' ? { required: true } : {}) }
    if (row.explode !== null) param.explode = row.explode
    add('3.2.1', id, location === 'path' ? `/${id}/{color}` : `/${id}`, 'get', { parameters: [param] })
    for (const ex of row.examples) rows.push({ id, location, row, ex })
  }
}
const versions = ['3.0.4', '3.1.2', '3.2.1']
cases.forEach((c, i) => {
  for (const version of c.versions ?? versions) {
    const op = {}
    if (c.parameters) op.parameters = c.parameters
    if (c.requestBody) op.requestBody = c.requestBody
    if (c.responses) op.responses = c.responses
    add(version, `c${i}`, `/c${i}${c.path ?? ''}`, c.method ?? 'get', op)
  }
})

// Generate, then compile every SDK at once.
const warnings = {}
const files = []
for (const [version, d] of Object.entries(docs)) {
  const name = `v${version.replace(/\./g, '_')}`
  writeFileSync(join(out, `${name}.json`), JSON.stringify(d, null, 1))
  const run = spawnSync(flowapi, [join(out, `${name}.json`), '--sdk', '-o', join(out, `${name}.ts`)], { encoding: 'utf8' })
  if (run.status !== 0) throw new Error(`${name}: flowapi failed:\n${run.stderr}`)
  warnings[version] = run.stderr
  files.push(join(out, `${name}.ts`))
}
let typeErrors = ''
try {
  execFileSync(tsc, [...strict, '--module', 'es2022', '--outDir', join(out, 'js'), ...files], { encoding: 'utf8' })
} catch (e) {
  typeErrors = e.stdout ?? String(e)
}
const sdk = {}
for (const version of Object.keys(docs)) {
  sdk[version] = await import(pathToFileURL(resolve(out, 'js', `v${version.replace(/\./g, '_')}.js`)).href)
}

const failures = []
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b)
let tableRun = 0
for (const { id, location, row, ex } of rows) {
  const input = { [location]: { color: table.values[ex.value] } }
  const r = sdk['3.2.1'][`${id}Request`](input)
  const got = location === 'path' ? r.path.slice(id.length + 2)
    : location === 'query' ? r.query
      : location === 'header' ? (r.headers.color ?? '')
        : (r.headers.cookie ?? '')
  tableRun++
  if (got !== ex.serialized) failures.push(`Style Examples line ${row.line} (${row.style}, explode ${row.explode}, ${location}, ${ex.value}): got ${JSON.stringify(got)}, the spec shows ${JSON.stringify(ex.serialized)}`)
}
let callsRun = 0
cases.forEach((c, i) => {
  for (const version of c.versions ?? versions) {
    const m = sdk[version]
    for (const call of c.calls) {
      callsRun++
      const what = `${c.name} (${version})`
      if ('status' in call) {
        const got = m[`c${i}Response`](call.status, call.headers ?? {}, call.body ?? '')
        for (const [k, v] of Object.entries(call.expect)) {
          if (!same(got[k], v)) failures.push(`${what}: response ${call.status} ${k}: got ${JSON.stringify(got[k])}, want ${JSON.stringify(v)}`)
        }
        continue
      }
      const got = m[`c${i}Request`](call.input ?? {})
      const want = call.expect
      if ('path' in want && got.path !== `/c${i}${want.path}`) failures.push(`${what}: path ${JSON.stringify(got.path)}, want ${JSON.stringify(`/c${i}${want.path}`)}`)
      for (const k of ['method', 'query', 'body']) {
        if (k in want && !same(got[k], want[k])) failures.push(`${what}: ${k} ${JSON.stringify(got[k])}, want ${JSON.stringify(want[k])}`)
      }
      if ('headers' in want && !same(got.headers, want.headers)) failures.push(`${what}: headers ${JSON.stringify(got.headers)}, want ${JSON.stringify(want.headers)}`)
      // Decoded as the receiver would: WHATWG's form-urlencoded parser for the query and a form
      // body, RFC 3986 percent-decoding for a path.
      if (want.pairs && !same([...new URLSearchParams(got.query)], want.pairs)) failures.push(`${what}: the query parses as ${JSON.stringify([...new URLSearchParams(got.query)])}, want ${JSON.stringify(want.pairs)}`)
      if (want.bodyPairs && !same([...new URLSearchParams(got.body)], want.bodyPairs)) failures.push(`${what}: the body parses as ${JSON.stringify([...new URLSearchParams(got.body)])}, want ${JSON.stringify(want.bodyPairs)}`)
      if (want.decodedPath && decodeURIComponent(got.path) !== `/c${i}${want.decodedPath}`) failures.push(`${what}: the path decodes as ${decodeURIComponent(got.path)}`)
    }
    if (c.warns && !warnings[version].includes(c.warns)) failures.push(`${c.name} (${version}): no \`${c.warns}\` diagnostic`)
  }
})

// The corpus: the SDK of each large description, type-checked one at a time.
const corpusRows = []
if (corpus) {
  for (const name of ['github', 'stripe', 'cloudflare']) {
    const input = `bench/data/${name}.json`
    if (!existsSync(input)) throw new Error(`${input} is missing: bench/corpus/fetch.sh`)
    const ts = join(out, `${name}.ts`)
    const run = spawnSync(flowapi, [input, '--sdk', '-q', '-o', ts], { encoding: 'utf8', maxBuffer: 1 << 26 })
    const generated = run.status === 0
    let errors = generated ? 0 : null
    if (generated) {
      try {
        execFileSync(tsc, [...strict, '--noEmit', ts], { encoding: 'utf8', maxBuffer: 1 << 28 })
      } catch (e) {
        errors = (e.stdout ?? '').split('\n').filter((l) => l.includes('error TS')).length || 1
      }
    }
    const text = generated ? readFileSync(ts, 'utf8') : ''
    const operations = (text.match(/^export function \w+Request\(/gm) ?? []).length
    corpusRows.push({ name, generated, operations, errors })
    if (!generated || errors !== 0) failures.push(`corpus ${name}: generated ${generated}, ${errors} type errors`)
  }
}
if (typeErrors) failures.unshift(`the generated SDKs do not type-check:\n${typeErrors}`)

const lines = [
  '# O8: the TypeScript SDK, run',
  '',
  'Written by `node conformance/sdk/run.mjs --corpus`; do not edit. Every request and response function is generated by `flowapi --sdk`, type-checked (`strict`, `exactOptionalPropertyTypes`), compiled and called under Node; strings are compared exactly.',
  '',
  '| Oracle | Checked | Differ |',
  '| --- | ---: | ---: |',
  `| OpenAPI 3.2.1 "Style Examples" (${table.source}:${table.heading}), each cell not _n/a_ | ${tableRun} | ${failures.filter((f) => f.startsWith('Style Examples')).length} |`,
  `| conformance/sdk/cases.json: ${cases.length} cases, each in its versions | ${callsRun} | ${failures.filter((f) => !f.startsWith('Style Examples') && !f.startsWith('corpus')).length} |`,
  '',
]
if (corpusRows.length) {
  lines.push('| Description | Operations | Type errors |', '| --- | ---: | ---: |')
  for (const r of corpusRows) lines.push(`| ${r.name} | ${r.operations} | ${r.errors ?? 'not generated'} |`)
  lines.push('')
}
lines.push('## Differences', '', ...(failures.length ? failures.map((f) => `- ${f}`) : ['None.']), '')
mkdirSync('conformance/results', { recursive: true })
writeFileSync('conformance/results/sdk.md', lines.join('\n'))
console.log(lines.join('\n'))
process.exit(failures.length ? 1 : 0)
