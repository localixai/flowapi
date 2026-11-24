# The Rust SDK on large descriptions

Written by `cargo run --release -j 1 --bin corpus` in `conformance/sdk-rust`; do not edit. Generation only (types and SDK, `Rust::with_sdk`, one run on this machine); the output of the three largest is not compiled (rustc needs more than 16 GB for GitHub's types).

| Description | Generation (ms) | Bytes | Items | Operations | Errors |
| --- | ---: | ---: | ---: | ---: | ---: |
| github | 497 | 14804825 | 25250 | 1281 | 0 |
| stripe | 262 | 10902620 | 18061 | 612 | 0 |
| cloudflare | 1437 | 39462230 | 75597 | 3935 | 0 |
| slack | 26 | 1435963 | 2857 | 174 | 0 |

Slack's output built (`cargo build -j 1`, debug, from clean, serde and serde_json included): built, 0:49.02 wall clock, peak resident memory 2684 MB, 0 warnings.
