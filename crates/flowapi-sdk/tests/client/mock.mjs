// The generated repository's mock server (src/repo/files/server.mjs), run on routes of its own in
// a temporary folder and asked over HTTP. M<n> are its pitfalls: M1 a concrete path matched
// before a templated one that also matches it (OpenAPI's Paths Object; GitHub's
// `POST /orgs/{org}/secret-scanning/custom-patterns` was answered as
// `POST /orgs/{org}/{security_product}/{enablement}`), M2 two templated paths ordered by the first
// segment where one is literal and the other a template, the literal one first; both whatever
// the order routes.mjs lists them in.
import { copyFileSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { request } from 'node:http'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { assert } from './stub.mjs'

const server = fileURLToPath(new URL('../../src/repo/files/server.mjs', import.meta.url))

const route = (method, path) => ({
  route: `${method} ${path}`, method, path, query: [], headers: [], cookies: [],
  body: null, status: 200, media: null, example: null,
})

const routes = [
  route('POST', '/orgs/{org}/{security_product}/{enablement}'),
  route('POST', '/orgs/{org}/secret-scanning/custom-patterns'),
  route('GET', '/a/{x}/b'),
  route('GET', '/a/b/{y}'),
]

/** A request by node:http (the platform's fetch keeps timers another test may have faked). */
function ask(url, method) {
  return new Promise((resolve, reject) => {
    const req = request(url, { method }, (res) => {
      const chunks = []
      res.on('data', (c) => chunks.push(c))
      res.on('end', () => resolve({ status: res.statusCode, text: Buffer.concat(chunks).toString() }))
    })
    req.on('error', reject)
    req.end()
  })
}

/** The route the mock server matched for each request, its routes listed in `order`. */
async function matched(order, requests) {
  const dir = mkdtempSync(join(tmpdir(), 'flowapi-mock-'))
  try {
    copyFileSync(server, join(dir, 'server.mjs'))
    writeFileSync(join(dir, 'routes.mjs'), `export default ${JSON.stringify(order)}\n`)
    writeFileSync(join(dir, 'package.json'), '{"type":"module"}')
    const { mock } = await import(pathToFileURL(join(dir, 'server.mjs')).href)
    const saved = process.env.TEST_API_BASE_URL
    delete process.env.TEST_API_BASE_URL
    const m = await mock()
    if (saved !== undefined) process.env.TEST_API_BASE_URL = saved
    try {
      const out = []
      for (const [method, path] of requests) {
        const res = await ask(`${m.url}${path}`, method)
        assert.equal(res.status, 200, `${method} ${path}: ${res.text}`)
        out.push(m.requests.at(-1).route)
      }
      return out
    } finally {
      await m.close()
    }
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}

export function tests() {
  const orders = [routes, [...routes].reverse()]
  return [
    ['M1: a concrete path is matched before a templated one', async () => {
      for (const order of orders) {
        assert.deepEqual(await matched(order, [
          ['POST', '/orgs/o/secret-scanning/custom-patterns'],
          ['POST', '/orgs/o/code-scanning/enable'],
        ]), [
          'POST /orgs/{org}/secret-scanning/custom-patterns',
          'POST /orgs/{org}/{security_product}/{enablement}',
        ])
      }
    }],
    ['M2: of two templated paths, the one literal at the first segment they differ wins', async () => {
      for (const order of orders) {
        assert.deepEqual(await matched(order, [['GET', '/a/b/b'], ['GET', '/a/c/b']]), ['GET /a/b/{y}', 'GET /a/{x}/b'])
      }
    }],
  ]
}
