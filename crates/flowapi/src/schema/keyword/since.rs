//! The draft that brought each keyword.

use super::Keyword;
use crate::source::Dialect;

impl Keyword {
    /// The first JSON Schema draft that has the keyword (draft 4 for those it had).
    pub(super) fn since(self) -> Dialect {
        if matches!(self, Self::Const | Self::Contains | Self::PropertyNames) {
            Dialect::Draft6
        } else if matches!(self, Self::If | Self::Then | Self::Else) {
            Dialect::Draft7
        } else if matches!(
            self,
            Self::MinContains
                | Self::MaxContains
                | Self::DependentRequired
                | Self::DependentSchemas
                | Self::UnevaluatedItems
                | Self::UnevaluatedProperties
        ) {
            Dialect::Draft2019
        } else if self == Self::PrefixItems {
            Dialect::Draft2020
        } else {
            Dialect::Draft4
        }
    }
}
