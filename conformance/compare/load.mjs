// The JSON-Schema-Test-Suite as the Rust harness reads it (conformance/suite/src/load.rs): every
// top-level file sorted by name, each group admitted the same way, each case's data printed as
// the same one-line literal (keys in written order, numbers as written).
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const DRAFT_2020 = 2020

// Whether a `compatibility` value admits `version`: load.rs `admits`, term by term.
export function admits(compatibility, version) {
  if (typeof compatibility !== 'string') return true
  let ok = true
  for (const raw of compatibility.split(',')) {
    const term = raw.trim()
    let op = ''
    let number = term
    if (term.startsWith('<=')) [op, number] = ['<=', term.slice(2)]
    else if (term.startsWith('=')) [op, number] = ['=', term.slice(1)]
    if (!/^[0-9]+$/.test(number)) return false
    const v = Number(number)
    ok = op === '<=' ? version <= v : op === '=' ? version === v : version >= v
    if (version <= v) break
  }
  return ok
}

// A JSON reader that returns, for every node, the plain value and its compact text as
// serde_json prints it with `preserve_order` and `arbitrary_precision`, and for an object its
// members as nodes. JSON.parse alone would move integer-like keys first and round long numbers.
function parse(text) {
  let i = 0
  const ws = () => {
    while (i < text.length && ' \t\n\r'.includes(text[i])) i++
  }
  const string = () => {
    const start = i
    i++
    while (text[i] !== '"') i += text[i] === '\\' ? 2 : 1
    i++
    return JSON.parse(text.slice(start, i))
  }
  const node = () => {
    ws()
    const c = text[i]
    if (c === '{') {
      i++
      const value = {}
      const members = new Map()
      ws()
      if (text[i] === '}') {
        i++
        return { value, text: '{}', members }
      }
      for (;;) {
        ws()
        const key = string()
        ws()
        i++ // ':'
        const v = node()
        Object.defineProperty(value, key, { value: v.value, enumerable: true, writable: true, configurable: true })
        members.set(key, v)
        ws()
        if (text[i++] === '}') {
          const parts = [...members].map(([k, m]) => `${JSON.stringify(k)}:${m.text}`)
          return { value, text: `{${parts.join(',')}}`, members }
        }
      }
    }
    if (c === '[') {
      i++
      const items = []
      ws()
      if (text[i] === ']') {
        i++
        return { value: [], text: '[]', items }
      }
      for (;;) {
        items.push(node())
        ws()
        if (text[i++] === ']') {
          return { value: items.map((n) => n.value), text: `[${items.map((n) => n.text).join(',')}]`, items }
        }
      }
    }
    if (c === '"') {
      const s = string()
      return { value: s, text: JSON.stringify(s) }
    }
    const m = /^(true|false|null|-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?)/.exec(text.slice(i, i + 400))
    if (!m) throw new Error(`bad JSON at ${i}`)
    i += m[0].length
    return { value: JSON.parse(m[0]), text: m[0] }
  }
  return node()
}

// Every keyword file of the copy: { name, groups: [{ index, description, schema, external,
// cases: [{ description, data, valid }], admission }] }, `data` the case literal's text.
export function suite(dir) {
  const names = readdirSync(dir).filter((n) => n.endsWith('.json')).sort()
  return names.map((n) => {
    const tree = parse(readFileSync(join(dir, n), 'utf8'))
    const groups = tree.members.get('tests').items.map((g, index) => group(index, g))
    return { name: n.slice(0, -'.json'.length), groups }
  })
}

function group(index, node) {
  const g = node.value
  const tagged = Array.isArray(g.tags) && g.tags.includes('format-assertion')
  const admission = !admits(g.compatibility, DRAFT_2020) ? 'dialect' : tagged ? 'tags' : 'run'
  const ext = g.externalSchemas
  const external = ext && typeof ext === 'object' && !Array.isArray(ext) ? Object.entries(ext) : []
  return {
    index,
    description: typeof g.description === 'string' ? g.description : '',
    schema: g.schema,
    external,
    cases: node.members.get('tests').items.map((c) => ({
      description: typeof c.value.description === 'string' ? c.value.description : '',
      data: c.members.get('data').text,
      valid: c.value.valid === true,
    })),
    admission,
  }
}
