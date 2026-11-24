# Writing a request: the flowapi SDK against openapi-fetch

Written by `node bench/competitors/sdk.mjs`; do not edit. 2026-11-24, Node.js v22.23.5, Intel(R) N150, 4 cores, 16.5 GB; openapi-fetch 0.17.0.

100,000 requests a round, one process, one shape after another; the median of 5 rounds after 3 warm-up rounds. flowapi: the whole `<op>Request` call (path, query, headers, body). openapi-fetch: `createFinalURL` with a query serializer made once and `defaultPathSerializer`, and `defaultBodySerializer` for the body; its `Request` and `Headers` are not made. Each pair is checked first to write the same request.

| Shape | flowapi | per request | openapi-fetch | per request | flowapi faster by |
| --- | ---: | ---: | ---: | ---: | ---: |
| path and a flag | 44.6 ms | 446 ns | 158.3 ms | 1583 ns | 3.6x |
| arrays and an object in the query | 85.4 ms | 854 ns | 134.4 ms | 1344 ns | 1.6x |
| path, query, header and a JSON body | 55.5 ms | 555 ns | 135.7 ms | 1357 ns | 2.4x |
