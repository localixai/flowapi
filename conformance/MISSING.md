# Requirements missing a test

Written by `cargo run --release --bin report` in `conformance/ledger/`, with `REPORT.md`. Do not edit by hand.
Every requirement `scope.json` puts in scope that no `req:` or `lossy:` marker names yet, per text and
section in the text's order. After each id, the scope rule that classifies it: its `note` names the
layer that answers for it.

**1 missing.**

## openapi 2.0 — `openapi/2.0.md`: 1 missing

### Specification > Schema > Operation Object > Fixed Fields (`#operationSchemes`, line 422)

- `oas-2.0.operation-object.schemes.must-values-list-http-https-ws-wss` (MUST, by `oas-*.operation-object`): Values MUST be from the list: `"http"`, `"https"`, `"ws"`, `"wss"`.
