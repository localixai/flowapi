// What a call sends and what it reads back: bodies, headers, URLs, the logs, redirects, the
// environment. Each test names the pitfall it holds (R<n>, the runtime's list): R16 an empty body,
// R17 JSON by its media type, R18 an error body that is not JSON, R19 no Content-Type without a
// body, R20 headers merged by name, R22 an environment variable of spaces, R23 an option over its
// variable, R24 a base URL and a path joined, R25 query parameters kept and replaced, R26 a method
// in lower case, R27 and R28 how fetch is called, R29 secrets kept out of the logs, R30 a log level
// that is none, R31 a client copied, R34 a redirect to another origin, R35 an environment that
// cannot be read, R42 bytes read as bytes. The rest: what `asResponse()` and `withResponse()` give,
// and a multipart body with the description's encoding.
import { assert, recorder, rejection, reply, stub } from './stub.mjs'

function shop(built, fetch, more = {}) {
  return new built.shop.Shop({ fetch, apiKeyAuth: 'key-1', logLevel: 'off', maxRetries: 0, ...more })
}

const pet = { id: 'p1', name: 'Rex' }

/** Runs `f` with these environment variables set (`undefined` unsets one), then puts them back. */
async function withEnv(vars, f) {
  const before = Object.fromEntries(Object.keys(vars).map((k) => [k, process.env[k]]))
  const put = (v) => {
    for (const [k, x] of Object.entries(v)) if (x === undefined) delete process.env[k]; else process.env[k] = x
  }
  put(vars)
  try {
    return await f()
  } finally {
    put(before)
  }
}

export function tests(built) {
  const { CrossOriginRedirectError, InternalServerError, NotFoundError, SdkError } = built.shop
  return [
    ['R16: a 204, and a 200 whose JSON body is empty, are no error', async () => {
      assert.equal(await shop(built, stub(() => reply(204, null))).pets.delete({ path: { pet_id: 'p1' } }), undefined)
      for (const headers of [{ 'content-type': 'application/json' }, { 'content-type': 'application/json', 'content-length': '0' }]) {
        assert.equal(await shop(built, stub(() => reply(200, '', headers))).pets.retrieve({ path: { pet_id: 'p1' } }), undefined)
      }
    }],
    ['R17: JSON is read by its media type, in any case and as +json', async () => {
      const got = await shop(built, stub(() => reply(200, JSON.stringify(pet), { 'content-type': 'Application/JSON; charset=UTF-8' }))).pets.retrieve({ path: { pet_id: 'p1' } })
      assert.deepEqual(got, pet)
      const e = await rejection(shop(built, stub(() => reply(404, JSON.stringify({ title: 'no pet' }), { 'content-type': 'application/problem+json' }))).pets.retrieve({ path: { pet_id: 'x' } }))
      assert.ok(e instanceof NotFoundError, String(e))
      assert.deepEqual(e.body, { title: 'no pet' })
    }],
    ['R18: an error body of HTML is its text, with no SyntaxError', async () => {
      const e = await rejection(shop(built, stub(() => reply(502, '<html>bad gateway</html>', { 'content-type': 'text/html' }))).system.health())
      assert.ok(e instanceof InternalServerError, String(e))
      assert.equal(e.body, '<html>bad gateway</html>')
      assert.equal(e.message, '502 <html>bad gateway</html>')
      assert.equal(e.cause, undefined)
    }],
    ['R19 R26 R27: no Content-Type without a body; the method in capitals; fetch given an init', async () => {
      const fetch = stub(() => reply(204, null))
      const client = shop(built, fetch)
      await client.pets.delete({ path: { pet_id: 'p1' } })
      await client.request('patch', '/pets/p1')
      assert.equal(fetch.calls[0].init.method, 'DELETE')
      assert.equal(fetch.calls[1].init.method, 'PATCH')
      for (const c of fetch.calls) {
        assert.equal(c.headers['content-type'], undefined)
        assert.ok(c.init !== undefined && c.init.body === undefined)
      }
    }],
    ['R20: headers merged by name without case; null removes one, undefined keeps it', async () => {
      const fetch = stub(() => reply(200, pet))
      const client = shop(built, fetch, { defaultHeaders: { 'X-Trace': 'a', 'X-Keep': 'k', 'X-Gone': 'g' } })
      await client.pets.retrieve({ path: { pet_id: 'p1' } }, { headers: [['x-TRACE', 'b'], ['x-gone', null]] })
      await client.pets.retrieve({ path: { pet_id: 'p1' } }, { headers: { 'X-Keep': undefined, 'x-api-key': null } })
      assert.deepEqual([fetch.calls[0].headers['x-trace'], fetch.calls[0].headers['x-keep'], 'x-gone' in fetch.calls[0].headers], ['b', 'k', false])
      assert.equal(Object.keys(fetch.calls[0].headers).filter((h) => h.toLowerCase() === 'x-trace').length, 1)
      assert.equal(fetch.calls[1].headers['x-keep'], 'k')
      assert.equal('x-api-key' in fetch.calls[1].headers, false, 'a null removes the credential and needs none')
    }],
    ['R22 R23: a variable of spaces is none, one is trimmed; an option comes first, null reads none', async () => {
      await withEnv({ SHOP_BASE_URL: '  ', SHOP_API_KEY: '  env-key  ', SHOP_TOKEN: undefined }, async () => {
        const fetch = stub(() => reply(200, pet))
        const plain = new built.shop.Shop({ fetch, logLevel: 'off' })
        assert.equal(plain.baseURL, 'https://api.shop.example/v1')
        await plain.pets.retrieve({ path: { pet_id: 'p1' } })
        await new built.shop.Shop({ fetch, apiKeyAuth: 'opt' }).pets.retrieve({ path: { pet_id: 'p1' } })
        assert.deepEqual(fetch.calls.map((c) => c.headers['x-api-key']), ['env-key', 'opt'])
        const e = await rejection(new built.shop.Shop({ fetch, apiKeyAuth: null }).pets.retrieve({ path: { pet_id: 'p1' } }))
        assert.ok(e instanceof SdkError && /`apiKeyAuth` or SHOP_API_KEY, or `bearerAuth` or SHOP_TOKEN/.test(e.message), e.message)
      })
      await withEnv({ SHOP_BASE_URL: ' https://env.example/v9 ' }, async () => {
        assert.equal(new built.shop.Shop({}).baseURL, 'https://env.example/v9')
        assert.equal(new built.shop.Shop({ baseURL: 'https://own.example' }).baseURL, 'https://own.example')
        assert.equal(new built.shop.Shop({ baseURL: null }).baseURL, 'https://api.shop.example/v1')
      })
      assert.equal(new built.shop.Shop({ environment: 'sandbox' }).baseURL, 'https://sandbox.shop.example/v1')
      assert.throws(() => new built.shop.Shop({ environment: 'sandbox', baseURL: 'https://x.example' }), TypeError)
      assert.throws(() => new built.shop.Shop({ environment: 'nowhere' }), RangeError)
    }],
    ['R24: one slash at the seam, the base\'s path and query kept, its fragment dropped', async () => {
      const fetch = stub(() => reply(200, pet))
      for (const baseURL of ['https://h.example/v1/', 'https://h.example/v1', 'https://h.example/v1#top', 'https://h.example/v1?tenant=a&tenant=b']) {
        await shop(built, fetch, { baseURL }).pets.retrieve({ path: { pet_id: 'p 1' } })
      }
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://h.example/v1/pets/p%201', 'https://h.example/v1/pets/p%201', 'https://h.example/v1/pets/p%201', 'https://h.example/v1/pets/p%201?tenant=a&tenant=b'])
      const e = await rejection(shop(built, fetch, { baseURL: '/relative' }).pets.retrieve({ path: { pet_id: 'p1' } }))
      assert.ok(e instanceof SdkError && /not absolute/.test(e.message), String(e))
    }],
    ['R25: repeated keys kept, the call\'s query over the operation\'s, nothing encoded twice', async () => {
      const fetch = stub(() => reply(200, { data: [], has_more: false }))
      const client = shop(built, fetch, { defaultQuery: { limit: 99, region: 'eu' } })
      await client.pets.list({ query: { tags: ['a b', 'c+/'], limit: 5 } }, { query: { extra: 1, limit: 7, region: null } })
      assert.equal(fetch.calls[0].url, 'https://api.shop.example/v1/pets?tags=a%20b&tags=c%2B%2F&extra=1&limit=7')
      await client.pets.list({ query: { limit: 5 } })
      assert.equal(fetch.calls[1].url, 'https://api.shop.example/v1/pets?limit=5&region=eu')
    }],
    ['R28: fetch is called with no this', async () => {
      assert.equal(await shop(built, stub(() => reply(200, 'up', { 'content-type': 'text/plain' }))).system.health(), 'up')
    }],
    ['R29: the logs mask credentials, secret query parameters, a URL\'s user and fragment', async () => {
      const log = recorder()
      const fetch = stub(() => reply(200, { ok: true }))
      const shopClient = shop(built, fetch, { logLevel: 'debug', logger: log.logger, baseURL: 'https://me:pw@h.example/v1#frag', bearerAuth: 'tok-secret' })
      await shopClient.pets.retrieve({ path: { pet_id: 'p1' } }, { headers: { cookie: 'sid=cookie-secret' } })
      const auth = new built.auth.AuthClient({ fetch, queryKey: 'query-secret', token: 'tok-secret', logLevel: 'debug', logger: log.logger })
      await auth.checks.both()
      const text = log.lines.map((l) => l[1]).join('\n')
      for (const secret of ['key-1', 'tok-secret', 'cookie-secret', 'query-secret', 'me:pw', '#frag']) assert.ok(!text.includes(secret), `${secret} in:\n${text}`)
      assert.ok(text.includes('"x-api-key":"***"') && text.includes('api_key=***'), text)
      assert.ok(fetch.calls[1].url.includes('api_key=query-secret'))
    }],
    ['R30: a log level that is none is warned about once and left at warn; the option needs none', async () => {
      await withEnv({ SHOP_LOG: 'loud' }, async () => {
        const log = recorder()
        const client = shop(built, stub(() => reply(200, pet)), { logLevel: undefined, logger: log.logger })
        await client.pets.retrieve({ path: { pet_id: 'p1' } })
        assert.equal(log.lines.length, 1)
        assert.match(log.lines[0][1], /SHOP_LOG is "loud"/)
        const quiet = recorder()
        await shop(built, stub(() => reply(200, pet)), { logLevel: 'info', logger: quiet.logger }).pets.retrieve({ path: { pet_id: 'p1' } })
        assert.ok(quiet.lines.every(([level]) => level === 'info') && quiet.lines.length === 2)
      })
    }],
    ['R31: withOptions keeps the class and every option; undefined keeps the old one', async () => {
      class Mine extends built.shop.Shop {}
      const fetch = stub(() => reply(200, pet))
      const a = new Mine({ fetch, apiKeyAuth: 'k', baseURL: 'https://a.example', timeout: 1000 })
      const b = a.withOptions({ timeout: 5000, baseURL: undefined })
      assert.ok(b instanceof Mine)
      assert.deepEqual([b.baseURL, b.timeout, a.timeout], ['https://a.example', 5000, 1000])
      await b.pets.retrieve({ path: { pet_id: 'p1' } })
      assert.equal(fetch.calls[0].headers['x-api-key'], 'k')
    }],
    // req: rfc9110.15.4.should-automatically-following-redirected-request-user-agent — a 303 followed as GET with no body or Content-Type, the other headers sent again
    // req: rfc6750.5.3.must-safeguard-bearer-tokens-client-implementations-ensure — a redirect to another origin refused, and allowed only without the credentials (the logs mask them: R29)
    ['R34: a redirect to another origin is refused; allowed, it loses the credentials', async () => {
      const away = (url) => (url.startsWith('https://api.') ? reply(307, null, { location: 'https://evil.example/steal' }) : reply(200, pet))
      const refused = stub(away)
      const e = await rejection(shop(built, refused).pets.retrieve({ path: { pet_id: 'p1' } }))
      assert.ok(e instanceof CrossOriginRedirectError && e.location === 'https://evil.example/steal', String(e))
      assert.equal(refused.calls.length, 1)
      const allowed = stub(away)
      await shop(built, allowed, { allowCrossOriginRedirects: true, bearerAuth: 't' }).pets.retrieve({ path: { pet_id: 'p1' } }, { headers: { 'x-extra': 'e' } })
      assert.deepEqual([allowed.calls[1].headers['x-api-key'], allowed.calls[1].headers['x-extra']], [undefined, 'e'])
      const same = stub((url, init) => (url.endsWith('/pets') && init.method === 'POST' ? reply(303, null, { location: '/v1/pets/p1' }) : reply(200, pet)))
      await shop(built, same).pets.create({ body: { contentType: 'application/json', value: pet } })
      assert.deepEqual([same.calls[1].init.method, same.calls[1].init.body, same.calls[1].headers['content-type'], same.calls[1].headers['x-api-key']], ['GET', undefined, undefined, 'key-1'])
    }],
    ['R35: an environment that throws when read is none', async () => {
      const deno = globalThis.Deno
      globalThis.Deno = { env: { get: () => { throw new Error('Requires env access') } } }
      try {
        const client = new built.shop.Shop({ fetch: stub(() => reply(200, 'up', { 'content-type': 'text/plain' })), logLevel: 'off' })
        assert.equal(await client.system.health(), 'up')
        const e = await rejection(client.pets.retrieve({ path: { pet_id: 'p1' } }))
        assert.ok(e instanceof SdkError && /needs a credential/.test(e.message), String(e))
      } finally {
        globalThis.Deno = deno
      }
    }],
    ['R42: bytes come back as they were sent', async () => {
      const bytes = new Uint8Array([0x00, 0x80, 0xff, 0xfe, 0x0d, 0x0a])
      const got = await shop(built, stub(() => reply(200, bytes, { 'content-type': 'application/octet-stream' }))).pets.photos.download({ path: { pet_id: 'p', photo_id: 'f' } })
      assert.deepEqual(new Uint8Array(got), bytes)
    }],
    ['asResponse gives the body unread; withResponse the data, the response and the request id', async () => {
      const fetch = stub(() => reply(200, pet, { 'x-request-id': 'req-7' }))
      const client = shop(built, fetch)
      const raw = await client.pets.retrieve({ path: { pet_id: 'p1' } }).asResponse()
      assert.equal(raw.bodyUsed, false)
      assert.deepEqual(await raw.json(), pet)
      const both = await client.pets.retrieve({ path: { pet_id: 'p1' } }).withResponse()
      assert.deepEqual([both.data, both.response.status, both.requestId], [pet, 200, 'req-7'])
      assert.equal(fetch.calls.length, 2)
    }],
    ['a multipart part takes the description\'s encoding, unless the input gives its own', async () => {
      const fetch = stub(() => reply(201, { id: 'f1' }))
      const client = shop(built, fetch)
      const text = (i) => new TextDecoder().decode(fetch.calls[i].init.body)
      await client.pets.photos.upload({ path: { pet_id: 'p1' }, body: { contentType: 'multipart/form-data', value: { photo: new Uint8Array([1, 2]), caption: 'hi' } } })
      assert.match(text(0), /name="photo"[^]*Content-Type: image\/png/i)
      assert.match(fetch.calls[0].headers['content-type'], /^multipart\/form-data; boundary=/)
      await client.pets.photos.upload({ path: { pet_id: 'p1' }, body: { contentType: 'multipart/form-data', value: { photo: new Uint8Array([1]) }, encoding: { photo: { contentType: 'image/jpeg' } } } })
      assert.match(text(1), /Content-Type: image\/jpeg/i)
      assert.doesNotMatch(text(1), /image\/png/)
    }],
  ]
}
