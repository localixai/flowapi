---
"@localix-ai/flowapi": patch
---

The SDK writes Swagger 2.0's `collectionFormat` in a path, a query and a header: `csv`, `ssv`,
`tsv` and `pipes` in each (`tsv` was written as `csv`), `multi` in a query, and an Items Object's
own `collectionFormat` for the arrays inside an array.
