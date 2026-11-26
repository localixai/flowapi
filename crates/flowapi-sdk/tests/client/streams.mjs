// Streams: server-sent events and JSON lines, read as they arrive. Each test names the pitfall it
// holds (P<n>, the lists' and streams' list; R5 of the runtime's): P15 a last event with no blank
// line, P16 and P17 the same events whatever the chunks (CR, CRLF and LF, a character cut, a byte
// order mark), P19 the fields as WHATWG reads them, P20 `id` kept and `retry` read, P21 a sentinel
// matched exactly, P23 an event of a name nobody knows, P25 an event over the size allowed, P26 a
// `break` and the caller's abort, P27 a stream read twice, P39 JSON lines, R5 a stream not retried.
import { assert, chunked, counted, rejection, stub } from './stub.mjs'

const enc = (s) => new TextEncoder().encode(s)
const SSE = { 'content-type': 'text/event-stream; charset=utf-8' }

const EVENTS = '﻿data: one\r\n\r\n: a comment\nevent: update\ndata:two\ndata:  lines\nid: 7\nretry: 3000\n\n'
  + 'retry: abc\ndata: ü€😀\r\rdata\n\nevent: strange\ndata: {"x":"[DONE]"}\n\ndata: last'
const WANT = [
  { event: undefined, data: 'one', id: undefined, retry: undefined },
  { event: 'update', data: 'two\n lines', id: '7', retry: 3000 },
  { event: undefined, data: 'ü€😀', id: '7', retry: 3000 },
  { event: undefined, data: '', id: '7', retry: 3000 },
  { event: 'strange', data: '{"x":"[DONE]"}', id: '7', retry: 3000 },
  { event: undefined, data: 'last', id: '7', retry: 3000 },
]

/** The chunkings of some bytes: cut once at every place, and a byte at a time. */
function cuts(bytes) {
  const out = []
  for (let i = 0; i <= bytes.length; i++) out.push([bytes.slice(0, i), bytes.slice(i)])
  out.push([...bytes].map((b) => new Uint8Array([b])))
  return out
}

async function all(iterable) {
  const out = []
  for await (const x of iterable) out.push(x)
  return out
}

function shop(built, fetch) {
  return new built.shop.Shop({ fetch, apiKeyAuth: 'k', logLevel: 'off' })
}

export function tests(built) {
  const { APIConnectionError, SdkError, Stream } = built.shop
  return [
    ['P15 P16 P17 P19 P20 P23: the same events at every cut of the bytes', async () => {
      for (const chunks of cuts(enc(EVENTS))) {
        const fetch = stub(() => new Response(chunked([...chunks]).stream, { status: 200, headers: SSE }))
        const got = await all(await shop(built, fetch).events.stream())
        assert.deepEqual(got, WANT, `cut into ${chunks.map((c) => c.length).join('+')}`)
      }
    }],
    ['P21: a sentinel ends the stream when it is the data, exactly', async () => {
      const text = 'data: a\n\ndata: {"t":"[DONE]"}\n\ndata: [DONE] \n\ndata: [DONE]\n\ndata: after\n\n'
      const s = Stream.of(new Response(text, { headers: SSE }), { sentinel: '[DONE]' }, () => {}, () => {})
      assert.deepEqual((await all(s)).map((e) => e.data), ['a', '{"t":"[DONE]"}', '[DONE] '])
      const none = Stream.of(new Response(text, { headers: SSE }), {}, () => {}, () => {})
      assert.equal((await all(none)).length, 5)
    }],
    ['P25: an event over maxEventSize, or a line without end, is refused', async () => {
      for (const text of ['data: 0123456789abc\n\n', 'data: 01234\ndata: 56789\n\n', 'data: 0123456789abcdef']) {
        const s = Stream.of(new Response(text, { headers: SSE }), { maxEventSize: 10 }, () => {}, () => {})
        const e = await rejection(all(s))
        assert.ok(e instanceof SdkError && /over 10 characters/.test(e.message), `${JSON.stringify(text)}: ${e}`)
      }
    }],
    ['P26: a break stops the request and cancels the body; the signal keeps no listener', async () => {
      const s = counted()
      const body = chunked(['data: a\n\n', 'data: b\n\n', 'data: c\n\n'])
      const fetch = stub(() => new Response(body.stream, { status: 200, headers: SSE }))
      const stream = await shop(built, fetch).events.stream({}, { signal: s.signal })
      assert.equal(s.listeners(), 1, 'the stream is the caller\'s abort to stop')
      for await (const e of stream) if (e.data === 'a') break
      assert.equal(body.cancelled(), true)
      assert.equal(fetch.calls[0].init.signal.aborted, true)
      assert.equal(s.listeners(), 0)
    }],
    ['P26: the caller\'s abort while a chunk is awaited ends the stream, with no error', async () => {
      const s = counted()
      const fetch = stub((_, init) => {
        const body = new ReadableStream({
          start(c) {
            c.enqueue(enc('data: a\n\n'))
            init.signal.addEventListener('abort', () => c.error(new DOMException('aborted', 'AbortError')))
          },
        })
        return new Response(body, { status: 200, headers: SSE })
      })
      const got = []
      for await (const e of await shop(built, fetch).events.stream({}, { signal: s.signal })) {
        got.push(e.data)
        setImmediate(() => s.controller.abort())
      }
      assert.deepEqual(got, ['a'])
      assert.equal(s.listeners(), 0)
    }],
    ['R5: a stream that breaks off is an APIConnectionError, and is not tried again', async () => {
      const fetch = stub(() => {
        let n = 0
        const body = new ReadableStream({
          pull(c) {
            if (n++ === 0) c.enqueue(enc('data: a\n\n'))
            else c.error(new TypeError('terminated'))
          },
        })
        return new Response(body, { status: 200, headers: SSE })
      })
      const got = []
      const e = await rejection((async () => {
        for await (const ev of await shop(built, fetch).events.stream()) got.push(ev.data)
      })())
      assert.ok(e instanceof APIConnectionError, String(e))
      assert.deepEqual([got, fetch.calls.length], [['a'], 1])
    }],
    ['P27: a stream is read once', async () => {
      const stream = await shop(built, stub(() => new Response('data: a\n\n', { headers: SSE }))).events.stream()
      await all(stream)
      assert.throws(() => stream[Symbol.asyncIterator](), (e) => e instanceof SdkError && /read once/.test(e.message))
    }],
    ['P39: JSON lines at every cut: CRLF, empty lines, a record separator, no end on the last', async () => {
      const text = '{"a":1}\r\n\n{"b":"ü"}\n\x1e{"c":3}\r\n  \n{"d":4}'
      for (const chunks of cuts(enc(text))) {
        const fetch = stub(() => new Response(chunked([...chunks]).stream, { status: 200, headers: { 'content-type': 'application/x-ndjson' } }))
        const got = await all(await new built.auth.AuthClient({ fetch, logLevel: 'off' }).checks.lines())
        assert.deepEqual(got, [{ a: 1 }, { b: 'ü' }, { c: 3 }, { d: 4 }])
      }
      const bad = stub(() => new Response('{"a":1}\nnot json\n', { headers: { 'content-type': 'application/x-ndjson' } }))
      const e = await rejection(all(await new built.auth.AuthClient({ fetch: bad, logLevel: 'off' }).checks.lines()))
      assert.ok(e instanceof built.auth.SdkError && e.message === 'a line of the stream is not JSON', String(e))
    }],
  ]
}
