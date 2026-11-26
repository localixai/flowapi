//! Swagger 2.0 has no Encoding Object: a `formData` array's `collectionFormat` says how it goes in
//! the form. The core reads it, and each Items Object's own for the arrays inside it, as it reads
//! one in a query (`Media::encoding`, reported there); here it is given as an Encoding Object's
//! style would say it. `csv`, the default, is `form` unexploded (`a,b`); `multi` is `form`
//! exploded (a pair or a part per item); `ssv` and `pipes` are `spaceDelimited` and
//! `pipeDelimited`; `tsv`, which no 3.x style is, is `tabDelimited` (items joined by a tab). An
//! inner array's `csv` is `form` too (`,`).

use crate::model::{Encoding, Part, Parts};
use flowapi::spec::{Operation, Serialization, Style};

/// The encodings of a 2.0 operation's form, one per media type the core lifted it under whose
/// fields include an array.
pub(super) fn encodings(operation: &Operation) -> Vec<Encoding> {
    let Some(body) = &operation.body else {
        return Vec::new();
    };
    body.content
        .iter()
        .filter(|m| !m.encoding.is_empty())
        .map(|m| Encoding {
            media_type: m.media_type.clone(),
            parts: Parts {
                by_name: m
                    .encoding
                    .iter()
                    .map(|(n, s)| (n.clone(), part(s)))
                    .collect(),
                ..Parts::default()
            },
        })
        .collect()
}

/// One field's Encoding Object: its style and `explode`, and its inner arrays' styles.
fn part(s: &Serialization) -> Part {
    let nested = s.nested.iter().map(|&style| {
        String::from(if style == Style::Simple {
            "form"
        } else {
            style.name()
        })
    });
    Part {
        style: Some(String::from(s.style.name())),
        explode: Some(s.explode),
        nested: nested.collect(),
        ..Part::default()
    }
}
