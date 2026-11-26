//! How a parameter or a header is serialized: 3.x's `style`, `explode`, `allowReserved` and
//! `content`, with the spec's defaults per `in`; Swagger 2.0's `collectionFormat` mapped onto the
//! same styles (3.2, "Style Values": `csv` is `simple` or `form` without `explode`, `multi` is
//! `form` with it, `ssv` is `spaceDelimited`, `pipes` is `pipeDelimited`; `tsv`, which no 3.x
//! style is, is `tabDelimited`), in each location 2.0 allows it, and an Items Object's own for
//! the arrays inside the value.

use super::{Location, Serialization, Style, reader::Reader, swagger::FORM_TYPES};
use crate::diag::Diagnostics;
use crate::id::{DocId, NodeId};
use crate::source::Version;

impl Reader<'_, '_> {
    /// How the parameter or header at `node`, sent in `location`, is written.
    pub(super) fn serialization(
        &self,
        doc: DocId,
        node: NodeId,
        location: Location,
        diags: &mut Diagnostics,
    ) -> Serialization {
        let default = default_style(location);
        if self.version == Version::V20 {
            return self.collection_format((doc, node), location, diags);
        }
        let Some(t) = self.tree(doc) else {
            return plain(default);
        };
        let media_type = t
            .get(node, "content")
            .and_then(|c| t.members(c).first().and_then(|m| t.string(m.key)))
            .map(std::borrow::Cow::into_owned);
        let style = match t.get(node, "style").map(|n| (n, t.as_str(n))) {
            None => default,
            Some((n, written)) => {
                let found = written.as_deref().and_then(parse);
                match found.filter(|&s| allowed(s, location, self.version)) {
                    Some(s) => s,
                    None => {
                        diags.warn(
                            self.style_rule(),
                            self.docs.loc(doc, n),
                            format!(
                                "`style` `{}` is not one a parameter in `{}` takes: read as `{}`",
                                written.as_deref().unwrap_or("(not a string)"),
                                location.name(),
                                default.name()
                            ),
                        );
                        default
                    }
                }
            }
        };
        let explode = t
            .get(node, "explode")
            .and_then(|n| t.as_bool(n))
            .unwrap_or(matches!(style, Style::Form | Style::Cookie));
        Serialization {
            explode,
            allow_reserved: self.flag(doc, node, "allowReserved"),
            media_type,
            ..plain(style)
        }
    }

    /// A request body's form (`application/x-www-form-urlencoded`) members by their Encoding
    /// Objects, in the order written: one with `style`, `explode` or `allowReserved` as a query
    /// parameter of that style is written (its `contentType` then unused); any other by its
    /// `contentType` alone, left out when it has none. Empty for another media type and for a
    /// response's (`request` false), which no Encoding Object applies to.
    pub(super) fn form_encoding(
        &self,
        (doc, media): (DocId, NodeId),
        media_type: &str,
        request: bool,
        diags: &mut Diagnostics,
    ) -> Vec<(String, Serialization)> {
        let bare = media_type.split(';').next().unwrap_or("").trim();
        let mut out = Vec::new();
        let form = request && bare.eq_ignore_ascii_case(FORM_TYPES[0]);
        let Some(t) = self.tree(doc).filter(|_| form) else {
            return out;
        };
        for m in t.get(media, "encoding").map_or(&[][..], |e| t.members(e)) {
            let styled = STYLED.iter().any(|k| t.get(m.value, k).is_some());
            let given = t.get(m.value, "contentType").and_then(|n| t.as_str(n));
            let s = match given.filter(|c| !c.trim().is_empty()) {
                _ if styled => self.serialization(doc, m.value, Location::Query, diags),
                Some(c) => Serialization {
                    media_type: Some(c.trim().to_owned()),
                    ..plain(Style::Form)
                },
                None => continue,
            };
            out.extend(t.string(m.key).map(|name| (name.into_owned(), s)));
        }
        out
    }

    /// Swagger 2.0: `collectionFormat` (default `csv`) as a 3.x style, and the formats of the
    /// arrays inside the value: each `items` that is an array, by its own `collectionFormat`.
    pub(super) fn collection_format(
        &self,
        (doc, node): (DocId, NodeId),
        location: Location,
        diags: &mut Diagnostics,
    ) -> Serialization {
        let (style, explode) = self.format(doc, node, Some(location), diags);
        let mut nested = Vec::new();
        let mut at = node;
        while let Some(items) = self.tree(doc).and_then(|t| {
            let items = t.get(at, "items")?;
            let ty = t.get(items, "type").and_then(|n| t.as_str(n));
            (ty.as_deref() == Some("array")).then_some(items)
        }) {
            nested.push(self.format(doc, items, None, diags).0);
            at = items;
        }
        Serialization {
            explode,
            nested,
            ..plain(style)
        }
    }

    /// The style a parameter's (in `location`) or an Items Object's `collectionFormat` names,
    /// and whether it explodes: `csv` is the location's default unexploded (`Simple` in an
    /// Items Object), `multi` is `form` exploded where 2.0 allows it (a query). Any other value
    /// is reported and read as `csv`.
    fn format(
        &self,
        doc: DocId,
        node: NodeId,
        location: Option<Location>,
        diags: &mut Diagnostics,
    ) -> (Style, bool) {
        let csv = location.map_or(Style::Simple, default_style);
        let found = self.tree(doc).and_then(|t| t.get(node, "collectionFormat"));
        let written = found.and_then(|n| self.tree(doc)?.as_str(n));
        match written.as_deref() {
            None | Some("csv") => (csv, false),
            Some("ssv") => (Style::SpaceDelimited, false),
            Some("tsv") => (Style::TabDelimited, false),
            Some("pipes") => (Style::PipeDelimited, false),
            Some("multi") if location == Some(Location::Query) => (Style::Form, true),
            Some(other) => {
                let place = location.map_or_else(
                    || String::from("an Items Object"),
                    |l| format!("a parameter in `{}`", l.name()),
                );
                diags.warn(
                    "swagger.collectionFormat",
                    self.docs.loc(doc, found.unwrap_or(node)),
                    format!(
                        "`collectionFormat` {other} is not one {place} takes: written as `csv`"
                    ),
                );
                (csv, false)
            }
        }
    }

    /// The requirement an unknown or misplaced `style` breaks, in the document's version.
    fn style_rule(&self) -> &'static str {
        match self.version {
            Version::V20 | Version::V30 => "oas-3.0.parameter-object.style",
            Version::V31 => "oas-3.1.parameter-object.style",
            Version::V32 => "oas-3.2.parameter-object.style",
        }
    }
}

/// The Encoding Object's fields any of which makes its member written by style.
const STYLED: [&str; 3] = ["style", "explode", "allowReserved"];

/// The spec's default style for a location: `form` for a query or a cookie, `simple` for a path
/// or a header (and for `querystring`, which `content` serializes).
fn default_style(location: Location) -> Style {
    match location {
        Location::Query | Location::Cookie => Style::Form,
        Location::Path | Location::Header | Location::QueryString => Style::Simple,
    }
}

/// A style with its default `explode` and nothing else.
fn plain(style: Style) -> Serialization {
    Serialization {
        style,
        explode: matches!(style, Style::Form | Style::Cookie),
        allow_reserved: false,
        media_type: None,
        nested: Vec::new(),
    }
}

fn parse(written: &str) -> Option<Style> {
    Some(match written {
        "matrix" => Style::Matrix,
        "label" => Style::Label,
        "simple" => Style::Simple,
        "form" => Style::Form,
        "spaceDelimited" => Style::SpaceDelimited,
        "pipeDelimited" => Style::PipeDelimited,
        "deepObject" => Style::DeepObject,
        "cookie" => Style::Cookie,
        _ => return None,
    })
}

/// The combinations of "Style Values" (3.2); `cookie` is 3.2's own.
fn allowed(style: Style, location: Location, version: Version) -> bool {
    match location {
        Location::Path => matches!(style, Style::Matrix | Style::Label | Style::Simple),
        Location::Query => matches!(
            style,
            Style::Form | Style::SpaceDelimited | Style::PipeDelimited | Style::DeepObject
        ),
        Location::Header | Location::QueryString => style == Style::Simple,
        Location::Cookie => {
            style == Style::Form || (style == Style::Cookie && version == Version::V32)
        }
    }
}
