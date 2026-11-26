---
"@localix-ai/flowapi": minor
---

The generated client follows RFC 9110 on retries and redirects and the OpenAPI text on servers. A
POST, a PATCH or another method that is not idempotent is retried only when the server answers
`x-should-retry: true`, or when `defaults.idempotency_key` in `flowapi-sdk.yml` names a header:
then each call sends a random key in it, the same on its retries. A 403 is never retried, and the
default `retries` is 1. Every 3xx with a `Location` is followed except 304, 305 and 306; a
redirect loop is refused at once, and a `Location` with no fragment keeps the request's. Server
variables take the caller's `serverVariables` (checked against each `enum`), a relative server URL
is resolved against `descriptionURL`, and a URL with a user name or password is never sent.
