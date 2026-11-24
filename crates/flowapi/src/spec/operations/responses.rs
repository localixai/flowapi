//! An operation's responses, each with its headers and content, Reference Objects followed.

use super::reader::Reader;
use super::{Location, Param, Response};
use crate::diag::Diagnostics;
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
            out.push(Param {
                name: name.into_owned(),
                location: Location::Header,
                required,
                deprecated,
                schema,
            });
        }
        out
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
            if status.starts_with("x-") {
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
