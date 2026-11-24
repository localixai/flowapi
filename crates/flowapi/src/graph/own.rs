//! A schema's own constraints, the keywords that narrow the kinds and shapes of values: `type`,
//! `enum`, `const`, the object and array keywords, `nullable` (3.0). Each narrows only its kind:
//! `properties` says nothing of a string, so without `type` a string still passes (2020-12 §7.6).

use super::arrays::bounded;
use super::build::Builder;
use super::narrow::restrict;
use super::{Array, Branches, Kinds};
use crate::id::{NodeId, TypeId};
use crate::json::{Tree, Value};
use crate::schema::{Keyword, Resource};
use std::borrow::Cow;

/// The nodes of the keywords that narrow a schema's own values.
#[derive(Default)]
pub(super) struct Own {
    pub(super) ty: Option<NodeId>,
    pub(super) enumeration: Option<NodeId>,
    pub(super) constant: Option<NodeId>,
    pub(super) properties: Option<NodeId>,
    pub(super) required: Option<NodeId>,
    pub(super) additional: Option<NodeId>,
    pub(super) patterns: Option<NodeId>,
    pub(super) items: Option<NodeId>,
    pub(super) prefix: Option<NodeId>,
    pub(super) additional_items: Option<NodeId>,
    pub(super) min_items: Option<NodeId>,
    pub(super) max_items: Option<NodeId>,
    pub(super) unevaluated_props: Option<NodeId>,
    pub(super) unevaluated_items: Option<NodeId>,
    pub(super) nullable: Option<NodeId>,
    pub(super) format: Option<NodeId>,
}

impl Own {
    pub(super) fn set(&mut self, keyword: Keyword, value: NodeId) {
        let slot = match keyword {
            Keyword::Type => &mut self.ty,
            Keyword::Enum => &mut self.enumeration,
            Keyword::Const => &mut self.constant,
            Keyword::Properties => &mut self.properties,
            Keyword::Required => &mut self.required,
            Keyword::AdditionalProperties => &mut self.additional,
            Keyword::PatternProperties => &mut self.patterns,
            Keyword::Items => &mut self.items,
            Keyword::PrefixItems => &mut self.prefix,
            Keyword::AdditionalItems => &mut self.additional_items,
            Keyword::MinItems => &mut self.min_items,
            Keyword::MaxItems => &mut self.max_items,
            Keyword::UnevaluatedProperties => &mut self.unevaluated_props,
            Keyword::UnevaluatedItems => &mut self.unevaluated_items,
            Keyword::Nullable => &mut self.nullable,
            Keyword::Format => &mut self.format,
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
            | Keyword::Defs
            | Keyword::AllOf
            | Keyword::AnyOf
            | Keyword::OneOf
            | Keyword::Not
            | Keyword::If
            | Keyword::Then
            | Keyword::Else
            | Keyword::DependentSchemas
            | Keyword::Dependencies
            | Keyword::Contains
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
            | Keyword::MinProperties
            | Keyword::DependentRequired
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
            | Keyword::Unknown => return,
        };
        *slot = Some(value);
    }

    fn narrows(&self) -> bool {
        [
            self.ty,
            self.enumeration,
            self.constant,
            self.properties,
            self.required,
            self.additional,
        ]
        .iter()
        .chain(
            [
                self.patterns,
                self.items,
                self.prefix,
                self.additional_items,
                self.min_items,
                self.max_items,
                self.unevaluated_props,
                self.unevaluated_items,
            ]
            .iter(),
        )
        .any(Option::is_some)
    }
}

impl<'t> Builder<'_, 't> {
    /// The values the schema's own keywords allow; `None` when none narrows them.
    pub(super) fn own(
        &mut self,
        at: &Resource,
        own: &Own,
        parts: &mut Vec<TypeId>,
    ) -> Option<Branches<'t>> {
        if !own.narrows() {
            return None;
        }
        let docs = self.docs;
        let tree = &docs.get(at.doc)?.tree;
        let mut b = Branches::of(self.kinds(tree, at, own.ty), self.any);
        if let (Some(n), Some(_)) = (own.nullable, own.ty) {
            b.null |= tree.as_bool(n) == Some(true);
        }
        if let Some(number) = b.number.as_mut() {
            number.integer = own.ty.is_some_and(|n| integer_only(tree, n));
        }
        if let (Some(s), Some(f)) = (b.string.as_mut(), own.format.and_then(|n| tree.as_str(n))) {
            s.format = Some(f);
        }
        if b.object.is_some() {
            b.object = Some(self.object(tree, at, own));
        }
        if b.array.is_some() {
            b.array = self.array(tree, at, own);
        }
        for node in [own.enumeration, own.constant].into_iter().flatten() {
            let values: Vec<NodeId> = if Some(node) == own.constant {
                vec![node]
            } else {
                tree.items(node).to_vec()
            };
            restrict(&mut b, tree, &values);
            parts.extend(self.literals(tree, &values));
        }
        Some(b)
    }

    fn kinds(&mut self, tree: &Tree<'t>, at: &Resource, ty: Option<NodeId>) -> Kinds {
        let Some(node) = ty else { return Kinds::ALL };
        let names: Vec<Cow<'t, str>> = match tree.value(node) {
            Value::String(_) => tree.as_str(node).into_iter().collect(),
            Value::Array(_) => tree
                .items(node)
                .iter()
                .filter_map(|&n| tree.as_str(n))
                .collect(),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => Vec::new(),
        };
        let mut kinds = Kinds::NONE;
        for name in &names {
            kinds = kinds.union(match name.as_ref() {
                "null" => Kinds::NULL,
                "boolean" => Kinds::BOOLEAN,
                "number" | "integer" => Kinds::NUMBER,
                "string" => Kinds::STRING,
                "array" => Kinds::ARRAY,
                "object" => Kinds::OBJECT,
                other => {
                    let loc = self.docs.loc(at.doc, node);
                    self.diags.error(
                        "schema.type",
                        loc,
                        format!("`type` names no JSON type: \"{other}\""),
                    );
                    Kinds::ALL
                }
            });
        }
        if names.is_empty() { Kinds::ALL } else { kinds }
    }

    fn array(&mut self, tree: &Tree<'t>, at: &Resource, own: &Own) -> Option<Array> {
        let tuple_items = at.dialect.tuple_items()
            && own
                .items
                .is_some_and(|n| matches!(tree.value(n), Value::Array(_)));
        let (prefix_node, rest_node) = if tuple_items {
            (own.items, own.additional_items)
        } else {
            (own.prefix, own.items)
        };
        let prefix: Vec<TypeId> = prefix_node
            .map_or(&[][..], |n| tree.items(n))
            .to_vec()
            .into_iter()
            .map(|n| self.sub(at, n))
            .collect();
        let rest = rest_node.map_or(self.any, |n| self.sub(at, n));
        let number = |node: Option<NodeId>| {
            node.and_then(|n| match tree.value(n) {
                Value::Number(span) => tree.raw(span).parse::<f64>().ok(),
                Value::Null
                | Value::Bool(_)
                | Value::String(_)
                | Value::Array(_)
                | Value::Object(_) => None,
            })
        };
        let never = TypeId::new(1).unwrap_or(self.any);
        let (mut prefix, mut rest) = (prefix, rest);
        // `unevaluatedItems: false`: no item past those some schema applied here could evaluate.
        if own.unevaluated_items.is_some()
            && rest_node.is_none()
            && let Some(found) = self.evaluated(at, false)
        {
            prefix.resize(prefix.len().max(found.items), self.any);
            rest = never;
        }
        bounded(
            prefix,
            rest,
            number(own.min_items),
            number(own.max_items),
            never,
        )
    }
}

/// Whether `type` names integers and not numbers.
fn integer_only(tree: &Tree<'_>, node: NodeId) -> bool {
    let names: Vec<Cow<'_, str>> = match tree.value(node) {
        Value::String(_) => tree.as_str(node).into_iter().collect(),
        Value::Array(_) => tree
            .items(node)
            .iter()
            .filter_map(|&n| tree.as_str(n))
            .collect(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => Vec::new(),
    };
    names.iter().any(|n| n == "integer") && !names.iter().any(|n| n == "number")
}
