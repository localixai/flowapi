//! An operation's responses, each with its headers and content, Reference Objects followed.

use super::reader::Reader;
use super::{Location, Param, Response};
use crate::diag::{Diagnostics, Loc};
use crate::id::{DocId, NodeId};
use crate::source::Version;

impl Reader<'_, '_> {
    /// A response's headers, by name, `Content-Type` left out.
    fn headers(&self, doc: DocId, response: NodeId, diags: &mut Diagnostics) -> Vec<Param> {
        let Some(t) = self.tree(doc) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for m in t.get(response, "headers").map_or(&[][..], |n| t.members(n)) {
            let Some(name) = t.string(m.key) else {
                continue;
            };
            if name.eq_ignore_ascii_case("content-type") {
                continue;
            }
            let Some((hd, h)) = self.follow_or_report(doc, m.value, "a header", diags) else {
                continue;
            };
            let (required, deprecated) =
                (self.flag(hd, h, "required"), self.flag(hd, h, "deprecated"));
            let schema = self.value_schema(hd, h, diags);
            let serialization = self.serialization(hd, h, Location::Header, diags);
            out.push(Param {
                name: name.into_owned(),
                location: Location::Header,
                required,
                deprecated,
                schema,
                serialization,
            });
        }
        out
    }

    /// Whether a key of a Responses Object is a status: `default`, a code from 100 to 599 (RFC
    /// 9110, 15) or a range `1XX` to `5XX`. Any other is reported and left out, never read as
    /// `default`; a range written with a lower-case `x` is read, and reported.
    fn status(status: &str, at: Loc, diags: &mut Diagnostics) -> bool {
        let b = status.as_bytes();
        let first = b.first().filter(|c| (b'1'..=b'5').contains(c)).is_some();
        let tail = b.get(1..).unwrap_or_default();
        let ok = status == "default"
            || (b.len() == 3 && first && (tail.iter().all(u8::is_ascii_digit) || tail == b"XX"));
        let lower = b.len() == 3 && first && tail.eq_ignore_ascii_case(b"XX") && tail != b"XX";
        if !ok {
            let message = if lower {
                format!("range {status:?} is written with a lower-case `x`: read as a range")
            } else {
                format!(
                    "response key {status:?} is not a status, `1XX` to `5XX` or `default`: left out"
                )
            };
            diags.warn("responses.status", at, message);
        }
        ok || lower
    }

    /// The responses of an operation, by status as written. `produces` is the media types a
    /// Swagger 2.0 operation's responses are in; empty in 3.x, whose responses name their own.
    pub(super) fn responses(
        &self,
        doc: DocId,
        node: NodeId,
        produces: &[String],
        diags: &mut Diagnostics,
    ) -> Vec<Response> {
        let Some(t) = self.tree(doc) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for m in t.members(node) {
            let Some(status) = t.string(m.key) else {
                continue;
            };
            if status.starts_with("x-")
                || !Self::status(
                    &status,
                    Loc {
                        doc,
                        offset: m.key.start,
                    },
                    diags,
                )
            {
                continue;
            }
            let what = format!("the response {status}");
            let Some((rd, r)) = self.follow_or_report(doc, m.value, &what, diags) else {
                continue;
            };
            let headers = self.headers(rd, r, diags);
            let content = match self.version {
                Version::V20 => self.produced(rd, r, produces),
                Version::V30 | Version::V31 | Version::V32 => self.content(rd, r, diags),
            };
            out.push(Response {
                status: status.into_owned(),
                headers,
                content,
            });
        }
        out
    }
}
