// OAuth 2.0's client credentials grant run by the client (the `tokens` case): how the token is
// asked for, what is read of the answer, how long it is held, one request for the calls that wait
// on it, and a 401 that asks for another once. Each pitfall named is one a client gets wrong.
import { assert, reply, stub } from './stub.mjs'

const API = 'https://api.tokens.example/v1'
const TOKEN_URL = 'https://api.tokens.example/oauth/token?tenant=main'

/** Whether a request goes to a token endpoint. */
const isToken = (url) => new URL(url).pathname.endsWith('/token')

/** A fetch that answers the token endpoint by `token(n)` (its n-th request) and the API by
 * `api(n)`; `asked` the token requests, `calls` every request. */
function server({ token = () => reply(200, { access_token: 'tok1', token_type: 'Bearer', expires_in: 3600 }), api = () => reply(200, { ok: true }) } = {}) {
  let tokens = 0
  let calls = 0
  const fetch = stub((url, init) => (isToken(url) ? token(++tokens, url, init) : api(++calls, url, init)))
  return Object.assign(fetch, { asked: () => fetch.calls.filter((c) => isToken(c.url)) })
}

const credentials = { clientId: 'my client', clientSecret: 'sé:cret' }

function client(built, fetch, more = {}) {
  return new built.tokens.Tokens({ fetch, logLevel: 'off', maxRetries: 0, client: credentials, ...more })
}

export function tests(built) {
  const { AuthenticationError, PermissionDeniedError, TokenError, InsecureCredentialError } = built.tokens
  return [
    // req: rfc6749.3.2.must-client-use-http-post-method-making — the token request is a POST
    // req: rfc6749.3.2.must-endpoint-uri-include-application-x-www — `?tenant=main` kept on the token URL
    // req: rfc6749.4.4.2.required-grant-type — `grant_type` sent
    // req: rfc6749.4.4.2.must-value-set-client-credentials — as `client_credentials`
    // req: rfc6749.4.4.2.optional-scope — `scope` sent with the operation's scopes
    // req: rfc6749.2.3.1.may-clients-possession-client-password-use-http — the client authenticates by HTTP basic
    // req: rfc6749.3.2.1.must-confidential-clients-other-clients-issued-client — with its id and secret on every token request
    // req: rfc6749.4.4.2.must-client-authenticate-authorization-server-described-section — as 3.2.1 says
    // req: rfc6749.2.3.1.must-not-parameters-only-transmitted-request-body-included — no credential in the token URL
    // req: rfc6749.2.3.1.not-recommended-including-client-credentials-request-body-using — no `client_id` or `client_secret` in the body
    // req: rfc6749.2.3.must-not-client-use-more-one-authentication-method — basic alone, nothing else
    // req: rfc6749.B.must-address-shortcoming-generating-payloads-using-media — the id and secret UTF-8, then form-encoded, before base64
    ['client credentials: a POST of the form to the token URL kept with its query, the client by HTTP basic alone', async () => {
      const fetch = server()
      await client(built, fetch).things.list()
      const [t, call] = fetch.calls
      assert.equal(t.url, TOKEN_URL)
      assert.equal(t.init.method, 'POST')
      assert.equal(t.headers['content-type'], 'application/x-www-form-urlencoded')
      assert.equal(t.init.body, 'grant_type=client_credentials&scope=read')
      assert.equal(t.headers.authorization, 'Basic ' + Buffer.from('my+client:s%C3%A9%3Acret', 'utf8').toString('base64'))
      assert.ok(!/client_(id|secret)/.test(t.init.body) && !/my|cret/.test(t.url), 'no client credential in the body or the URL')
      assert.equal(call.url, `${API}/things`)
      assert.equal(call.headers.authorization, 'Bearer tok1')
    }],
    // req: rfc6749.3.2.must-not-request-response-parameters-included-more-once — each parameter, and each scope, once
    // req: rfc6749.10.3.should-client-request-access-tokens-minimal-scope — the scopes of the operation called, not every scope
    ['the scopes the operation needs, each once; another set of scopes is another token', async () => {
      const fetch = server({ token: (n) => reply(200, { access_token: `tok${n}`, token_type: 'bearer', expires_in: '3600' }) })
      const c = client(built, fetch)
      await c.things.list()
      await c.things.list()
      await c.things.write()
      await c.things.write()
      const bodies = fetch.asked().map((t) => t.init.body)
      assert.deepEqual(bodies, ['grant_type=client_credentials&scope=read', 'grant_type=client_credentials&scope=read+write'])
      for (const b of bodies) assert.equal(new URLSearchParams(b).getAll('grant_type').length, 1)
      assert.deepEqual(fetch.calls.filter((c) => !isToken(c.url)).map((c) => c.headers.authorization), ['Bearer tok1', 'Bearer tok1', 'Bearer tok2', 'Bearer tok2'])
      const own = server()
      await client(built, own, { partner: { ...credentials, scopes: ['a', 'b', 'a'] } }).others.partner()
      assert.equal(own.asked()[0].init.body, 'grant_type=client_credentials&scope=a+b')
      assert.equal(own.asked()[0].url, 'https://login.tokens.example/token')
    }],
    ['a token is held until a little before it expires; one that expires at once is asked for again', async () => {
      const fetch = server({ token: (n) => reply(200, { access_token: `tok${n}`, token_type: 'Bearer', expires_in: 0 }) })
      const c = client(built, fetch)
      await c.things.list()
      await c.things.list()
      assert.equal(fetch.asked().length, 2)
      const lasting = server({ token: (n) => reply(200, { access_token: `tok${n}`, token_type: 'Bearer' }) })
      const d = client(built, lasting)
      await d.things.list()
      await d.things.list()
      assert.equal(lasting.asked().length, 1, 'with no expires_in the token is held')
    }],
    ['calls that wait on the same token make one token request', async () => {
      let release
      const gate = new Promise((r) => (release = r))
      const fetch = server({ token: async () => (await gate, reply(200, { access_token: 'tok', token_type: 'Bearer', expires_in: 60 })) })
      const c = client(built, fetch)
      const all = Promise.all([c.things.list(), c.things.list(), c.things.list()])
      await new Promise((r) => setTimeout(r, 5))
      release()
      await all
      assert.equal(fetch.asked().length, 1)
    }],
    // req: rfc6749.5.1.must-client-ignore-unrecognized-value-names-response — members the client does not know are ignored
    // req: rfc6749.7.1.must-not-client-use-access-token-does-not — a token of a type other than bearer is not used
    ['the answer: unknown members ignored; a token of a type the client does not know is never sent', async () => {
      const fetch = server({ token: () => reply(200, { access_token: 'tok', token_type: 'Bearer', expires_in: 60, refresh_hint: { x: 1 }, scope: 'read' }) })
      await client(built, fetch).things.list()
      assert.equal(fetch.calls[1].headers.authorization, 'Bearer tok')
      const mac = server({ token: () => reply(200, { access_token: 'tok', token_type: 'mac' }) })
      await assert.rejects(client(built, mac).things.list(), (e) => e instanceof TokenError && e.error === 'unsupported_token_type')
      assert.equal(mac.calls.length, 1, 'the API was not called')
      const none = server({ token: () => reply(200, { token_type: 'Bearer' }) })
      await assert.rejects(client(built, none).things.list(), (e) => e instanceof TokenError && e.error === 'invalid_response')
    }],
    ['an error answer of the token endpoint is a TokenError with its code, description and URI', async () => {
      const fetch = server({ token: () => reply(401, { error: 'invalid_client', error_description: 'unknown client', error_uri: 'https://docs.example/e' }) })
      await assert.rejects(client(built, fetch).things.list(), (e) => e instanceof TokenError && e.status === 401 && e.error === 'invalid_client' && e.description === 'unknown client' && e.uri === 'https://docs.example/e')
      const text = server({ token: () => reply(500, 'down') })
      await assert.rejects(client(built, text).things.list(), (e) => e instanceof TokenError && e.status === 500 && e.description === 'down')
    }],
    // req: rfc6750.3.1.may-client-request-new-access-token-retry — a 401 to a token the client requested: another token, the call once more
    ['a 401 to a requested token drops it and makes the call once more with a new one; never twice', async () => {
      const fetch = server({ token: (n) => reply(200, { access_token: `tok${n}`, token_type: 'Bearer', expires_in: 3600 }), api: (n) => reply(n === 1 ? 401 : 200, { ok: n !== 1 }) })
      const c = client(built, fetch)
      await c.things.list()
      assert.deepEqual(fetch.calls.map((x) => x.headers.authorization), [fetch.calls[0].headers.authorization, 'Bearer tok1', fetch.calls[0].headers.authorization, 'Bearer tok2'])
      const always = server({ api: () => reply(401, { error: 'no' }) })
      await assert.rejects(client(built, always).things.list(), AuthenticationError)
      assert.equal(always.calls.length, 4, 'token, call, token, call: one renewal')
    }],
    // req: rfc9110.15.5.4.should-not-client-automatically-repeat-request-same-credentials — a 403 is not repeated, nor a 401 to a token the caller gave
    ['a 403, and a 401 to a token the caller gave, are not repeated with the same credentials', async () => {
      const denied = server({ api: () => reply(403, { error: 'forbidden' }) })
      await assert.rejects(client(built, denied, { maxRetries: 3 }).things.list(), PermissionDeniedError)
      assert.equal(denied.calls.length, 2, 'one token request, one call')
      const given = server({ api: () => reply(401, { error: 'no' }) })
      await assert.rejects(client(built, given, { client: 'caller-token', maxRetries: 3 }).things.list(), AuthenticationError)
      assert.equal(given.calls.length, 1)
      assert.equal(given.calls[0].headers.authorization, 'Bearer caller-token')
    }],
    // req: rfc6749.10.3.must-access-token-credentials-only-transmitted-using — the token endpoint only over TLS
    // req: rfc6749.10.8.must-not-access-tokens-refresh-tokens-resource-owner — client credentials never sent in the clear
    // req: rfc6750.5.2.must-second-confidentiality-protection-exchanges-between-client — both exchanges, with the token endpoint and with the API
    ['the token endpoint only over https or to a loopback host, unless allowInsecureCredentials', async () => {
      const fetch = server()
      await assert.rejects(client(built, fetch, { client: { ...credentials, tokenURL: 'http://login.example/token' } }).things.list(), InsecureCredentialError)
      assert.equal(fetch.calls.length, 0)
      await client(built, fetch, { client: { ...credentials, tokenURL: 'http://localhost:8080/token' }, baseURL: 'http://127.0.0.1:9/v1' }).things.list()
      await client(built, fetch, { client: { ...credentials, tokenURL: 'http://login.example/token' }, allowInsecureCredentials: true }).things.list()
      assert.deepEqual(fetch.asked().map((t) => t.url), ['http://localhost:8080/token', 'http://login.example/token'])
    }],
    ['a call the TLS rule refuses requests no token: the rule holds the base URL before a token is asked for', async () => {
      const fetch = server()
      const over = { client: { ...credentials, tokenURL: 'https://login.tokens.example/token' }, baseURL: 'http://api.tokens.example/v1' }
      await assert.rejects(client(built, fetch, over).things.list(), (e) => e instanceof InsecureCredentialError && e.message.includes('http://api.tokens.example/v1'))
      assert.equal(fetch.calls.length, 0, 'no token request for a call that is not sent')
    }],
  ]
}
