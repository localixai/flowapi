// `flowapi generate` through the package against the Rust command: each run in two copies of
// the same folder, one per command, and every file both leave (bytes and the executable bit),
// `.flowapi/lock.json` among them, compared with their standard output, standard error and exit
// code. On the `shop` and `auth` client cases and GitHub's description with a drafted config,
// then each rule of the writing (crates/flowapi-cli/tests/generate_cli.rs): a second run, a
// generated file edited, the user's files, orphans, `--check`, paths out of the folder. And the
// lock's hashes, ours (crates/flowapi-sdk/src/sha256.rs), against node:crypto's.
// Run after the build (`bun run build`); the Rust command is built here if it is stale.

import { beforeAll, describe, expect, test } from 'bun:test'
import { createHash } from 'node:crypto'
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { both, buildRust, github, js, root, run, rust, type Run } from './commands.ts'

beforeAll(buildRust)

const client = join(root, 'crates', 'flowapi-sdk', 'tests', 'client')

/** Every file under a folder: its bytes and whether it is executable, by path. */
function tree(dir: string): Record<string, string> {
  const out: Record<string, string> = {}
  const walk = (at: string, prefix: string) => {
    for (const entry of readdirSync(at, { withFileTypes: true })) {
      const path = join(at, entry.name)
      const name = `${prefix}${entry.name}`
      if (entry.isDirectory()) walk(path, `${name}/`)
      else if (entry.isSymbolicLink()) out[name] = 'a symbolic link'
      else {
        const x = (statSync(path).mode & 0o111) !== 0 ? 'x ' : ''
        out[name] = `${x}${readFileSync(path).toString('base64')}`
      }
    }
  }
  walk(dir, '')
  return out
}

/** Two copies of a folder, one for each command; every step changes both alike. */
class Pair {
  readonly dirs: { rust: string; js: string }

  constructor(files: Record<string, string>) {
    const base = mkdtempSync(join(tmpdir(), 'flowapi-generate-'))
    this.dirs = { rust: join(base, 'rust'), js: join(base, 'js') }
    for (const dir of Object.values(this.dirs)) {
      mkdirSync(dir)
      for (const [name, from] of Object.entries(files)) cpSync(from, join(dir, name))
    }
  }

  /** A change made in both folders. */
  edit(change: (dir: string) => void): void {
    for (const dir of Object.values(this.dirs)) change(dir)
  }

  /** `generate flowapi-sdk.yml --out sdk` and `more` by each command in its folder: the same
   * output and exit code, and the same files left behind. */
  generate(more: readonly string[] = [], config = 'flowapi-sdk.yml'): Run {
    const args = ['generate', config, '--out', 'sdk', ...more]
    const expected = run(rust, args, this.dirs.rust)
    const actual = run('node', [js, ...args], this.dirs.js)
    expect(actual).toEqual(expected)
    expect(tree(this.dirs.js)).toEqual(tree(this.dirs.rust))
    return expected
  }
}

function replace(path: string, from: string, to: string): void {
  const text = readFileSync(path, 'utf8')
  expect(text).toContain(from)
  writeFileSync(path, text.replace(from, to))
}

const fixture = (name: string) => ({
  'flowapi-sdk.yml': join(client, name, 'flowapi-sdk.yml'),
  'spec.json': join(client, name, 'spec.json'),
})

describe('generate writes what the Rust command writes', () => {
  test.each(['shop', 'auth'])('%s: a fresh repository, a second run, --check', (name) => {
    const pair = new Pair(fixture(name))
    expect(pair.generate().stderr).toContain(' written, 0 deleted, 0 unchanged\n')
    expect(pair.generate().stderr).toContain(': 0 written, 0 deleted, ')
    expect(pair.generate(['--check'])).toEqual({
      status: 0,
      stdout: '',
      stderr: 'flowapi: sdk is as generated\n',
    })
  })

  test('the lock holds the SHA-256 node:crypto gives of each generated file', () => {
    const pair = new Pair(fixture('shop'))
    pair.generate()
    const sdk = join(pair.dirs.js, 'sdk')
    const lock = JSON.parse(readFileSync(join(sdk, '.flowapi', 'lock.json'), 'utf8')) as {
      files: Record<string, string>
      spec_sha256: string
      config_sha256: string
    }
    const sha = (path: string) => createHash('sha256').update(readFileSync(path)).digest('hex')
    expect(Object.keys(lock.files).length).toBeGreaterThan(40)
    for (const [path, hash] of Object.entries(lock.files)) {
      expect([path, sha(join(sdk, path))]).toEqual([path, hash])
    }
    expect(lock.spec_sha256).toBe(sha(join(pair.dirs.js, 'spec.json')))
    expect(lock.config_sha256).toBe(sha(join(pair.dirs.js, 'flowapi-sdk.yml')))
  })

  test.skipIf(github === undefined)('GitHub with a drafted config', () => {
    const pair = new Pair({})
    const draft = ['config', 'draft', github as string, '-o', 'flowapi-sdk.yml']
    pair.edit((dir) => expect(run(rust, draft, dir).status).toBe(0))
    const timed = (command: string, args: string[], cwd: string) => {
      const start = performance.now()
      const done = run(command, args, cwd)
      return { done, ms: Math.round(performance.now() - start) }
    }
    const args = ['generate', 'flowapi-sdk.yml', '--out', 'sdk']
    const expected = timed(rust, args, pair.dirs.rust)
    const actual = timed('node', [js, ...args], pair.dirs.js)
    console.log(`generate on GitHub: Rust ${expected.ms} ms, npm ${actual.ms} ms`)
    expect(expected.done.status).toBe(0)
    expect(actual.done).toEqual(expected.done)
    expect(tree(pair.dirs.js)).toEqual(tree(pair.dirs.rust))
    expect(pair.generate().stderr).toContain(': 0 written, 0 deleted, ')
  }, 60_000)
})

describe('generate keeps the Rust command\'s rules', () => {
  const config = (dir: string) => join(dir, 'flowapi-sdk.yml')
  const bump = (dir: string) => replace(config(dir), 'version: 1.2.0', 'version: 1.3.0')

  test('a generated file edited by hand is a conflict, with and without --check', () => {
    const pair = new Pair(fixture('shop'))
    pair.generate()
    pair.edit((dir) => writeFileSync(join(dir, 'sdk', 'README.md'), '# mine\n'))
    pair.edit(bump)
    for (const more of [[], ['--check']]) {
      const done = pair.generate(more)
      expect(done.status).toBe(1)
      expect(done.stderr).toContain('conflict: README.md was changed')
    }
    pair.edit((dir) => rmSync(join(dir, 'sdk', 'README.md')))
    expect(pair.generate().status).toBe(0)
  })

  test('a file in the way, the user\'s files kept, a changed one with a warning', () => {
    const pair = new Pair(fixture('shop'))
    pair.edit((dir) => {
      mkdirSync(join(dir, 'sdk'))
      writeFileSync(join(dir, 'sdk', 'package.json'), '{}\n')
    })
    expect(pair.generate().stderr).toContain('package.json is in the way')
    pair.edit((dir) => rmSync(join(dir, 'sdk', 'package.json')))
    pair.generate()
    pair.edit((dir) => {
      mkdirSync(join(dir, 'sdk', 'src', 'lib'), { recursive: true })
      writeFileSync(join(dir, 'sdk', 'src', 'lib', 'x.ts'), 'export const x = 1\n')
      writeFileSync(join(dir, 'sdk', 'examples', '.keep'), 'mine\n')
      writeFileSync(join(dir, 'sdk', 'CHANGELOG.md'), '# Changelog\n\n## 1.2.0\n')
      writeFileSync(join(dir, 'sdk', 'notes.txt'), 'not generated\n')
    })
    pair.edit(bump)
    const done = pair.generate()
    expect(done.status).toBe(0)
    expect(done.stderr).toContain('examples/.keep was changed: it is yours, and kept')
    const kept = readFileSync(join(pair.dirs.js, 'sdk', 'src', 'lib', 'x.ts'), 'utf8')
    expect(kept).toBe('export const x = 1\n')
  })

  test('a file no longer generated is deleted, or kept with a warning when changed', () => {
    const pair = new Pair(fixture('shop'))
    pair.generate()
    pair.edit((dir) => {
      replace(config(dir), '  system:\n    methods:\n      health: health\n', '')
      const tags = '  tags:\n    methods:\n      list: { op: listTags, pagination: one }\n'
      replace(config(dir), tags, '')
      const skip = [
        'skip:',
        '  - { op: health, reason: internal }',
        '  - { op: listTags, reason: internal }',
        '',
      ].join('\n')
      replace(config(dir), 'readme:', `${skip}readme:`)
      writeFileSync(join(dir, 'sdk', 'src', 'resources', 'tags.ts'), '// mine\n')
    })
    const done = pair.generate()
    expect(done.status).toBe(0)
    expect(done.stderr).toContain('src/resources/tags.ts is no longer generated, but was changed')
    expect(done.stderr).not.toContain(' 0 deleted')
  })

  test('--check writes nothing and says what differs', () => {
    const pair = new Pair(fixture('shop'))
    const fresh = pair.generate(['--check'])
    expect(fresh.status).toBe(1)
    expect(fresh.stderr).toContain('write package.json\n')
    pair.generate()
    pair.edit(bump)
    const changed = pair.generate(['--check'])
    expect(changed.status).toBe(1)
    expect(changed.stderr).toContain('flowapi: sdk differs from what generate writes in ')
  })

  test('a lock naming a path out of the folder, a symbolic link, a lock that is not one', () => {
    const pair = new Pair(fixture('shop'))
    pair.generate()
    const lock = (dir: string) => join(dir, 'sdk', '.flowapi', 'lock.json')
    pair.edit((dir) => replace(lock(dir), '"api.md"', '"../outside.md"'))
    expect(pair.generate().stderr).toContain('is not a path inside the repository')
    pair.edit((dir) => writeFileSync(lock(dir), '{"version": 2}\n'))
    expect(pair.generate().stderr).toContain('this flowapi reads version 1')
    const linked = new Pair(fixture('shop'))
    linked.edit((dir) => {
      mkdirSync(join(dir, 'elsewhere'))
      mkdirSync(join(dir, 'sdk'))
      symlinkSync(join(dir, 'elsewhere'), join(dir, 'sdk', 'src'))
    })
    expect(linked.generate().stderr).toContain('is a symbolic link')
  })

  test('a missing description, a config with errors, one naming no spec or a URL', () => {
    const pair = new Pair(fixture('shop'))
    pair.edit((dir) => replace(config(dir), 'client: Shop', 'client: shop'))
    expect(pair.generate().stderr).toContain('error[config.package.client]')
    pair.edit((dir) => replace(config(dir), 'spec: spec.json', 'spec: https://example.com/a.json'))
    expect(pair.generate().stderr).toContain('which generate does not fetch')
    pair.edit((dir) => replace(config(dir), 'spec: https://example.com/a.json\n', ''))
    expect(pair.generate().stderr).toContain('names no description')
    // A missing file: the same exit code and first words; the OS's reason is worded by each.
    const missing = new Pair(fixture('shop'))
    missing.edit((dir) => rmSync(join(dir, 'spec.json')))
    const args = ['generate', 'flowapi-sdk.yml', '--out', 'sdk']
    const expected = run(rust, args, missing.dirs.rust)
    const actual = run('node', [js, ...args], missing.dirs.js)
    const start = (r: Run) => ({ status: r.status, start: r.stderr.split(': ').slice(0, 2) })
    expect(start(actual)).toEqual(start(expected))
    expect(expected.stderr).toStartWith('flowapi: cannot read spec.json: ')
  })

  test('a bad command line: the same first line and exit code', () => {
    for (const args of [
      ['generate'],
      ['generate', 'a.yml'],
      ['generate', 'a.yml', '--out'],
      ['generate', 'a.yml', 'b.yml', '--out', 'x'],
      ['generate', 'a.yml', '--out', 'x', '--json'],
    ]) {
      const { rust: expected, js: actual } = both(args)
      const first = (r: Run) => ({ status: r.status, line: r.stderr.split('\n')[0] })
      expect({ args, ...first(actual) }).toEqual({ args, ...first(expected) })
      expect(expected.status).toBe(2)
    }
  })
})
