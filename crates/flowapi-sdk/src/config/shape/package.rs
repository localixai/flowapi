//! What the package and its README say of themselves: `package` (its npm name, version, class,
//! license, description and keywords) and `readme` (its title, opening paragraph and the methods
//! it shows). Part of `shape.rs`, by which the checker, its reader and the JSON Schema all go.

use super::{Field, Shape, Str, TEXT, f, req};

static KEYWORDS: Shape = Shape::List(&TEXT);

pub(crate) const PACKAGE: &[Field] = &[
    req("name", &Shape::Str(Str::Package), "The npm package's name"),
    f(
        "version",
        &Shape::Str(Str::Release),
        "The package's version (0.1.0 unless given); the release tool raises it",
    ),
    req(
        "client",
        &Shape::Str(Str::Client),
        "The client class's name, in PascalCase",
    ),
    f(
        "license",
        &TEXT,
        "The package's license, an SPDX expression",
    ),
    f("repository", &TEXT, "The package's repository URL"),
    f(
        "description",
        &TEXT,
        "One or two sentences of plain text: `package.json`'s `description` and, without \
         `readme.intro`, the README's first paragraph. Unless given, made from the description's \
         `info.title` and the first sentence of its `info.summary` or `info.description`",
    ),
    f(
        "keywords",
        &KEYWORDS,
        "`package.json`'s `keywords`, for npm's search; none unless given",
    ),
];

pub(crate) const README: &[Field] = &[
    f(
        "title",
        &TEXT,
        "The README's heading. Unless given, the description's `info.title` without a last \
         `API`, then `TypeScript SDK` (`Twilio Studio API` is `Twilio Studio TypeScript SDK`); \
         the package's name when the description has no title",
    ),
    f(
        "intro",
        &TEXT,
        "The README's first paragraph, Markdown; `package.description` unless given",
    ),
    f(
        "headline_method",
        &TEXT,
        "The method the README shows first, as `resource.method`",
    ),
    f(
        "pagination_method",
        &TEXT,
        "The method the README shows pagination with",
    ),
];
