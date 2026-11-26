// The behavior tests: each SDK, through its adapter, against the shared mock, and what the mock
// recorded held to what the case expects (`cases.mjs`). A test's result is `pass`, `fail` (the
// SDK has a surface for the feature, and what it did was not what the specification asks: the
// observation says what it did), or `n/a` (no surface for it in the generated code, or nothing
// in the description for it). Every result carries the observation it was judged by.
import { isDeepStrictEqual } from 'node:util'

const LIMIT_MS = 15_000
const WAIT_S = 1

const pairs = (text) => [...new URLSearchParams(text)]
/** Pairs grouped by name, each name's values in order: the order between names is free. */
const grouped = (list) => JSON.stringify(Object.entries(Object.groupBy(list, ([k]) => k)).map(([k, v]) => [k, v.map(([, x]) => x)]).sort())
const hex = (path) => path.replace(/%[0-9a-f]{2}/gi, (m) => m.toUpperCase())

function within(promise) {
  let timer
  const limit = new Promise((_, fail) => (timer = setTimeout(() => fail(new Error(`no answer in ${LIMIT_MS} ms`)), LIMIT_MS)))
  return Promise.race([promise, limit]).finally(() => clearTimeout(timer))
}

const said = (e) => String(e?.message ?? e).split('\n')[0].slice(0, 160)
const seen = (r) => (r ? `${r.method} ${r.target}` : 'no request')

/** What is wrong with `got` against `expect`, or null. */
function compare(got, expect) {
  if (!got) return 'no request reached the server'
  const wrong = []
  if (expect.method && got.method !== expect.method) wrong.push(`method ${got.method}`)
  if (expect.path && hex(got.path) !== hex(expect.path)) wrong.push(`path ${got.path}`)
  if (expect.query && grouped(pairs(got.query)) !== grouped(expect.query)) wrong.push(`query ${got.query}`)
  if (expect.json) {
    let body
    try {
      body = JSON.parse(got.body)
    } catch {
      body = got.body
    }
    if (!isDeepStrictEqual(body, expect.json)) wrong.push(`body ${got.body.slice(0, 120)}`)
    if (!/^application\/json/i.test(got.headers['content-type'] ?? '')) wrong.push(`content-type ${got.headers['content-type']}`)
  }
  if (expect.form) {
    if (grouped(pairs(got.body)) !== grouped(expect.form)) wrong.push(`body ${got.body.slice(0, 160)}`)
    if (!/^application\/x-www-form-urlencoded/i.test(got.headers['content-type'] ?? '')) wrong.push(`content-type ${got.headers['content-type']}`)
  }
  if (expect.header && got.headers[expect.header[0]] !== expect.header[1]) wrong.push(`${expect.header[0]}: ${got.headers[expect.header[0]] ?? '(none)'}`)
  if (expect.queryHas) {
    const [k, v] = expect.queryHas
    if (!pairs(got.query).some(([a, b]) => a === k && b === v)) wrong.push(`query ${got.query}`)
  }
  if (expect.cookie) {
    const cookies = (got.headers.cookie ?? '').split(/;\s*/).map((c) => c.split(/=(.*)/s).slice(0, 2))
    if (!cookies.some(([k, v]) => k === expect.cookie[0] && v === expect.cookie[1])) wrong.push(`cookie ${got.headers.cookie ?? '(none)'}`)
  }
  return wrong.length > 0 ? wrong.join('; ') : null
}

/** Every test of `kase` run with `adapter` on the built `sdk`; `ops` the description's operations. */
export async function behave(name, adapter, sdk, kase, ops, server) {
  const results = {}
  const op = (id) => {
    const found = ops.get(id)
    if (!found) throw new Error(`${kase.name}: no operation ${id}`)
    return found
  }
  const run = async (id, credentials, body) => {
    server.reset()
    try {
      results[id] = await within(body(adapter.client(sdk, credentials, kase)))
    } catch (e) {
      results[id] = { result: 'fail', observed: `${said(e)} (${seen(server.requests[0])})` }
    }
  }

  for (const test of kase.requests) {
    await run(test.id, kase.credentials, async (client) => {
      await adapter.call(client, op(test.op), test.args)
      const wrong = compare(server.requests[0], test.expect)
      return wrong ? { result: 'fail', observed: wrong } : { result: 'pass', observed: seen(server.requests[0]) }
    })
  }

  for (const test of kase.auth) {
    if (!adapter.auth || (test.declared && name !== 'flowapi')) {
      const why = adapter.auth ? 'the description declares no scheme' : 'no option for the scheme: the caller writes the credential'
      results[test.id] = { result: 'n/a', observed: why }
      continue
    }
    const expect = test.expect.query ? { queryHas: test.expect.query } : test.expect
    await run(test.id, { [test.scheme]: test.value }, async (client) => {
      await adapter.call(client, op(test.op), test.args)
      const got = server.requests[0]
      const wrong = compare(got, expect)
      const sent = test.expect.header ? `${test.expect.header[0]}: ${got?.headers[test.expect.header[0]]}` : test.expect.cookie ? `cookie: ${got?.headers.cookie}` : ''
      return wrong ? { result: 'fail', observed: wrong } : { result: 'pass', observed: `${seen(got)} ${sent}`.trim() }
    })
  }

  if (kase.retry) {
    for (const status of [429, 503]) {
      await run(`retry-${status}`, kase.credentials, async (client) => {
        server.answer({ status, headers: { 'retry-after': String(WAIT_S) }, body: { message: 'busy' } })
        const got = await adapter.call(client, op(kase.retry.op), kase.retry.args)
        const sent = server.requests
        const waited = sent.length > 1 ? sent[1].at - sent[0].at : 0
        const observed = `${sent.length} request(s)${sent.length > 1 ? `, the second after ${Math.round(waited)} ms` : ''}, then ${got.error ? `an error ${got.error.status}` : 'data'}`
        if (!adapter.retries) return { result: 'n/a', observed: `no retry option; ${observed}` }
        const ok = sent.length === 2 && waited >= WAIT_S * 1000 - 100 && !got.error
        return { result: ok ? 'pass' : 'fail', observed }
      })
    }
  }

  if (kase.paging) {
    const paging = kase.paging
    if (!adapter.page) results.paging = { result: 'n/a', observed: 'no pagination surface: the caller follows the next page' }
    else {
      await run('paging', kase.credentials, async (client) => {
        server.answer(...paging.pages)
        const items = await adapter.page(client, op(paging.op), paging.args)
        if (items === null) return { result: 'n/a', observed: 'this method does not page' }
        const sent = server.requests
        const [k, v] = paging.next
        const next = sent[1] && pairs(sent[1].query).some(([a, b]) => a === k && b === v)
        const observed = `${items.length} item(s) in ${sent.length} request(s)${sent[1] ? `; the second ${seen(sent[1])}` : ''}`
        return { result: items.length === paging.items && sent.length === 2 && next ? 'pass' : 'fail', observed }
      })
    }
  }

  if (kase.error) {
    await run('error', kase.credentials, async (client) => {
      server.answer(kase.error.answer)
      const got = await adapter.call(client, op(kase.error.op), kase.error.args)
      if (!got.error) return { result: 'fail', observed: `data returned for a ${kase.error.answer.status}: ${JSON.stringify(got.data)?.slice(0, 80)}` }
      const ok = got.error.status === kase.error.answer.status && isDeepStrictEqual(got.error.body, kase.error.answer.body)
      return { result: ok ? 'pass' : 'fail', observed: `status ${got.error.status}, body ${JSON.stringify(got.error.body)?.slice(0, 80)}` }
    })
  }
  return results
}
