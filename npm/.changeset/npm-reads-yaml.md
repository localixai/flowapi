---
"@localix-ai/flowapi": minor
---

YAML descriptions are read, as the Rust command reads them and by the same code: `.yaml` and
`.yml` files, and under any other name a text that does not open with `{` or `[`, the YAML files a
`$ref` names too. A diagnostic names the YAML line and column, and a YAML text that JSON cannot
hold is an error with the code `yaml.syntax`, printed by the command as the Rust command prints
it. In the API a document's `uri` says whether its text is YAML; `references(text, uri)` takes it
too (without one the text is JSON, as before). The module is about 43 KB larger gzipped.
