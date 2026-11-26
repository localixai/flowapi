//! The npm package's second WebAssembly module, `flowapi-sdk.wasm`: `flowapi-wasm` built with its
//! `sdk` feature, so it has every export of the first module and `flowapi_sdk`, the commands about
//! an SDK's config (`summary`, `draft`, `schema`, `spec`, `check`) and `generate`'s two halves
//! (`repo`, `plan`). The package loads it only when one of them is first used: a host that only
//! generates types never downloads it. The exports and all their code are `flowapi-wasm`'s; this
//! crate only links them into a module of its own.

pub use flowapi_wasm::{generate, references, sdk};
