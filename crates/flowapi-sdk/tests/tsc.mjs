// The SDK layer's tables type-checked and run: for each description in tests/cases/, `flowapi
// --sdk` writes the TypeScript SDK, the table kept beside the description is appended, the
// TypeScript compiler the oracles use checks the whole under strict and exactOptionalPropertyTypes
// and emits JavaScript, and Node imports it: every operation's `request` and `response` is the
// SDK's function, its method a string, and every scheme a requirement names is in
// `flowapiSecuritySchemes` unless the kept diagnostics say why not.
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
if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log(`${names.length} tables type-checked and imported`)
