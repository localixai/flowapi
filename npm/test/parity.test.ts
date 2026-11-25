// The package's command against the Rust one, built from the same sources: the same standard
// output, the same diagnostics on standard error and the same exit code, byte for byte, on
// petstore in JSON and YAML, every kept case of crates/flowapi-ts, a description split over three
// files, and YAML: files reached from a YAML description, its diagnostics and its errors.
// Run after the build (`bun run build`); the Rust command is built here if it is stale.

import { beforeAll, describe, expect, test } from 'bun:test'
import { spawnSync } from 'node:child_process'
import { mkdirSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const npm = join(import.meta.dir, '..')
const root = join(npm, '..')
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target')
const rust = join(target, 'release', 'flowapi')
const js = join(npm, 'dist', 'cli.js')

interface Run {
  readonly status: number | null
  readonly stdout: string
  readonly stderr: string
}

function run(command: string, args: readonly string[], cwd: string): Run {
  const done = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 1 << 28 })
  if (done.error) throw done.error
  return { status: done.status, stdout: done.stdout, stderr: done.stderr }
}

/** Both commands on the same arguments, from the same directory. */
function both(args: readonly string[], cwd = root): { rust: Run; js: Run } {
  return { rust: run(rust, args, cwd), js: run('node', [js, ...args], cwd) }
}

beforeAll(() => {
  const built = run('cargo', ['build', '--locked', '--release', '-p', 'flowapi-cli'], root)
  if (built.status !== 0) throw new Error(`the Rust command did not build:\n${built.stderr}`)
})

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

  test('the exit codes of a bad command line and a missing file', () => {
    for (const args of [[], ['--nope'], ['a.json', 'b.json'], ['-o'], ['missing.json']]) {
      const { rust: expected, js: actual } = both(args)
      expect({ args, status: actual.status }).toEqual({ args, status: expected.status })
    }
  })
})
