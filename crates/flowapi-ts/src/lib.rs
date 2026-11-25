//! flowapi's TypeScript target: each declaration of a [`Program`] printed as an exported type
//! alias. Every type accepts every value its schema accepts, checked against the official JSON
//! Schema Test Suite under `strict` and `exactOptionalPropertyTypes` (`conformance/`).

mod object;
mod pattern;
mod print;
mod sdk;

/// What the SDK decides that a layer above the target prints by (`flowapi-sdk`): its functions'
/// names, each request's method, how a body is read, a TypeScript string literal.
pub use sdk::{ReadAs, bases as sdk_bases, lit as string_literal, method as sdk_method};

use flowapi::lower::Capabilities;
use flowapi::{File, IdentifierRules, Output, Program, Target};

/// The TypeScript target.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeScript {
    /// The path of the file written.
    pub path: String,
    /// Whether the file holds an SDK after the types: for each operation a function that writes
    /// its request and one that reads its response, sending nothing (`sdk.rs`).
    pub sdk: bool,
}

impl TypeScript {
    /// The target writing `path`, types only.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            sdk: false,
        }
    }

    /// The same target, writing the SDK after the types.
    #[must_use]
    pub fn with_sdk(self) -> Self {
        Self { sdk: true, ..self }
    }
}

/// Names TypeScript keeps, its reserved words and the names a type alias cannot take, and the
/// global types the output refers to (`Record`, `Object`), which a declaration of that name
/// would shadow; with the SDK, also those its functions and runtime name as types and the names
/// it exports (`FlowapiRequestError`, `FlowapiFile`, `FlowapiEncoding`). A schema of
/// one of these names is declared under another, with a warning.
macro_rules! reserved {
    ($($word:literal,)*) => {
        const RESERVED: &[&str] = &[$($word,)* "Record", "Object"];
        const RESERVED_SDK: &[&str] = &[
            $($word,)* "Record", "Object", "NonNullable", "ReturnType", "RegExpExecArray",
            "Uint8Array", "ArrayBuffer", "Promise", "FlowapiRequestError", "FlowapiFile",
            "FlowapiEncoding",
        ];
    };
}

reserved! {
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
}

impl Target for TypeScript {
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
        let mut text = print::program(program);
        if self.sdk {
            sdk::sdk(&mut text, program);
        }
        Output {
            files: vec![File {
                path: self.path.clone(),
                text,
            }],
            diagnostics: flowapi::Diagnostics::default(),
        }
    }
}
