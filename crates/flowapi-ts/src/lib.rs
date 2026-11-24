//! flowapi's TypeScript target: each declaration of a [`Program`] printed as an exported type
//! alias. Every type accepts every value its schema accepts, checked against the official JSON
//! Schema Test Suite under `strict` and `exactOptionalPropertyTypes` (`conformance/`).

mod object;
mod pattern;
mod print;

use flowapi::lower::Capabilities;
use flowapi::{File, IdentifierRules, Output, Program, Target};

/// The TypeScript target.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeScript {
    /// The path of the file written.
    pub path: String,
}

impl TypeScript {
    /// The target writing `path`.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// Names TypeScript keeps: its reserved words, and the names a type alias cannot take.
const RESERVED: &[&str] = &[
    "any",
    "unknown",
    "never",
    "number",
    "bigint",
    "boolean",
    "string",
    "symbol",
    "void",
    "object",
    "undefined",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "while",
    "with",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "await",
];

impl Target for TypeScript {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            intersections: true,
            unions: true,
        }
    }

    fn identifiers(&self) -> IdentifierRules {
        IdentifierRules { reserved: RESERVED }
    }

    fn emit(&self, program: &Program<'_>) -> Output {
        let text = print::program(program);
        Output {
            files: vec![File {
                path: self.path.clone(),
                text,
            }],
            diagnostics: flowapi::Diagnostics::default(),
        }
    }
}
