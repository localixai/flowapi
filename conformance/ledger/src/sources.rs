//! The pinned specification texts the ledger reads (`conformance/specs/`, `SOURCE.md`).

/// How a text is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The OAI's Markdown.
    Markdown,
    /// An xml2rfc v2 source.
    Rfc,
}

/// One pinned text.
#[derive(Debug, Clone, Copy)]
pub struct Source {
    /// `openapi` or `json-schema`.
    pub spec: &'static str,
    /// The release: `3.1.2`, `2020-12`.
    pub version: &'static str,
    /// The path under `conformance/specs/`.
    pub file: &'static str,
    /// What every id from this text starts with.
    pub prefix: &'static str,
    /// How it is written.
    pub format: Format,
}

/// Every text, in the order the report lists them.
pub const SOURCES: [Source; 5] = [
    Source {
        spec: "openapi",
        version: "3.0.4",
        file: "openapi/3.0.4.md",
        prefix: "oas-3.0",
        format: Format::Markdown,
    },
    Source {
        spec: "openapi",
        version: "3.1.2",
        file: "openapi/3.1.2.md",
        prefix: "oas-3.1",
        format: Format::Markdown,
    },
    Source {
        spec: "openapi",
        version: "3.2.1",
        file: "openapi/3.2.1.md",
        prefix: "oas-3.2",
        format: Format::Markdown,
    },
    Source {
        spec: "json-schema",
        version: "2020-12",
        file: "json-schema-2020-12/jsonschema-core.xml",
        prefix: "jsonschema-2020-12.core",
        format: Format::Rfc,
    },
    Source {
        spec: "json-schema",
        version: "2020-12",
        file: "json-schema-2020-12/jsonschema-validation.xml",
        prefix: "jsonschema-2020-12.validation",
        format: Format::Rfc,
    },
];

/// The text a file under `conformance/specs/` is.
pub fn by_file(file: &str) -> Option<&'static Source> {
    SOURCES.iter().find(|source| source.file == file)
}
