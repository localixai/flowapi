// The SDK layer's tables type-checked and run: for each description in tests/cases/, `flowapi
// --sdk` writes the TypeScript SDK, the table kept beside the description is appended, the
// TypeScript compiler the oracles use checks the whole under strict and exactOptionalPropertyTypes
// and emits JavaScript, and Node imports it: every operation's `request` and `response` is the
// SDK's function, its method a string, and every scheme a requirement names is in
// `flowapiSecuritySchemes` unless the kept diagnostics say why not. A case's `x-flowapi-calls`
// write requests through the table as the client does: an operation's `request` given the input,
// its body with the table's encoding of its media type where the input gives none (calls.ts),
// and the body and headers compared exactly: the description's encoding read, then written.
// Run from the repository root after `cargo build --release -p flowapi-cli`:
//   node crates/flowapi-sdk/tests/tsc.mjs [path/to/flowapi]
// Exits 1 when anything fails.
import { execFileSync, spawnSync } from 'node:child_process'
import { mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const flowapi = process.argv[2] ?? 'target/release/flowapi'
const tsc = 'conformance/ts/node_modules/.bin/tsc'
const cases = 'crates/flowapi-sdk/tests/cases'
const out = 'conformance/out/sdk-table'
const strict = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--target', 'es2022', '--skipLibCheck', '--pretty', 'false']
rmSync(out, { recursive: true, force: true })
mkdirSync(out, { recursive: true })

const names = readdirSync(cases).filter((f) => f.endsWith('.json')).map((f) => f.slice(0, -5)).sort()
const files = []
for (const name of names) {
  const ts = join(out, `${name}.ts`)
  const run = spawnSync(flowapi, [join(cases, `${name}.json`), '--sdk', '-q', '-o', ts], { encoding: 'utf8' })
  if (run.status !== 0) {
    console.error(`${name}: flowapi failed\n${run.stderr}`)
    process.exit(1)
  }
  writeFileSync(ts, readFileSync(ts, 'utf8') + readFileSync(join(cases, `${name}.ts`), 'utf8'))
  files.push(ts)
}
try {
  execFileSync(tsc, [...strict, '--module', 'es2022', '--outDir', join(out, 'js'), ...files], { encoding: 'utf8' })
} catch (e) {
  console.error(`the TypeScript compiler refused the tables:\n${e.stdout}`)
  process.exit(1)
}
writeFileSync(join(out, 'js', 'package.json'), '{"type":"module"}')

const failures = []
for (const name of names) {
  const m = await import(pathToFileURL(resolve(out, 'js', `${name}.js`)).href)
  if (!Array.isArray(m.flowapiServers)) failures.push(`${name}: flowapiServers is not a list`)
  const kept = readFileSync(join(cases, `${name}.ts`), 'utf8')
  for (const [op, entry] of Object.entries(m.flowapiOperations)) {
    if (entry.request !== m[`${op}Request`]) failures.push(`${name}.${op}: request is not ${op}Request`)
    if (entry.response !== m[`${op}Response`]) failures.push(`${name}.${op}: response is not ${op}Response`)
    if (typeof entry.method !== 'string') failures.push(`${name}.${op}: no method`)
    for (const alternative of entry.security) {
      for (const scheme of Object.keys(alternative)) {
        if (!(scheme in m.flowapiSecuritySchemes) && !kept.includes(`names \`${scheme}\``)) {
          failures.push(`${name}.${op}: scheme ${scheme} is neither declared nor warned about`)
        }
      }
    }
  }
}
// The table's encoding of a body's media type put in the input as the client's `encoded` does.
const bare = (t) => (t.split(';')[0] ?? '').trim().toLowerCase()
function encoded(entry, input) {
  const body = input.body
  if (entry.encoding === undefined || body === undefined || 'encoding' in body || 'prefixEncoding' in body || 'itemEncoding' in body) return input
  const key = Object.keys(entry.encoding).find((k) => bare(k) === bare(body.contentType))
  return key === undefined ? input : { ...input, body: { ...body, ...entry.encoding[key] } }
}
let calls = 0
for (const name of names) {
  const described = JSON.parse(readFileSync(join(cases, `${name}.json`), 'utf8'))
  const m = await import(pathToFileURL(resolve(out, 'js', `${name}.js`)).href)
  for (const call of described['x-flowapi-calls'] ?? []) {
    calls++
    const entry = m.flowapiOperations[call.operation]
    // A call's `randoms` (hex, 16 bytes each) are what `crypto.getRandomValues` gives, in turn.
    const real = Object.getOwnPropertyDescriptor(globalThis, 'crypto')
    const queue = (call.randoms ?? []).map((h) => Uint8Array.from(h.match(/../g), (b) => parseInt(b, 16)))
    if (call.randoms) Object.defineProperty(globalThis, 'crypto', { configurable: true, value: { getRandomValues: (b) => { b.set(queue.shift()); return b } } })
    let r
    try {
      r = entry.request(encoded(entry, call.input))
    } finally {
      Object.defineProperty(globalThis, 'crypto', real)
    }
    const body = r.body instanceof Uint8Array ? new TextDecoder().decode(r.body) : r.body
    if (body !== call.expect.body) failures.push(`${name}.${call.operation}: body ${JSON.stringify(body)}, want ${JSON.stringify(call.expect.body)}`)
    for (const [k, v] of Object.entries(call.expect.headers ?? {})) {
      if (r.headers[k] !== v) failures.push(`${name}.${call.operation}: header ${k} ${JSON.stringify(r.headers[k])}, want ${JSON.stringify(v)}`)
    }
  }
}
if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log(`${names.length} tables type-checked and imported, ${calls} requests written through them`)
