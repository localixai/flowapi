// The descriptions `sdk-compare.mjs` builds an SDK of, and the calls each SDK is made to send.
// Every expected request is read from the description and the specification's text, never from
// what a generator sends:
// - a path parameter is `simple` style (OpenAPI 3.x 4.8.12.4): RFC 6570's simple expansion, so
//   a space is `%20` and `/` is `%2F` in its segment;
// - a query array with no `style` is `form` with `explode: true`: one `name=value` pair each;
// - a JSON body is the value's JSON with `Content-Type: application/json`;
// - a form body is `application/x-www-form-urlencoded`, its members by their Encoding Objects
//   (Stripe's `metadata` is `deepObject`: `metadata[plan]=pro`), a `+` sent as `%2B`;
// - each security scheme as the Security Scheme Object says (apiKey's `in` and `name`, RFC 6750
//   bearer, RFC 7617 basic of `user:password`).
// A query and a form body are compared decoded, pair by pair (`URLSearchParams`), in order within
// a name; a path is compared byte for byte, hex digits' case aside.

const b64 = (text) => Buffer.from(text, 'utf8').toString('base64')
const basic = (username, password) => ({ basic: { username, password } })

/** A JSON answer the mock gives. */
const json = (status, body, headers = {}) => ({ status, body, headers })

export const cases = [
  {
    name: 'shop',
    file: 'crates/flowapi-sdk/tests/client/shop/spec.json',
    about: 'a small description of the flowapi tests: two security schemes, a cursor list',
    credentials: { ApiKeyAuth: 'key-0' },
    requests: [
      { id: 'path', op: 'getPet', args: { path: { pet_id: 'a b/c' } }, expect: { method: 'GET', path: '/v1/pets/a%20b%2Fc' } },
      { id: 'query-array', op: 'listPets', args: { query: { tags: ['red', 'big dog'], limit: 2 } }, expect: { method: 'GET', path: '/v1/pets', query: [['tags', 'red'], ['tags', 'big dog'], ['limit', '2']] } },
      { id: 'json-body', op: 'createPet', args: { body: { id: 'p9', name: 'Rex & co' }, type: 'application/json' }, expect: { method: 'POST', path: '/v1/pets', json: { id: 'p9', name: 'Rex & co' } } },
    ],
    auth: [
      { id: 'apikey-header', scheme: 'ApiKeyAuth', value: 'key-1', op: 'getPet', args: { path: { pet_id: 'p1' } }, expect: { header: ['x-api-key', 'key-1'] } },
      { id: 'bearer', scheme: 'BearerAuth', value: 'tok-1', op: 'getPet', args: { path: { pet_id: 'p1' } }, expect: { header: ['authorization', 'Bearer tok-1'] } },
    ],
    paging: {
      op: 'listPets',
      args: { query: { limit: 2 } },
      pages: [json(200, { data: [{ id: 'p1', name: 'a' }, { id: 'p2', name: 'b' }], next_page_token: 'n2', has_more: true }), json(200, { data: [{ id: 'p3', name: 'c' }], next_page_token: null, has_more: false })],
      items: 3,
      next: ['page_token', 'n2'],
    },
    retry: { op: 'getPet', args: { path: { pet_id: 'p1' } } },
    error: { op: 'getPet', args: { path: { pet_id: 'gone' } }, answer: json(404, { message: 'no pet', code: 'missing' }) },
  },
  {
    name: 'auth',
    file: 'crates/flowapi-sdk/tests/client/auth/spec.json',
    about: 'a small description of the flowapi tests: an API key in a query and in a cookie, HTTP basic and bearer',
    credentials: {},
    requests: [],
    auth: [
      { id: 'apikey-query', scheme: 'QueryKey', value: 'key-1', op: 'byQuery', args: { query: { q: 'x' } }, expect: { query: ['api_key', 'key-1'] } },
      { id: 'apikey-cookie', scheme: 'CookieKey', value: 's-1', op: 'byCookie', args: {}, expect: { cookie: ['session', 's-1'] } },
      { id: 'basic', scheme: 'Basic', value: basic('ann', 'p:w'), op: 'byBasic', args: {}, expect: { header: ['authorization', `Basic ${b64('ann:p:w')}`] } },
      { id: 'bearer', scheme: 'Token', value: 'tok-1', op: 'byEither', args: {}, expect: { header: ['authorization', 'Bearer tok-1'] } },
    ],
  },
  {
    name: 'github',
    file: 'bench/data/github.json',
    about: "GitHub's REST API; it declares no security scheme, so flowapi's config declares a bearer token",
    config: { auth: { Token: { type: 'http', scheme: 'bearer', env_var: 'GITHUB_TOKEN' } }, security: [{ Token: [] }, {}] },
    credentials: {},
    requests: [
      { id: 'path', op: 'repos/get', args: { path: { owner: 'octo cat', repo: 'hello.world' } }, expect: { method: 'GET', path: '/repos/octo%20cat/hello.world' } },
      { id: 'query-array', op: 'orgs/list-pat-grant-requests', args: { path: { org: 'acme' }, query: { owner: ['octo cat', 'hubot'], per_page: 2 } }, expect: { method: 'GET', path: '/orgs/acme/personal-access-token-requests', query: [['owner', 'octo cat'], ['owner', 'hubot'], ['per_page', '2']] } },
      { id: 'json-body', op: 'issues/create', args: { path: { owner: 'o', repo: 'r' }, body: { title: 'Bug: a & b', labels: ['bug', 'p1'] }, type: 'application/json' }, expect: { method: 'POST', path: '/repos/o/r/issues', json: { title: 'Bug: a & b', labels: ['bug', 'p1'] } } },
    ],
    auth: [{ id: 'bearer', scheme: 'Token', value: 'ghp-1', op: 'repos/get', args: { path: { owner: 'o', repo: 'r' } }, expect: { header: ['authorization', 'Bearer ghp-1'] }, declared: true }],
    paging: {
      op: 'issues/list-for-repo',
      args: { path: { owner: 'o', repo: 'r' }, query: { per_page: 2 } },
      pages: [json(200, [{ id: 1 }, { id: 2 }], { link: '<https://api.github.com/repositories/1/issues?per_page=2&page=2>; rel="next", <https://api.github.com/repositories/1/issues?per_page=2&page=2>; rel="last"' }), json(200, [{ id: 3 }])],
      items: 3,
      next: ['page', '2'],
    },
    retry: { op: 'repos/get', args: { path: { owner: 'o', repo: 'r' } } },
    error: { op: 'repos/get', args: { path: { owner: 'o', repo: 'gone' } }, answer: json(404, { message: 'Not Found', documentation_url: 'https://docs.github.com/rest', status: '404' }) },
  },
  {
    name: 'stripe',
    file: 'bench/data/stripe.json',
    about: "Stripe's API: form bodies with deepObject members, HTTP basic or bearer, a list by the last item's id",
    credentials: { bearerAuth: 'sk_test_0' },
    requests: [
      { id: 'path', op: 'GetCustomersCustomer', args: { path: { customer: 'cus 1/x' } }, expect: { method: 'GET', path: '/v1/customers/cus%201%2Fx' } },
      { id: 'form-body', op: 'PostCustomers', args: { body: { email: 'a+b@c.example', name: 'A & B', metadata: { plan: 'pro' } }, type: 'application/x-www-form-urlencoded' }, expect: { method: 'POST', path: '/v1/customers', form: [['email', 'a+b@c.example'], ['name', 'A & B'], ['metadata[plan]', 'pro']] } },
    ],
    auth: [
      { id: 'basic', scheme: 'basicAuth', value: basic('sk_test_1', ''), op: 'GetCustomersCustomer', args: { path: { customer: 'cus_1' } }, expect: { header: ['authorization', `Basic ${b64('sk_test_1:')}`] } },
      { id: 'bearer', scheme: 'bearerAuth', value: 'sk_test_1', op: 'GetCustomersCustomer', args: { path: { customer: 'cus_1' } }, expect: { header: ['authorization', 'Bearer sk_test_1'] } },
    ],
    paging: {
      op: 'GetCustomers',
      args: { query: { limit: 2 } },
      pages: [json(200, { object: 'list', url: '/v1/customers', has_more: true, data: [{ id: 'cus_1' }, { id: 'cus_2' }] }), json(200, { object: 'list', url: '/v1/customers', has_more: false, data: [{ id: 'cus_3' }] })],
      items: 3,
      next: ['starting_after', 'cus_2'],
    },
    retry: { op: 'GetCustomersCustomer', args: { path: { customer: 'cus_1' } } },
    error: { op: 'GetCustomersCustomer', args: { path: { customer: 'gone' } }, answer: json(404, { error: { message: 'No such customer', type: 'invalid_request_error' } }) },
  },
  {
    name: 'twilio',
    file: 'bench/data/twilio.json',
    about: "Twilio's API (2010): form bodies, HTTP basic, a list with the next page's URI",
    credentials: { accountSid_authToken: basic('AC0', 't0') },
    requests: [
      { id: 'path', op: 'FetchCall', args: { path: { AccountSid: 'AC 1', Sid: 'CA/2' } }, expect: { method: 'GET', path: '/2010-04-01/Accounts/AC%201/Calls/CA%2F2.json' } },
      { id: 'form-body', op: 'CreateMessage', args: { path: { AccountSid: 'AC1' }, body: { To: '+15551234567', From: '+15557654321', Body: 'Hi & bye' }, type: 'application/x-www-form-urlencoded' }, expect: { method: 'POST', path: '/2010-04-01/Accounts/AC1/Messages.json', form: [['To', '+15551234567'], ['From', '+15557654321'], ['Body', 'Hi & bye']] } },
    ],
    auth: [{ id: 'basic', scheme: 'accountSid_authToken', value: basic('AC1', 'tok'), op: 'FetchCall', args: { path: { AccountSid: 'AC1', Sid: 'CA1' } }, expect: { header: ['authorization', `Basic ${b64('AC1:tok')}`] } }],
    paging: {
      op: 'ListMessage',
      args: { path: { AccountSid: 'AC1' }, query: { PageSize: 2 } },
      pages: [json(200, { messages: [{ sid: 'SM1' }, { sid: 'SM2' }], page: 0, page_size: 2, next_page_uri: '/2010-04-01/Accounts/AC1/Messages.json?PageSize=2&Page=1&PageToken=PASM2' }), json(200, { messages: [{ sid: 'SM3' }], page: 1, page_size: 2, next_page_uri: null })],
      items: 3,
      next: ['PageToken', 'PASM2'],
    },
    retry: { op: 'FetchCall', args: { path: { AccountSid: 'AC1', Sid: 'CA1' } } },
    error: { op: 'FetchCall', args: { path: { AccountSid: 'AC1', Sid: 'gone' } }, answer: json(404, { code: 20404, message: 'The requested resource was not found', status: 404 }) },
  },
]

/** The kind of each test, the feature table's rows, in order. */
export const features = [
  ['path', 'Path parameters encoded'],
  ['query-array', 'Query arrays'],
  ['json-body', 'JSON body'],
  ['form-body', 'Form body by its Encoding Objects'],
  ['apikey-header', 'Credential: API key in a header'],
  ['apikey-query', 'Credential: API key in a query'],
  ['apikey-cookie', 'Credential: API key in a cookie'],
  ['bearer', 'Credential: HTTP bearer'],
  ['basic', 'Credential: HTTP basic'],
  ['retry-429', 'Retry on 429 after Retry-After'],
  ['retry-503', 'Retry on 503 after Retry-After'],
  ['paging', 'Pagination to the last page'],
  ['error', 'An error as a value with its status and body'],
]
