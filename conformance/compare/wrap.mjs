// A suite group as an OpenAPI 3.1 document: its schema as `components.schemas.Root`, each of
// its `externalSchemas` as `components.schemas.Remote<i>`, and every `$ref` and `$dynamicRef`
// that resolves (against the base URI that `$id` sets) to one of those documents or to a
// resource or anchor embedded in them rewritten into a local `#/components/schemas/...`
// pointer. A reference that resolves to nothing known is left as written.

// The URI the Rust harness adds a group's schema under (conformance/suite/src/generate.rs).
export const rootUri = (file, group) => `https://json-schema.org/flowapi-suite/${file}/${group}.json`

const MAPS = ['properties', 'patternProperties', '$defs', 'definitions', 'dependentSchemas', 'dependencies']
const LISTS = ['allOf', 'anyOf', 'oneOf', 'prefixItems', 'items']
const SINGLE = [
  'not', 'if', 'then', 'else', 'contains', 'additionalProperties', 'propertyNames', 'items',
  'unevaluatedItems', 'unevaluatedProperties', 'additionalItems', 'contentSchema',
]

const isSchema = (v) => typeof v === 'boolean' || (v !== null && typeof v === 'object' && !Array.isArray(v))

// The schemas directly under a schema object, as [key path, subschema].
function children(schema) {
  const out = []
  for (const k of MAPS) {
    const m = schema[k]
    if (m && typeof m === 'object' && !Array.isArray(m)) {
      for (const [name, s] of Object.entries(m)) if (isSchema(s)) out.push([[k, name], s])
    }
  }
  for (const k of LISTS) {
    if (Array.isArray(schema[k])) schema[k].forEach((s, i) => isSchema(s) && out.push([[k, String(i)], s]))
  }
  for (const k of SINGLE) if (isSchema(schema[k])) out.push([[k], schema[k]])
  return out
}

function resolve(ref, base) {
  try {
    return new URL(ref, base).href
  } catch {
    return null
  }
}

const withoutFragment = (uri) => uri.split('#')[0]

// A JSON Pointer as a URI fragment: `~` and `/` escaped per segment, then percent-encoded.
function fragment(segments) {
  const enc = (s) =>
    s.replace(/~/g, '~0').replace(/\//g, '~1').replace(/[^A-Za-z0-9\-._~!$&'()*+,;=:@]/g, (c) =>
      [...Buffer.from(c, 'utf8')].map((b) => `%${b.toString(16).toUpperCase().padStart(2, '0')}`).join(''),
    )
  return segments.map((s) => `/${enc(s)}`).join('')
}

// Every resource (by its URI without fragment) and anchor (by `uri#name`) in a document, as the
// pointer of its schema in the OpenAPI document.
function index(schema, base, at, resources, anchors) {
  const stack = [[schema, base, at]]
  while (stack.length > 0) {
    const [s, b, path] = stack.pop()
    if (!isSchema(s) || typeof s === 'boolean') continue
    let here = b
    if (typeof s.$id === 'string') {
      const id = resolve(s.$id, b)
      if (id) {
        here = withoutFragment(id)
        if (!resources.has(here)) resources.set(here, path)
      }
    }
    for (const k of ['$anchor', '$dynamicAnchor']) {
      if (typeof s[k] === 'string') {
        const key = `${here}#${s[k]}`
        if (!anchors.has(key)) anchors.set(key, path)
      }
    }
    for (const [keys, sub] of children(s)) stack.push([sub, here, [...path, ...keys]])
  }
}

// A copy of `schema` with its references rewritten.
function rewrite(schema, base, known) {
  if (!isSchema(schema) || typeof schema === 'boolean') return schema
  let here = base
  if (typeof schema.$id === 'string') here = withoutFragment(resolve(schema.$id, base) ?? base)
  const out = {}
  const subs = new Map(children(schema).map(([keys, sub]) => [keys.join('\u0000'), sub]))
  for (const [k, v] of Object.entries(schema)) {
    let value = v
    if ((k === '$ref' || k === '$dynamicRef') && typeof v === 'string') value = local(v, here, known) ?? v
    else if (MAPS.includes(k) && v && typeof v === 'object' && !Array.isArray(v)) {
      value = {}
      for (const [name, s] of Object.entries(v)) {
        const sub = subs.has(`${k}\u0000${name}`) ? rewrite(s, here, known) : s
        Object.defineProperty(value, name, { value: sub, enumerable: true, writable: true, configurable: true })
      }
    } else if (LISTS.includes(k) && Array.isArray(v)) {
      value = v.map((s, i) => (subs.has(`${k}\u0000${i}`) ? rewrite(s, here, known) : s))
    } else if (SINGLE.includes(k) && isSchema(v)) value = rewrite(v, here, known)
    Object.defineProperty(out, k, { value, enumerable: true, writable: true, configurable: true })
  }
  return out
}

// The local `#/...` form of a reference, or null when it resolves to no known document.
function local(ref, base, { resources, anchors }) {
  const abs = resolve(ref, base)
  if (!abs) return null
  const hash = abs.indexOf('#')
  const doc = hash < 0 ? abs : abs.slice(0, hash)
  const frag = hash < 0 ? '' : abs.slice(hash + 1)
  if (frag === '' || frag.startsWith('/')) {
    const at = resources.get(doc)
    return at ? `#${fragment(at)}${frag}` : null
  }
  const at = anchors.get(`${doc}#${frag}`) ?? anchors.get(`${doc}#${decodeURIComponent(frag)}`)
  return at ? `#${fragment(at)}` : null
}

// The OpenAPI document of one group, and the component names of its documents.
export function wrap(file, g) {
  const docs = [['Root', rootUri(file, g.index), g.schema]]
  g.external.forEach(([uri, s], i) => docs.push([`Remote${i}`, uri, s]))
  const resources = new Map()
  const anchors = new Map()
  for (const [name, uri, s] of docs) {
    const at = ['components', 'schemas', name]
    const retrieval = withoutFragment(resolve(uri, uri) ?? uri)
    resources.set(retrieval, at)
    index(s, retrieval, at, resources, anchors)
  }
  const known = { resources, anchors }
  const schemas = {}
  for (const [name, uri, s] of docs) schemas[name] = rewrite(s, withoutFragment(resolve(uri, uri) ?? uri), known)
  return {
    openapi: '3.1.0',
    jsonSchemaDialect: 'https://json-schema.org/draft/2020-12/schema',
    info: { title: `${file} #${g.index}`, version: '0.0.0' },
    paths: {},
    components: { schemas },
  }
}
