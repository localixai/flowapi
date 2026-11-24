//! One keyword of a Schema Object, put where the graph reads it: a reference resolved, an
//! applicator's schemas made parts, a keyword of the schema's own values set aside for
//! `own.rs`, one no type expresses recorded as lossy, an annotation passed by. The match has no
//! `_` arm: a keyword added to [`Keyword`] does not build until it is placed here (G2).

use super::Kind;
use super::build::{Builder, Read};
use super::own::Own;
use crate::id::{NodeId, TypeId};
use crate::schema::{Keyword, Resource};
use crate::source::Dialect;

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
                let nodes = items(value);
                let members: Vec<TypeId> = nodes.iter().map(|&n| self.sub(at, n)).collect();
                let union = self.make(Kind::Union(members));
                let part = if keyword == Keyword::OneOf {
                    self.one_of(at, &nodes, union, &mut read.lossy)
                } else {
                    union
                };
                read.parts.push(part);
            }
            Keyword::Then => read.then_else.0 = Some(value),
            Keyword::Else => read.then_else.1 = Some(value),
            Keyword::If => read.test = Some(value),
            // Swagger 2.0's `file`, of a form field or a response: bytes, which no JSON type
            // describes, so any value (sound), the narrowing recorded as lost.
            Keyword::Type
                if at.dialect == Dialect::Swagger20
                    && tree.and_then(|t| t.as_str(value)).as_deref() == Some("file") =>
            {
                read.lossy.push(keyword);
            }
            // A Swagger 2.0 parameter or header read as its own schema: its `required` is the
            // parameter's flag (the operation reads it), not the keyword's list of names.
            Keyword::Required
                if at.dialect == Dialect::Swagger20
                    && tree.and_then(|t| t.as_bool(value)).is_some() => {}
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
            | Keyword::Format
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
            | Keyword::MaxContains
            | Keyword::MinContains
            | Keyword::MaxProperties => own.set(keyword, value),
            Keyword::UnevaluatedItems | Keyword::UnevaluatedProperties => {
                self.unevaluated_keyword(at, keyword, value, read, own);
            }
            Keyword::DependentRequired | Keyword::DependentSchemas | Keyword::Dependencies => {
                self.dependent_keyword(at, keyword, value, read);
            }
            Keyword::Not => self.not_keyword(at, value, read),
            Keyword::UniqueItems | Keyword::MinProperties => {
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

    /// `not`: the kinds its schema accepts every value of are ruled out (`not: {"type":
    /// "string"}` is every value but strings); stated in full when it tests kinds alone.
    fn not_keyword(&mut self, at: &Resource, value: NodeId, read: &mut Read) {
        let local = self.local(at, value);
        if let Some(l) = &local {
            read.exclude = read.exclude.union(l.lower());
        }
        if !local.is_some_and(|l| l.kinds_only()) {
            read.lossy.push(Keyword::Not);
        }
    }
}
