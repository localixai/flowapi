# Writing a request: the Rust SDK against progenitor's way

Written by `cargo run --release -j 1` in `bench/sdk-rust`; do not edit. 100000 requests a round, the median of 5 rounds after a warm-up, nanoseconds a request, one thread on a machine of 4 cores. flowapi's is the whole `<op>_request` call into its `sdk::Request`; progenitor's is its `encode_path`, its `QueryParam` pairs through serde_urlencoded and serde_json for the body, into the same four parts (src/baseline.rs). Both mean the same request (checked first: path, query pairs and body decoded alike).

| Shape | flowapi (ns) | progenitor's way (ns) | Ratio |
| --- | ---: | ---: | ---: |
| getRepo: path and a query flag | 134 | 306 | 2.28x |
| search: an array, a deepObject, an integer | 360 | 696 | 1.93x |
| putPet: path, query, header, JSON body | 282 | 314 | 1.12x |
