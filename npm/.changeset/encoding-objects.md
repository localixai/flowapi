---
"@localix-ai/flowapi": minor
---

The SDK writes request bodies by the description's Encoding Objects: by name, by position
(3.2's `prefixEncoding` and `itemEncoding`) and nested, with `style`, `explode` and
`allowReserved` for form members, and Swagger 2.0's `formData` `collectionFormat`. A form body
written from its content is percent-encoded by WHATWG's rules (a space as `+`), as OpenAPI 3.2
asks.
