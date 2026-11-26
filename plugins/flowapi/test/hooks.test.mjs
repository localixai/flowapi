// The guard as each client runs it: the core decision on `file.modify` profiles, the hook in both
// answer formats (Claude Code's `hookSpecificOutput`, the core's `{ decision, reason }`), and the
// two hook files kept in agreement, as AGENTS.md asks: the same events the core defines, the same
// script.
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'
import { decideCore } from '../lib/guard.mjs'

const ROOT = fileURLToPath(new URL('..', import.meta.url))
const HOOK = join(ROOT, 'hooks', 'guard.mjs')
/** The events Agent Plugins core 1.1.0 defines (§7.3.4 to §7.3.8). */
const CORE_EVENTS = ['SessionStart', 'UserPromptSubmit', 'PreToolUse', 'PostToolUse', 'Stop']

function repo() {
  const root = mkdtempSync(join(tmpdir(), 'flowapi-plugin-hooks-'))
  mkdirSync(join(root, '.flowapi'))
  const files = { 'package.json': 'a', 'src/client.ts': 'b', 'src/lib/own.ts': 'c' }
  writeFileSync(join(root, '.flowapi', 'lock.json'), JSON.stringify({ version: 1, files }))
  return root
}

const coreEvent = (cwd, operations) => ({
  hook_event_name: 'PreToolUse',
  session_id: 's',
  cwd,
  tool_use_id: 't',
  tool_name: 'AnyClientTool',
  tool_input: {},
  tool_profile: { name: 'file.modify', input: { operations } },
})

test('core: every operation on a generated file is refused', () => {
  const root = repo()
  const client = join(root, 'src/client.ts')
  for (const operations of [
    [{ kind: 'edit', path: client }],
    [{ kind: 'write', path: client }],
    [{ kind: 'delete', path: client }],
    [{ kind: 'move', from: client, to: join(root, 'src/other.ts') }],
    [{ kind: 'move', from: join(root, 'src/other.ts'), to: client }],
    [{ kind: 'write', path: join(root, 'notes.md') }, { kind: 'edit', path: client }],
    [{ kind: 'write', path: join(root, '.flowapi/lock.json') }],
  ]) {
    const decision = decideCore(coreEvent(root, operations))
    assert.ok(decision, JSON.stringify(operations))
    assert.match(decision.reason, /flowapi-sdk\.yml/)
  }
})

test('core: own code, other files, other profiles and odd events stay silent', () => {
  const root = repo()
  assert.equal(decideCore(coreEvent(root, [{ kind: 'edit', path: join(root, 'src/lib/own.ts') }])), null)
  assert.equal(decideCore(coreEvent(root, [{ kind: 'write', path: join(root, 'src/new.ts') }])), null)
  const shell = { ...coreEvent(root, []), tool_profile: { name: 'shell', input: { command: 'rm package.json' } } }
  assert.equal(decideCore(shell), null)
  const native = { ...coreEvent(root, []), tool_name: 'Edit', tool_input: { file_path: join(root, 'package.json') } }
  delete native.tool_profile
  assert.equal(decideCore(native), null, 'the core form reads the profile, never a native tool name')
  for (const odd of [null, {}, { tool_profile: { name: 'file.modify' } }, coreEvent(root, 'x'), coreEvent(root, [null, 7])]) {
    assert.equal(decideCore(odd), null)
  }
})

const run = (args, input) => spawnSync(process.execPath, [HOOK, ...args], { input, encoding: 'utf8' })

test('the hook, core form: a denial as { decision, reason }, silence otherwise, exit 0', () => {
  const root = repo()
  const denied = run(['core'], JSON.stringify(coreEvent(root, [{ kind: 'edit', path: join(root, 'package.json') }])))
  assert.equal(denied.status, 0)
  assert.deepEqual(Object.keys(JSON.parse(denied.stdout)).sort(), ['decision', 'reason'])
  assert.equal(JSON.parse(denied.stdout).decision, 'deny')
  const own = JSON.stringify(coreEvent(root, [{ kind: 'edit', path: join(root, 'src/lib/own.ts') }]))
  for (const input of [own, 'not json', '']) {
    const quiet = run(['core'], input)
    assert.equal(quiet.status, 0)
    assert.equal(quiet.stdout, '', 'no opinion is no output, never `allow` or `{}`')
    assert.equal(quiet.stderr, '')
  }
})

test('the hook, Claude Code form: the core event is not its format, and the reverse', () => {
  const root = repo()
  const claude = { tool_name: 'Edit', tool_input: { file_path: join(root, 'package.json') }, cwd: root }
  assert.equal(JSON.parse(run([], JSON.stringify(claude)).stdout).hookSpecificOutput.permissionDecision, 'deny')
  assert.equal(run(['core'], JSON.stringify(claude)).stdout, '')
  const core = coreEvent(root, [{ kind: 'edit', path: join(root, 'package.json') }])
  assert.equal(run([], JSON.stringify(core)).stdout, '')
})

const readJson = (rel) => JSON.parse(readFileSync(join(ROOT, rel), 'utf8'))

test('hooks.json and hooks/hooks.json: the same core events, the same guard', () => {
  const core = readJson('hooks.json')
  const claude = readJson('hooks/hooks.json')
  assert.equal(core.$schema, 'https://agent-plugins.org/schemas/1.1.0/hooks.schema.json')
  const claudeCoreEvents = Object.keys(claude.hooks).filter((e) => CORE_EVENTS.includes(e))
  assert.deepEqual(Object.keys(core.hooks).sort(), claudeCoreEvents.sort())
  for (const event of Object.keys(core.hooks)) {
    for (const handler of core.hooks[event]) {
      assert.deepEqual(Object.keys(handler).sort(), ['args', 'command'])
      assert.equal(handler.command, './hooks/guard.mjs')
      assert.deepEqual(handler.args, ['core'])
    }
    for (const matcher of claude.hooks[event]) {
      for (const handler of matcher.hooks) {
        assert.equal(handler.command, 'node')
        assert.deepEqual(handler.args, ['${CLAUDE_PLUGIN_ROOT}/hooks/guard.mjs'])
      }
    }
  }
})
