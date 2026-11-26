# Writing a request: the flowapi SDK against openapi-fetch

Written by `node bench/competitors/sdk.mjs`; do not edit. 2026-11-26, Node.js v22.23.5, Intel(R) N150, 4 cores, 16.5 GB; openapi-fetch 0.17.0.

100,000 requests a round, one process, one shape after another; the median of 5 rounds after 3 warm-up rounds. flowapi: the whole `<op>Request` call (path, query, headers, body). openapi-fetch: `createFinalURL` with a query serializer made once and `defaultPathSerializer`, and `defaultBodySerializer` for the body; its `Request` and `Headers` are not made. Each pair is checked first to write the same request.

| Shape | flowapi | per request | openapi-fetch | per request | flowapi faster by |
| --- | ---: | ---: | ---: | ---: | ---: |
| path and a flag | 43.3 ms | 433 ns | 137.2 ms | 1372 ns | 3.2x |
| arrays and an object in the query | 82.3 ms | 823 ns | 122.4 ms | 1224 ns | 1.5x |
| path, query, header and a JSON body | 54.4 ms | 544 ns | 122.2 ms | 1222 ns | 2.2x |
