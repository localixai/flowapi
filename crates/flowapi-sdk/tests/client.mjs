// The clients kept in tests/client/ type-checked, compiled and run. For each case its kept src/
// and the runtime (src/client/core/) are written to a temporary folder; the TypeScript compiler
// the oracles use checks them under strict and exactOptionalPropertyTypes with `lib: ["es2022"]`
// alone (the runtime types what it needs of fetch itself), then again with the DOM's declarations
// beside a file that hands the client the platform's own fetch, Response and AbortSignal, and
// emits JavaScript that Node imports. Then the tests of every module of tests/client/ but
// `stub.mjs` (what they run against) run against a stub fetch, each naming the pitfall it holds:
// R<n> of the runtime's list, P<n> of the lists' and streams', each module's head saying what its
// numbers are. Every runtime file holds at most 300 lines of working code (G10, as for Rust).
// Run from the repository root, after `(cd conformance/ts && npm ci)`:
//   node crates/flowapi-sdk/tests/client.mjs
// Exits 1 when anything fails.
import { execFileSync } from 'node:child_process'
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const tsc = resolve('conformance/ts/node_modules/.bin/tsc')
const cases = 'crates/flowapi-sdk/tests/client'
const core = 'crates/flowapi-sdk/src/client/core'
const strict = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--target', 'es2022', '--module', 'nodenext', '--pretty', 'false']
const failures = []

// G10: a runtime file's working lines: not blank, not a comment.
for (const name of readdirSync(core)) {
  const lines = readFileSync(join(core, name), 'utf8').split('\n').map((l) => l.trim())
  const working = lines.filter((l) => l !== '' && !l.startsWith('//') && !l.startsWith('/*') && !l.startsWith('*')).length
  if (working > 300) failures.push(`${core}/${name}: ${working} lines of working code, over 300`)
}

const root = mkdtempSync(join(tmpdir(), 'flowapi-client-'))
const built = {}
for (const name of readdirSync(cases).filter((n) => !n.includes('.')).sort()) {
  const dir = join(root, name)
  cpSync(join(cases, name, 'src'), join(dir, 'src'), { recursive: true })
  cpSync(core, join(dir, 'src', 'core'), { recursive: true })
  writeFileSync(join(dir, 'package.json'), '{"type":"module"}')
  const files = (sub) => readdirSync(join(dir, sub), { recursive: true }).filter((f) => f.endsWith('.ts')).map((f) => join(dir, sub, f))
  const started = Date.now()
  try {
    execFileSync(tsc, [...strict, '--lib', 'es2022', '--rootDir', join(dir, 'src'), '--outDir', join(dir, 'js'), ...files('src')], { encoding: 'utf8' })
  } catch (e) {
    failures.push(`${name}: the TypeScript compiler refused it with lib es2022:\n${e.stdout}`)
    continue
  }
  const client = /export class (\w+)/.exec(readFileSync(join(dir, 'src', 'client.ts'), 'utf8'))[1]
  mkdirSync(join(dir, 'dom'))
  writeFileSync(join(dir, 'dom', 'platform.ts'), [
    `import { ${client} } from "../src/index.js";`,
    `const controller = new AbortController();`,
    `const client = new ${client}({ fetch: globalThis.fetch, fetchOptions: { credentials: "include" }, logger: console });`,
    `export const raw = client.request("GET", "/", { signal: controller.signal }).asResponse();`,
    `export const fetchResponse: (r: Response) => import("../src/index.js").FetchResponse = (r) => r;`,
  ].join('\n'))
  try {
    execFileSync(tsc, [...strict, '--lib', 'es2022,dom', '--noEmit', ...files('src'), join(dir, 'dom', 'platform.ts')], { encoding: 'utf8' })
  } catch (e) {
    failures.push(`${name}: the platform's fetch is not the runtime's (lib dom):\n${e.stdout}`)
  }
  console.log(`${name}: type-checked and compiled in ${((Date.now() - started) / 1000).toFixed(1)} s`)
  built[name] = await import(pathToFileURL(join(dir, 'js', 'index.js')).href)
}

let passed = 0
if (built.shop !== undefined && built.auth !== undefined) {
  const modules = readdirSync(cases).filter((n) => n.endsWith('.mjs') && n !== 'stub.mjs').sort()
  const all = []
  for (const m of modules) all.push(...(await import(`./client/${m}`)).tests(built))
  for (const [name, test] of all) {
    try {
      await test()
      passed++
    } catch (e) {
      failures.push(`${name}: ${e?.stack ?? e}`)
    }
  }
}
rmSync(root, { recursive: true, force: true })
if (failures.length > 0) {
  console.error(failures.join('\n\n'))
  process.exit(1)
}
console.log(`${passed} runtime tests passed`)
