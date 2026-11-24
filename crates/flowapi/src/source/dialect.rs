//! What a document is: the OpenAPI version it states, or the JSON Schema draft its `$schema`
//! names. The schema dialect decides which keywords a Schema Object takes.

use super::DocKind;
use crate::json::Tree;

/// An OpenAPI version this build reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    /// 3.0.x: Schema Objects are the extended subset of JSON Schema Wright draft 00.
    V30,
    /// 3.1.x: Schema Objects are JSON Schema 2020-12 with the OpenAPI vocabulary.
    V31,
    /// 3.2.x: as 3.1, with `$self` and the new Discriminator and XML fields.
    V32,
}

/// The rules a Schema Object is read by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dialect {
    /// OpenAPI 3.0's Schema Object: `nullable`, `$ref` with its siblings ignored, no `const`,
    /// no type arrays.
    Oas30,
    /// JSON Schema draft 4.
    Draft4,
    /// JSON Schema draft 6.
    Draft6,
    /// JSON Schema draft 7.
    Draft7,
    /// JSON Schema 2019-09.
    Draft2019,
    /// JSON Schema 2020-12, and OpenAPI 3.1's and 3.2's default dialect.
    Draft2020,
    /// JSON Schema 2020-12 under a meta-schema whose `$vocabulary` leaves validation out: `type`,
    /// `enum`, `required` and the other validation keywords are annotations only.
    Draft2020Annotations,
}

impl Dialect {
    /// The dialect a `$schema` URI names; `None` for one this build does not know.
    #[must_use]
    pub fn from_uri(uri: &str) -> Option<Self> {
        let uri = uri.trim_end_matches('#');
        let uri = uri
            .strip_prefix("http://")
            .or_else(|| uri.strip_prefix("https://"))
            .unwrap_or(uri);
        match uri {
            "json-schema.org/draft/2020-12/schema"
            | "spec.openapis.org/oas/3.1/dialect/base"
            | "spec.openapis.org/oas/3.1/dialect/2024-11-10"
            | "spec.openapis.org/oas/3.2/dialect/2025-09-17" => Some(Self::Draft2020),
            "json-schema.org/draft/2019-09/schema" => Some(Self::Draft2019),
            "json-schema.org/draft-07/schema" => Some(Self::Draft7),
            "json-schema.org/draft-06/schema" => Some(Self::Draft6),
            "json-schema.org/draft-04/schema" => Some(Self::Draft4),
            _ => uri
                .starts_with("spec.openapis.org/oas/3.")
                .then_some(Self::Draft2020),
        }
    }

    /// Whether `items` may be an array (a tuple) and `additionalItems` applies: before 2020-12.
    #[must_use]
    pub fn tuple_items(self) -> bool {
        matches!(
            self,
            Self::Draft4 | Self::Draft6 | Self::Draft7 | Self::Draft2019
        )
    }

    /// Whether a `$ref`'s sibling keywords are ignored: OpenAPI 3.0 and drafts before 2019-09.
    #[must_use]
    pub fn ref_overrides(self) -> bool {
        matches!(
            self,
            Self::Oas30 | Self::Draft4 | Self::Draft6 | Self::Draft7
        )
    }

    /// The keyword naming a schema resource: `id` in draft 4, `$id` after.
    #[must_use]
    pub fn id_keyword(self) -> &'static str {
        if self == Self::Draft4 { "id" } else { "$id" }
    }
}

/// What a tree is, by its root's `openapi`, `swagger` or `$schema`.
pub(super) fn detect(tree: &Tree<'_>) -> Result<DocKind, String> {
    let root = tree.root();
    if let Some(v) = tree.get(root, "openapi").and_then(|n| tree.as_str(n)) {
        return match v.split('.').take(2).collect::<Vec<_>>().as_slice() {
            ["3", "0"] => Ok(DocKind::OpenApi(Version::V30)),
            ["3", "1"] => Ok(DocKind::OpenApi(Version::V31)),
            ["3", "2"] => Ok(DocKind::OpenApi(Version::V32)),
            _ => Err(format!(
                "OpenAPI {v} is not read by this version (3.0, 3.1 and 3.2 are)"
            )),
        };
    }
    if tree.get(root, "swagger").is_some() {
        return Err(String::from(
            "Swagger 2.0 is not read yet (OpenAPI 3.0, 3.1 and 3.2 are)",
        ));
    }
    let named = tree.get(root, "$schema").and_then(|n| tree.as_str(n));
    Ok(DocKind::Schema(
        named
            .and_then(|uri| Dialect::from_uri(&uri))
            .unwrap_or(Dialect::Draft2020),
    ))
}
