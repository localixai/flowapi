// The runtime's attempts: retries, their waits, timeouts and the caller's abort. Each test names the
// pitfall it holds (R<n>, the runtime's list): R1 a server's wait out of range, R2 one that cannot
// be read, R3 and R40 which header decides, R4 an abort during the wait, R6 and R7 options out of
// range, R8 a fresh signal for each attempt, R9 the caller's signal left with no listener, R11 the
// timeout covering the body, R12 an abort is not a timeout, R13 a runtime's own connect timeout,
// R37 a date to wait until, R38 the first wait not zero, R39 a fetch that throws, R41 a body
// cancelled before a retry.
import { assert, counted, chunked, hanging, hangingBody, rejection, reply, stub, timed } from './stub.mjs'

const ok = () => reply(200, 'up', { 'content-type': 'text/plain' })

/** A shop client with a key, quiet, no timeout unless asked. */
function shop(built, fetch, more = {}) {
  return new built.shop.Shop({ fetch, apiKeyAuth: 'key-1', logLevel: 'off', timeout: null, ...more })
}

/** The waits a 503 with these headers, then a 200, makes the client ask for. */
async function waits(built, headers, random = 0) {
  return timed(async (c) => {
    const fetch = stub((_, __, n) => (n === 1 ? reply(503, 'busy', headers) : ok()))
    assert.equal(await shop(built, fetch).system.health(), 'up')
    assert.equal(fetch.calls.length, 2)
    return c.asked
  }, random)
}

export function tests(built) {
  const { APIConnectionError, APIConnectionTimeoutError, APIUserAbortError, InternalServerError, NotFoundError, RateLimitError } = built.shop
  return [
    ['R1 R2 R38: an unreadable or spent retry-after waits the backoff, 0.375 to 0.5 s', async () => {
      const past = new Date(Date.now() - 60_000).toUTCString()
      for (const value of ['junk', '0', '-1', '', '  ', 'NaN', past]) {
        assert.deepEqual(await waits(built, { 'retry-after': value }, 0), [500], value)
        const [low] = await waits(built, { 'retry-after': value }, 0.9999)
        assert.ok(low >= 375 && low < 376, `${value}: ${low}`)
      }
      for (const value of ['junk', '-5', '']) assert.deepEqual(await waits(built, { 'retry-after-ms': value }), [500], value)
    }],
    ['R1: a wait past maxRetryDelay is refused: the error, one request', async () => {
      for (const headers of [{ 'retry-after': '2147484' }, { 'retry-after': '21' }, { 'retry-after-ms': '20001' }]) {
        const fetch = stub(() => reply(503, { message: 'later' }, headers))
        const e = await rejection(shop(built, fetch).system.health())
        assert.ok(e instanceof InternalServerError, String(e))
        assert.equal(fetch.calls.length, 1)
      }
      const fetch = stub((_, __, n) => (n === 1 ? reply(429, 'slow', { 'retry-after': '21' }) : ok()))
      await timed(async (c) => {
        assert.equal(await shop(built, fetch, { maxRetryDelay: null }).system.health(), 'up')
        assert.deepEqual(c.asked, [21_000])
      })
    }],
    ['R3 R40: retry-after-ms decides, 0 included; then seconds', async () => {
      assert.deepEqual(await waits(built, { 'retry-after-ms': '0', 'retry-after': '5' }), [0])
      assert.deepEqual(await waits(built, { 'retry-after-ms': '120', 'retry-after': '5' }), [120])
      assert.deepEqual(await waits(built, { 'retry-after': '2' }), [2000])
      assert.deepEqual(await waits(built, { 'retry-after': '1.5' }), [1500])
    }],
    ['R37: a retry-after date in the future, as an HTTP date and in ISO form', async () => {
      const at = Date.now() + 10_000
      for (const date of [new Date(at).toUTCString(), new Date(at).toISOString()]) {
        const [ms] = await waits(built, { 'retry-after': date })
        assert.ok(ms > 8000 && ms <= 10_000, `${date}: ${ms}`)
      }
    }],
    ['R38: the backoff doubles from 0.5 s, at most 8 s', async () => {
      await timed(async (c) => {
        const fetch = stub(() => reply(500, 'down'))
        await rejection(shop(built, fetch, { maxRetries: 6 }).system.health())
        assert.deepEqual(c.asked, [500, 1000, 2000, 4000, 8000, 8000])
        assert.equal(fetch.calls.length, 7)
      })
    }],
    // req: rfc9110.15.5.9.may-client-outstanding-request-transit-repeat-request — a 408 retried
    ['R5: which statuses are retried; x-should-retry decides alone', async () => {
      const cases = [[408, {}, 2], [409, {}, 2], [429, {}, 2], [500, {}, 2], [503, {}, 2], [400, {}, 1], [404, {}, 1], [503, { 'x-should-retry': 'false' }, 1], [400, { 'x-should-retry': 'true' }, 2]]
      for (const [status, headers, calls] of cases) {
        await timed(async () => {
          const fetch = stub((_, __, n) => (n === 1 ? reply(status, 'no', headers) : ok()))
          const p = shop(built, fetch, { maxRetries: 1 }).system.health()
          if (calls === 2) assert.equal(await p, 'up')
          else await rejection(p)
          assert.equal(fetch.calls.length, calls, `${status} ${JSON.stringify(headers)}`)
        })
      }
    }],
    ['R4 R9: an abort during a server-asked wait ends the call at once, its listener gone', async () => {
      await timed(async (c) => {
        const s = counted()
        const fetch = stub(() => {
          setImmediate(() => s.controller.abort())
          return reply(503, 'busy', { 'retry-after': '60' })
        })
        const e = await rejection(shop(built, fetch, { maxRetryDelay: 120_000 }).system.health({}, { signal: s.signal }))
        assert.ok(e instanceof APIUserAbortError, String(e))
        assert.equal(fetch.calls.length, 1)
        assert.ok(c.now() < 60_000)
        assert.equal(s.listeners(), 0)
        assert.equal(c.pending(), 0)
      })
    }],
    ['R6 R7: maxRetries, timeout and maxRetryDelay out of range are a RangeError', async () => {
      const fetch = stub(ok)
      for (const bad of [{ maxRetries: 1.5 }, { maxRetries: -1 }, { maxRetries: NaN }, { timeout: 0 }, { timeout: 2 ** 31 }, { timeout: 1.5 }, { maxRetryDelay: -1 }, { maxPages: 0 }]) {
        assert.throws(() => shop(built, fetch, bad), RangeError, JSON.stringify(bad))
      }
      for (const bad of [{ maxRetries: 1.5 }, { maxRetries: -1 }, { timeout: 0 }, { timeout: 2 ** 31 }]) {
        assert.ok((await rejection(shop(built, fetch).system.health({}, bad))) instanceof RangeError, JSON.stringify(bad))
      }
      assert.equal(fetch.calls.length, 0)
    }],
    ['R8 R13: an attempt that times out is retried with a fresh signal and timer', async () => {
      await timed(async (c) => {
        const fetch = stub((_, init, n) => (n === 1 ? hanging(init) : ok()))
        assert.equal(await shop(built, fetch, { timeout: 1000, maxRetries: 1 }).system.health(), 'up')
        const [first, second] = fetch.calls.map((x) => x.init.signal)
        assert.notEqual(first, second)
        assert.equal(first.aborted, true)
        assert.equal(second.aborted, false)
        assert.equal(c.pending(), 0)
      })
    }],
    ['R9: the caller\'s signal holds no listener after any end of a call', async () => {
      await timed(async () => {
        const ends = [
          ['success', () => ok(), (p) => p],
          ['an error status', () => reply(404, { message: 'gone' }), rejection],
          ['a failed connection', () => Promise.reject(new TypeError('fetch failed')), rejection],
          ['a retry', (n) => (n === 1 ? reply(503, 'busy') : ok()), (p) => p],
          ['a body that breaks', () => new Response(new ReadableStream({ pull: (c) => c.error(new TypeError('terminated')) }), { status: 200 }), rejection],
          ['the raw response', () => ok(), (p) => p],
        ]
        for (const [name, answer, end] of ends) {
          const s = counted()
          const fetch = stub((_, __, n) => answer(n))
          const call = shop(built, fetch, { maxRetries: 1, timeout: 5000 }).system.health({}, { signal: s.signal })
          await end(name === 'the raw response' ? call.asResponse() : call)
          assert.equal(s.listeners(), 0, name)
        }
      })
    }],
    ['R11 R13: the timeout covers the body: one that hangs times out, and is retried', async () => {
      await timed(async (c) => {
        const fetch = stub((_, init) => hangingBody(init, { 'content-type': 'text/plain' }))
        const e = await rejection(shop(built, fetch, { timeout: 2000, maxRetries: 1 }).system.health())
        assert.ok(e instanceof APIConnectionTimeoutError, String(e))
        assert.equal(fetch.calls.length, 2)
        assert.equal(c.pending(), 0)
      })
    }],
    ['R12: the caller\'s abort is APIUserAbortError and is not retried', async () => {
      await timed(async () => {
        const s = counted()
        const fetch = stub((_, init) => {
          setImmediate(() => s.controller.abort())
          return hanging(init)
        })
        const e = await rejection(shop(built, fetch, { timeout: 60_000 }).system.health({}, { signal: s.signal }))
        assert.ok(e instanceof APIUserAbortError && !(e instanceof APIConnectionTimeoutError), String(e))
        assert.equal(fetch.calls.length, 1)
        assert.equal(s.listeners(), 0)
        const before = counted()
        before.controller.abort()
        const f2 = stub(ok)
        assert.ok((await rejection(shop(built, f2).system.health({}, { signal: before.signal }))) instanceof APIUserAbortError)
        assert.equal(f2.calls.length, 0)
      })
    }],
    ['R13: a runtime\'s connect timeout, named in the cause, is a timeout and is retried', async () => {
      await timed(async () => {
        const cause = Object.assign(new Error('Connect Timeout Error (attempt timed out after 10000ms)'), { name: 'ConnectTimeoutError' })
        const fetch = stub((_, __, n) => (n === 1 ? Promise.reject(new TypeError('fetch failed', { cause })) : ok()))
        assert.equal(await shop(built, fetch, { maxRetries: 1 }).system.health(), 'up')
        const f2 = stub(() => Promise.reject(new TypeError('fetch failed', { cause })))
        assert.ok((await rejection(shop(built, f2, { maxRetries: 0 }).system.health())) instanceof APIConnectionTimeoutError)
      })
    }],
    ['R39: a fetch that throws is retried maxRetries times, then its error is the cause', async () => {
      await timed(async () => {
        const boom = new TypeError('boom from the caller\'s fetch')
        const fetch = stub(() => {
          throw boom
        })
        const e = await rejection(shop(built, fetch).system.health())
        assert.ok(e instanceof APIConnectionError, String(e))
        assert.equal(e.cause, boom)
        assert.equal(fetch.calls.length, 3)
      })
    }],
    ['R41: the body of an answer that is retried is cancelled, not left open', async () => {
      await timed(async () => {
        const bodies = []
        const fetch = stub((_, __, n) => {
          if (n > 1) return ok()
          const b = chunked(['busy', ' still'])
          bodies.push(b)
          return new Response(b.stream, { status: 503 })
        })
        assert.equal(await shop(built, fetch).system.health(), 'up')
        assert.equal(bodies[0].cancelled(), true)
      })
    }],
    ['R14: an error\'s name is its class\'s, a subclass\'s too; statuses have their classes', async () => {
      const { SdkError, APIError } = built.shop
      class Mine extends SdkError {}
      assert.equal(new Mine('x').name, 'Mine')
      assert.equal(new NotFoundError(404, {}, null, null).name, 'NotFoundError')
      assert.equal(APIError.of(429, {}, null, null).constructor, RateLimitError)
      assert.equal(APIError.of(418, {}, null, null).constructor, APIError)
      assert.equal(APIError.of(599, {}, null, null).name, 'InternalServerError')
    }],
  ]
}
