# `flowapi-sdk.yml`: the catalogue

Version 1 of the format, as `flowapi config schema` prints it; that schema is the authority. A key
not listed here is an error.

## Top level

| Key | Meaning |
| --- | --- |
| `version` | The format's version: `1` |
| `generator.flowapi` | The flowapi version the SDK is generated with (`0.2.0`, or `0.2.x` for any patch) |
| `spec` | The description: a path relative to the config, or a URL |
| `package.name` | The npm package's name (`@acme/sdk`) |
| `package.version` | The package's version, `0.1.0` unless given; the release tool raises it |
| `package.client` | The client class, in PascalCase (`Acme`) |
| `package.license` | An SPDX expression; MIT and Apache-2.0 get their text written |
| `package.repository` | The repository URL, for `package.json` |
| `package.description` | `package.json`'s description, plain text, and the README's first paragraph unless `readme.intro` is given. Default: `TypeScript SDK for the <title> API.` and the first short sentence of `info.summary` or `info.description` |
| `package.keywords` | `package.json`'s keywords, a list (`[pets, shop]`); none unless given |
| `environments.<name>` | A base URL by name |
| `environments.default` | The environment used when the caller names none |
| `environments.env_var` | The variable that overrides the base URL (`ACME_BASE_URL`) |
| `auth.<scheme>` | A security scheme the client sends: one of the description's by its name, given options only, or one the description lacks, declared with `type` (below) |
| `auth.<scheme>.env_var` | The variable the credential is read from: the key or the token; HTTP basic's `username:password` |
| `auth.<scheme>.username_env_var`, `password_env_var` | HTTP basic: the user-id and the password from two variables, given together |
| `auth.<scheme>.client_id_env_var`, `client_secret_env_var` | OAuth 2.0 with a client credentials flow: the client requests its own token with these, given together |
| `auth.<scheme>.scopes` | That flow's scopes for every token, instead of each operation's |
| `auth.<scheme>.description` | What the scheme is, for the README |
| `security` | The security of every operation with none of its own, over the description's: `[{ Token: [] }, {}]` is a token, optional; `[]` is none, said on purpose |
| `defaults.retries` | Retries of a failed request (2) |
| `defaults.timeout` | Seconds a request may take, its body read (60) |
| `defaults.max_retry_delay` | The longest wait in seconds a server may ask for before a retry; longer is an error, not a sleep |
| `defaults.pagination` | The scheme of every method named `list` that names none |
| `pagination.<name>` | A pagination scheme, by a name methods refer to |
| `resources.<name>.methods.<name>` | A method: an `operationId`, `"METHOD /path"`, or an object below |
| `resources.<name>.resources` | Resources nested under it |
| `skip` | `[{ op, reason }]`: operations the SDK leaves out |
| `readme.title` | The README's heading. Default: `info.title` without a last `API`, then `TypeScript SDK` (`Twilio Studio API` is `Twilio Studio TypeScript SDK`); the package's name without a title |
| `readme.intro` | The README's first paragraph, Markdown; `package.description` unless given |
| `readme.headline_method` | The method the README shows first, as `resource.method` |
| `readme.pagination_method` | The method the README shows paging with |

A method as an object: `op` (the operation), `pagination` (a scheme's name, or `false`),
`retries`, `stream: true` (the response is read as server-sent events or JSON lines), `security`
(the method's own, over everything; `[]` for none).

## Security schemes

A scheme the description declares takes options only (`env_var`, ...); saying its `type` again
is an error (`config.auth.declared`). One it lacks is declared with `type` and the members of its
kind, named as OpenAPI's Security Scheme Object names them:

| `type` | Members | The client sends |
| --- | --- | --- |
| `apiKey` | `in` (`header`, `query`, `cookie`), `name` | The key in that header, query parameter or cookie |
| `http` | `scheme`: `bearer`, `basic`, or another of IANA's registry | `Authorization: Bearer <token>`, `Basic <base64 of user:password>`, `<scheme> <credential>`; a scheme that answers a challenge (`digest`, `negotiate`, ...) is not sent |
| `oauth2` | `token_url` for the client credentials flow, or none for a token the caller gives | `Authorization: Bearer <token>`, requested at `token_url` with the client's id and secret |
| `mutualTLS` | none | Nothing: the caller's `fetch` presents the certificate, and the option is `true` |

An alternative of `security` lists schemes needed together; the client sends the first
alternative it holds every credential of. Credentials go only over `https:` or to a loopback
host unless the client is given `allowInsecureCredentials: true`.

```yaml
auth:
  Token: { type: http, scheme: bearer, env_var: ACME_TOKEN, description: A personal access token }
  Admin: { type: http, scheme: basic, username_env_var: ACME_USER, password_env_var: ACME_PASSWORD }
  Service:
    type: oauth2
    token_url: https://login.acme.com/oauth/token
    client_id_env_var: ACME_CLIENT_ID
    client_secret_env_var: ACME_CLIENT_SECRET
    scopes: [projects.read]
security:
  - Token: []
  - {}
```

## Pagination schemes

Each has `type`, `request` (parameter names) and `response` (paths in the body, `data.items`; the
body itself when `items` is not given).

| `type` | `request` | `response` |
| --- | --- | --- |
| `cursor` | `cursor`, `limit` | `items`, `next` (the token), `has_more` |
| `last_item_id` | `cursor`, `limit` | `items`, `id` (the item member the next page starts after), `has_more` |
| `offset` | `offset`, `limit` | `items`, `total` |
| `page` | `page`, `per_page`; `start` (first page number) | `items`, `total_pages`, `has_more` |
| `link` | `limit` | `items`; `next` (a URL in the body) or `header` (`Link` with `rel="next"`) |
| `single` | none | `items`: one page, read as a list |

## Example

```yaml
version: 1
generator: { flowapi: "0.2.0" }
spec: openapi.json
package:
  name: "@acme/sdk"
  client: Acme
  license: MIT
  description: Unofficial TypeScript SDK for the Acme API.
  keywords: [acme, projects]
environments:
  production: https://api.acme.com
  sandbox: https://sandbox.acme.com
  default: production
  env_var: ACME_BASE_URL
auth:
  BearerAuth: { env_var: ACME_API_KEY }
pagination:
  cursor:
    type: cursor
    request: { cursor: after, limit: limit }
    response: { items: data, next: next_cursor }
resources:
  projects:
    methods:
      list: { op: listProjects, pagination: cursor }
      create: createProject
      retrieve: getProject
      archive: "POST /projects/{project_id}/archive"
    resources:
      members:
        methods:
          list: { op: listMembers, pagination: cursor }
          remove: { op: removeMember, retries: 0 }
skip:
  - { op: "GET /internal/health", reason: not public }
readme:
  title: Acme TypeScript SDK (unofficial)
  headline_method: projects.list
```
