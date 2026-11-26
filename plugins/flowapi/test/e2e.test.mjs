// The pipeline end to end with a real flowapi: draft, check, generate, and the judge. flowapi is
// FLOWAPI_PLUGIN_FLOWAPI when set, else the repository's own release build
// (target/release/flowapi, from `cargo build --release -p flowapi-cli`); without one the test is
// skipped. By default the judge runs step 0 only (no install, no network);
// FLOWAPI_PLUGIN_E2E=1 (`npm run test:e2e`) installs the SDK's dependency and runs steps 0-6.

import assert from 'node:assert/strict'
import { execFileSync, spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'

const ROOT = fileURLToPath(new URL('..', import.meta.url))
const FULL = process.env.FLOWAPI_PLUGIN_E2E === '1'

function localFlowapi() {
  if (process.env.FLOWAPI_PLUGIN_FLOWAPI) return process.env.FLOWAPI_PLUGIN_FLOWAPI
  const built = join(ROOT, '..', '..', 'target', 'release', 'flowapi')
  return existsSync(built) ? built : null
}

const flowapi = localFlowapi()
const env = { ...process.env, FLOWAPI_PLUGIN_FLOWAPI: flowapi ?? '' }

function F(dir, ...args) {
  return spawnSync(join(ROOT, 'bin', 'flowapi'), args, { cwd: dir, env, encoding: 'utf8' })
}

test('draft, check, generate and judge a small API', { skip: flowapi ? false : 'no target/release/flowapi: cargo build --release -p flowapi-cli, or set FLOWAPI_PLUGIN_FLOWAPI', timeout: 600_000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'flowapi-plugin-e2e-'))
  copyFileSync(join(ROOT, 'test', 'fixtures', 'pets.json'), join(dir, 'pets.json'))

  const draft = F(dir, 'config', 'draft', 'pets.json', '-o', 'flowapi-sdk.yml')
  assert.equal(draft.status, 0, draft.stderr)

  const check = F(dir, 'config', 'check', 'flowapi-sdk.yml', '--json')
  assert.equal(check.status, 0, check.stdout + check.stderr)
  assert.equal(JSON.parse(check.stdout).errors, 0)

  const generate = F(dir, 'generate', 'flowapi-sdk.yml', '--out', '.')
  assert.equal(generate.status, 0, generate.stderr)
  const lock = JSON.parse(readFileSync(join(dir, '.flowapi', 'lock.json'), 'utf8'))
  assert.ok(Object.keys(lock.files).includes('package.json'))

  const again = F(dir, 'generate', 'flowapi-sdk.yml', '--out', '.', '--check')
  assert.equal(again.status, 0, again.stdout + again.stderr)

  if (FULL) execFileSync('npm', ['install', '--no-audit', '--no-fund'], { cwd: dir, stdio: 'ignore' })
  const judged = spawnSync('node', [join(ROOT, 'bin', 'judge.mjs'), dir, '--steps', FULL ? '0-6' : '0', '--json', '--keep-going'], { env, encoding: 'utf8' })
  const report = JSON.parse(judged.stdout)
  for (const step of report.steps) console.log(`step ${step.step} ${step.name}: ${step.status} ${step.duration_ms} ms`)
  assert.equal(report.verdict, 'pass', judged.stdout)
})
