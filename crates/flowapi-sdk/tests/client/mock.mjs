// The generated repository's mock server (src/repo/files/server.mjs), run on routes of its own in
// a temporary folder and asked over HTTP. M<n> are its pitfalls: M1 a concrete path matched
// before a templated one that also matches it (OpenAPI's Paths Object; GitHub's
// `POST /orgs/{org}/secret-scanning/custom-patterns` was answered as
// `POST /orgs/{org}/{security_product}/{enablement}`), M2 two templated paths ordered by the first
// segment where one is literal and the other a template, the literal one first; both whatever
// the order routes.mjs lists them in. M3 a request that meets none of its operation's security
// alternatives is a 401 naming what each lacks (the mock never checked credentials, so a client
// that sent none passed its tests), and one alternative met, or `{}`, is enough. M4 a token
// endpoint gives a token only to a client credentials grant as RFC 6749 asks for one: POST, form
// encoded, each parameter once, the client in HTTP basic and not in the body.
import { copyFileSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { request } from 'node:http'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { assert } from './stub.mjs'

const server = fileURLToPath(new URL('../../src/repo/files/server.mjs', import.meta.url))

const route = (method, path, security = []) => ({
  route: `${method} ${path}`, method, path, query: [], headers: [], cookies: [],
  body: null, security, status: 200, media: null, example: null,
})

const routes = [
  route('POST', '/orgs/{org}/{security_product}/{enablement}'),
  route('POST', '/orgs/{org}/secret-scanning/custom-patterns'),
  route('GET', '/a/{x}/b'),
  route('GET', '/a/b/{y}'),
]

/** A request by node:http (the platform's fetch keeps timers another test may have faked). */
function ask(url, method, headers = {}, body = undefined) {
  return new Promise((resolve, reject) => {
    const req = request(url, { method, headers }, (res) => {
      const chunks = []
      res.on('data', (c) => chunks.push(c))
      res.on('end', () => resolve({ status: res.statusCode, text: Buffer.concat(chunks).toString() }))
    })
    req.on('error', reject)
    req.end(body)
  })
}

/** `f` run against the mock server on these routes and token endpoints, in a temporary folder. */
async function withMock(order, tokens, f) {
  const dir = mkdtempSync(join(tmpdir(), 'flowapi-mock-'))
  try {
    copyFileSync(server, join(dir, 'server.mjs'))
    writeFileSync(join(dir, 'routes.mjs'), `export default ${JSON.stringify(order)}\nexport const tokens = ${JSON.stringify(tokens)}\n`)
    writeFileSync(join(dir, 'package.json'), '{"type":"module"}')
    const { mock } = await import(pathToFileURL(join(dir, 'server.mjs')).href)
    const saved = process.env.TEST_API_BASE_URL
    delete process.env.TEST_API_BASE_URL
    const m = await mock()
    if (saved !== undefined) process.env.TEST_API_BASE_URL = saved
    try {
      return await f(m)
    } finally {
      await m.close()
    }
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}

/** The route the mock server matched for each request, its routes listed in `order`. */
function matched(order, requests) {
  return withMock(order, [], async (m) => {
    const out = []
    for (const [method, path] of requests) {
      const res = await ask(`${m.url}${path}`, method)
      assert.equal(res.status, 200, `${method} ${path}: ${res.text}`)
      out.push(m.requests.at(-1).route)
    }
    return out
  })
}

const key = { scheme: 'Key', kind: 'apiKey', in: 'header', name: 'x-key' }
const bearer = { scheme: 'Token', kind: 'bearer' }
const basic = { scheme: 'Basic', kind: 'basic' }
const form = { 'content-type': 'application/x-www-form-urlencoded' }
const pair = `Basic ${Buffer.from('id:secret').toString('base64')}`

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
    ['M3: a request that meets no security alternative is a 401 naming what each lacks', () => withMock([
      route('GET', '/both', [[key, bearer], [basic]]),
      route('GET', '/maybe', [[bearer], []]),
    ], [], async (m) => {
      const none = await ask(`${m.url}/both`, 'GET')
      assert.equal(none.status, 401)
      assert.deepEqual(JSON.parse(none.text).missing, [['Key', 'Token'], ['Basic']])
      assert.match(JSON.parse(none.text).error, /send Key and Token, or Basic/)
      const half = await ask(`${m.url}/both`, 'GET', { 'x-key': 'k' })
      assert.deepEqual(JSON.parse(half.text).missing, [['Token'], ['Basic']])
      assert.equal((await ask(`${m.url}/both`, 'GET', { 'x-key': 'k', authorization: 'Bearer t' })).status, 200)
      assert.equal((await ask(`${m.url}/both`, 'GET', { authorization: pair })).status, 200)
      assert.equal((await ask(`${m.url}/both`, 'GET', { authorization: 'Bearer not a token' })).status, 401)
      assert.equal((await ask(`${m.url}/maybe`, 'GET')).status, 200)
    })],
    ['M4: a token endpoint answers a client credentials grant as RFC 6749 4.4 asks for it', () => withMock([], ['/token'], async (m) => {
      const ok = await ask(`${m.url}/token`, 'POST', { ...form, authorization: pair }, 'grant_type=client_credentials&scope=read')
      assert.equal(ok.status, 200, ok.text)
      assert.deepEqual(JSON.parse(ok.text), { access_token: 'mock-token', token_type: 'Bearer', expires_in: 3600, scope: 'read' })
      assert.equal(m.requests.at(-1).route, 'token')
      const refused = async (status, error, headers, body, method = 'POST') => {
        const r = await ask(`${m.url}/token`, method, headers, body)
        assert.equal(r.status, status, r.text)
        assert.equal(JSON.parse(r.text).error, error)
      }
      await refused(401, 'invalid_client', form, 'grant_type=client_credentials&client_id=id&client_secret=secret')
      await refused(401, 'invalid_client', { ...form, authorization: pair }, 'grant_type=client_credentials&client_secret=secret')
      await refused(400, 'unsupported_grant_type', { ...form, authorization: pair }, 'grant_type=password')
      await refused(400, 'invalid_request', { ...form, authorization: pair }, 'grant_type=client_credentials&grant_type=client_credentials')
      await refused(400, 'invalid_request', { 'content-type': 'application/json', authorization: pair }, '{}')
      await refused(405, 'invalid_request', { authorization: pair }, undefined, 'GET')
    })],
  ]
}
