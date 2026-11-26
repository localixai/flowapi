// Where the credentials meet the client's use of HTTP (the `hosts` case, its optional OAuth 2.0
// client, and the `tokens` and `auth` cases): the auth layer's renewal beside RFC 9110's retries,
// the token request beside the retry layer, credentials beside redirects and the servers. Each
// test names the pitfall it holds (R<n>, the runtime's list): R56 a 401 renewal counted as a retry,
// or made with another idempotency key, R57 a POST sent more than twice through a renewal, R58 the
// retries made anew after a renewal, R59 a token request retried or redirected to another origin,
// R60 a credential carried by a redirect to another origin, R61 a credential sent in the clear to
// a base URL the servers resolved to `http:`.
import { assert, rejection, reply, stub, timed } from './stub.mjs'

const credentials = { clientId: 'id', clientSecret: 'secret' }
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/
const ok = () => reply(200, [{ id: 1, name: 'a' }])
/** Asks for a retry at once, so that no backoff is waited. */
const now = { 'retry-after-ms': '0' }

const isToken = (url) => new URL(url).pathname.endsWith('/token')

/** A fetch answering a token endpoint by `token(n)`, its n-th request, and the API by `api(n)`. */
function server(api, token = (n) => reply(200, { access_token: `tok${n}`, token_type: 'Bearer', expires_in: 3600 })) {
  let tokens = 0
  let calls = 0
  const fetch = stub((url, init) => (isToken(url) ? token(++tokens, url, init) : api(++calls, url, init)))
  return Object.assign(fetch, { api: () => fetch.calls.filter((c) => !isToken(c.url)), asked: () => fetch.calls.filter((c) => isToken(c.url)) })
}

/** A hosts client with the client's own id and secret, quiet. */
function hosts(built, fetch, more = {}) {
  return new built.hosts.Hosts({ fetch, logLevel: 'off', timeout: null, client: credentials, ...more })
}

const item = { body: { contentType: 'application/json', value: { name: 'a' } } }

export function tests(built) {
  const { APIConnectionError, CrossOriginRedirectError, InsecureCredentialError, InternalServerError, TokenError } = built.hosts
  return [
    ['R56: a 401 renewal is the auth layer\'s one, made with no retry left, and keeps the call\'s idempotency key', () => timed(async () => {
      const fetch = server((n) => (n === 1 ? reply(401, { error: 'expired' }) : reply(201, { id: 1, name: 'a' })))
      await hosts(built, fetch, { maxRetries: 0 }).items.create(item)
      const [first, again] = fetch.api()
      assert.equal(fetch.api().length, 2)
      assert.deepEqual([first.init.method, again.init.method], ['POST', 'POST'])
      assert.deepEqual([first.headers.authorization, again.headers.authorization], ['Bearer tok1', 'Bearer tok2'])
      assert.match(first.headers['idempotency-key'], UUID)
      assert.equal(again.headers['idempotency-key'], first.headers['idempotency-key'], 'the renewed call is the same call')
    })],
    // req: rfc9110.9.2.2.should-not-client-automatically-retry-request-non-idempotent — a POST after a renewal is not retried on its own
    ['R57: a POST is sent at most twice through a renewal, whatever maxRetries says', () => timed(async () => {
      const make = (fetch) => new built.tokens.Tokens({ fetch, logLevel: 'off', timeout: null, maxRetries: 3, client: credentials })
      const down = server((n) => (n === 1 ? reply(401, { error: 'expired' }) : reply(503, 'down', now)))
      await assert.rejects(make(down).things.write(), built.tokens.InternalServerError)
      assert.deepEqual(down.api().map((c) => c.init.method), ['POST', 'POST'])
      const denied = server(() => reply(401, { error: 'no' }))
      await assert.rejects(make(denied).things.write(), built.tokens.AuthenticationError)
      assert.deepEqual(denied.calls.map((c) => (isToken(c.url) ? 'token' : c.init.method)), ['token', 'POST', 'token', 'POST'])
    })],
    ['R58: the retries of a call are counted across its renewal: what is left after it, not maxRetries again', () => timed(async () => {
      const answers = [() => reply(503, 'down', now), () => reply(401, { error: 'expired' }), () => reply(503, 'down', now), ok]
      const fetch = server((n) => answers[n - 1]())
      await assert.rejects(hosts(built, fetch, { maxRetries: 1 }).items.all(), InternalServerError)
      assert.equal(fetch.api().length, 3, '503, its retry 401, the renewed call 503 with no retry left')
      const plenty = server((n) => answers[n - 1]())
      await hosts(built, plenty, { maxRetries: 2 }).items.all()
      assert.equal(plenty.api().length, 4)
    })],
    // req: rfc6749.3.2.must-client-use-http-post-method-making — the token request is a POST, sent once
    ['R59: a token request is never retried by the retry layer, nor followed to another origin', () => timed(async () => {
      const busy = server(ok, () => reply(503, { error: 'temporarily_unavailable' }, now))
      await assert.rejects(hosts(built, busy, { maxRetries: 3 }).items.all(), (e) => e instanceof TokenError && e.status === 503)
      assert.deepEqual([busy.asked().length, busy.api().length], [1, 0])
      assert.equal(busy.asked()[0].init.method, 'POST')
      const failing = server(ok, () => {
        throw new TypeError('fetch failed')
      })
      await assert.rejects(hosts(built, failing, { maxRetries: 3 }).items.all(), APIConnectionError)
      assert.deepEqual([failing.asked().length, failing.api().length], [1, 0])
      const moved = server(ok, () => reply(307, null, { location: 'https://elsewhere.example/token' }))
      const e = await rejection(hosts(built, moved, { allowCrossOriginRedirects: true }).items.all())
      assert.ok(e instanceof CrossOriginRedirectError, String(e))
      assert.ok(!moved.calls.some((c) => c.url.startsWith('https://elsewhere.example')), 'the client secret stayed home')
    })],
    // req: rfc9110.15.4.should-automatically-following-redirected-request-user-agent — Authorization and Cookie removed when the redirect leaves the origin
    ['R60: a redirect to another origin carries no credential: refused by default, stripped when allowed', () => timed(async () => {
      const away = (location) => server((n) => (n === 1 ? reply(307, null, { location }) : ok()))
      const refused = away('https://other.example/items')
      await assert.rejects(hosts(built, refused).items.all(), CrossOriginRedirectError)
      assert.equal(refused.api().length, 1)
      const allowed = away('https://other.example/items')
      await hosts(built, allowed, { allowCrossOriginRedirects: true }).items.all()
      assert.deepEqual(allowed.api().map((c) => [c.url, c.headers.authorization]), [['https://eu.hosts.example/v1/items', 'Bearer tok1'], ['https://other.example/items', undefined]])
      const home = away('https://eu.hosts.example/v1/moved')
      await hosts(built, home).items.all()
      assert.equal(home.api()[1].headers.authorization, 'Bearer tok1', 'the same origin keeps it')
      const cookies = stub((_, __, n) => (n === 1 ? reply(307, null, { location: 'https://other.example/cookie' }) : ok()))
      await new built.auth.AuthClient({ fetch: cookies, logLevel: 'off', cookieKey: 'c1', allowCrossOriginRedirects: true }).checks.cookie({ cookie: { theme: 'dark' } })
      assert.match(cookies.calls[0].headers.cookie, /c1/)
      assert.equal(cookies.calls[1].headers.cookie, undefined)
    })],
    // req: rfc6750.5.3.must-always-use-tls-https-clients-always — a bearer token only over TLS, to the URL the servers resolved to
    ['R61: the TLS rule holds the base URL the servers resolved, a relative one against descriptionURL', () => timed(async () => {
      const plain = { descriptionURL: 'http://docs.hosts.example/api/openapi.json' }
      const given = server(ok)
      const e = await rejection(hosts(built, given, { ...plain, client: 'tok' }).places.local())
      assert.ok(e instanceof InsecureCredentialError && e.message.includes('http://docs.hosts.example/api/'), String(e))
      assert.equal(given.calls.length, 0)
      const requested = server(ok)
      await assert.rejects(hosts(built, requested, plain).places.local(), InsecureCredentialError)
      assert.equal(requested.api().length, 0, 'the API never saw the token')
      const none = server(ok)
      await hosts(built, none, { ...plain, client: null }).places.local()
      await hosts(built, none, { ...plain, client: 'tok', allowInsecureCredentials: true }).places.local()
      await hosts(built, none, { descriptionURL: 'https://docs.hosts.example/api/openapi.json', client: 'tok' }).places.local()
      assert.deepEqual(none.api().map((c) => [c.url, c.headers.authorization]), [
        ['http://docs.hosts.example/api/status', undefined],
        ['http://docs.hosts.example/api/status', 'Bearer tok'],
        ['https://docs.hosts.example/api/status', 'Bearer tok'],
      ])
    })],
  ]
}
