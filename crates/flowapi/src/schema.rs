//! Schema Objects read keyword by keyword. Every keyword of JSON Schema 2020-12 (and the drafts
//! before it), of the OpenAPI vocabulary and of OpenAPI 3.0 is a variant of [`Keyword`]; the
//! graph matches it without a `_` arm, so a keyword is either given a type, recorded as
//! something the target cannot express, or named as an annotation, never dropped (G2).

mod index;
mod keyword;
mod sub;

pub use index::{Index, Resource, Target, scoped};
pub use keyword::Keyword;
pub use sub::subschemas;
