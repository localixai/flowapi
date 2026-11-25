//! A description's text as flowapi's core reads it, shared by both hosts, the `flowapi` command
//! and the WebAssembly module of the npm package, so that both read YAML alike: which document is
//! YAML, the JSON it is read into, and the YAML line and column a diagnostic names (G8). The core
//! reads JSON only and has no dependency (G5, G11); this crate is where YAML is read, and it does
//! no I/O either: the host reads the file and gives its text.

mod reader;
mod text;

pub use reader::{Error, Json, MAX_ALIAS_BYTES, MAX_ALIAS_NODES, Marks, to_json};
pub use text::{CODE, Text, is_yaml};
