// How each generated SDK is called, as its own output asks to be called: each adapter makes a
// client with the credentials a test gives, calls an operation by its id with the test's path,
// query, header and body values, and returns what the SDK gave as `{ data }` or
// `{ error: { status, body } }`. An adapter states only the features the generated code has a
// surface for (`page`, `retries`, `auth`); a feature it has no surface for is `n/a` in the
// results. Nothing here sends a request itself: every byte the mock sees is the SDK's.
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { pathToFileURL } from 'node:url'
import createFetchClient from 'openapi-fetch'
import { parse } from 'yaml'

const load = (dir, file) => import(pathToFileURL(join(dir, file)).href)
/** A name compared as generators spell it: `repos/get`, `reposGet` and `ReposGet` are one. */
const plain = (name) => String(name).toLowerCase().replace(/[^a-z0-9]/g, '')

/** The names of a function's parameters in TypeScript source, from the `(` at `at`. */
function parameters(source, at) {
  const names = []
  let depth = 0
  let current = ''
  for (let i = at + 1; i < source.length; i++) {
    const c = source[i]
    if ('({[<'.includes(c)) depth++
    else if (')}]>'.includes(c) && !(c === '>' && source[i - 1] === '=')) {
      if (depth === 0) break
      depth--
    }
    if (depth === 0 && c === ',') {
      names.push(current)
      current = ''
    } else current += c
  }
  names.push(current)
  return names.map((p) => p.trim().match(/^[A-Za-z_$][\w$]*/)?.[0]).filter(Boolean)
}

/** A positional call's arguments from the parameters' names (orval, swagger-typescript-api). */
function positional(names, op, args, { query, body, options }) {
  return names.map((name) => {
    if (name === query) return args.query
    if (name === options) return args.header ? { headers: args.header } : undefined
    const path = op.params.find((p) => p.in === 'path' && plain(p.name) === plain(name))
    if (path) return args.path?.[path.name]
    if (name === 'headers') return args.header
    if (name === body || (body === null && args.body !== undefined)) return args.body
    return undefined
  })
}

/** A status of 400 or more as an error, else data. */
const outcome = (status, data, error) => (status >= 400 ? { error: { status, body: error } } : { data })

/** The resources' path to each operation in a flowapi config: `listPets` → `['pets', 'list']`. */
function methods(resources, at = [], out = {}) {
  const camel = (name) => name.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase())
  for (const [name, resource] of Object.entries(resources ?? {})) {
    for (const [method, spec] of Object.entries(resource.methods ?? {})) {
      out[typeof spec === 'string' ? spec : spec.op] = [...at, camel(name), camel(method)]
    }
    methods(resource.resources, [...at, camel(name)], out)
  }
  return out
}

export const adapters = {
  flowapi: {
    async load(dir) {
      const config = parse(readFileSync(join(dir, '..', 'flowapi-sdk.yml'), 'utf8'))
      const client = readFileSync(join(dir, 'src', 'client.ts'), 'utf8')
      const options = Object.fromEntries([...client.matchAll(/"([^"]+)": \{ option: "(\w+)"/g)].map((m) => [m[1], m[2]]))
      const module = await load(dir, 'js/index.js')
      return { Client: module[config.package.client], options, paths: methods(config.resources) }
    },
    client(sdk, credentials) {
      const given = Object.fromEntries(Object.entries(sdk.options).map(([scheme, option]) => [option, credentials[scheme]?.basic ?? credentials[scheme] ?? null]))
      return { sdk, api: new sdk.Client({ ...given, logLevel: 'off' }) }
    },
    method(client, op) {
      const path = client.sdk.paths[op.id]
      if (!path) throw new Error(`no method for ${op.id} in the config`)
      const owner = path.slice(0, -1).reduce((at, key) => at[key], client.api)
      return (params) => owner[path.at(-1)](params)
    },
    params: (args) => ({ ...(args.path && { path: args.path }), ...(args.query && { query: args.query }), ...(args.header && { header: args.header }), ...(args.body !== undefined && { body: { contentType: args.type, value: args.body } }) }),
    async call(client, op, args) {
      try {
        return { data: await this.method(client, op)(this.params(args)) }
      } catch (e) {
        if (typeof e?.status === 'number' && 'body' in e) return { error: { status: e.status, body: e.body } }
        throw e
      }
    },
    async page(client, op, args) {
      const pages = this.method(client, op)(this.params(args))
      if (typeof pages?.[Symbol.asyncIterator] !== 'function') return null
      const items = []
      for await (const item of pages) items.push(item)
      return items
    },
    retries: true,
    auth: true,
  },
  'hey-api': {
    async load(dir) {
      return { sdk: await load(dir, 'js/sdk.gen.js'), runtime: await load(dir, 'js/client/index.js') }
    },
    client(sdk, credentials, kase) {
      // hey-api's `auth` callback is asked for each scheme an operation names, by its type and
      // where it goes; the token is given as its runtime writes it (basic: `user:password`).
      const auth = (wanted) => {
        const scheme = Object.entries(kase.schemes).find(([, s]) => s.type === wanted.type && (s.type !== 'http' || s.scheme?.toLowerCase() === wanted.scheme) && (s.type !== 'apiKey' || (s.name === wanted.name && (s.in ?? 'header') === (wanted.in ?? 'header'))))
        const value = scheme && credentials[scheme[0]]
        return value?.basic ? `${value.basic.username}:${value.basic.password}` : value
      }
      return { sdk, api: sdk.runtime.createClient(sdk.runtime.createConfig({ baseUrl: kase.server, auth })) }
    },
    async call(client, op, args) {
      const fn = Object.entries(client.sdk.sdk).find(([name]) => plain(name) === plain(op.id))?.[1]
      if (!fn) throw new Error(`no function for ${op.id}`)
      const { data, error, response } = await fn({ client: client.api, path: args.path, query: args.query, headers: args.header, body: args.body })
      if (!response) throw error
      return outcome(response.status, data, error)
    },
    page: null,
    retries: false,
    auth: true,
  },
  'openapi-typescript': {
    // A client by composition: openapi-fetch's runtime typed by openapi-typescript's `paths`.
    load: async () => ({}),
    client: (sdk, credentials, kase) => ({ api: createFetchClient({ baseUrl: kase.server }) }),
    async call(client, op, args) {
      const headers = args.type && args.type !== 'application/json' ? { 'Content-Type': args.type } : undefined
      const init = { params: { path: args.path, query: args.query, header: args.header }, body: args.body, headers }
      const { data, error, response } = await client.api[op.method](op.template, init)
      return outcome(response.status, data, error)
    },
    page: null,
    retries: false,
    auth: false,
  },
  orval: {
    async load(dir) {
      return { module: await load(dir, 'js/api.js'), source: readFileSync(join(dir, 'api.ts'), 'utf8') }
    },
    client: (sdk) => ({ sdk }),
    async call(client, op, args) {
      const [name, fn] = Object.entries(client.sdk.module).find(([n, f]) => typeof f === 'function' && plain(n) === plain(op.id)) ?? []
      if (!fn) throw new Error(`no function for ${op.id}`)
      const at = client.sdk.source.indexOf('(', client.sdk.source.indexOf(`export const ${name} = async`))
      const values = positional(parameters(client.sdk.source, at), op, args, { query: 'params', body: null, options: 'options' })
      const { data, status } = await fn(...values)
      return outcome(status, data, data)
    },
    page: null,
    retries: false,
    auth: false,
  },
  'swagger-typescript-api': {
    async load(dir) {
      return { module: await load(dir, 'js/api.js'), source: readFileSync(join(dir, 'api.ts'), 'utf8') }
    },
    client: (sdk, credentials, kase) => ({ sdk, api: new sdk.module.Api({ baseUrl: kase.server.replace(/\/+$/, '') }) }),
    async call(client, op, args) {
      let found = null
      for (const group of Object.values(client.api)) {
        if (!group || typeof group !== 'object') continue
        const name = Object.keys(group).find((n) => plain(n) === plain(op.id))
        if (name) found = [name, group[name]]
      }
      if (!found) throw new Error(`no method for ${op.id}`)
      const at = client.sdk.source.indexOf('(', client.sdk.source.indexOf(`    ${found[0]}: (`))
      const values = positional(parameters(client.sdk.source, at), op, args, { query: 'query', body: 'data', options: 'params' })
      try {
        const response = await found[1](...values)
        return { data: response.data }
      } catch (e) {
        if (typeof e?.status === 'number') return { error: { status: e.status, body: e.error } }
        throw e
      }
    },
    page: null,
    retries: false,
    auth: false,
  },
}
