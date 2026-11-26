// How the client uses HTTP (RFC 9110): which requests it sends twice, how it reads a status, how
// it follows a redirect, and what content each method carries. Each test names the pitfall it holds
// (R<n>, the runtime's list): R46 a request that is not idempotent retried, R47 its idempotency key,
// R48 a failed retry retried, R49 a 403 repeated, R50 a status no class is known for, or none, R51
// a redirect's method, R52 a redirect loop, R53 a redirect's fragment, R54 content with GET, HEAD
// and DELETE, R55 content with OPTIONS.
import { assert, rejection, reply, stub, timed } from './stub.mjs'

const ok = () => reply(200, { ok: true })
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/

/** A shop client (retries 2, no idempotency key), quiet. */
function shop(built, fetch, more = {}) {
  return new built.shop.Shop({ fetch, apiKeyAuth: 'key-1', logLevel: 'off', timeout: null, ...more })
}

/** A hosts client (retries 1, `Idempotency-Key`), quiet. */
function hosts(built, fetch, more = {}) {
  return new built.hosts.Hosts({ fetch, logLevel: 'off', timeout: null, ...more })
}

/** A fetch whose first answer is `first` (a status, or a thrown error), then 200. */
function once(first, headers = {}) {
  return stub((_, __, n) => {
    if (n > 1) return ok()
    if (typeof first === 'number') return reply(first, 'no', headers)
    throw first
  })
}

/** A response with a status no `Response` can be made with. */
function odd(status) {
  return { status, headers: new Headers(), url: '', type: 'basic', body: null, text: async () => '', arrayBuffer: async () => new ArrayBuffer(0) }
}

export function tests(built) {
  const { APIError, InternalServerError, SdkError } = built.shop
  return [
    // req: rfc9110.9.2.2.should-not-client-automatically-retry-request-non-idempotent — POST, PATCH and QUERY not retried on a 503 or a failed connection; PUT, DELETE and the safe methods are; the server's x-should-retry: true is the means to know
    ['R46: a request that is not idempotent is not retried unless the server asks', () => timed(async () => {
      for (const method of ['POST', 'PATCH', 'QUERY']) {
        for (const first of [503, new TypeError('fetch failed')]) {
          const fetch = once(first)
          await rejection(shop(built, fetch).request(method, '/x', { body: { a: 1 } }))
          assert.equal(fetch.calls.length, 1, `${method} ${first}`)
        }
        const asked = once(503, { 'x-should-retry': 'true' })
        await shop(built, asked).request(method, '/x')
        assert.equal(asked.calls.length, 2, `${method} asked`)
      }
      for (const method of ['GET', 'HEAD', 'OPTIONS', 'TRACE', 'PUT', 'DELETE', 'put']) {
        for (const first of [503, new TypeError('fetch failed')]) {
          const fetch = once(first)
          await shop(built, fetch).request(method, '/x')
          assert.equal(fetch.calls.length, 2, `${method} ${first}`)
        }
      }
    })],
    // req: rfc9110.9.2.2.should-not-client-automatically-retry-request-non-idempotent — an idempotency key is the means to know a POST or a PATCH was not applied twice: with it, they are retried
    ['R47: a call of a method that is not idempotent sends one idempotency key on all its attempts', () => timed(async () => {
      const fetch = once(503)
      const client = hosts(built, fetch)
      await client.items.create({ body: { contentType: 'application/json', value: { name: 'a' } } })
      const [first, second] = fetch.calls.map((c) => c.headers['idempotency-key'])
      assert.ok(UUID.test(first) && first === second, `${first} ${second}`)
      const failed = once(new TypeError('fetch failed'))
      await hosts(built, failed).items.edit({ path: { id: 1 }, body: { contentType: 'application/json', value: { name: 'b' } } })
      assert.equal(failed.calls.length, 2)
      assert.equal(failed.calls[0].headers['idempotency-key'], failed.calls[1].headers['idempotency-key'])
      const each = stub(ok)
      await client.withOptions({ fetch: each }).items.create({ body: { contentType: 'application/json', value: { name: 'a' } } })
      await client.withOptions({ fetch: each }).request('POST', '/items')
      await client.withOptions({ fetch: each }).items.all()
      await client.withOptions({ fetch: each }).items.replace({ path: { id: 1 }, body: { contentType: 'application/json', value: { name: 'c' } } })
      const keys = each.calls.map((c) => c.headers['idempotency-key'])
      assert.ok(UUID.test(keys[0]) && UUID.test(keys[1]) && keys[0] !== keys[1] && keys[0] !== first, keys.join(' '))
      assert.deepEqual(keys.slice(2), [undefined, undefined])
      const mine = once(500)
      await hosts(built, mine).items.create({ body: { contentType: 'application/json', value: { name: 'a' } } }, { headers: { 'Idempotency-Key': 'mine-1' } })
      assert.deepEqual(mine.calls.map((c) => c.headers['idempotency-key']), ['mine-1', 'mine-1'])
    })],
    // req: rfc9110.9.2.2.should-not-client-automatically-retry-failed-automatic-retry — one retry by default: a retry that failed is the error; more only when the caller asks
    ['R48: a failed retry is not retried by default; maxRetries above 1 is the caller\'s', () => timed(async () => {
      const fetch = stub(() => reply(503, 'busy'))
      const e = await rejection(new built.edge.Edge({ fetch, logLevel: 'off' }).zones.status())
      assert.ok(e instanceof built.edge.InternalServerError)
      assert.equal(fetch.calls.length, 2)
      await rejection(new built.edge.Edge({ fetch, logLevel: 'off', maxRetries: 2 }).zones.status())
      assert.equal(fetch.calls.length, 5)
      assert.equal(new built.edge.Edge({ fetch }).maxRetries, 1)
    })],
    // req: rfc9110.15.5.4.should-not-client-automatically-repeat-request-same-credentials — a 403 is never retried, whatever x-should-retry says
    ['R49: a 403 is not sent again with the same credentials', () => timed(async () => {
      for (const headers of [{}, { 'x-should-retry': 'true' }]) {
        const fetch = once(403, headers)
        assert.ok((await rejection(shop(built, fetch).system.health())) instanceof built.shop.PermissionDeniedError)
        assert.equal(fetch.calls.length, 1)
      }
    })],
    // req: rfc9110.15.must-however-client-understand-class-any-status — 299 a success, 499 a client's error not retried like 400, 599 a server's retried like 500, 399 with no Location an error like 300 (R51: with one, followed as a 300 is)
    // req: rfc9110.15.should-client-receives-response-invalid-status-code — 600, 0 and 99.5 read as a 5xx: InternalServerError, retried
    ['R50: a status no code is known for is its class\'s x00; a status that is none is a 5xx', () => timed(async () => {
      const cases = [[299, 1, null], [499, 1, APIError], [599, 3, InternalServerError], [399, 1, APIError], [304, 1, APIError]]
      for (const [status, calls, error] of cases) {
        const fetch = stub(() => reply(status, status === 304 ? null : { ok: status }, status === 399 ? {} : { location: '/elsewhere' }))
        const p = shop(built, fetch).request('GET', '/x')
        if (error === null) assert.deepEqual(await p, { ok: 299 })
        else {
          const e = await rejection(p)
          assert.ok(e.constructor === error && e.status === status, `${status}: ${e}`)
        }
        assert.equal(fetch.calls.length, calls, String(status))
      }
      for (const status of [600, 0, 99.5]) {
        const fetch = stub(() => odd(status))
        const e = await rejection(shop(built, fetch).request('GET', '/x'))
        assert.ok(e instanceof InternalServerError && e.status === status, String(e))
        assert.equal(fetch.calls.length, 3)
      }
    })],
    // req: rfc9110.15.4.2.may-user-agent-use-location-field-value — a 301 followed
    // req: rfc9110.15.4.2.may-note-historical-reasons-user-agent-change — a POST redirected by 301 is sent again as a GET with no content, as browsers do; a PUT stays a PUT
    // req: rfc9110.15.4.3.may-user-agent-use-location-field-value — a 302 followed
    // req: rfc9110.15.4.3.may-note-historical-reasons-user-agent-change — a POST redirected by 302 is sent again as a GET with no content
    // req: rfc9110.15.4.8.may-user-agent-use-location-field-value — a 307 followed
    // req: rfc9110.15.4.8.must-not-307-temporary-redirect-status-code-indicates — a POST redirected by 307 stays a POST with its content
    // req: rfc9110.15.4.9.may-user-agent-use-location-field-value — a 308 followed, method and content kept
    // req: rfc9110.15.4.1.may-user-agent-use-location-field-value — a 300 with a Location followed, method and content kept
    // req: rfc9110.15.4.may-location-header-field-section-10-2 — a 399, which no one defines, with a Location followed as a 300 is; a 304, 305 or 306 with one is not
    ['R51: a redirect\'s method: 301 and 302 turn a POST into a GET; 300, 307, 308 and a code no one defines keep it and its content', async () => {
      const cases = [[301, 'POST', 'GET'], [302, 'POST', 'GET'], [301, 'PUT', 'PUT'], [307, 'POST', 'POST'], [308, 'POST', 'POST'], [308, 'PATCH', 'PATCH'], [300, 'POST', 'POST'], [399, 'POST', 'POST']]
      for (const [status, method, then] of cases) {
        const fetch = stub((url) => (url.endsWith('/a') ? reply(status, null, { location: '/v1/b' }) : ok()))
        assert.deepEqual(await shop(built, fetch).request(method, '/a', { body: { n: 1 } }), { ok: true })
        const second = fetch.calls[1]
        const kept = then === method
        assert.deepEqual([second.url, second.init.method, second.init.body, second.headers['content-type']], ['https://api.shop.example/v1/b', then, kept ? '{"n":1}' : undefined, kept ? 'application/json' : undefined], `${status} ${method}`)
      }
      for (const status of [304, 305, 306]) {
        const fetch = stub(() => reply(status, null, { location: '/v1/b' }))
        assert.equal((await rejection(shop(built, fetch).request('GET', '/a'))).status, status)
        assert.equal(fetch.calls.length, 1, String(status))
      }
    }],
    // req: rfc9110.15.4.should-client-detect-intervene-cyclical-redirections-i — a request sent again where it went is refused at once; more than 20 redirects are refused
    ['R52: a redirect loop is refused at once, a long chain after 20', async () => {
      const loop = stub((url) => reply(302, null, { location: url.endsWith('/a') ? '/v1/b' : '/v1/a' }))
      const e = await rejection(shop(built, loop).request('GET', '/a'))
      assert.ok(e instanceof SdkError && /redirect loop: GET https:\/\/api\.shop\.example\/v1\/a/.test(e.message), String(e))
      assert.equal(loop.calls.length, 2)
      const chain = stub((url) => reply(307, null, { location: `/v1/r${Number(url.split('/r').at(-1) ?? 0) + 1}` }))
      const long = await rejection(shop(built, chain).request('GET', '/r0'))
      assert.ok(long instanceof SdkError && /more than 20 redirects/.test(long.message), String(long))
      assert.equal(chain.calls.length, 21)
    }],
    // req: rfc9110.10.2.2.must-location-value-provided-3xx-redirection-response — a Location with no fragment inherits the request's; one with a fragment keeps its own
    ['R53: a redirect keeps the request\'s fragment where the Location has none', async () => {
      const at = (url) => new URL(url).pathname
      const fetch = stub((url) => (at(url) === '/v1/a' ? reply(302, null, { location: '/v1/b' }) : at(url) === '/v1/b' ? reply(302, null, { location: '/v1/c#own' }) : ok()))
      await shop(built, fetch).request('GET', '/a#top')
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://api.shop.example/v1/a#top', 'https://api.shop.example/v1/b#top', 'https://api.shop.example/v1/c#own'])
    }],
    // req: rfc9110.9.3.1.should-not-client-generate-content-get-request-unless — a GET sends no content: one given is refused before anything is sent
    // req: rfc9110.9.3.2.should-not-client-generate-content-head-request-unless — the same for HEAD
    // req: rfc9110.9.3.5.should-not-client-generate-content-delete-request-unless — a DELETE sends content only where the description says it takes one (the server's indication)
    ['R54: GET, HEAD and DELETE send no content unless the description declares it', async () => {
      const fetch = stub((_, init) => (init.method === 'GET' ? reply(200, []) : reply(init.method === 'HEAD' ? 200 : 204, null)))
      const client = hosts(built, fetch)
      await client.items.all()
      await client.items.check({ path: { id: 1 } })
      await client.items.delete({ path: { id: 1 } })
      await client.items.purge({ body: { contentType: 'application/json', value: [1, 2] } })
      assert.deepEqual(fetch.calls.map((c) => [c.init.method, c.init.body, c.headers['content-type']]), [['GET', undefined, undefined], ['HEAD', undefined, undefined], ['DELETE', undefined, undefined], ['DELETE', '[1,2]', 'application/json']])
      for (const method of ['GET', 'HEAD']) {
        const e = await rejection(client.request(method, '/items', { body: { a: 1 } }))
        assert.ok(e instanceof built.hosts.SdkError && /carries no content/.test(e.message), String(e))
      }
      assert.equal(fetch.calls.length, 4)
    }],
    // req: rfc9110.9.3.7.must-client-generates-options-request-containing-content — content with OPTIONS goes with its Content-Type; without one it is refused
    ['R55: an OPTIONS request with content sends its Content-Type', async () => {
      const fetch = stub(() => reply(204, null))
      const client = hosts(built, fetch)
      await client.items.describe({ path: { id: 1 }, body: { contentType: 'text/plain', value: 'why' } })
      await client.items.describe({ path: { id: 1 } })
      await client.request('OPTIONS', '/items', { body: { a: 1 } })
      assert.deepEqual(fetch.calls.map((c) => [c.init.body, c.headers['content-type']]), [['why', 'text/plain'], [undefined, undefined], ['{"a":1}', 'application/json']])
      const e = await rejection(client.items.describe({ path: { id: 1 }, body: { contentType: 'text/plain', value: 'why' } }, { headers: { 'Content-Type': null } }))
      assert.ok(e instanceof built.hosts.SdkError && /needs a Content-Type/.test(e.message), String(e))
      assert.equal(fetch.calls.length, 3)
    }],
  ]
}
