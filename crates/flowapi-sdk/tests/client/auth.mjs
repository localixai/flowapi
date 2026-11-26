// Credentials sent by an operation's effective security, and where a call goes. Each kind of
// scheme (an API key in a header, a query and a cookie, HTTP basic and bearer, an OAuth 2.0 token),
// the first alternative the client can meet, schemes needed together, `security: []`, a credential
// that is missing, an operation's own servers and a server variable's default; also the names that
// are TypeScript's reserved words and resources three levels deep.
import { assert, rejection, reply, stub } from './stub.mjs'

const ok = () => reply(200, { ok: true })

function client(built, fetch, more = {}) {
  return new built.auth.AuthClient({ fetch, logLevel: 'off', ...more })
}

const env = ['AUTH_QUERY_KEY', 'AUTH_BASIC', 'AUTH_TOKEN', 'SHOP_API_KEY', 'SHOP_TOKEN']

/** Runs `f` with none of the cases' credential variables set. */
async function clean(f) {
  const before = env.map((k) => process.env[k])
  for (const k of env) delete process.env[k]
  try {
    return await f()
  } finally {
    env.forEach((k, i) => {
      if (before[i] !== undefined) process.env[k] = before[i]
    })
  }
}

export function tests(built) {
  const { SdkError } = built.auth
  return [
    // req: rfc6750.2.1.should-clients-make-authenticated-requests-bearer-token — Bearer t in the Authorization header
    ['an API key in a header, else a bearer token: the first alternative that can be met', () => clean(async () => {
      const fetch = stub(() => reply(200, { id: 'p1', name: 'Rex' }))
      await new built.shop.Shop({ fetch, apiKeyAuth: 'k', bearerAuth: 't' }).pets.retrieve({ path: { pet_id: 'p1' } })
      await new built.shop.Shop({ fetch, bearerAuth: 't' }).pets.retrieve({ path: { pet_id: 'p1' } })
      assert.deepEqual(fetch.calls.map((c) => [c.headers['x-api-key'], c.headers['authorization']]), [['k', undefined], [undefined, 'Bearer t']])
    })],
    ['an API key in the query and in a cookie, beside the operation\'s own', () => clean(async () => {
      const fetch = stub(ok)
      const c = client(built, fetch, { queryKey: 'q k', cookieKey: 's1' })
      await c.checks.queryKey({ query: { q: 'x' } })
      await c.checks.cookie({ cookie: { theme: 'dark' } })
      assert.equal(fetch.calls[0].url, 'https://auth.example/v2/query?q=x&api_key=q%20k')
      assert.equal(fetch.calls[1].headers.cookie, 'theme=dark; session=s1')
    })],
    // req: rfc7617.2.2.may-client-preemptively-send-corresponding-authorization-header — Basic sent with the first request, no challenge awaited
    ['HTTP basic: the UTF-8 pair in base64, from the option or the variable', () => clean(async () => {
      const fetch = stub(ok)
      await client(built, fetch, { basic: { username: 'jürgen', password: 'p:ß' } }).checks.basic()
      process.env.AUTH_BASIC = 'u:p'
      await client(built, fetch).checks.basic()
      await client(built, fetch, { basic: async () => ({ username: 'a', password: 'b' }) }).checks.basic()
      const b64 = (s) => 'Basic ' + Buffer.from(s, 'utf8').toString('base64')
      assert.deepEqual(fetch.calls.map((c) => c.headers.authorization), [b64('jürgen:p:ß'), b64('u:p'), b64('a:b')])
    })],
    ['alternatives: a token, else basic; schemes needed together are sent together', () => clean(async () => {
      const fetch = stub(ok)
      await client(built, fetch, { basic: { username: 'u', password: 'p' } }).checks.either()
      await client(built, fetch, { token: 't', basic: { username: 'u', password: 'p' } }).checks.either()
      await client(built, fetch, { token: 't', queryKey: 'k' }).checks.both()
      assert.deepEqual(fetch.calls.map((c) => c.headers.authorization), ['Basic dTpw', 'Bearer t', 'Bearer t'])
      assert.ok(fetch.calls[2].url.endsWith('/both?api_key=k'))
      const e = await rejection(client(built, fetch, { token: 't' }).checks.both())
      assert.ok(e instanceof SdkError, String(e))
      assert.match(e.message, /`GET \/both` needs a credential: set `queryKey` or AUTH_QUERY_KEY and `token` or AUTH_TOKEN/)
      assert.equal(fetch.calls.length, 3)
    })],
    ['security: [] sends nothing, whatever the client holds', () => clean(async () => {
      const fetch = stub(ok)
      await client(built, fetch, { token: 't', queryKey: 'k', cookieKey: 'c' }).checks.open()
      assert.equal(fetch.calls[0].url, 'https://auth.example/v2/open')
      assert.deepEqual(fetch.calls[0].headers, {})
    })],
    // req: rfc6750.2.1.should-clients-make-authenticated-requests-bearer-token — the OAuth 2.0 token sent as Bearer in the Authorization header
    ['an OAuth 2.0 token is a bearer token; a function is asked for each request', () => clean(async () => {
      const fetch = stub(ok)
      let n = 0
      const c = client(built, fetch, { oauth: () => `tok-${++n}` })
      await c.checks.oauth()
      await c.checks.oauth()
      assert.deepEqual(fetch.calls.map((x) => x.headers.authorization), ['Bearer tok-1', 'Bearer tok-2'])
      const e = await rejection(client(built, fetch, { oauth: () => ' ' }).checks.oauth())
      assert.match(e.message, /credential of `Oauth` \(the option `oauth`\) is not a string/)
      const none = await rejection(client(built, fetch).checks.oauth())
      assert.match(none.message, /needs a credential: set `oauth`/)
    })],
    ['a credential given as a header by the caller meets its scheme', () => clean(async () => {
      const fetch = stub(ok)
      await client(built, fetch).checks.oauth({}, { headers: { Authorization: 'Bearer mine' } })
      await client(built, fetch, { defaultQuery: { api_key: 'mine' }, token: 't' }).checks.both()
      assert.equal(fetch.calls[0].headers.authorization, 'Bearer mine')
      assert.ok(fetch.calls[1].url.endsWith('/both?api_key=mine'))
    })],
    // req: oas-3.0.server-variable-object.default — {version} read as its default, v2
    // req: oas-3.0.server-variable-object.default.shall-default-value-use-substitution-sent-alternate — no value given: https://auth.example/v2
    ['an operation\'s own servers; a server variable\'s default; a base URL given goes over both', () => clean(async () => {
      const fetch = stub(ok)
      await client(built, fetch).checks.elsewhere()
      await client(built, fetch).outer.middle.inner.get()
      await client(built, fetch, { baseURL: 'https://mine.example' }).checks.elsewhere()
      await client(built, fetch).checks.elsewhere({}, { baseURL: 'https://call.example/x' })
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://other.example/api/elsewhere', 'https://auth.example/v2/deep', 'https://mine.example/elsewhere', 'https://call.example/x/elsewhere'])
    })],
    ['methods named by reserved words; a raw request sends every credential it has', () => clean(async () => {
      const fetch = stub((_, init) => (init.method === 'DELETE' ? reply(204, null) : ok()))
      const c = client(built, fetch, { token: 't', queryKey: 'k' })
      assert.equal(await c.default.delete({ path: { id: 1 } }), undefined)
      assert.deepEqual(await c.default.new({ path: { id: 2 } }), { ok: true })
      assert.deepEqual(await c.request('POST', '/hooks', { body: { a: 1 }, query: { x: 'y' } }), { ok: true })
      assert.deepEqual(fetch.calls.map((x) => `${x.init.method} ${x.url}`), ['DELETE https://auth.example/v2/things/1', 'PUT https://auth.example/v2/things/2', 'POST https://auth.example/v2/hooks?x=y&api_key=k'])
      assert.deepEqual([fetch.calls[2].headers.authorization, fetch.calls[2].headers['content-type'], fetch.calls[2].init.body], ['Bearer t', 'application/json', '{"a":1}'])
      assert.deepEqual(fetch.calls[0].headers, {})
    })],
  ]
}
