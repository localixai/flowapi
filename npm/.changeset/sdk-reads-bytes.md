---
"@localix-ai/flowapi": minor
---

The TypeScript SDK reads binary responses as bytes, as the Rust SDK does. `<op>Response(status,
headers, body)` takes `body: string | Uint8Array` and reads it by the type sent: JSON parsed,
`text/*` a `string`, any other a `Uint8Array` (an `application/octet-stream` or `image/png` body
was a string before, and bytes that were not UTF-8 were lost on the way). The response union says
`Uint8Array` for such a type. Bytes given for JSON or text are read as UTF-8, and bytes that are
not UTF-8 there read as `response: null` with the body as given; a string given for a binary type
is encoded as UTF-8. `flowapiReadsBytes(contentType)` (exported) says which bodies are read as
bytes, so a client can read those with `arrayBuffer()` and the others with `text()`.
