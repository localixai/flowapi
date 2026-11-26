// The plugin's two packagings kept equal (the core's root plugin.json and Claude Code's
// .claude-plugin/plugin.json), one version everywhere (the workspace's Cargo.toml, both
// manifests, package.json and lib/flowapi-version, which bin/flowapi runs), the repository's
// marketplace entry pointing here, the skills in the Agent Skills form, and the executables
// runnable as a client runs them.
import assert from 'node:assert/strict'
import { accessSync, constants, existsSync, readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'

const ROOT = fileURLToPath(new URL('..', import.meta.url))
const REPO = join(ROOT, '..', '..')
const read = (...parts) => readFileSync(join(...parts), 'utf8')
const json = (...parts) => JSON.parse(read(...parts))
const inRepo = existsSync(join(REPO, 'Cargo.toml')) ? false : 'not inside the flowapi repository'

test('the two manifests: the same fields, the core one with its $schema', () => {
  const core = json(ROOT, 'plugin.json')
  const claude = json(ROOT, '.claude-plugin', 'plugin.json')
  const { $schema, ...rest } = core
  assert.equal($schema, 'https://agent-plugins.org/schemas/1.1.0/plugin.schema.json')
  assert.deepEqual(rest, claude)
  assert.equal(claude.name, 'flowapi')
})

test('one version: Cargo.toml, both manifests, package.json, lib/flowapi-version', { skip: inRepo }, () => {
  const cargo = /^\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m.exec(read(REPO, 'Cargo.toml'))
  assert.ok(cargo, 'Cargo.toml has [workspace.package] version')
  const version = cargo[1]
  assert.equal(json(ROOT, 'plugin.json').version, version, 'plugin.json')
  assert.equal(json(ROOT, '.claude-plugin', 'plugin.json').version, version, '.claude-plugin/plugin.json')
  assert.equal(json(ROOT, 'package.json').version, version, 'package.json')
  assert.equal(read(ROOT, 'lib', 'flowapi-version').trim(), version, 'lib/flowapi-version')
})

test('the marketplace at the repository root installs this folder', { skip: inRepo }, () => {
  const market = json(REPO, '.claude-plugin', 'marketplace.json')
  const manifest = json(ROOT, '.claude-plugin', 'plugin.json')
  assert.equal(market.name, 'flowapi')
  assert.equal(market.owner.name, 'Localix Lab')
  const entry = market.plugins.find((p) => p.name === manifest.name)
  assert.ok(entry, 'an entry named as the manifest')
  assert.equal(entry.source, './plugins/flowapi')
  assert.equal(join(REPO, entry.source), join(ROOT, '.'))
  assert.equal(entry.description, manifest.description)
  assert.equal(entry.version, undefined, 'plugin.json alone says the version')
})

test('skills: name equal to the directory, a description, Agent Skills fields only', () => {
  const allowed = new Set(['name', 'description', 'license', 'compatibility', 'metadata', 'allowed-tools'])
  const dirs = readdirSync(join(ROOT, 'skills'))
  assert.deepEqual(dirs.sort(), ['flowapi', 'sdk'])
  for (const dir of dirs) {
    const text = read(ROOT, 'skills', dir, 'SKILL.md')
    const front = /^---\n([\s\S]*?)\n---\n/.exec(text)
    assert.ok(front, `${dir}: frontmatter`)
    const fields = Object.fromEntries(
      front[1].split('\n').map((line) => /^([a-z-]+):\s*(.*)$/.exec(line)).filter(Boolean).map((m) => [m[1], m[2]]),
    )
    assert.equal(fields.name, dir)
    assert.match(dir, /^[a-z0-9]+(-[a-z0-9]+)*$/)
    assert.ok(fields.description && fields.description.length <= 1024, `${dir}: description`)
    for (const key of Object.keys(fields)) assert.ok(allowed.has(key), `${dir}: ${key} is not an Agent Skills field`)
  }
})

test('no command hides a skill: a command and a skill never share a name', () => {
  const commands = readdirSync(join(ROOT, 'commands')).map((f) => f.replace(/\.md$/, ''))
  for (const skill of readdirSync(join(ROOT, 'skills'))) assert.ok(!commands.includes(skill), skill)
})

test('the executables a client runs as they are can be run', () => {
  for (const file of ['bin/flowapi', 'bin/judge.mjs', 'hooks/guard.mjs', 'scripts/docs.mjs']) {
    accessSync(join(ROOT, file), constants.X_OK)
    assert.match(read(ROOT, file), /^#!\/(usr\/)?bin\/(env )?(node|sh)\n/, `${file}: interpreter line`)
  }
})
