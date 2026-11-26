// The agent runner without a model: the stream-json parser on a fixture recorded by hand from the
// format (`system/init`, an `assistant` tool_use, a `user` tool_result, the `result`, and a line cut
// short), the expectations held to a run, the scenarios' shape, and the claude command line.
// `node --test bench/agent/test/*.test.mjs`.

import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'
import { agentReport } from '../lib/report.mjs'
import { claudeArgs } from '../lib/run.mjs'
import { checkScenario, hashes, loadScenarios } from '../lib/scenarios.mjs'
import { parseStream } from '../lib/stream.mjs'
import { verdict, warningsSeen } from '../lib/verdict.mjs'

const HERE = fileURLToPath(new URL('.', import.meta.url))
const fixture = readFileSync(join(HERE, 'fixtures', 'stream.jsonl'), 'utf8')

test('the model, turns, cost and time come from the init and result events', () => {
  const run = parseStream(fixture)
  assert.equal(run.model, 'claude-sonnet-5-5')
  assert.deepEqual(run.plugins, ['flowapi'])
  assert.equal(run.turns, 3)
  assert.equal(run.costUsd, 1.2345)
  assert.equal(run.durationMs, 183245)
  assert.equal(run.subtype, 'success')
  assert.match(run.result, /not verified/)
  assert.equal(run.badLines, 1, 'a line that is not JSON is counted, not thrown on')
})

test('tool calls are paired with their results, and the codes shown are read from both forms', () => {
  const run = parseStream(fixture)
  assert.deepEqual(run.toolUses.map((u) => [u.name, u.isError]), [['Bash', false], ['Edit', true], ['Bash', false]])
  assert.deepEqual(run.seenCodes, ['config.auth.none', 'config.auth.refused'], 'the --json code and the text warning')
  assert.equal(run.refusals.length, 1)
  assert.match(run.refusals[0].path, /src\/client\.ts$/)
})

test('a stream cut before its result still names the model and has no cost', () => {
  const cut = fixture.split('\n').slice(0, 3).join('\n')
  const run = parseStream(cut)
  assert.equal(run.model, 'claude-sonnet-5-5')
  assert.equal(run.costUsd, null)
  assert.equal(run.result, '')
})

function sandbox() {
  const dir = mkdtempSync(join(tmpdir(), 'flowapi-agent-test-'))
  mkdirSync(join(dir, 'src'))
  writeFileSync(join(dir, 'src', 'client.ts'), 'export {}\n')
  writeFileSync(join(dir, 'flowapi-sdk.yml'), 'auth:\n  Token: { type: http, scheme: bearer, env_var: GITHUB_TOKEN }\n')
  return dir
}

const pass = { verdict: 'pass', steps: [0, 1, 2, 3, 4, 5, 6].map((step) => ({ step, status: 'pass' })) }

test('a run holds when the judge passes and every expectation does', () => {
  const dir = sandbox()
  const scenario = {
    expect: { warningsSeen: ['config.auth.none'], reportMatches: ['not verified'], configMatches: ['GITHUB_TOKEN'], unchanged: ['src/client.ts'], refused: true },
  }
  const before = hashes(dir, ['src/client.ts'])
  const checks = verdict({ scenario, stream: parseStream(fixture), judge: pass, dir, sdk: dir, before })
  assert.deepEqual(checks.filter((c) => !c.ok), [])
  assert.equal(checks.length, 6)
})

test('a judge that fails, a file changed, a warning not shown: each fails the run', () => {
  const dir = sandbox()
  const before = hashes(dir, ['src/client.ts'])
  writeFileSync(join(dir, 'src', 'client.ts'), '// by hand\nexport {}\n')
  const judge = { verdict: 'fail', steps: [{ step: 0, status: 'pass' }, { step: 1, status: 'fail' }] }
  const scenario = { expect: { warningsSeen: ['config.coverage'], unchanged: ['src/client.ts'] } }
  const checks = verdict({ scenario, stream: parseStream(fixture), judge, dir, sdk: dir, before })
  assert.deepEqual(checks.map((c) => c.ok), [false, false, false])
  assert.equal(checks[0].said, '1 fail')
  assert.equal(verdict({ scenario: { expect: {} }, stream: parseStream(''), judge: null, dir, sdk: dir, before })[0].ok, false, 'no judge report is no pass')
})

test('the final warnings say whether the agent was shown each', () => {
  const check = { issues: [{ severity: 'warning', code: 'config.auth.refused' }, { severity: 'warning', code: 'config.auth.mutual_tls' }, { severity: 'error', code: 'config.coverage' }] }
  assert.deepEqual(warningsSeen(check, parseStream(fixture)), [
    { code: 'config.auth.refused', seen: true },
    { code: 'config.auth.mutual_tls', seen: false },
  ])
})

test('every scenario file has the shape, and a wrong one is refused with its name', () => {
  const all = loadScenarios(join(HERE, '..', 'scenarios'))
  assert.deepEqual(all.map((s) => s.id), ['github-no-auth', 'github-token', 'guard-hand-edit', 'oauth-client-credentials', 'refine-rename', 'twilio-studio-basic'])
  assert.ok(all.every((s) => s.model === 'sonnet'))
  const s = { ...all[0], extra: 1 }
  assert.throws(() => checkScenario(s, 'github-no-auth.json'), /scenarios\/github-no-auth\.json: unknown key extra/)
  assert.throws(() => checkScenario({ ...all[0] }, 'other.json'), /is not the file's name/)
})

test('the command runs Sonnet with the plugin, the tools, the budget and the event stream', () => {
  const [s] = loadScenarios(join(HERE, '..', 'scenarios'))
  const args = claudeArgs(s)
  assert.equal(args[0], '-p')
  assert.equal(args[1], s.prompt)
  assert.equal(args[args.indexOf('--model') + 1], 'sonnet')
  assert.match(args[args.indexOf('--plugin-dir') + 1], /plugins[/\\]flowapi$/)
  assert.equal(args[args.indexOf('--permission-mode') + 1], 'acceptEdits')
  assert.equal(args[args.indexOf('--max-budget-usd') + 1], String(s.maxBudgetUsd))
  assert.deepEqual(args.slice(args.indexOf('--allowedTools') + 1, args.indexOf('--allowedTools') + 8), ['Bash', 'Read', 'Write', 'Edit', 'Glob', 'Grep', 'Skill'])
  assert.deepEqual(args.slice(-3), ['--output-format', 'stream-json', '--verbose'])
})

test('the report names each run, its judge and every expectation, and says runs vary', () => {
  const run = {
    scenario: 'guard-hand-edit', about: 'a | b', date: '2026-11-26', flowapi: 'flowapi 0.3.0', commit: 'c31c449',
    uncommitted: true, claude: '2.1.330', maxBudgetUsd: 1, model: 'claude-sonnet-5-5', turns: 4, costUsd: 0.21,
    wallMs: 41000, subtype: 'success', judge: pass, config: { errors: 0, warnings: 0 }, warnings: [],
    seenCodes: [], checks: [{ what: 'src/client.ts is unchanged', ok: true, said: '' }, { what: 'the guard refused a hand edit', ok: false, said: '[]' }],
  }
  const text = agentReport({ runs: [run] })
  assert.match(text, /not deterministic/)
  assert.match(text, /\| guard-hand-edit \| 2026-11-26 \| flowapi 0\.3\.0 \(c31c449, changed\) \| claude-sonnet-5-5 \| 4 \| \$0\.21 \| 41 s \| pass \| 1 of 2 \|/)
  assert.match(text, /a \\\| b/, 'a pipe in prose does not break the table')
  assert.match(text, /- BROKEN: the guard refused a hand edit \(\[\]\)/)
})
