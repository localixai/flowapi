//! Swagger 2.0 has no Encoding Object: a `formData` parameter's `collectionFormat` says how its
//! array goes in the form, read here as an Encoding Object's style would say it. `csv`, the
//! default, is `form` unexploded (`a,b`); `multi` is `form` exploded (a pair or a part per
//! item); `ssv` and `pipes` are `spaceDelimited` and `pipeDelimited`; `tsv`, which no 3.x style
//! is, is `tabDelimited` (items joined by a tab). An array inside the array is joined by its
//! Items Object's own `collectionFormat`, level by level, as the core reads one in a path, a
//! query or a header (2.0, "Items Object").

use crate::doc::{At, Doc};
use crate::model::{Encoding, Part, Parts};
use flowapi::Diagnostics;
use flowapi::spec::{Operation, Payload};

impl Doc<'_, '_> {
    /// The encodings of a 2.0 operation's form, one per media type the core lifted it under
    /// whose fields include an array.
    pub(super) fn swagger_encodings(
        &self,
        operation: &Operation,
        diags: &mut Diagnostics,
    ) -> Vec<Encoding> {
        let Some(body) = &operation.body else {
            return Vec::new();
        };
        let mut parts = Parts::default();
        let Some(fields) = body.content.iter().find_map(|m| match &m.payload {
            Payload::Form(fields) => Some(fields),
            Payload::Schema(_) => None,
        }) else {
            return Vec::new();
        };
        for field in fields {
            let Some(at) = field.schema.as_ref().map(|s| (s.doc, s.node)) else {
                continue;
            };
            if self.str(at, "type").as_deref() != Some("array") {
                continue;
            }
            let format = self.str(at, "collectionFormat");
            let (style, explode) = match format.as_deref() {
                None | Some("csv") => ("form", false),
                Some("multi") => ("form", true),
                Some("ssv") => ("spaceDelimited", false),
                Some("pipes") => ("pipeDelimited", false),
                Some("tsv") => ("tabDelimited", false),
                Some(other) => {
                    let message = format!(
                        "`collectionFormat` {other} is none of csv, ssv, tsv, pipes, multi: \
                         written as `csv`"
                    );
                    let place = self.get(at, "collectionFormat").unwrap_or(at);
                    self.warn(diags, "swagger.collectionFormat", place, message);
                    ("form", false)
                }
            };
            let mut nested = Vec::new();
            let mut level = at;
            while let Some(items) = self
                .get(level, "items")
                .filter(|&i| self.str(i, "type").as_deref() == Some("array"))
            {
                nested.push(String::from(self.items_style(items, diags)));
                level = items;
            }
            let part = Part {
                style: Some(String::from(style)),
                explode: Some(explode),
                nested,
                ..Part::default()
            };
            parts.by_name.push((field.name.clone(), part));
        }
        if parts.is_empty() {
            return Vec::new();
        }
        body.content
            .iter()
            .filter(|m| matches!(m.payload, Payload::Form(_)))
            .map(|m| Encoding {
                media_type: m.media_type.clone(),
                parts: parts.clone(),
            })
            .collect()
    }

    /// The style an Items Object's `collectionFormat` joins its array by (`csv` by default);
    /// `multi`, which only a parameter takes, or an unknown value is reported and read as `csv`.
    fn items_style(&self, items: At, diags: &mut Diagnostics) -> &'static str {
        match self.str(items, "collectionFormat").as_deref() {
            None | Some("csv") => "form",
            Some("ssv") => "spaceDelimited",
            Some("pipes") => "pipeDelimited",
            Some("tsv") => "tabDelimited",
            Some(other) => {
                let message = format!(
                    "`collectionFormat` {other} is not one an Items Object takes: written as `csv`"
                );
                let place = self.get(items, "collectionFormat").unwrap_or(items);
                self.warn(diags, "swagger.collectionFormat", place, message);
                "form"
            }
        }
    }
}
