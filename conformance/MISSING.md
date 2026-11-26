# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**3 missing.**

## openapi 2.0 — `openapi/2.0.md`: 3 missing

### Specification > Schema > Operation Object > Fixed Fields (`#operationSchemes`, line 422)

- `oas-2.0.operation-object.schemes.must-values-list-http-https-ws-wss` (MUST, by `oas-*.operation-object`): Values MUST be from the list: `"http"`, `"https"`, `"ws"`, `"wss"`.

### Specification > Schema > Parameter Object > Fixed Fields (`#parameterCollectionFormat`, line 598)

- `oas-2.0.parameter-object.collectionFormat` (field, by `oas-*.parameter-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. `multi` - corresponds to multiple parameter instances instead of multiple values for a single instance `foo=bar&foo=baz`. This is valid only for parameters `in` "query" or "formData". Default value is `csv`.

### Specification > Schema > Items Object > Fixed Fields (`#itemsCollectionFormat`, line 778)

- `oas-2.0.items-object.collectionFormat` (field, by `oas-2.0.items-object`): Determines the format of the array if type array is used. Possible values are: `csv` - comma separated values `foo,bar`. `ssv` - space separated values `foo bar`. `tsv` - tab separated values `foo\tbar`. `pipes` - pipe separated values foo&#124;bar. Default value is `csv`.
