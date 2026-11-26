// The guard: its decision on throwaway folders, the Windows rules through path.win32, and the hook
// end to end as Claude Code runs it (the event on stdin, the answer on stdout, exit code 0).
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path, { join } from 'node:path'
import { test } from 'node:test'
import { decide } from '../lib/guard.mjs'

const HOOK = new URL('../hooks/guard.mjs', import.meta.url).pathname
const FILES = {
  'package.json': 'a1',
  'src/client.ts': 'b2',
  'src/resources/projects.ts': 'c3',
  'src/lib/helpers.ts': 'd4',
  'examples/basic.ts': 'e5',
}

function repo({ lock } = {}) {
  const root = mkdtempSync(join(tmpdir(), 'flowapi-plugin-guard-'))
  mkdirSync(join(root, '.flowapi'))
  const text = lock ?? JSON.stringify({ version: 1, flowapi: '0.2.0', files: FILES })
  writeFileSync(join(root, '.flowapi', 'lock.json'), text)
  return root
}

const edit = (file_path, cwd, tool_name = 'Edit') => ({ tool_name, tool_input: { file_path }, cwd })

test('no lock above the file: allowed', () => {
  const root = mkdtempSync(join(tmpdir(), 'flowapi-plugin-none-'))
  assert.equal(decide(edit(join(root, 'src/client.ts'), root)), null)
})

test('a generated file listed in the lock: denied, with the way to change it', () => {
  const root = repo()
  for (const tool of ['Edit', 'Write', 'MultiEdit']) {
    const decision = decide(edit(join(root, 'src/client.ts'), root, tool))
    assert.match(decision.reason, /src\/client\.ts is generated/)
    assert.match(decision.reason, /change flowapi-sdk\.yml and run generate again/)
  }
})

test('a notebook edit names its file as notebook_path', () => {
  const root = repo({ lock: JSON.stringify({ files: { 'notes.ipynb': 'x' } }) })
  const event = {
    tool_name: 'NotebookEdit',
    tool_input: { notebook_path: join(root, 'notes.ipynb') },
    cwd: root,
  }
  assert.ok(decide(event))
})

test('a relative path is taken from the event cwd', () => {
  const root = repo()
  assert.ok(decide(edit('src/resources/projects.ts', root)))
  assert.ok(decide(edit('resources/projects.ts', join(root, 'src'))))
})

test('the custom zones stay editable even when listed', () => {
  const root = repo()
  assert.equal(decide(edit(join(root, 'src/lib/helpers.ts'), root)), null)
  assert.equal(decide(edit(join(root, 'src/lib/new.ts'), root)), null)
  assert.equal(decide(edit(join(root, 'examples/basic.ts'), root)), null)
})

test('a file not in the lock and flowapi-sdk.yml: allowed', () => {
  const root = repo()
  assert.equal(decide(edit(join(root, 'flowapi-sdk.yml'), root)), null)
  assert.equal(decide(edit(join(root, 'src/other.ts'), root)), null)
  assert.equal(decide(edit(join(root, 'notes/package.json'), root)), null)
})

test('the lock itself: denied, with or without a lock above it', () => {
  const root = repo()
  assert.match(
    decide(edit(join(root, '.flowapi/lock.json'), root, 'Write')).reason,
    /written by flowapi generate alone/,
  )
  const bare = mkdtempSync(join(tmpdir(), 'flowapi-plugin-bare-'))
  assert.ok(decide(edit(join(bare, '.flowapi/lock.json'), bare, 'Write')))
})

test('a broken or odd lock fails open', () => {
  for (const lock of ['{not json', '[]', '{"files": []}', '{"files": "x"}', '']) {
    const root = repo({ lock })
    assert.equal(
      decide(edit(join(root, 'src/client.ts'), root)),
      null,
      `lock ${JSON.stringify(lock)}`,
    )
  }
})

test('the nearest lock wins: an SDK nested in another one', () => {
  const outer = repo({ lock: JSON.stringify({ files: { 'sdk/src/client.ts': 'x' } }) })
  const inner = join(outer, 'sdk')
  mkdirSync(join(inner, '.flowapi'), { recursive: true })
  writeFileSync(
    join(inner, '.flowapi', 'lock.json'),
    JSON.stringify({ files: { 'src/index.ts': 'y' } }),
  )
  assert.equal(decide(edit(join(inner, 'src/client.ts'), outer)), null)
  assert.ok(decide(edit(join(inner, 'src/index.ts'), outer)))
  assert.ok(decide(edit(join(inner, 'src/deep/../index.ts'), outer)))
})

test('other tools and malformed events: silent', () => {
  const root = repo()
  assert.equal(
    decide({
      tool_name: 'Read',
      tool_input: { file_path: join(root, 'src/client.ts') },
      cwd: root,
    }),
    null,
  )
  assert.equal(decide({ tool_name: 'Edit', tool_input: {} }), null)
  assert.equal(decide({ tool_name: 'Edit' }), null)
  assert.equal(decide(null), null)
})

test('Windows paths: backslashes, a drive, any case', () => {
  const locks = { 'C:\\work\\sdk\\.flowapi\\lock.json': JSON.stringify({ files: FILES }) }
  const readFile = (file) => {
    if (file in locks) return locks[file]
    throw Object.assign(new Error('missing'), { code: 'ENOENT' })
  }
  const win = { path: path.win32, readFile, platform: 'win32' }
  const at = (file_path) =>
    decide({ tool_name: 'Edit', tool_input: { file_path }, cwd: 'C:\\work' }, win)
  assert.match(at('C:\\work\\sdk\\src\\client.ts').reason, /src\/client\.ts is generated/)
  assert.ok(at('C:/work/sdk/src/Client.ts'))
  assert.ok(at('sdk\\src\\resources\\projects.ts'))
  assert.equal(at('C:\\work\\sdk\\src\\lib\\helpers.ts'), null)
  assert.equal(at('C:\\work\\sdk\\SRC\\LIB\\helpers.ts'), null)
  assert.ok(at('C:\\work\\sdk\\.FLOWAPI\\lock.json'))
  assert.equal(at('C:\\work\\other\\src\\client.ts'), null)
})

test('the hook: a denial as PreToolUse JSON, silence otherwise, exit code 0 always', () => {
  const root = repo()
  const hook = (input) => spawnSync(process.execPath, [HOOK], { input, encoding: 'utf8' })
  const denied = hook(JSON.stringify(edit(join(root, 'src/client.ts'), root)))
  assert.equal(denied.status, 0)
  const answer = JSON.parse(denied.stdout)
  assert.equal(answer.hookSpecificOutput.hookEventName, 'PreToolUse')
  assert.equal(answer.hookSpecificOutput.permissionDecision, 'deny')
  assert.match(answer.hookSpecificOutput.permissionDecisionReason, /flowapi-sdk\.yml/)
  for (const input of [JSON.stringify(edit(join(root, 'src/lib/x.ts'), root)), 'not json', '']) {
    const quiet = hook(input)
    assert.equal(quiet.status, 0)
    assert.equal(quiet.stdout, '')
    assert.equal(quiet.stderr, '')
  }
})
