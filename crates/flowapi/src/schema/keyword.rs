//! The keywords: each key of a Schema Object read as one, in its dialect.

mod name;
mod since;

use crate::source::Dialect;

/// A keyword of a Schema Object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyword {
    // Core (2020-12 §8).
    /// `$schema`.
    Schema,
    /// `$id` (`id` in draft 4).
    Id,
    /// `$ref`.
    Ref,
    /// `$anchor`.
    Anchor,
    /// `$dynamicRef`.
    DynamicRef,
    /// `$dynamicAnchor`.
    DynamicAnchor,
    /// `$recursiveRef` (2019-09).
    RecursiveRef,
    /// `$recursiveAnchor` (2019-09).
    RecursiveAnchor,
    /// `$vocabulary`.
    Vocabulary,
    /// `$comment`.
    Comment,
    /// `$defs`, and `definitions` before 2019-09.
    Defs,
    // Applicators (2020-12 §10).
    /// `allOf`.
    AllOf,
    /// `anyOf`.
    AnyOf,
    /// `oneOf`.
    OneOf,
    /// `not`.
    Not,
    /// `if`.
    If,
    /// `then`.
    Then,
    /// `else`.
    Else,
    /// `dependentSchemas`.
    DependentSchemas,
    /// `dependencies` (before 2019-09: schemas and required lists in one).
    Dependencies,
    /// `prefixItems`.
    PrefixItems,
    /// `items`.
    Items,
    /// `additionalItems` (before 2020-12).
    AdditionalItems,
    /// `contains`.
    Contains,
    /// `properties`.
    Properties,
    /// `patternProperties`.
    PatternProperties,
    /// `additionalProperties`.
    AdditionalProperties,
    /// `propertyNames`.
    PropertyNames,
    /// `unevaluatedItems`.
    UnevaluatedItems,
    /// `unevaluatedProperties`.
    UnevaluatedProperties,
    // Validation (2020-12 validation §6).
    /// `type`.
    Type,
    /// `enum`.
    Enum,
    /// `const`.
    Const,
    /// `multipleOf`.
    MultipleOf,
    /// `maximum`.
    Maximum,
    /// `exclusiveMaximum`.
    ExclusiveMaximum,
    /// `minimum`.
    Minimum,
    /// `exclusiveMinimum`.
    ExclusiveMinimum,
    /// `maxLength`.
    MaxLength,
    /// `minLength`.
    MinLength,
    /// `pattern`.
    Pattern,
    /// `maxItems`.
    MaxItems,
    /// `minItems`.
    MinItems,
    /// `uniqueItems`.
    UniqueItems,
    /// `maxContains`.
    MaxContains,
    /// `minContains`.
    MinContains,
    /// `maxProperties`.
    MaxProperties,
    /// `minProperties`.
    MinProperties,
    /// `required`.
    Required,
    /// `dependentRequired`.
    DependentRequired,
    /// `format`.
    Format,
    /// `contentEncoding`.
    ContentEncoding,
    /// `contentMediaType`.
    ContentMediaType,
    /// `contentSchema`.
    ContentSchema,
    // Meta-data (2020-12 validation §9).
    /// `title`.
    Title,
    /// `description`.
    Description,
    /// `default`.
    Default,
    /// `deprecated`.
    Deprecated,
    /// `readOnly`.
    ReadOnly,
    /// `writeOnly`.
    WriteOnly,
    /// `examples`.
    Examples,
    // The OpenAPI vocabulary (3.1, 3.2) and 3.0's Schema Object.
    /// `discriminator`.
    Discriminator,
    /// `xml`.
    Xml,
    /// `externalDocs`.
    ExternalDocs,
    /// `example`.
    Example,
    /// `nullable` (OpenAPI 3.0).
    Nullable,
    /// A specification extension, `x-…`.
    Extension,
    /// A key no vocabulary defines: an annotation with no meaning (2020-12 §6.5).
    Unknown,
}

impl Keyword {
    /// The keyword a key is in a dialect.
    #[must_use]
    pub fn of(key: &str, dialect: Dialect) -> Self {
        let keyword = Self::any_of(key, dialect);
        if dialect == Dialect::Draft2020Annotations && keyword.validates() {
            return Self::Unknown;
        }
        // A keyword a later draft brought is an unknown word, so nothing, in an earlier one.
        let since = keyword.since();
        if dialect >= Dialect::Draft4 && dialect < since {
            return Self::Unknown;
        }
        let listed = match dialect {
            Dialect::Oas30 => keyword.in_oas30(),
            Dialect::Swagger20 => keyword.in_swagger20(),
            Dialect::Draft4
            | Dialect::Draft6
            | Dialect::Draft7
            | Dialect::Draft2019
            | Dialect::Draft2020
            | Dialect::Draft2020Annotations => true,
        };
        if !listed {
            // Swagger 2.0's and OpenAPI 3.0's Schema Objects take a fixed list; any other key
            // means nothing there.
            return if key.starts_with("x-") {
                Self::Extension
            } else {
                Self::Unknown
            };
        }
        keyword
    }

    /// Whether the keyword is of the validation vocabulary (2020-12 validation §6).
    fn validates(self) -> bool {
        matches!(
            self,
            Self::Type
                | Self::Enum
                | Self::Const
                | Self::MultipleOf
                | Self::Maximum
                | Self::ExclusiveMaximum
                | Self::Minimum
                | Self::ExclusiveMinimum
                | Self::MaxLength
                | Self::MinLength
                | Self::Pattern
                | Self::MaxItems
                | Self::MinItems
                | Self::UniqueItems
                | Self::MaxContains
                | Self::MinContains
                | Self::MaxProperties
                | Self::MinProperties
                | Self::Required
                | Self::DependentRequired
        )
    }

    /// Whether OpenAPI 3.0's Schema Object has the keyword (3.0.4, "Schema Object").
    fn in_oas30(self) -> bool {
        matches!(
            self,
            Self::Ref
                | Self::AllOf
                | Self::AnyOf
                | Self::OneOf
                | Self::Not
                | Self::Items
                | Self::Properties
                | Self::AdditionalProperties
                | Self::Type
                | Self::Enum
                | Self::MultipleOf
                | Self::Maximum
                | Self::ExclusiveMaximum
                | Self::Minimum
                | Self::ExclusiveMinimum
                | Self::MaxLength
                | Self::MinLength
                | Self::Pattern
                | Self::MaxItems
                | Self::MinItems
                | Self::UniqueItems
                | Self::MaxProperties
                | Self::MinProperties
                | Self::Required
                | Self::Format
                | Self::Title
                | Self::Description
                | Self::Default
                | Self::Deprecated
                | Self::ReadOnly
                | Self::WriteOnly
                | Self::Discriminator
                | Self::Xml
                | Self::ExternalDocs
                | Self::Example
                | Self::Nullable
                | Self::Extension
                | Self::Unknown
        )
    }

    /// Whether Swagger 2.0's Schema Object has the keyword (2.0, "Schema Object"): 3.0's list
    /// without `anyOf`, `oneOf`, `not`, `deprecated`, `writeOnly` and `nullable`, which came
    /// in 3.0; `Nullable` stands for the `x-nullable` extension here.
    fn in_swagger20(self) -> bool {
        self.in_oas30()
            && !matches!(
                self,
                Self::AnyOf | Self::OneOf | Self::Not | Self::Deprecated | Self::WriteOnly
            )
    }

    /// The keyword a key is in a dialect, before Swagger 2.0's or OpenAPI 3.0's list is applied.
    fn any_of(key: &str, dialect: Dialect) -> Self {
        match key {
            "$schema" => Self::Schema,
            "$id" if dialect != Dialect::Draft4 => Self::Id,
            "id" if dialect == Dialect::Draft4 => Self::Id,
            "$ref" => Self::Ref,
            "$anchor" => Self::Anchor,
            "$dynamicRef" => Self::DynamicRef,
            "$dynamicAnchor" => Self::DynamicAnchor,
            "$recursiveRef" => Self::RecursiveRef,
            "$recursiveAnchor" => Self::RecursiveAnchor,
            "$vocabulary" => Self::Vocabulary,
            "$comment" => Self::Comment,
            "$defs" | "definitions" => Self::Defs,
            "allOf" => Self::AllOf,
            "anyOf" => Self::AnyOf,
            "oneOf" => Self::OneOf,
            "not" => Self::Not,
            "if" => Self::If,
            "then" => Self::Then,
            "else" => Self::Else,
            "dependentSchemas" => Self::DependentSchemas,
            "dependencies" => Self::Dependencies,
            "prefixItems" => Self::PrefixItems,
            "items" => Self::Items,
            "additionalItems" => Self::AdditionalItems,
            "contains" => Self::Contains,
            "properties" => Self::Properties,
            "patternProperties" => Self::PatternProperties,
            "additionalProperties" => Self::AdditionalProperties,
            "propertyNames" => Self::PropertyNames,
            "unevaluatedItems" => Self::UnevaluatedItems,
            "unevaluatedProperties" => Self::UnevaluatedProperties,
            "type" => Self::Type,
            "enum" => Self::Enum,
            "const" => Self::Const,
            "multipleOf" => Self::MultipleOf,
            "maximum" => Self::Maximum,
            "exclusiveMaximum" => Self::ExclusiveMaximum,
            "minimum" => Self::Minimum,
            "exclusiveMinimum" => Self::ExclusiveMinimum,
            "maxLength" => Self::MaxLength,
            "minLength" => Self::MinLength,
            "pattern" => Self::Pattern,
            "maxItems" => Self::MaxItems,
            "minItems" => Self::MinItems,
            "uniqueItems" => Self::UniqueItems,
            "maxContains" => Self::MaxContains,
            "minContains" => Self::MinContains,
            "maxProperties" => Self::MaxProperties,
            "minProperties" => Self::MinProperties,
            "required" => Self::Required,
            "dependentRequired" => Self::DependentRequired,
            "format" => Self::Format,
            "contentEncoding" => Self::ContentEncoding,
            "contentMediaType" => Self::ContentMediaType,
            "contentSchema" => Self::ContentSchema,
            "title" => Self::Title,
            "description" => Self::Description,
            "default" => Self::Default,
            "deprecated" => Self::Deprecated,
            "readOnly" => Self::ReadOnly,
            "writeOnly" => Self::WriteOnly,
            "examples" => Self::Examples,
            "discriminator" => Self::Discriminator,
            "xml" => Self::Xml,
            "externalDocs" => Self::ExternalDocs,
            "example" => Self::Example,
            "nullable" if dialect == Dialect::Oas30 => Self::Nullable,
            // A vendor extension most Swagger 2.0 tools honour as 3.0's `nullable`. It only adds
            // `null` to what the schema accepts, so reading it keeps every type sound.
            "x-nullable" if dialect == Dialect::Swagger20 => Self::Nullable,
            _ if key.starts_with("x-") => Self::Extension,
            _ => Self::Unknown,
        }
    }
}
