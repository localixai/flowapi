//! One keyword of a Schema Object, put where the graph reads it: a reference resolved, an
//! applicator's schemas made parts, a keyword of the schema's own values set aside for
//! `own.rs`, one no type expresses recorded as lossy, an annotation passed by. The match has no
//! `_` arm: a keyword added to [`Keyword`] does not build until it is placed here (G2).

use super::build::{Builder, Read};
use super::own::Own;
use super::{Kind, Kinds};
use crate::id::{NodeId, TypeId};
use crate::json::{Tree, Value};
use crate::schema::{Keyword, Resource};

impl Builder<'_, '_> {
    pub(super) fn keyword(
        &mut self,
        at: &Resource,
        keyword: Keyword,
        value: NodeId,
        read: &mut Read,
        own: &mut Own,
    ) {
        let tree = self.docs.get(at.doc).map(|d| &d.tree);
        let items = |n: NodeId| tree.map(|t| t.items(n).to_vec()).unwrap_or_default();
        match keyword {
            Keyword::Ref => read.reference = Some(self.reference(at, value)),
            Keyword::DynamicRef | Keyword::RecursiveRef => {
                // Resolved by the dynamic scope, which a type cannot follow: any value (sound).
                read.lossy.push(keyword);
            }
            Keyword::AllOf => {
                for n in items(value) {
                    let t = self.sub(at, n);
                    read.parts.push(t);
                }
            }
            Keyword::AnyOf | Keyword::OneOf => {
                let members: Vec<TypeId> =
                    items(value).into_iter().map(|n| self.sub(at, n)).collect();
                let union = self.make(Kind::Union(members));
                read.parts.push(union);
                if keyword == Keyword::OneOf {
                    read.lossy.push(keyword);
                }
            }
            Keyword::Then => read.then_else.0 = Some(value),
            Keyword::Else => read.then_else.1 = Some(value),
            Keyword::If => read.has_if = true,
            Keyword::Type
            | Keyword::Enum
            | Keyword::Const
            | Keyword::Properties
            | Keyword::Required
            | Keyword::AdditionalProperties
            | Keyword::PatternProperties
            | Keyword::Items
            | Keyword::PrefixItems
            | Keyword::AdditionalItems
            | Keyword::MinItems
            | Keyword::MaxItems
            | Keyword::Nullable
            | Keyword::Format => own.set(keyword, value),
            Keyword::UnevaluatedItems | Keyword::UnevaluatedProperties => {
                self.unevaluated_keyword(at, keyword, value, read, own);
            }
            Keyword::DependentRequired | Keyword::DependentSchemas | Keyword::Dependencies => {
                self.dependent_keyword(at, keyword, value, read);
            }
            Keyword::Not => self.not_keyword(at, value, read),
            Keyword::Contains
            | Keyword::PropertyNames
            | Keyword::MultipleOf
            | Keyword::Maximum
            | Keyword::ExclusiveMaximum
            | Keyword::Minimum
            | Keyword::ExclusiveMinimum
            | Keyword::MaxLength
            | Keyword::MinLength
            | Keyword::Pattern
            | Keyword::UniqueItems
            | Keyword::MaxContains
            | Keyword::MinContains
            | Keyword::MaxProperties
            | Keyword::MinProperties => {
                read.lossy.push(keyword);
            }
            Keyword::Schema
            | Keyword::Id
            | Keyword::Anchor
            | Keyword::DynamicAnchor
            | Keyword::RecursiveAnchor
            | Keyword::Vocabulary
            | Keyword::Comment
            | Keyword::Defs
            | Keyword::ContentEncoding
            | Keyword::ContentMediaType
            | Keyword::ContentSchema
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
            | Keyword::Extension
            | Keyword::Unknown => {}
        }
    }
}

impl Builder<'_, '_> {
    /// The kinds a `not` rules out, when it is a boolean or names only a `type` (beside
    /// annotations): `not: {"type": "string"}` is every value but strings. `integer` rules out
    /// nothing a type can show (numbers stay). `None` for any other `not`.
    fn excluded(&self, at: &Resource, node: NodeId) -> Option<Kinds> {
        let tree = &self.docs.get(at.doc)?.tree;
        match tree.value(node) {
            Value::Bool(b) => return Some(if b { Kinds::ALL } else { Kinds::NONE }),
            Value::Object(_) => {}
            Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => return None,
        }
        let mut kinds = Kinds::ALL;
        for m in tree.members(node) {
            let key = tree.string(m.key)?;
            match key.as_ref() {
                "type" => kinds = named_kinds(tree, m.value)?,
                "title" | "description" | "$comment" | "examples" | "default" => {}
                _ => return None,
            }
        }
        Some(kinds)
    }
}

fn named_kinds(tree: &Tree<'_>, node: NodeId) -> Option<Kinds> {
    let names: Vec<NodeId> = match tree.value(node) {
        Value::String(_) => vec![node],
        Value::Array(_) => tree.items(node).to_vec(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => return None,
    };
    let mut kinds = Kinds::NONE;
    for n in names {
        kinds = kinds.union(match tree.as_str(n)?.as_ref() {
            "null" => Kinds::NULL,
            "boolean" => Kinds::BOOLEAN,
            "number" => Kinds::NUMBER,
            "integer" => Kinds::NONE,
            "string" => Kinds::STRING,
            "array" => Kinds::ARRAY,
            "object" => Kinds::OBJECT,
            _ => return None,
        });
    }
    Some(kinds)
}

impl Builder<'_, '_> {
    /// `unevaluatedProperties` or `unevaluatedItems`: `false` closes the shape where it can
    /// (`own.rs`); a schema is not checked.
    fn unevaluated_keyword(
        &mut self,
        at: &Resource,
        keyword: Keyword,
        value: NodeId,
        read: &mut Read,
        own: &mut Own,
    ) {
        let closed = self.docs.get(at.doc).and_then(|d| d.tree.as_bool(value)) == Some(false);
        if closed {
            own.set(keyword, value);
        } else {
            read.lossy.push(keyword);
        }
    }

    /// `dependentRequired`, `dependentSchemas`, `dependencies`: unions of each side.
    fn dependent_keyword(
        &mut self,
        at: &Resource,
        keyword: Keyword,
        value: NodeId,
        read: &mut Read,
    ) {
        match self.dependent(at, value) {
            Some(unions) => read.parts.extend(unions),
            None => read.lossy.push(keyword),
        }
    }

    /// `not`: the kinds it rules out, when it only names a type.
    fn not_keyword(&mut self, at: &Resource, value: NodeId, read: &mut Read) {
        match self.excluded(at, value) {
            Some(kinds) => read.exclude = read.exclude.union(kinds),
            None => read.lossy.push(Keyword::Not),
        }
    }
}
