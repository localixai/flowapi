// Lists read a page at a time, by each scheme of the shop case. Each test names the pitfall it holds
// (P<n>, the lists' and streams' list): P1 an empty page that still leads on, P2 each scheme's end,
// P3 no request past the last page, P4 an offset grown by the items read, P5 a cursor that is none,
// P6 a last item with no id, P7 the next request is the last one with one parameter changed, P9 no
// next page asked for, P10 the call's options on every page, P12 a link resolved and kept to its
// origin, P13 a server that loops. The streams are in streams.mjs.
import { assert, counted, hanging, recorder, rejection, reply, stub } from './stub.mjs'

function shop(built, fetch, more = {}) {
  return new built.shop.Shop({ fetch, apiKeyAuth: 'k', logLevel: 'off', maxRetries: 0, ...more })
}

const HEADERS = Symbol('headers')

/** A body with headers, for `pages`. */
function withHeaders(body, headers) {
  return { [HEADERS]: [body, headers] }
}

/** A fetch answering the bodies in turn, as JSON. */
function pages(...bodies) {
  return stub((_, __, n) => {
    const b = bodies[n - 1]
    if (b === undefined) throw new Error(`no page ${n}`)
    return b[HEADERS] === undefined ? reply(200, b) : reply(200, ...b[HEADERS])
  })
}

async function all(iterable) {
  const out = []
  for await (const x of iterable) out.push(x)
  return out
}

const query = (call) => Object.fromEntries(new URL(call.url).searchParams)

export function tests(built) {
  const { SdkError, APIUserAbortError } = built.shop
  return [
    ['P1 P2: a cursor past an empty page, to has_more false', async () => {
      const fetch = pages({ data: [], next_page_token: 'a', has_more: true }, { data: [1, 2], next_page_token: 'b', has_more: true }, { data: [3], next_page_token: 'c', has_more: false })
      assert.deepEqual(await all(shop(built, fetch).pets.list()), [1, 2, 3])
      assert.deepEqual(fetch.calls.map((c) => query(c).page_token), [undefined, 'a', 'b'])
    }],
    ['P5: a cursor that is empty, null, missing or no string ends the list', async () => {
      for (const next of ['', null, undefined, 0]) {
        const fetch = pages({ data: [1], next_page_token: next })
        const page = await shop(built, fetch).pets.list()
        assert.equal(page.hasNextPage(), false, String(next))
        assert.deepEqual(await all(page), [1])
        assert.equal(fetch.calls.length, 1)
      }
    }],
    ['P7 P10: the next request is the last with the cursor changed; the input is not', async () => {
      const fetch = pages({ data: [1], next_page_token: 'a' }, { data: [2] })
      const input = { query: { limit: 1, tags: ['x', 'y'] } }
      const copy = structuredClone(input)
      assert.deepEqual(await all(shop(built, fetch).pets.list(input, { headers: { 'x-h': '1' }, query: { extra: 'e' } })), [1, 2])
      assert.deepEqual(input, copy)
      const second = new URL(fetch.calls[1].url).searchParams
      assert.deepEqual([second.get('page_token'), second.get('limit'), second.getAll('tags'), second.get('extra')], ['a', '1', ['x', 'y'], 'e'])
      assert.deepEqual(fetch.calls.map((c) => c.headers['x-h']), ['1', '1'])
    }],
    ['P10: the call\'s signal reaches every page', async () => {
      const s = counted()
      const fetch = stub((_, init, n) => {
        if (n === 1) return reply(200, { data: [1], next_page_token: 'a' })
        setImmediate(() => s.controller.abort())
        return hanging(init)
      })
      const page = await shop(built, fetch).pets.list({}, { signal: s.signal })
      assert.ok((await rejection(page.getNextPage())) instanceof APIUserAbortError)
      assert.equal(s.listeners(), 0)
    }],
    ['P9: no next page asked for is an error that says to ask hasNextPage()', async () => {
      const page = await shop(built, pages({ tags: ['a', 'b'] })).tags.list()
      assert.deepEqual([page.items, page.hasNextPage(), page.nextRequest()], [['a', 'b'], false, null])
      assert.throws(() => page.getNextPage(), (e) => e instanceof SdkError && /hasNextPage\(\)/.test(e.message))
    }],
    ['P2 P6: the last item\'s id leads on; a last item with no id is an error', async () => {
      const fetch = pages({ data: [{ id: 'u1' }, { id: 'u2' }], has_more: true }, { data: [{ id: 'u3' }], has_more: false })
      assert.deepEqual((await all(shop(built, fetch).users.list())).map((u) => u.id), ['u1', 'u2', 'u3'])
      assert.equal(query(fetch.calls[1]).after, 'u2')
      const e = await rejection(shop(built, pages({ data: [{ name: 'x' }], has_more: true })).users.list())
      assert.ok(e instanceof SdkError && /no `id`/.test(e.message), String(e))
      const empty = pages({ data: [], has_more: true })
      assert.deepEqual(await all(shop(built, empty).users.list()), [])
      assert.equal(empty.calls.length, 1)
    }],
    ['P3 P4: an offset grows by the items read; total and a short page end without a request', async () => {
      const fetch = pages({ items: ['a', 'b', 'c'], total: 5 }, { items: ['d', 'e'], total: 5 })
      assert.deepEqual(await all(shop(built, fetch).pets.photos.list({ path: { pet_id: 'p' } })), ['a', 'b', 'c', 'd', 'e'])
      assert.deepEqual(fetch.calls.map((c) => query(c).offset), [undefined, '3'])
      const short = pages({ items: ['a', 'b', 'c'], total: 100 })
      assert.deepEqual(await all(shop(built, short).pets.photos.list({ path: { pet_id: 'p' }, query: { limit: 5 } })), ['a', 'b', 'c'])
      assert.equal(short.calls.length, 1)
    }],
    ['P3: page numbers from 1, to total_pages, with no request past it', async () => {
      const fetch = pages({ items: [1], total_pages: 3 }, { items: [2], total_pages: 3 }, { items: [3], total_pages: 3 })
      const list = await shop(built, fetch).pages.list()
      let n = 0
      for await (const page of list.iterPages()) n += page.items.length
      assert.equal(n, 3)
      assert.deepEqual(fetch.calls.map((c) => query(c).page), [undefined, '2', '3'])
      const empty = pages({ items: [], total_pages: 9 })
      assert.deepEqual(await all(shop(built, empty).pages.list()), [])
    }],
    ['P12: a Link header\'s next, relative or absolute; another origin is refused when asked for', async () => {
      const fetch = pages(withHeaders(['o1'], { link: '</v1/owners?page=2>; rel="next", </v1/owners?page=9>; rel="last"' }), withHeaders(['o2'], { link: '<https://api.shop.example/v1/owners?page=3>; rel=next' }), ['o3'])
      assert.deepEqual(await all(shop(built, fetch).owners.list({ query: { per_page: 1 } })), ['o1', 'o2', 'o3'])
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://api.shop.example/v1/owners?per_page=1', 'https://api.shop.example/v1/owners?page=2', 'https://api.shop.example/v1/owners?page=3'])
      const away = pages(withHeaders(['o1'], { link: '<https://evil.example/owners?page=2>; rel="next"' }))
      const page = await shop(built, away).owners.list()
      assert.deepEqual(page.items, ['o1'])
      assert.throws(() => page.getNextPage(), (e) => e instanceof SdkError && /another origin/.test(e.message))
      assert.equal(away.calls.length, 1)
    }],
    ['P13: a cursor that repeats ends the list with a warning; maxPages refuses an endless one', async () => {
      const log = recorder()
      const loop = pages({ data: [1], next_page_token: 'a' }, { data: [2], next_page_token: 'a' })
      assert.deepEqual(await all(shop(built, loop, { logLevel: 'warn', logger: log.logger }).pets.list()), [1, 2])
      assert.equal(loop.calls.length, 2)
      assert.match(log.lines[0][1], /leads back to a page already read/)
      let n = 0
      const endless = stub(() => reply(200, { data: [++n], next_page_token: `t${n}` }))
      const e = await rejection(all(shop(built, endless, { maxPages: 3 }).pets.list()))
      assert.ok(e instanceof SdkError && /maxPages/.test(e.message), String(e))
      assert.equal(endless.calls.length, 3)
      await rejection(all(shop(built, endless).pets.list({}, { maxPages: 2 })))
      assert.equal(endless.calls.length, 5)
    }],
  ]
}
