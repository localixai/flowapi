//! JSON (RFC 8259) read into a flat tree that keeps the byte offset of every node, so a
//! diagnostic can say where (G8). Strings and numbers are spans of the text, decoded only when
//! read: a document is never copied (G6). Nothing here knows OpenAPI or JSON Schema.

mod parse;
mod text;
mod tree;

pub use parse::{ParseError, parse};
pub use tree::{Member, Span, Tree, Value};
