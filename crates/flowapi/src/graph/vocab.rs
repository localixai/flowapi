//! A meta-schema's `$vocabulary`: which keywords a schema written against it means. Without the
//! applicator and validation vocabularies its assertions are only annotations, so its type is
//! any value (sound); a vocabulary a meta-schema requires and flowapi does not know is an error,
//! as is one with no core (2020-12 §8.1.2).

use super::build::Builder;
use crate::schema::{Resource, Target};
use crate::source::Dialect;
use crate::uri;

/// What a schema's keywords mean.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Meaning {
    /// Everything they say.
    All,
    /// Applicators only: validation keywords are annotations.
    NoValidation,
    /// Nothing a type shows: no applicator vocabulary either.
    Nothing,
}

/// The vocabularies flowapi knows, of 2020-12 and 2019-09.
const KNOWN: &[&str] = &[
    "https://json-schema.org/draft/2020-12/vocab/core",
    "https://json-schema.org/draft/2020-12/vocab/applicator",
    "https://json-schema.org/draft/2020-12/vocab/unevaluated",
    "https://json-schema.org/draft/2020-12/vocab/validation",
    "https://json-schema.org/draft/2020-12/vocab/meta-data",
    "https://json-schema.org/draft/2020-12/vocab/format-annotation",
    "https://json-schema.org/draft/2020-12/vocab/format-assertion",
    "https://json-schema.org/draft/2020-12/vocab/content",
    "https://json-schema.org/draft/2019-09/vocab/core",
    "https://json-schema.org/draft/2019-09/vocab/applicator",
    "https://json-schema.org/draft/2019-09/vocab/validation",
    "https://json-schema.org/draft/2019-09/vocab/meta-data",
    "https://json-schema.org/draft/2019-09/vocab/format",
    "https://json-schema.org/draft/2019-09/vocab/content",
    "https://spec.openapis.org/oas/3.1/vocab/base",
    "https://spec.openapis.org/oas/3.2/vocab/base",
];

impl Builder<'_, '_> {
    /// What the schema's keywords mean under its meta-schema's vocabularies.
    pub(super) fn vocabularies(&mut self, at: &Resource) -> Meaning {
        let docs = self.docs;
        let Some(tree) = docs.get(at.doc).map(|d| &d.tree) else {
            return Meaning::All;
        };
        let Some(named) = tree.get(at.node, "$schema").and_then(|n| tree.as_str(n)) else {
            return Meaning::All;
        };
        if Dialect::from_uri(&named).is_some() {
            return Meaning::All;
        }
        let meta_uri = uri::resolve(&at.base, &named);
        // The meta-schema by its `$id` or by the URI of its document, as any reference resolves.
        let found = match self.index.resolve(docs, &at.base, &named, at.dialect) {
            Target::Found(r) => Some((r.doc, r.node)),
            Target::Missing(_) => None,
        };
        let Some((meta, meta_root)) = found.and_then(|(d, n)| Some((&docs.get(d)?.tree, n))) else {
            return Meaning::All;
        };
        let Some(vocabularies) = meta.get(meta_root, "$vocabulary") else {
            return Meaning::All;
        };
        let named_vocab = |part: &str| {
            meta.members(vocabularies)
                .iter()
                .any(|m| meta.string(m.key).is_some_and(|k| k.ends_with(part)))
        };
        let (core, applicator, validation) = (
            named_vocab("/vocab/core"),
            named_vocab("/vocab/applicator"),
            named_vocab("/vocab/validation"),
        );
        let loc = docs.loc(at.doc, at.node);
        if !core {
            self.diags.error(
                "jsonschema-2020-12.core.$vocabulary",
                loc,
                format!("the meta-schema {meta_uri} names no core vocabulary"),
            );
        }
        for m in meta.members(vocabularies) {
            let Some(name) = meta.string(m.key) else {
                continue;
            };
            if meta.as_bool(m.value) == Some(true) && !KNOWN.contains(&name.as_ref()) {
                self.diags.error(
                    "jsonschema-2020-12.core.$vocabulary",
                    loc,
                    format!("the meta-schema {meta_uri} requires the vocabulary {name}, which flowapi does not know"),
                );
            }
        }
        match (applicator, validation) {
            (true, true) => Meaning::All,
            (true, false) => Meaning::NoValidation,
            (false, true | false) => Meaning::Nothing,
        }
    }
}
