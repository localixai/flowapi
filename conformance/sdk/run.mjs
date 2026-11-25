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
    // A path parameter after a literal `x`: the table is about the expansion, and a whole
    // segment left empty or `.` (as it shows for undefined) is refused, by the cases.
    add('3.2.1', id, location === 'path' ? `/${id}/x{color}` : `/${id}`, 'get', { parameters: [param] })
    for (const ex of row.examples) rows.push({ id, location, row, ex })
  }
}
const versions = ['3.0.4', '3.1.2', '3.2.1']
// A case names its `targets` when only some SDKs can be given its calls (bytes that are not UTF-8
// as a body); with none it is every target's.
const ours = (c) => !c.targets || c.targets.includes('typescript')
cases.forEach((c, i) => {
  if (!ours(c)) return
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
// `{ "$number": "NaN" }` (or "Infinity", "-Infinity") is the number JSON cannot write.
const revive = (v) => {
  if (Array.isArray(v)) return v.map(revive)
  if (v === null || typeof v !== 'object') return v
  const keys = Object.keys(v)
  if (keys.length === 1 && keys[0] === '$number') return Number(v.$number)
  return Object.fromEntries(keys.map((k) => [k, revive(v[k])]))
}
// A multipart input's files: `{ "$bytes": hex }` bytes, `{ "$file": { bytes or text, name, type } }`
// flowapiFile's, `{ "$blob": { bytes or text, name, type, as } }` a Blob (a File named `as`) read by
// flowapiBlob, `{ "$unread": { bytes or text } }` a Blob given as it is.
const data = (f) => 'bytes' in f ? fromHex(f.bytes) : f.text
async function given(m, v) {
  if (Array.isArray(v)) return Promise.all(v.map((x) => given(m, x)))
  if (v === null || typeof v !== 'object') return v
  const keys = Object.keys(v)
  if (keys.length === 1 && keys[0] === '$bytes') return fromHex(v.$bytes)
  if (keys.length === 1 && keys[0] === '$file') return m.flowapiFile(data(v.$file), v.$file.name, v.$file.type)
  if (keys.length === 1 && keys[0] === '$blob') {
    const b = v.$blob
    const blob = b.as === undefined ? new Blob([data(b)], { type: b.blobType ?? '' }) : new File([data(b)], b.as, { type: b.blobType ?? '' })
    return m.flowapiBlob(blob, b.name, b.type)
  }
  if (keys.length === 1 && keys[0] === '$unread') return new Blob([data(v.$unread)])
  if (keys.length === 1 && keys[0] === '$undefined') return undefined
  return Object.fromEntries(await Promise.all(keys.map(async (k) => [k, await given(m, v[k])])))
}
// A call's `randoms` (hex, 16 bytes each) are what `crypto.getRandomValues` gives, in turn.
function draws(randoms) {
  const real = Object.getOwnPropertyDescriptor(globalThis, 'crypto')
  const queue = randoms.map(fromHex)
  Object.defineProperty(globalThis, 'crypto', { configurable: true, value: { getRandomValues: (b) => { b.set(queue.shift()); return b } } })
  return () => Object.defineProperty(globalThis, 'crypto', real)
}
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b)
// Bytes as hex: a call's `bytes` is the body given as bytes, an expected `bytes` the body read
// back as exactly those bytes. An expected `body` string for a body read as bytes is held to its
// UTF-8, as the Rust runner reads bytes as their text.
const fromHex = (h) => Uint8Array.from(h.match(/../g) ?? [], (b) => parseInt(b, 16))
const hex = (b) => Buffer.from(b).toString('hex')
const show = (v) => v instanceof Uint8Array ? `bytes ${hex(v)}` : JSON.stringify(v)
const holds = (got, k, v) => {
  if (k === 'bytes') return got.body instanceof Uint8Array && hex(got.body) === v
  if (k === 'body' && got.body instanceof Uint8Array) return typeof v === 'string' && hex(got.body) === hex(Buffer.from(v, 'utf8'))
  return same(got[k], v)
}
let tableRun = 0
for (const { id, location, row, ex } of rows) {
  const input = { [location]: { color: table.values[ex.value] } }
  const r = sdk['3.2.1'][`${id}Request`](input)
  const got = location === 'path' ? r.path.slice(id.length + 3)
    : location === 'query' ? r.query
      : location === 'header' ? (r.headers.color ?? '')
        : (r.headers.cookie ?? '')
  tableRun++
  if (got !== ex.serialized) failures.push(`Style Examples line ${row.line} (${row.style}, explode ${row.explode}, ${location}, ${ex.value}): got ${JSON.stringify(got)}, the spec shows ${JSON.stringify(ex.serialized)}`)
}
let callsRun = 0
for (const [i, c] of cases.entries()) {
  if (!ours(c)) continue
  for (const version of c.versions ?? versions) {
    const m = sdk[version]
    for (const call of c.calls) {
      callsRun++
      const what = `${c.name} (${version})`
      if ('readsBytes' in call) {
        const got = m.flowapiReadsBytes(call.readsBytes)
        if (got !== call.expect) failures.push(`${what}: flowapiReadsBytes(${JSON.stringify(call.readsBytes)}) is ${got}, want ${call.expect}`)
        continue
      }
      if ('status' in call) {
        const body = 'bytes' in call ? fromHex(call.bytes) : call.body ?? ''
        const got = m[`c${i}Response`](call.status, call.headers ?? {}, body)
        for (const [k, v] of Object.entries(call.expect)) {
          const at = k === 'bytes' ? 'body' : k
          if (!holds(got, k, v)) failures.push(`${what}: response ${call.status} ${k}: got ${show(got[at])}, want ${JSON.stringify(v)}`)
        }
        continue
      }
      const want = structuredClone(call.expect)
      const input = await given(m, revive(call.input ?? {}))
      const restore = call.randoms ? draws(call.randoms) : () => {}
      let got
      try {
        got = m[`c${i}Request`](input)
      } catch (e) {
        if (e?.name !== 'FlowapiRequestError') throw e
        const refused = { location: e.location, parameter: e.parameter, reason: e.reason }
        if (!same(refused, want.refused)) failures.push(`${what}: refused ${JSON.stringify(refused)}, want ${JSON.stringify(want.refused ?? 'a request')}`)
        continue
      } finally {
        restore()
      }
      // A boundary drawn at random: 128 bits as hex after `flowapi-`, put for `{boundary}` in what
      // is wanted.
      if (want.boundary === 'drawn') {
        const b = /; boundary=(flowapi-[0-9a-f]{32})$/.exec(got.headers['content-type'] ?? '')?.[1]
        if (b === undefined) failures.push(`${what}: no boundary drawn in ${JSON.stringify(got.headers)}`)
        const put = (x) => x.replaceAll('{boundary}', b ?? '')
        if (typeof want.body === 'string') want.body = put(want.body)
        if (want.headers) want.headers = Object.fromEntries(Object.entries(want.headers).map(([k, v]) => [k, put(v)]))
      }
      if (want.refused) {
        failures.push(`${what}: wrote ${JSON.stringify({ ...got, body: got.body instanceof Uint8Array ? show(got.body) : got.body })}, want it refused ${JSON.stringify(want.refused)}`)
        continue
      }
      if ('path' in want && got.path !== `/c${i}${want.path}`) failures.push(`${what}: path ${JSON.stringify(got.path)}, want ${JSON.stringify(`/c${i}${want.path}`)}`)
      for (const k of ['method', 'query']) {
        if (k in want && !same(got[k], want[k])) failures.push(`${what}: ${k} ${JSON.stringify(got[k])}, want ${JSON.stringify(want[k])}`)
      }
      for (const k of ['body', 'bytes']) {
        if (k in want && !holds(got, k, want[k])) failures.push(`${what}: body ${show(got.body)}, want ${k} ${JSON.stringify(want[k])}`)
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
}

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
