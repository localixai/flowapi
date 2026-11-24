// Oracle O2: the OAI's own test descriptions. Every `pass` description must generate with no
// error and its TypeScript must type-check; each `fail` description's outcome is recorded.
// Run from the repository root after `cargo build --release`: node conformance/oai/run.mjs
import { execFileSync, spawnSync } from 'node:child_process'
import { mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const suites = 'conformance/suites/oai-schema-tests'
const out = 'conformance/oai/out'
const flowapi = 'target/release/flowapi'
const tsc = 'conformance/ts/node_modules/.bin/tsc'
rmSync(out, { recursive: true, force: true })
mkdirSync(out, { recursive: true })

const rows = []
for (const version of readdirSync(suites).filter((d) => d.startsWith('v')).sort()) {
  for (const kind of ['pass', 'fail']) {
    let files = []
    try {
      files = readdirSync(join(suites, version, kind)).sort()
    } catch {
      continue
    }
    for (const file of files) {
      const ts = join(out, `${version}-${kind}-${file.replace(/\W/g, '_')}.ts`)
      const run = spawnSync(flowapi, [join(suites, version, kind, file), '-q', '-o', ts], { encoding: 'utf8' })
      const errors = (run.stderr.match(/^error\[[^\]]+\]/gm) ?? []).map((e) => e.slice(6, -1))
      rows.push({ version, kind, file, generated: run.status === 0, errors, ts: run.status === 0 ? ts : null })
    }
  }
}
const written = rows.filter((r) => r.ts).map((r) => r.ts)
const flags = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--noEmit', '--target', 'es2022', '--skipLibCheck', '--pretty', 'false']
let diagnostics = ''
try {
  execFileSync(tsc, [...flags, ...written], { encoding: 'utf8' })
} catch (e) {
  diagnostics = e.stdout ?? ''
}
for (const r of rows) r.typeChecks = r.ts ? !diagnostics.split('\n').some((l) => l.startsWith(r.ts)) : null

const pass = rows.filter((r) => r.kind === 'pass')
const passOk = pass.filter((r) => r.generated && r.typeChecks)
const lines = [
  '# O2: the OAI test descriptions',
  '',
  'Written by `node conformance/oai/run.mjs`; do not edit. Every `pass` description must generate with no error and type-check; a `fail` description is invalid against the OAI schema, often where no type looks, so its outcome is recorded only.',
  '',
  `**pass: ${passOk.length} of ${pass.length} generate and type-check.**`,
  '',
  '| Version | Kind | Description | Generated | Errors | Type-checks |',
  '| --- | --- | --- | --- | --- | --- |',
  ...rows.map((r) => `| ${r.version} | ${r.kind} | ${r.file} | ${r.generated ? 'yes' : 'no'} | ${r.errors.join(', ') || '-'} | ${r.typeChecks === null ? '-' : r.typeChecks ? 'yes' : 'no'} |`),
  '',
]
mkdirSync('conformance/results', { recursive: true })
writeFileSync('conformance/results/oai.md', lines.join('\n'))
writeFileSync('conformance/results/oai.json', `${JSON.stringify(rows.map(({ ts, ...r }) => r), null, 2)}\n`)
console.log(`pass: ${passOk.length} of ${pass.length} generate and type-check; results in conformance/results/oai.md`)
if (passOk.length !== pass.length) process.exit(1)
