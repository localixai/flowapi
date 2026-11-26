// The one mock server every SDK of `sdk-compare.mjs` is run against, made from the description:
// it finds the operation a request is for (the servers' paths taken off, then each path template
// matched segment by segment), records the request as it arrived on the wire (the raw target,
// every header, the body's bytes), and answers it with an example of its first success response,
// unless a test has queued answers of its own (`answer`), which are given first, in order.
import { createServer } from 'node:http'
import { performance } from 'node:perf_hooks'

const METHODS = ['get', 'put', 'post', 'delete', 'options', 'head', 'patch', 'trace']

/** A JSON Pointer reference of the same document, followed. */
export function deref(doc, node) {
  let seen = 0
  while (node && typeof node.$ref === 'string' && node.$ref.startsWith('#/') && seen++ < 32) {
    node = node.$ref
      .slice(2)
      .split('/')
      .map((part) => decodeURIComponent(part).replaceAll('~1', '/').replaceAll('~0', '~'))
      .reduce((at, key) => at?.[key], doc)
  }
  return node
}

/** A value of `schema`: its example, else one made from its type, to `depth` levels. */
export function example(doc, schema, depth = 0) {
  schema = deref(doc, schema)
  if (!schema || typeof schema !== 'object' || depth > 6) return null
  if (schema.example !== undefined) return schema.example
  if (Array.isArray(schema.examples) && schema.examples.length > 0) return schema.examples[0]
  if (schema.default !== undefined) return schema.default
  if (schema.const !== undefined) return schema.const
  if (Array.isArray(schema.enum) && schema.enum.length > 0) return schema.enum[0]
  const alternatives = schema.oneOf ?? schema.anyOf
  if (Array.isArray(alternatives) && alternatives.length > 0) return example(doc, alternatives[0], depth + 1)
  if (Array.isArray(schema.allOf)) {
    return Object.assign({}, ...schema.allOf.map((part) => example(doc, part, depth + 1)).filter((v) => v && typeof v === 'object'))
  }
  const type = Array.isArray(schema.type) ? schema.type.find((t) => t !== 'null') : schema.type
  if (type === 'array') return [example(doc, schema.items, depth + 1)]
  if (type === 'object' || schema.properties) {
    const out = {}
    for (const name of schema.required ?? []) out[name] = example(doc, schema.properties?.[name] ?? {}, depth + 1)
    return out
  }
  if (type === 'integer' || type === 'number') return schema.minimum ?? 1
  if (type === 'boolean') return true
  if (type === 'string') return schema.format === 'date-time' ? '2026-01-01T00:00:00Z' : 'string'
  return null
}

/** The paths of a description's servers, at the top, of a path and of an operation. */
function prefixes(doc) {
  const urls = new Set()
  const add = (servers) => {
    for (const server of servers ?? []) {
      const url = String(server.url ?? '/').replace(/\{([^}]+)\}/g, (_, name) => server.variables?.[name]?.default ?? '')
      urls.add(new URL(url, 'http://mock.invalid').pathname.replace(/\/+$/, ''))
    }
  }
  add(doc.servers)
  for (const item of Object.values(doc.paths ?? {})) {
    add(item.servers)
    for (const method of METHODS) add(item[method]?.servers)
  }
  if (doc.basePath) urls.add(String(doc.basePath).replace(/\/+$/, ''))
  urls.add('')
  return [...urls].sort((a, b) => b.length - a.length)
}

/** A template's segment as a pattern: `{Sid}.json` matches `CA1.json`. */
function segment(part) {
  const escaped = part.replace(/[.*+?^$()|[\]\\]/g, '\\$&')
  return new RegExp(`^${escaped.replace(/\{[^}]+\}/g, '[^/]+')}$`)
}

function routes(doc) {
  const all = []
  for (const [template, item] of Object.entries(doc.paths ?? {})) {
    const parts = template.split('/').slice(1).map(segment)
    for (const method of METHODS) {
      if (item[method]) all.push({ method: method.toUpperCase(), template, parts, op: item[method] })
    }
  }
  // A literal segment before a template's: `/pets/mine` before `/pets/{id}`.
  return all.sort((a, b) => (a.template.match(/\{/g)?.length ?? 0) - (b.template.match(/\{/g)?.length ?? 0))
}

function find(table, bases, method, path) {
  for (const base of bases) {
    if (!path.startsWith(`${base}/`) && path !== base) continue
    const rest = path.slice(base.length).split('/').slice(1)
    const hit = table.find((r) => r.method === method && r.parts.length === rest.length && r.parts.every((p, i) => p.test(rest[i])))
    if (hit) return hit
  }
  return null
}

/** The example a success response of `op` gives: its status and its JSON. */
function success(doc, op) {
  const status = Object.keys(op.responses ?? {}).find((s) => /^2/.test(s)) ?? '200'
  const response = deref(doc, op.responses?.[status]) ?? {}
  const media = Object.entries(response.content ?? {}).find(([type]) => /json/i.test(type))
  const value = media?.[1]?.example ?? (media ? example(doc, media[1].schema) : response.schema ? example(doc, response.schema) : null)
  const code = Number(status) || 200
  return { status: code, body: code === 204 ? undefined : value }
}

/** Starts the mock on a free port of 127.0.0.1. */
export async function mock(doc) {
  const table = routes(doc)
  const bases = prefixes(doc)
  const requests = []
  const queued = []
  const server = createServer((req, res) => {
    const chunks = []
    req.on('data', (chunk) => chunks.push(chunk))
    req.on('end', () => {
      const [path, query = ''] = (req.url ?? '/').split(/\?(.*)/s)
      const route = find(table, bases, req.method, path)
      requests.push({ method: req.method, target: req.url, path, query, headers: req.headers, body: Buffer.concat(chunks).toString('utf8'), op: route?.op.operationId ?? null, at: performance.now() })
      const answer = queued.shift() ?? (route ? success(doc, route.op) : { status: 404, body: { message: `no operation for ${req.method} ${path}` } })
      // A connection per request: a client's pooled connection, left idle while the next SDK is
      // generated, would otherwise be closed under it and read as that SDK's failure.
      const headers = { connection: 'close', ...(answer.body === undefined ? {} : { 'content-type': 'application/json' }), ...answer.headers }
      res.writeHead(answer.status, headers)
      res.end(answer.body === undefined ? undefined : JSON.stringify(answer.body))
    })
  })
  await new Promise((done) => server.listen(0, '127.0.0.1', done))
  const { port } = server.address()
  return {
    origin: `http://127.0.0.1:${port}`,
    requests,
    /** Answers given to the next requests, in order, before any example. */
    answer: (...answers) => queued.push(...answers),
    reset: () => {
      requests.length = 0
      queued.length = 0
    },
    close: () => new Promise((done) => server.close(done)),
  }
}
