// Where a call goes: the servers of the description as the client uses them. Each test names the
// pitfall it holds (R<n>, the runtime's list): R43 a URL with a user or a password, R44 a server
// variable's value (the caller's, else its default; one outside its `enum` refused), R45 a relative
// server URL, resolved against where the description is served. `hosts` is OpenAPI 3.2, `edge`
// 3.1: the runtime reads both alike, and each version's statements are held by its own case.
import { assert, rejection, reply, stub } from './stub.mjs'

const ok = () => reply(200, { up: true })

function hosts(built, fetch, more = {}) {
  return new built.hosts.Hosts({ fetch, logLevel: 'off', timeout: null, ...more })
}

function edge(built, fetch, more = {}) {
  return new built.edge.Edge({ fetch, logLevel: 'off', timeout: null, ...more })
}

export function tests(built) {
  const { SdkError } = built.hosts
  return [
    // req: rfc9110.4.2.4.must-not-sender-generate-userinfo-subcomponent-delimiter-http — a base URL, a description URL or a redirect's Location with a user or a password is never sent
    ['R43: a URL with a user or a password is refused, never sent nor logged', async () => {
      const fetch = stub(ok)
      const e = await rejection(hosts(built, fetch, { baseURL: 'https://me:secret@h.example' }).places.local())
      assert.ok(e instanceof SdkError && /user name or password/.test(e.message) && !e.message.includes('secret'), String(e))
      assert.throws(() => hosts(built, fetch, { descriptionURL: 'https://me@d.example/openapi.json' }), /user name or password/)
      assert.equal(fetch.calls.length, 0)
      const away = stub((url) => (url.includes('/items') ? reply(302, null, { location: 'https://me:secret@eu.hosts.example/v1/other' }) : ok()))
      const r = await rejection(hosts(built, away).items.all())
      assert.ok(r instanceof SdkError && /user name or password/.test(r.message), String(r))
      assert.equal(away.calls.length, 1)
    }],
    // req: oas-3.2.server-variable-object.default — the default sent: eu and v1
    // req: oas-3.2.server-variable-object.default.shall-default-value-use-substitution-sent-alternate — no value given: https://eu.hosts.example/v1; one given replaces it
    // req: oas-3.2.server-variable-object.enum — `region` takes eu or us; mars is refused before anything is sent
    // req: oas-3.2.server-object.variables — each `{name}` of the URL replaced by its variable's value
    // req: oas-3.2.operation-object.servers — an operation's own server, its variable checked against its enum when it is used
    ['R44: a server variable is the caller\'s value, else its default; a value outside its enum is refused', async () => {
      const fetch = stub(ok)
      await hosts(built, fetch).items.all()
      await hosts(built, fetch, { serverVariables: { region: 'us', version: 'v2' } }).items.all()
      await hosts(built, fetch).places.edge()
      await hosts(built, fetch, { serverVariables: { zone: 'b' } }).places.edge()
      await hosts(built, fetch, { serverVariables: { region: 'us' }, baseURL: 'https://mine.example' }).items.all()
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://eu.hosts.example/v1/items', 'https://us.hosts.example/v2/items', 'https://a.edge.example/edge', 'https://b.edge.example/edge', 'https://mine.example/items'])
      assert.equal(hosts(built, fetch, { serverVariables: { version: 'anything/at all' } }).baseURL, 'https://eu.hosts.example/anything/at all')
      assert.throws(() => hosts(built, fetch, { serverVariables: { region: 'mars' } }), (e) => e instanceof RangeError && /`mars` is not a value of the server variable `region`.*`eu`, `us`/.test(e.message))
      const zone = await rejection(hosts(built, fetch, { serverVariables: { zone: 'c' } }).places.edge())
      assert.ok(zone instanceof RangeError && /`c` is not a value of the server variable `zone`/.test(zone.message), String(zone))
      assert.equal(fetch.calls.length, 5)
    }],
    // req: oas-3.1.server-variable-object.default — the default sent: a and v3
    // req: oas-3.1.server-variable-object.default.shall-default-value-use-substitution-sent-alternate — no value given: https://a.edge.example/v3; one given replaces it
    // req: oas-3.1.server-variable-object.enum — `zone` takes a or b; z is refused
    ['R44: the same in OpenAPI 3.1', async () => {
      const fetch = stub(ok)
      await edge(built, fetch).zones.status()
      await edge(built, fetch, { serverVariables: { zone: 'b', version: 'v4' } }).zones.status()
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://a.edge.example/v3/status', 'https://b.edge.example/v4/status'])
      assert.throws(() => edge(built, fetch, { serverVariables: { zone: 'z' } }), RangeError)
    }],
    // req: oas-3.2.server-object.relative-references-in-api-urls.may-unless-specified-otherwise-all-fields-urls — a server URL `.` resolved against the description's retrieval URL, as the specification's example
    // req: oas-3.2.server-object.relative-references-in-api-urls.may-note-themselves-relative-referring-document — `.` of a Path Item is the description's own folder
    // req: oas-3.2.server-object.url.may-url-supports-server-variables-relative-indicate — a relative URL sends to where the description is served; without `descriptionURL` it cannot be sent
    // req: oas-3.2.path-item-object.servers — a Path Item's relative server serves its operation
    ['R45: a relative server URL is resolved against where the description is served', async () => {
      const fetch = stub(ok)
      const none = await rejection(hosts(built, fetch).places.local())
      assert.ok(none instanceof SdkError && /not absolute.*`descriptionURL`/.test(none.message), String(none))
      await hosts(built, fetch, { descriptionURL: 'https://device1.example.com' }).places.local()
      await hosts(built, fetch, { descriptionURL: 'https://device1.example.com/specs/hosts.json' }).places.local()
      await hosts(built, fetch, { descriptionURL: 'https://device1.example.com/specs/hosts.json' }).items.all()
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://device1.example.com/status', 'https://device1.example.com/specs/status', 'https://eu.hosts.example/v1/items'])
      assert.throws(() => hosts(built, fetch, { descriptionURL: '/specs/hosts.json' }), (e) => e instanceof TypeError && /absolute URL/.test(e.message))
    }],
    // req: oas-3.1.relative-references-in-api-urls.may-unless-specified-otherwise-all-fields-urls — `./near` of a Path Item resolved against the description's URL
    // req: oas-3.1.relative-references-in-api-urls.may-note-themselves-relative-referring-document — relative to the description that holds it, its folder
    // req: oas-3.1.server-object.url.may-url-supports-server-variables-relative-indicate — the relative URL sent to where the description is served
    ['R45: the same in OpenAPI 3.1', async () => {
      const fetch = stub(ok)
      await edge(built, fetch, { descriptionURL: 'https://cdn.example/specs/edge.json' }).zones.near()
      assert.deepEqual(fetch.calls.map((c) => c.url), ['https://cdn.example/specs/near/near'])
      assert.ok((await rejection(edge(built, fetch).zones.near())) instanceof built.edge.SdkError)
    }],
  ]
}
