//! How a parameter or a header is serialized: 3.x's `style`, `explode`, `allowReserved` and
//! `content`, with the spec's defaults per `in`; Swagger 2.0's `collectionFormat` mapped onto the
//! same styles (3.2, "Style Values": `csv` is `simple` or `form` without `explode`, `multi` is
//! `form` with it, `ssv` is `spaceDelimited`, `pipes` is `pipeDelimited`).

use super::reader::Reader;
use super::{Location, Serialization, Style};
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
            return self.collection_format(doc, node, location, diags);
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
            style,
            explode,
            allow_reserved: self.flag(doc, node, "allowReserved"),
            media_type,
        }
    }

    /// Swagger 2.0: `collectionFormat` (default `csv`) as a 3.x style.
    fn collection_format(
        &self,
        doc: DocId,
        node: NodeId,
        location: Location,
        diags: &mut Diagnostics,
    ) -> Serialization {
        let default = default_style(location);
        let Some(t) = self.tree(doc) else {
            return plain(default);
        };
        let found = t.get(node, "collectionFormat");
        let written = found.and_then(|n| t.as_str(n));
        let (style, explode) = match (written.as_deref(), location) {
            (None | Some("csv"), _) => (default, false),
            (Some("multi"), Location::Query) => (Style::Form, true),
            (Some("ssv"), Location::Query) => (Style::SpaceDelimited, false),
            (Some("pipes"), Location::Query) => (Style::PipeDelimited, false),
            (Some(other), _) => {
                let at = self.docs.loc(doc, found.unwrap_or(node));
                diags.warn(
                    "swagger.collectionFormat",
                    at,
                    format!(
                        "`collectionFormat` {other} has no 3.x style in `{}`: written as `csv`",
                        location.name()
                    ),
                );
                (default, false)
            }
        };
        Serialization {
            style,
            explode,
            allow_reserved: false,
            media_type: None,
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
