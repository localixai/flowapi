// What the runtime tests run against: a stub fetch that records each call and answers with the
// platform's own Response, bodies that hang until the attempt's signal aborts, a fake clock that
// records every wait asked for and fires the earliest timer whenever the program has nothing else
// to do (so a test of a 60 s wait takes no time), and a signal that counts its listeners.
import assert from 'node:assert/strict'

export { assert }

/** A fetch answering by `answer(url, init, n)`; `calls` the requests it was given. It throws when
 * called with a `this` (R28: a platform's fetch refuses one). */
export function stub(answer) {
  const calls = []
  const fetch = async function (url, init) {
    if (this !== undefined) throw new TypeError('fetch was called with a this')
    calls.push({ url, init, headers: { ...init.headers } })
    return answer(url, init, calls.length)
  }
  return Object.assign(fetch, { calls })
}

/** A response: JSON for an object, else the body as given. */
export function reply(status, body, headers = {}) {
  if (body !== null && typeof body === 'object' && !(body instanceof Uint8Array) && !(body instanceof ReadableStream)) {
    return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json', ...headers } })
  }
  return new Response(body ?? null, { status, headers })
}

const aborted = () => new DOMException('This operation was aborted', 'AbortError')

/** A response whose headers came and whose body never ends, until the attempt aborts. */
export function hangingBody(init, headers = { 'content-type': 'application/json' }) {
  const body = new ReadableStream({
    start(c) {
      init.signal.addEventListener('abort', () => c.error(aborted()))
    },
  })
  return new Response(body, { status: 200, headers })
}

/** A fetch that never answers, until the attempt aborts. */
export function hanging(init) {
  return new Promise((_, reject) => init.signal.addEventListener('abort', () => reject(aborted())))
}

/** A body given in chunks; `cancelled()` says whether the reader cancelled it. */
export function chunked(chunks) {
  let cancelled = false
  const stream = new ReadableStream({
    pull(c) {
      const next = chunks.shift()
      if (next === undefined) c.close()
      else c.enqueue(typeof next === 'string' ? new TextEncoder().encode(next) : next)
    },
    cancel() {
      cancelled = true
    },
  })
  return { stream, cancelled: () => cancelled }
}

/** The fake clock, installed until `restore()`. `asked` holds each timer's milliseconds. */
export function clock() {
  const real = { setTimeout: globalThis.setTimeout, clearTimeout: globalThis.clearTimeout }
  const timers = new Map()
  const asked = []
  let now = 0
  let seq = 0
  let on = true
  globalThis.setTimeout = (fn, ms) => {
    const id = ++seq
    asked.push(ms)
    timers.set(id, { at: now + ms, fn, id })
    return id
  }
  globalThis.clearTimeout = (id) => timers.delete(id)
  const idle = () => new Promise((r) => setImmediate(r))
  ;(async () => {
    while (on) {
      for (let i = 0; i < 20 && on; i++) await idle()
      if (!on || timers.size === 0) continue
      const t = [...timers.values()].sort((a, b) => a.at - b.at || a.id - b.id)[0]
      timers.delete(t.id)
      now = t.at
      t.fn()
    }
  })()
  return {
    asked,
    now: () => now,
    pending: () => timers.size,
    restore() {
      on = false
      globalThis.setTimeout = real.setTimeout
      globalThis.clearTimeout = real.clearTimeout
    },
  }
}

/** Runs `f` with the fake clock and `Math.random` fixed at `random`. */
export async function timed(f, random = 0) {
  const c = clock()
  const r = Math.random
  Math.random = () => random
  try {
    return await f(c)
  } finally {
    Math.random = r
    c.restore()
  }
}

/** An abort controller whose signal counts the listeners it holds (R9). */
export function counted() {
  const controller = new AbortController()
  const s = controller.signal
  const live = new Map()
  const add = s.addEventListener.bind(s)
  const remove = s.removeEventListener.bind(s)
  s.addEventListener = (type, listener, options) => {
    const wrapped = () => {
      if (options?.once) live.delete(listener)
      listener()
    }
    live.set(listener, wrapped)
    add(type, wrapped, options)
  }
  s.removeEventListener = (type, listener) => {
    const wrapped = live.get(listener)
    live.delete(listener)
    if (wrapped !== undefined) remove(type, wrapped)
  }
  return { controller, signal: s, listeners: () => live.size }
}

/** The error a promise rejects with; fails when it resolves. */
export async function rejection(p) {
  try {
    await p
  } catch (e) {
    return e
  }
  assert.fail('it resolved')
}

/** A logger that keeps what it is given. */
export function recorder() {
  const lines = []
  const keep = (level) => (...a) => lines.push([level, a.map((x) => (typeof x === 'string' ? x : JSON.stringify(x))).join(' ')])
  return { lines, logger: { error: keep('error'), warn: keep('warn'), info: keep('info'), debug: keep('debug') } }
}
