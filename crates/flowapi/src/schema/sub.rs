//! Where a schema holds other schemas: the keywords whose value is a schema, a list of them, or
//! a map of them.

use super::keyword::Keyword;
use crate::id::NodeId;
use crate::json::{Tree, Value};
use crate::source::Dialect;

/// The schemas a schema holds, with the keyword each is under, in the order written. A
/// keyword's value of the wrong shape holds none (the graph reports it).
#[must_use]
pub fn subschemas(tree: &Tree<'_>, schema: NodeId, dialect: Dialect) -> Vec<(Keyword, NodeId)> {
    let mut out = Vec::new();
    for member in tree.members(schema) {
        let Some(key) = tree.string(member.key) else {
            continue;
        };
        let keyword = Keyword::of(&key, dialect);
        let value = member.value;
        match keyword {
            Keyword::Not
            | Keyword::If
            | Keyword::Then
            | Keyword::Else
            | Keyword::Contains
            | Keyword::AdditionalProperties
            | Keyword::PropertyNames
            | Keyword::UnevaluatedItems
            | Keyword::UnevaluatedProperties
            | Keyword::ContentSchema
            | Keyword::AdditionalItems => {
                out.push((keyword, value));
            }
            Keyword::Items => match tree.value(value) {
                Value::Array(_) => out.extend(tree.items(value).iter().map(|&n| (keyword, n))),
                Value::Null
                | Value::Bool(_)
                | Value::Number(_)
                | Value::String(_)
                | Value::Object(_) => {
                    out.push((keyword, value));
                }
            },
            Keyword::AllOf | Keyword::AnyOf | Keyword::OneOf | Keyword::PrefixItems => {
                out.extend(tree.items(value).iter().map(|&n| (keyword, n)));
            }
            Keyword::Defs
            | Keyword::Properties
            | Keyword::PatternProperties
            | Keyword::DependentSchemas => {
                out.extend(tree.members(value).iter().map(|m| (keyword, m.value)));
            }
            Keyword::Dependencies => out.extend(
                tree.members(value)
                    .iter()
                    .filter(|m| !matches!(tree.value(m.value), Value::Array(_)))
                    .map(|m| (keyword, m.value)),
            ),
            Keyword::Schema
            | Keyword::Id
            | Keyword::Ref
            | Keyword::Anchor
            | Keyword::DynamicRef
            | Keyword::DynamicAnchor
            | Keyword::RecursiveRef
            | Keyword::RecursiveAnchor
            | Keyword::Vocabulary
            | Keyword::Comment
            | Keyword::Type
            | Keyword::Enum
            | Keyword::Const
            | Keyword::MultipleOf
            | Keyword::Maximum
            | Keyword::ExclusiveMaximum
            | Keyword::Minimum
            | Keyword::ExclusiveMinimum
            | Keyword::MaxLength
            | Keyword::MinLength
            | Keyword::Pattern
            | Keyword::MaxItems
            | Keyword::MinItems
            | Keyword::UniqueItems
            | Keyword::MaxContains
            | Keyword::MinContains
            | Keyword::MaxProperties
            | Keyword::MinProperties
            | Keyword::Required
            | Keyword::DependentRequired
            | Keyword::Format
            | Keyword::ContentEncoding
            | Keyword::ContentMediaType
            | Keyword::Title
            | Keyword::Description
            | Keyword::Default
            | Keyword::Deprecated
            | Keyword::ReadOnly
            | Keyword::WriteOnly
            | Keyword::Examples
            | Keyword::Discriminator
            | Keyword::Xml
            | Keyword::ExternalDocs
            | Keyword::Example
            | Keyword::Nullable
            | Keyword::Extension
            | Keyword::Unknown => {}
        }
    }
    out
}
