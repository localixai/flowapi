// The `flowapi` skill's references are what scripts/docs.mjs writes from flowapi's docs: a
// reference edited by hand, or a source changed without running the script, fails here.
import assert from 'node:assert/strict'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { test } from 'node:test'
import { OUT, REPO, render } from '../scripts/docs.mjs'

const inRepo = existsSync(join(REPO, 'Cargo.toml')) ? false : 'not inside the flowapi repository'

test('every reference equals what the script writes, and nothing else is there', { skip: inRepo }, () => {
  const wanted = render()
  assert.deepEqual(readdirSync(OUT).sort(), [...wanted.keys()].sort())
  for (const [name, text] of wanted) {
    assert.equal(readFileSync(join(OUT, name), 'utf8'), text, `${name} is stale: node scripts/docs.mjs`)
  }
})

test('each reference names its source on its first line, and links stay absolute', { skip: inRepo }, () => {
  for (const [name, text] of render()) {
    assert.match(text.split('\n')[0], /^<!-- generated from \S.* by plugins\/flowapi\/scripts\/docs\.mjs/, name)
    for (const [, target] of text.matchAll(/\]\(([^)\s]+)\)/g)) {
      assert.match(target, /^(https?:|mailto:)/, `${name}: relative link ${target}`)
    }
  }
})

test('the commands help is copied whole, from both commands', { skip: inRepo }, () => {
  const cli = render().get('cli.md')
  assert.match(cli, /## The Rust command\n\n```text\nflowapi: types from/)
  assert.match(cli, /## The npm command\n\n```text\nflowapi: types from/)
  assert.match(cli, /--target ts\|rust/)
  assert.ok(!cli.includes('\\`'), 'template-literal escapes are undone')
})
