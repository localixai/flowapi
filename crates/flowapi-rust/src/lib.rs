//! flowapi's Rust target: each declaration of a [`Program`] printed as a Rust item with serde
//! derives, in one file, `types.rs`. The generated code needs `serde` (with `derive`) and
//! `serde_json`, nothing else. Every type deserializes every value its schema accepts and
//! serializes it back equal, checked against the official JSON Schema Test Suite
//! (`conformance/suite-rust`).

mod cycles;
mod doc;
mod emit;
mod ident;
mod item;
mod print;
mod sdk;
mod shape;

use flowapi::lower::Capabilities;
use flowapi::{File, IdentifierRules, Output, Program, Target};

/// The Rust target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rust {
    /// The path of the file written.
    pub path: String,
    /// Whether the SDK follows the types: per operation a function that writes its request and
    /// one that reads its response, sending nothing (`sdk.rs`).
    pub sdk: bool,
}

impl Default for Rust {
    fn default() -> Self {
        Self::new("types.rs")
    }
}

impl Rust {
    /// The target writing `path`.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            sdk: false,
        }
    }

    /// The target with the SDK after the types.
    #[must_use]
    pub fn with_sdk(self) -> Self {
        Self { sdk: true, ..self }
    }
}

/// The names no item may take, and those the SDK adds: its runtime's module.
macro_rules! reserved {
    ($($name:literal,)*) => {
        /// Names no item may take: Rust's keywords, its primitive types, the prelude's types
        /// and variants the output uses, the crates it names, and the output's own helpers.
        pub(crate) const RESERVED: &[&str] = &[$($name,)*];
        /// With the SDK: its runtime's module, `sdk`, too.
        pub(crate) const RESERVED_SDK: &[&str] = &[$($name,)* "sdk"];
    };
}

reserved! {
    "bool",
    "char",
    "str",
    "f32",
    "f64",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "Option",
    "Some",
    "None",
    "Result",
    "Ok",
    "Err",
    "Box",
    "Vec",
    "String",
    "BTreeMap",
    "Serialize",
    "Deserialize",
    "Never",
    "deserialize_some",
    "Integer",
    "std",
    "core",
    "alloc",
    "serde",
    "serde_json",
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "async",
    "await",
    "dyn",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "try",
    "gen",
    "_",
}

impl Target for Rust {
    /// The lowering keeps intersections and unions whatever is asked; this target states a
    /// union as an untagged enum and merges an intersection of objects itself (else it is
    /// `serde_json::Value`, sound).
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            intersections: true,
            unions: true,
        }
    }

    fn identifiers(&self) -> IdentifierRules {
        IdentifierRules {
            reserved: if self.sdk { RESERVED_SDK } else { RESERVED },
        }
    }

    fn emit(&self, program: &Program<'_>) -> Output {
        Output {
            files: vec![File {
                path: self.path.clone(),
                text: print::program(program, self.sdk),
            }],
            diagnostics: flowapi::Diagnostics::default(),
        }
    }
}
