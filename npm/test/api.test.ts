// The package's API, run on the built package (dist/), as a user imports it.

import { describe, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import {
  configCheck,
  configDraft,
  configSchema,
  formatDiagnostic,
  generate,
  load,
  loadSdk,
  summary,
  type Diagnostic,
} from '../dist/index.js'
import { ReadError, generateFile, readDescription } from '../dist/node.js'

const root = join(import.meta.dir, '..', '..')
const dist = join(import.meta.dir, '..', 'dist')
const petstore = join(root, 'crates', 'flowapi-cli', 'tests', 'yaml', 'petstore.json')

describe('generate', () => {
  test('a description as text, one file at the path given', async () => {
    const result = await generate(readFileSync(petstore, 'utf8'), { output: 'pets.ts' })
    expect(result.diagnostics).toEqual([])
    expect(result.files.map((f) => f.path)).toEqual(['pets.ts'])
    expect(result.files[0]?.text).toContain('export type Pet')
  })

  test('the same bytes from text, a document, a list and a file', async () => {
    const text = readFileSync(petstore, 'utf8')
    const flowapi = await load()
    const outputs = [
      (await generate(text)).files,
      flowapi.generate({ uri: petstore, text }).files,
      flowapi.generate([{ uri: petstore, text }]).files,
      (await generateFile(petstore)).files,
    ]
    for (const files of outputs) expect(files).toEqual(outputs[0] as typeof files)
    expect(outputs[0]?.[0]?.path).toBe('types.ts')
  })

  test('a document that is not JSON is an error with its line and column, and no file', async () => {
    const result = await generate('{\n  "openapi": "3.1.0",\n  "info": tru\n}')
    expect(result.files).toEqual([])
    const [d] = result.diagnostics as [Diagnostic]
    expect(result.diagnostics).toHaveLength(1)
    expect(d).toMatchObject({ severity: 'error', code: 'json.syntax', uri: 'input.json' })
    expect({ line: d.line, column: d.column, offset: d.offset }).toEqual({
      line: 3,
      column: 11,
      offset: 34,
    })
    expect(formatDiagnostic(d)).toMatch(/^error\[json\.syntax\]: .*\n {2}at input\.json:3:11$/u)
  })

  test('a column counts characters, an offset counts UTF-8 bytes', async () => {
    const result = await generate('{"é€": tru}')
    const [d] = result.diagnostics as [Diagnostic]
    expect({ offset: d.offset, column: d.column }).toEqual({ offset: 10, column: 8 })
  })

  test('a warning names its document, line and column', async () => {
    const uri = 'warn.json'
    const text =
      '{\n  "$defs": {\n    "a": { "$id": "https://example.com/x" },\n' +
      '    "b": { "$id": "https://example.com/x" }\n  }\n}'
    const { files, diagnostics } = await generate({ uri, text })
    expect(files).toHaveLength(1)
    expect(diagnostics).toHaveLength(1)
    const [d] = diagnostics as [Diagnostic]
    expect(d).toMatchObject({ severity: 'warning', code: 'jsonschema-2020-12.core.$id', uri })
    const before = new TextDecoder().decode(new TextEncoder().encode(text).subarray(0, d.offset))
    expect(d.line).toBe(before.split('\n').length)
    expect(d.column).toBe([...(before.split('\n').at(-1) ?? '')].length + 1)
    expect(d.line).toBeGreaterThan(1)
  })

  test('an empty list is refused, not thrown', async () => {
    const result = await generate([])
    expect(result.files).toEqual([])
    expect(result.diagnostics[0]).toMatchObject({ severity: 'error', code: 'flowapi.request' })
  })
})

describe('load', () => {
  test('references name other documents, in order', async () => {
    const flowapi = await load()
    const text = JSON.stringify({
      a: { $ref: '#/b' },
      c: [{ $ref: 'other.json#/x' }, { $schema: 'https://json-schema.org/draft/2020-12/schema' }],
    })
    expect(flowapi.references(text)).toEqual([
      'other.json#/x',
      'https://json-schema.org/draft/2020-12/schema',
    ])
    expect(flowapi.references('not json')).toEqual([])
  })

  test('a thousand calls hold no memory between them', async () => {
    const flowapi = await load()
    const text = readFileSync(petstore, 'utf8')
    const first = flowapi.generate(text)
    for (let i = 0; i < 1000; i++) flowapi.generate(text)
    expect(flowapi.generate(text)).toEqual(first)
  })
})

describe('readDescription', () => {
  test('a YAML file generates what its JSON twin does', async () => {
    const yaml = join(root, 'crates', 'flowapi-cli', 'tests', 'yaml', 'petstore.yaml')
    const documents = await readDescription(yaml)
    expect(documents.map((d) => d.uri)).toEqual([yaml])
    expect((await generateFile(yaml)).files).toEqual((await generateFile(petstore)).files)
  })

  test('a YAML text is told by its URI, and bare text is JSON', async () => {
    const flowapi = await load()
    const yaml = 'a:\n  $ref: other.yaml#/x\n'
    expect(flowapi.references(yaml, 'spec.yaml')).toEqual(['other.yaml#/x'])
    expect(flowapi.references(yaml, 'spec')).toEqual(['other.yaml#/x'])
    expect(flowapi.references(yaml)).toEqual([])
    const bad = flowapi.generate({ uri: 'bad.yaml', text: 'a: 1\nb:\n  c: 2\n d: 3\n' })
    expect(bad.files).toEqual([])
    expect(bad.diagnostics).toHaveLength(1)
    expect(bad.diagnostics[0]).toMatchObject({ code: 'yaml.syntax', uri: 'bad.yaml', line: 4 })
  })

  test('a missing file is a ReadError', async () => {
    expect(readDescription(join(root, 'missing.json'))).rejects.toBeInstanceOf(ReadError)
  })
})

describe('the commands about an SDK\'s config', () => {
  const spec = join(root, 'crates', 'flowapi-sdk', 'tests', 'config', 'spec.json')
  const description = { uri: 'openapi.json', text: readFileSync(spec, 'utf8') }
  const refs = join(root, 'crates', 'flowapi-sdk', 'tests', 'config', 'refs.yml')

  test('only the second module has the SDK commands', () => {
    const exports = (file: string) =>
      WebAssembly.Module.exports(new WebAssembly.Module(readFileSync(join(dist, file)))).map(
        (e) => e.name,
      )
    expect(exports('flowapi.wasm')).not.toContain('flowapi_sdk')
    expect(exports('flowapi-sdk.wasm')).toContain('flowapi_sdk')
    expect(exports('flowapi-sdk.wasm')).toEqual(expect.arrayContaining(exports('flowapi.wasm')))
  })

  test('summary: compact JSON, filtered; null for what is not a description', async () => {
    const all = await summary(description)
    expect(all.diagnostics).toEqual([])
    expect(JSON.parse(all.text ?? '')).toHaveProperty('info')
    const owners = await summary(description, { tag: 'owners' })
    expect(owners.text).toContain('"id":"listOwners"')
    expect((await summary(description, { maxBytes: 500 })).text).toContain('"truncated"')
    expect(await summary('{"type": "string"}')).toEqual({ text: null, diagnostics: [] })
    const bad = await summary({ uri: 'bad.yaml', text: 'a: 1\nb:\n  c: 2\n d: 3\n' })
    expect(bad.text).toBeNull()
    expect(bad.diagnostics[0]).toMatchObject({ code: 'yaml.syntax', uri: 'bad.yaml', line: 4 })
    const refused = await summary(description, { maxBytes: 1.5 })
    expect(refused.diagnostics[0]).toMatchObject({ code: 'flowapi.request' })
  })

  test('a draft names its spec, passes its check, and the schema is the kept one', async () => {
    const draft = await configDraft(description, '../openapi.json')
    expect(draft.text).toStartWith('# A first sdk-anything.yml')
    expect(draft.text).toContain('spec: "../openapi.json"')
    expect((await configDraft(description)).text).toContain('spec: openapi.json')
    const checked = await configCheck({ uri: 'sdk.yml', text: draft.text ?? '' }, description)
    expect(checked).toMatchObject({ ok: true, errors: 0, issues: [], text: '', diagnostics: [] })
    const kept = readFileSync(join(root, 'crates', 'flowapi-sdk', 'tests', 'schema.json'), 'utf8')
    expect(await configSchema()).toBe(kept)
  })

  test('a check gives every issue as data, text and JSON, at the YAML lines', async () => {
    const flowapi = await loadSdk()
    const config = { uri: 'refs.yml', text: readFileSync(refs, 'utf8') }
    expect(flowapi.configSpec(config)).toEqual({ text: 'spec.json', diagnostics: [] })
    const found = flowapi.configCheck(config, description)
    expect(found.ok).toBe(false)
    expect(found.errors).toBeGreaterThan(1)
    expect(found.issues).toContainEqual(
      expect.objectContaining({ code: 'config.op.unknown', line: 10, column: 13 }),
    )
    expect(found.text).toContain('  at refs.yml:10:13 (/resources/pets/methods/list)')
    expect(JSON.parse(found.json ?? '')).toMatchObject({ ok: false, errors: found.errors })
    const unread = flowapi.configCheck({ uri: 'c.yml', text: 'a:\n  b: 1\n c: 2\n' })
    expect(unread).toMatchObject({ ok: false, text: null, json: null, issues: [] })
    expect(unread.diagnostics[0]).toMatchObject({ code: 'yaml.syntax', uri: 'c.yml' })
  })
})
