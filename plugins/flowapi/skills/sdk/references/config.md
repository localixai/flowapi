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
| `auth.<scheme>.env_var` | The variable a scheme's credential is read from; `<scheme>` is its name in the description |
| `defaults.retries` | Retries of a failed request (1). A POST, a PATCH or another method that is not idempotent (RFC 9110 9.2.2) is retried only with an idempotency key, or when the server answers `x-should-retry: true`; a 403 never |
| `defaults.idempotency_key` | The header (`Idempotency-Key`) each call of a method that is not idempotent sends a random key in, the same on its retries, so that those calls are retried too; none unless given. Set it only when the API reads that header |
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
`retries`, `stream: true` (the response is read as server-sent events or JSON lines).

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
