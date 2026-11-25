---
"@localix-ai/flowapi": minor
---

The TypeScript SDK writes `multipart/form-data` request bodies (it took a string you had encoded).
`<op>Request` takes the body's value as its schema says, a string member also as a file
(`Uint8Array`, `flowapiFile(bytes or text, name, type)`, or `await flowapiBlob(blobOrFile)`), and
returns the parts as bytes with `content-type: multipart/form-data; boundary=...`: files as file
parts (the name's last path segment, `blob` when none), strings, numbers and booleans as text, objects
as JSON, an array's items one part each under its name in order. The body's `encoding` (the
Encoding Object's `contentType` and `headers`, given by you: the description's is not read yet)
sets a part's type and headers. The boundary is the content type's, or 128 random bits from
`globalThis.crypto`. A request body neither JSON, text nor a form may be given as a `Uint8Array`,
and `$Request.body` is `string | Uint8Array`. `<op>Request` refuses names, file names, types and
headers that would change the parts (a CR, LF or NUL, a quote, `..`), `null`, a Blob not read yet
and a boundary a part holds, with new `FlowapiRequestError` reasons. A schema named `Promise`,
`ArrayBuffer`, `FlowapiRequestError`, `FlowapiFile` or `FlowapiEncoding` is declared under another
name with the SDK, with a warning.
