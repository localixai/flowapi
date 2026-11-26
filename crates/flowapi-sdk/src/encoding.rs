//! What a `multipart/form-data` request body's Encoding Objects say a part is sent as: the
//! `contentType` of each property, read from the Media Object of the operation's request body
//! (a Reference Object followed). `<op>Request` takes it as `body.encoding`, and the client
//! passes the description's when the caller gives none. A `contentType` that is a list or a range
//! names no one type to send: it is left out, with a warning, and the part goes as its value's
//! default type. Swagger 2.0 has no Encoding Object; its forms read nothing here.

use crate::doc::{At, Doc, oas};
use crate::model::Encoding;
use flowapi::Diagnostics;

impl Doc<'_, '_> {
    /// The encodings of an operation's request body, one per `multipart/form-data` media type
    /// that names at least one part's type, in the order written.
    pub(crate) fn encodings(&self, op: Option<At>, diags: &mut Diagnostics) -> Vec<Encoding> {
        let Some(body) = op
            .and_then(|op| self.get(op, "requestBody"))
            .and_then(|b| self.resolve(b))
        else {
            return Vec::new();
        };
        let Some(content) = self.get(body, "content") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (media_type, media) in self.members(content) {
            let bare = media_type.split(';').next().unwrap_or("").trim();
            if !bare.eq_ignore_ascii_case("multipart/form-data") {
                continue;
            }
            let Some(encoding) = self.get(media, "encoding") else {
                continue;
            };
            let mut parts = Vec::new();
            for (name, object) in self.members(encoding) {
                let Some(at) = self.get(object, "contentType") else {
                    continue;
                };
                let Some(value) = self.str(object, "contentType") else {
                    continue;
                };
                let value = value.trim().to_owned();
                if value.contains(',') || value.contains('*') || value.is_empty() {
                    let code = oas!(
                        self.version,
                        "swagger.encoding",
                        "encoding-object.contentType"
                    );
                    let message = format!(
                        "the part `{name}` is encoded as `{value}`, a list or a range of types: no \
                         one type to send it as, so it is sent as its value's default type"
                    );
                    self.warn(diags, code, at, message);
                    continue;
                }
                parts.push((name, value));
            }
            if !parts.is_empty() {
                out.push(Encoding { media_type, parts });
            }
        }
        out
    }
}
