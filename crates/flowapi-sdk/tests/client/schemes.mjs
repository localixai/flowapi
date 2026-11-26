// Every kind of scheme the `tokens`, `github` and `swagger` cases declare, as the client sends it:
// credentials only over TLS or to a loopback host, a bearer token only in `Authorization`, HTTP
// basic's control characters refused, an HTTP scheme the client cannot answer refused, a client
// certificate said to be given, `{}` making a credential optional, schemes the config declares
// for a description with none, Swagger 2.0's, and no credential in a log line or across origins.
import { readFileSync } from 'node:fs'
import { assert, reply, stub } from './stub.mjs'

const ok = () => reply(200, { ok: true })
const env = ['GITHUB_TOKEN', 'GITHUB_APP_KEY', 'GITHUB_ADMIN_USER', 'GITHUB_ADMIN_PASSWORD', 'SWAGGER_BASIC', 'SWAGGER_HEADER_KEY', 'SWAGGER_QUERY_KEY', 'SWAGGER_CLIENT_ID', 'SWAGGER_CLIENT_SECRET', 'TOKENS_KEY', 'TOKENS_GNAP', 'TOKENS_OIDC']

/** Runs `f` with the variables `set` set and the cases' other credential variables unset. */
async function clean(set, f) {
  const before = env.map((k) => process.env[k])
  for (const k of env) delete process.env[k]
  Object.assign(process.env, set)
  try {
    return await f()
  } finally {
    env.forEach((k, i) => {
      if (before[i] === undefined) delete process.env[k]
      else process.env[k] = before[i]
    })
  }
}

export function tests(built) {
  // Each case has its own runtime, so an error is known by its class's name.
  const credential = (e) => e?.name === 'CredentialError'
  const insecure = (e) => e?.name === 'InsecureCredentialError'
  const tokens = (fetch, more = {}) => new built.tokens.Tokens({ fetch, logLevel: 'off', maxRetries: 0, ...more })
  return [
    // req: rfc6750.5.3.must-always-use-tls-https-clients-always — a bearer token only over https, or to a loopback host
    // req: rfc6749.10.3.must-access-token-credentials-well-any-confidential — no token sent in the clear
    // req: rfc7617.4.should-not-because-basic-authentication-involves-cleartext-transmission — basic's pair too
    // req: rfc6750.5.2.must-not-therefore-bearer-tokens-stored-cookies-sent — a cookie credential is held to the same rule
    // req: rfc6750.5.3.must-not-don-t-store-bearer-tokens-cookies — the client keeps no cookie: a bearer token goes in no cookie
    ['credentials over http: refused before sending but to a loopback host or with allowInsecureCredentials', () => clean({}, async () => {
      const fetch = stub(ok)
      await assert.rejects(tokens(fetch, { oidc: 't', baseURL: 'http://api.example' }).others.oidc(), insecure)
      await assert.rejects(new built.swagger.Swagger({ fetch, basic: 'u:p', baseURL: 'http://api.example' }).pets.list(), insecure)
      await assert.rejects(new built.auth.AuthClient({ fetch, cookieKey: 's', baseURL: 'http://api.example' }).checks.cookie({ cookie: { theme: 'x' } }), insecure)
      assert.equal(fetch.calls.length, 0)
      for (const base of ['http://localhost:3000', 'http://127.0.0.1', 'http://[::1]:8080']) await tokens(fetch, { oidc: 't', baseURL: base }).others.oidc()
      await tokens(fetch, { oidc: 't', baseURL: 'http://api.example', allowInsecureCredentials: true }).others.oidc()
      await new built.github.GitHub({ fetch, token: null, baseURL: 'http://api.example' }).meta.get()
      assert.equal(fetch.calls.length, 5, 'a call that carries no credential goes over http')
      assert.ok(fetch.calls.every((c) => c.headers.cookie === undefined), 'no bearer token in a cookie')
    })],
    // req: rfc6750.2.must-not-clients-use-more-one-method-transmit — the token in one place only
    // req: rfc6750.2.2.must-not-client-use-method-unless-all-following — never in a form body
    // req: rfc6750.2.3.should-not-because-security-weaknesses-associated-uri-method — never in the URI
    // req: rfc6750.5.3.should-instead-bearer-tokens-passed-http-message — in the Authorization header
    // req: rfc6750.5.3.should-not-don-t-pass-bearer-tokens-page — no token in a page URL
    ['a bearer token goes in Authorization alone: not in the query, the body or a cookie', () => clean({}, async () => {
      const fetch = stub(ok)
      await tokens(fetch, { oidc: 'abc.def-1~+/==' }).others.oidc()
      await tokens(fetch, { client: 'xyz' }).things.write()
      for (const c of fetch.calls) {
        assert.ok(!c.url.includes('abc') && !c.url.includes('xyz') && !c.url.includes('access_token'), c.url)
        assert.equal(c.init.body, undefined)
        assert.equal(c.headers.cookie, undefined)
      }
      assert.deepEqual(fetch.calls.map((c) => c.headers.authorization), ['Bearer abc.def-1~+/==', 'Bearer xyz'])
      await assert.rejects(tokens(fetch, { oidc: 'a b' }).others.oidc(), (e) => credential(e) && /RFC 6750 2\.1/.test(e.message))
    })],
    // req: rfc7617.2.must-not-user-id-password-contain-any-control — a control character in either half is refused
    ['HTTP basic: a control character in the user-id or the password, or a user-id with a colon, is refused', () => clean({}, async () => {
      const fetch = stub(ok)
      const call = (basic) => new built.swagger.Swagger({ fetch, basic }).pets.list()
      await assert.rejects(call({ username: 'a\u0001', password: 'p' }), credential)
      await assert.rejects(call({ username: 'a', password: 'p\n' }), credential)
      await assert.rejects(call({ username: 'a:b', password: 'p' }), /RFC 7617/)
      await assert.rejects(call('no-colon'), credential)
      assert.equal(fetch.calls.length, 0)
      await call('ü:p:ß')
      assert.equal(fetch.calls[0].headers.authorization, 'Basic ' + Buffer.from('ü:p:ß', 'utf8').toString('base64'))
    })],
    ['an HTTP scheme sent as given; one that answers a challenge refused, unless the call gives Authorization', () => clean({ TOKENS_GNAP: 'g1' }, async () => {
      const fetch = stub(ok)
      await tokens(fetch).others.gnap()
      assert.equal(fetch.calls[0].headers.authorization, 'gnap g1')
      await assert.rejects(tokens(fetch, { digest: 'x' }).others.digest(), (e) => credential(e) && /digest answers a challenge/.test(e.message))
      await tokens(fetch).others.digest({}, { headers: { Authorization: 'Digest username="u", response="r"' } })
      assert.equal(fetch.calls[1].headers.authorization, 'Digest username="u", response="r"')
    })],
    // req: oas-3.2.security-scheme-object.deprecated.should-consumers-refrain-usage-declared-scheme — a deprecated scheme still works, its option marked @deprecated and the config warned (tests/client/tokens/issues.txt)
    ['a client certificate is `true`; `{}` makes a credential optional; a deprecated key still sent', () => clean({}, async () => {
      const fetch = stub(ok)
      await assert.rejects(tokens(fetch).others.cert(), (e) => credential(e) && /cert/.test(e.message))
      await tokens(fetch, { cert: true }).others.cert()
      await tokens(fetch).others.optional()
      await tokens(fetch, { legacy: 'k' }).others.optional()
      await assert.rejects(tokens(fetch).others.legacy(), credential)
      process.env.TOKENS_KEY = 'from-env'
      await tokens(fetch).others.legacy()
      assert.deepEqual(fetch.calls.map((c) => [c.headers.authorization, c.headers['x-api-key']]), [[undefined, undefined], [undefined, undefined], [undefined, 'k'], [undefined, 'from-env']])
      const kept = (file) => readFileSync(new URL(`./tokens/${file}`, import.meta.url), 'utf8')
      assert.match(kept('src/client.ts'), /`Legacy`[^\n]*@deprecated[^\n]*\n {2}legacy\?:/)
      assert.match(kept('issues.txt'), /warning\[config\.auth\.deprecated\]: `Legacy` is deprecated/)
    })],
    ['schemes the config declares for a description with none: an optional token, none, a key, basic from two variables', () => clean({}, async () => {
      const fetch = stub(ok)
      const gh = (more) => new built.github.GitHub({ fetch, ...more })
      await gh().users.me()
      process.env.GITHUB_TOKEN = 'ghp_1'
      await gh().users.me()
      await gh().meta.get()
      await assert.rejects(gh().apps.hook(), (e) => credential(e) && /GITHUB_APP_KEY/.test(e.message))
      await gh({ appKey: 'app' }).apps.hook()
      await assert.rejects(gh().enterprise.settings(), (e) => /GITHUB_ADMIN_USER with GITHUB_ADMIN_PASSWORD/.test(e.message))
      Object.assign(process.env, { GITHUB_ADMIN_USER: 'root', GITHUB_ADMIN_PASSWORD: 'pw' })
      await gh().enterprise.settings()
      assert.deepEqual(fetch.calls.map((c) => [c.url, c.headers.authorization, c.headers['x-app-key']]), [
        ['https://api.github.example/user', undefined, undefined],
        ['https://api.github.example/user', 'Bearer ghp_1', undefined],
        ['https://api.github.example/meta', undefined, undefined],
        ['https://api.github.example/app/hook', undefined, 'app'],
        ['https://api.github.example/enterprise/settings', 'Basic ' + Buffer.from('root:pw').toString('base64'), undefined],
      ])
    })],
    ["Swagger 2.0: basic by default, a header key else a query key, the application flow's token, none", () => clean({ SWAGGER_BASIC: 'u:p', SWAGGER_QUERY_KEY: 'q' }, async () => {
      const fetch = stub((url) => (String(url).endsWith('/oauth/token') ? reply(200, { access_token: 'sw', token_type: 'Bearer' }) : ok()))
      const sw = (more) => new built.swagger.Swagger({ fetch, ...more })
      await sw().pets.list()
      await sw().pets.add()
      await sw({ header: 'h' }).pets.add()
      await sw({ app: { clientId: 'c', clientSecret: 's' } }).pets.mine()
      await sw({ basic: null }).health.get()
      const [list, byQuery, byHeader, token, mine, health] = fetch.calls
      assert.equal(list.headers.authorization, 'Basic ' + Buffer.from('u:p').toString('base64'))
      assert.equal(byQuery.url, 'https://api.swagger.example/v1/pets?key=q')
      assert.deepEqual([byHeader.headers['x-key'], byHeader.url], ['h', 'https://api.swagger.example/v1/pets'])
      assert.deepEqual([token.url, token.init.body], ['https://swagger.example/oauth/token', 'grant_type=client_credentials&scope=pets%3Aread'])
      assert.equal(mine.headers.authorization, 'Bearer sw')
      assert.equal(health.headers.authorization, undefined)
    })],
    ['no credential in a log line: headers masked, a key in the query redacted, the token request without its secret', () => clean({}, async () => {
      const lines = []
      const logger = Object.fromEntries(['error', 'warn', 'info', 'debug'].map((l) => [l, (...a) => lines.push(JSON.stringify(a))]))
      const fetch = stub((url) => (String(url).includes('/oauth/token') ? reply(200, { access_token: 'SECRET-TOKEN', token_type: 'Bearer' }) : ok()))
      await tokens(fetch, { logLevel: 'debug', logger, client: { clientId: 'id', clientSecret: 'SECRET-CLIENT' }, legacy: 'SECRET-KEY' }).things.list()
      await tokens(fetch, { logLevel: 'debug', logger, legacy: 'SECRET-KEY' }).others.legacy()
      await new built.swagger.Swagger({ fetch, logLevel: 'debug', logger, query: 'SECRET-QUERY' }).pets.add()
      assert.ok(lines.length > 4, 'something was logged')
      for (const line of lines) assert.ok(!/SECRET|U0VDUkVU/.test(line), line)
    })],
    ['a redirect to another origin drops every credential header when it is followed', () => clean({}, async () => {
      const fetch = stub((url, init, n) => (n === 1 ? reply(302, '', { location: 'https://elsewhere.example/x' }) : ok()))
      await tokens(fetch, { gnap: 'g', allowCrossOriginRedirects: true }).others.gnap()
      const legacy = stub((url, init, n) => (n === 1 ? reply(307, '', { location: 'https://elsewhere.example/x' }) : ok()))
      await tokens(legacy, { legacy: 'k', allowCrossOriginRedirects: true }).others.legacy()
      assert.deepEqual([fetch.calls[0].headers.authorization, fetch.calls[1].headers.authorization], ['gnap g', undefined])
      assert.deepEqual([legacy.calls[0].headers['x-api-key'], legacy.calls[1].headers['x-api-key']], ['k', undefined])
    })],
  ]
}
