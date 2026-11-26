// The package's command against the Rust one, built from the same sources: the same standard
// output, the same diagnostics on standard error and the same exit code, byte for byte, on
// petstore in JSON and YAML, every kept case of crates/flowapi-ts, a description split over three
// files, and YAML: files reached from a YAML description, its diagnostics and its errors; and the
// commands about an SDK's config, `summary` and `config draft|schema|check`.
// Run after the build (`bun run build`); the Rust command is built here if it is stale. `generate`
// is held to it in generate.test.ts.

import { beforeAll, describe, expect, test } from 'bun:test'
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { both, buildRust, github, js, root, run, rust, type Run } from './commands.ts'

beforeAll(buildRust)

const cases = readdirSync(join(root, 'crates', 'flowapi-ts', 'tests', 'cases'))
  .filter((name) => name.endsWith('.json'))
  .sort()
  .map((name) => join('crates', 'flowapi-ts', 'tests', 'cases', name))

describe('the command prints what the Rust command prints', () => {
  const petstore = ['petstore.json', 'petstore.yaml'].map((name) =>
    join('crates', 'flowapi-cli', 'tests', 'yaml', name),
  )
  test.each([...petstore, ...cases])(
    '%s',
    (path) => {
      const { rust: expected, js: actual } = both([path])
      expect(expected.stdout.length + expected.stderr.length).toBeGreaterThan(0)
      expect(actual).toEqual(expected)
    },
  )

  const sdkCases = readdirSync(join(root, 'crates', 'flowapi-ts', 'tests', 'sdk'))
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((name) => join('crates', 'flowapi-ts', 'tests', 'sdk', name))
  test.each(sdkCases)('--sdk %s', (path) => {
    const { rust: expected, js: actual } = both([path, '--sdk'])
    expect(expected.stdout.length).toBeGreaterThan(0)
    expect(actual).toEqual(expected)
  })

  test('a description whose references reach two more files', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    mkdirSync(join(dir, 'defs'))
    const openapi = {
      openapi: '3.1.0',
      info: { title: 'Split', version: '1' },
      paths: {},
      components: {
        schemas: {
          Pet: { $ref: './defs/pet.json#/Pet' },
          Missing: { $ref: './defs/none.json' },
        },
      },
    }
    const pet = { Pet: { type: 'object', properties: { owner: { $ref: '../owner.json' } } } }
    const owner = { type: 'object', required: ['name'], properties: { name: { type: 'string' } } }
    writeFileSync(join(dir, 'openapi.json'), JSON.stringify(openapi, null, 2))
    writeFileSync(join(dir, 'defs', 'pet.json'), JSON.stringify(pet, null, 2))
    writeFileSync(join(dir, 'owner.json'), JSON.stringify(owner, null, 2))
    const { rust: expected, js: actual } = both(['openapi.json', '-o', '-'], dir)
    expect(actual).toEqual(expected)
    expect(expected.stdout + expected.stderr).toContain('name')
  })

  test('a document that is not JSON: the same error, line and column', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    writeFileSync(join(dir, 'bad.json'), '{\n  "openapi": "3.1.0",\n  "info": tru\n}\n')
    const { rust: expected, js: actual } = both(['bad.json'], dir)
    expect(expected.status).toBe(1)
    expect(expected.stderr).toContain('bad.json:3:')
    expect(actual).toEqual(expected)
  })

  test('a YAML description whose references reach a YAML and a JSON file', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    mkdirSync(join(dir, 'defs'))
    const openapi = [
      'openapi: 3.1.0',
      'info: {title: Split, version: "1"}',
      'paths: {}',
      'components:',
      '  schemas:',
      '    Pet: {$ref: "./defs/pet.yml#/Pet"}',
      // Two names that make one identifier: a warning at the second one's YAML line.
      '    pet.kind: {type: string}',
      '    pet_kind: {type: string}',
    ]
    const pet = ['Pet:', '  type: object', '  properties:', '    owner: {$ref: ../owner.json}']
    const owner = { type: 'object', required: ['name'], properties: { name: { type: 'string' } } }
    writeFileSync(join(dir, 'openapi.yaml'), `${openapi.join('\n')}\n`)
    writeFileSync(join(dir, 'defs', 'pet.yml'), `${pet.join('\n')}\n`)
    writeFileSync(join(dir, 'owner.json'), JSON.stringify(owner, null, 2))
    const { rust: expected, js: actual } = both(['openapi.yaml'], dir)
    expect(expected.stdout).toContain('name')
    expect(expected.stderr).toContain('\n  at openapi.yaml:8:15\n')
    expect(actual).toEqual(expected)
  })

  test('a YAML text that JSON cannot hold: the same error, line and column', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    writeFileSync(join(dir, 'bad.yaml'), 'openapi: 3.1.0\ninfo:\n  title: t\n version: "1"\n')
    writeFileSync(join(dir, 'twice.yml'), 'openapi: 3.1.0\nopenapi: 3.0.0\n')
    for (const name of ['bad.yaml', 'twice.yml']) {
      const { rust: expected, js: actual } = both([name], dir)
      expect(expected.status).toBe(1)
      expect(expected.stderr).toStartWith('error[yaml.syntax]: ')
      expect(expected.stderr).toContain(`at ${name}:`)
      expect(actual).toEqual(expected)
    }
  })

  test('the help is the Rust command\'s, less its Rust target', () => {
    const { rust: expected, js: actual } = both(['--help'])
    const lines = expected.stdout.split('\n')
    const other: string[] = []
    let at = 0
    for (const line of actual.stdout.split('\n')) {
      const found = lines.indexOf(line, at)
      if (found === -1) other.push(line)
      else at = found + 1
    }
    expect(other).toEqual([
      'Usage: flowapi <description> [-o <types.ts>] [--sdk] [--quiet]',
      '                        reads its response, with no HTTP library and no I/O',
    ])
    expect(actual.status).toBe(0)
  })

  test('the exit codes of a bad command line and a missing file', () => {
    for (const args of [[], ['--nope'], ['a.json', 'b.json'], ['-o'], ['missing.json']]) {
      const { rust: expected, js: actual } = both(args)
      expect({ args, status: actual.status }).toEqual({ args, status: expected.status })
    }
  })
})

describe('the commands about an SDK\'s config print what the Rust command prints', () => {
  const sdk = join('crates', 'flowapi-sdk', 'tests', 'config')
  const small = join(sdk, 'spec.json')
  const yaml = join('crates', 'flowapi-cli', 'tests', 'yaml', 'petstore.yaml')
  const configs = readdirSync(join(root, sdk))
    .filter((name) => name.endsWith('.yml'))
    .sort()
    .map((name) => join(sdk, name))

  /** Rows of a table test: each command line under its own name. */
  const named = (rows: readonly (readonly string[])[]): [string, readonly string[]][] =>
    rows.map((args) => [args.join(' '), args])

  test.each(named([
    ['summary', small],
    ['summary', small, '--tag', 'owners'],
    ['summary', small, '--path', '/pets', '--max-bytes', '+900'],
    ['summary', small, '--max-bytes', '500'],
    ['summary', yaml],
    ['config', 'draft', small],
    ['config', 'draft', yaml],
    ['config', 'draft', small, '-o', '-'],
    ['config', 'schema'],
  ]))('%s', (_, args) => {
    const { rust: expected, js: actual } = both(args)
    expect(expected.status).toBe(0)
    expect(expected.stdout.length).toBeGreaterThan(0)
    expect(actual).toEqual(expected)
  })

  const withGithub = test.skipIf(github === undefined)
  withGithub.each(named([[], ['--tag', 'repos']]))('summary of GitHub %s', (_, flags) => {
    const { rust: expected, js: actual } = both(['summary', github as string, ...flags])
    expect(expected.stdout).toStartWith('{"info":')
    expect(actual).toEqual(expected)
  })

  withGithub('config draft of GitHub', () => {
    const { rust: expected, js: actual } = both(['config', 'draft', github as string])
    expect(expected.stdout).toStartWith('# A first flowapi-sdk.yml')
    expect(actual).toEqual(expected)
  })

  const checks = named(configs.flatMap((config) => [[config], [config, '--json']]))
  test.each(checks)('config check %s', (_, args) => {
    const { rust: expected, js: actual } = both(['config', 'check', ...args])
    expect(expected.stdout.length).toBeGreaterThan(0)
    expect(actual).toEqual(expected)
  })

  test('a config with several issues, a passing one, and their exit codes', () => {
    const failing = both(['config', 'check', join(sdk, 'refs.yml')])
    expect(failing.rust.status).toBe(1)
    expect(failing.rust.stdout.match(/^error\[/gm)?.length).toBeGreaterThan(1)
    const passing = both(['config', 'check', join(sdk, 'valid.yml')])
    const stdout = `${join(sdk, 'valid.yml')}: no issues\n`
    expect(passing.rust).toEqual({ status: 0, stdout, stderr: '' })
    expect([failing.js, passing.js]).toEqual([failing.rust, passing.rust])
  })

  test('a draft written beside another folder names its description relative to the config', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    mkdirSync(join(dir, 'specs'))
    mkdirSync(join(dir, 'out'))
    writeFileSync(join(dir, 'specs', 'openapi.json'), readFileSync(join(root, small)))
    const spec = join('specs', 'openapi.json')
    const draft = (name: string) => ['config', 'draft', spec, '-o', join('out', name)]
    const expected = run(rust, draft('rust.yml'), dir)
    const actual = run('node', [js, ...draft('js.yml')], dir)
    expect(actual).toEqual(expected)
    const written = readFileSync(join(dir, 'out', 'rust.yml'), 'utf8')
    expect(written).toContain('spec: "../specs/openapi.json"')
    expect(readFileSync(join(dir, 'out', 'js.yml'), 'utf8')).toBe(written)
    for (const flags of [[], ['--json']]) {
      const checked = both(['config', 'check', join('out', 'js.yml'), ...flags], dir)
      expect(checked.rust.status).toBe(0)
      expect(checked.js).toEqual(checked.rust)
    }
  })

  test('a config whose spec is a URL, is given by --spec, is missing, or is not YAML', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    writeFileSync(join(dir, 'openapi.json'), readFileSync(join(root, small)))
    const rest = 'package: {name: a, client: A}\nresources: {}\n'
    writeFileSync(join(dir, 'url.yml'), `version: 1\nspec: https://example.com/openapi.json\n${rest}`)
    writeFileSync(join(dir, 'none.yml'), `version: 1\n${rest}`)
    writeFileSync(join(dir, 'bad.yml'), 'version: 1\nspec:\n  a: 2\n b: 3\n')
    writeFileSync(join(dir, 'bad.yaml'), 'openapi: 3.1.0\ninfo:\n  title: t\n version: "1"\n')
    for (const args of [
      ['url.yml'],
      ['url.yml', '--spec', 'openapi.json'],
      ['url.yml', '--spec', 'openapi.json', '--json'],
      ['url.yml', '--spec', 'bad.yaml'],
      ['none.yml'],
      ['bad.yml'],
    ]) {
      const { rust: expected, js: actual } = both(['config', 'check', ...args], dir)
      expect({ args, status: expected.status }).toEqual({ args, status: 1 })
      expect({ args, ...actual }).toEqual({ args, ...expected })
    }
  })

  test('a document that is not a description, or cannot be read as one', () => {
    const dir = mkdtempSync(join(tmpdir(), 'flowapi-parity-'))
    writeFileSync(join(dir, 'schema.json'), '{"type": "string"}')
    writeFileSync(join(dir, 'bad.json'), '{"openapi": 3.1.0')
    const commands = [['summary', 'schema.json'], ['config', 'draft', 'schema.json']]
    for (const args of [...commands, ['summary', 'bad.json']]) {
      const { rust: expected, js: actual } = both(args, dir)
      expect({ args, status: expected.status }).toEqual({ args, status: 1 })
      expect({ args, ...actual }).toEqual({ args, ...expected })
    }
  })

  test('a bad command line: the same first line and exit code', () => {
    for (const args of [
      ['summary'],
      ['summary', 'a', 'b'],
      ['summary', 'x', '--max-bytes', 'abc'],
      ['summary', 'x', '--max-bytes', '99999999999999999999'],
      ['config'],
      ['config', 'make'],
      ['config', 'check'],
      ['config', 'schema', 'x'],
      ['config', 'schema', '--json'],
      ['config', 'check', 'x', '-o', 'y'],
      ['config', 'draft', 'x', '-o'],
    ]) {
      const { rust: expected, js: actual } = both(args)
      const first = (r: Run) => ({ status: r.status, line: r.stderr.split('\n')[0] })
      expect({ args, ...first(actual) }).toEqual({ args, ...first(expected) })
      expect(expected.status).toBe(2)
    }
  })
})
