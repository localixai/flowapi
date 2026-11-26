//! An OAuth 2.0 scheme's flows: their names (3.x's `flows` keys, 2.0's `flow` lifted to 3.x's
//! names), and the client credentials flow's token URL and scopes, the one flow the client runs
//! itself (2.0's `application`). The OAuth Flows Object's `x-` members are Specification
//! Extensions, no flow; so are 2.0's Scopes Object's, no scope, while 3.x's `scopes` is a plain
//! map whose every key is a scope. A token URL may be relative: the client resolves it against
//! the server's URL ("Relative References in URLs").

use crate::doc::{At, Doc, oas};
use crate::model::TokenFlow;
use flowapi::{Diagnostics, Version};

/// 2.0's OAuth 2.0 flows and their 3.x names (2.0, "Security Scheme Object", `flow`).
const FLOWS_20: [(&str, &str); 4] = [
    ("implicit", "implicit"),
    ("password", "password"),
    ("application", "clientCredentials"),
    ("accessCode", "authorizationCode"),
];

impl Doc<'_, '_> {
    /// An OAuth 2.0 scheme's flows by name: 3.x's `flows` keys but its extensions, 2.0's `flow`
    /// lifted.
    pub(super) fn flows(&self, at: At) -> Vec<String> {
        if self.version == Version::V20 {
            let flow = self.str(at, "flow");
            return FLOWS_20
                .iter()
                .filter(|(old, _)| flow.as_deref() == Some(*old))
                .map(|(_, new)| (*new).to_owned())
                .collect();
        }
        self.get(at, "flows")
            .map(|f| {
                self.members(f)
                    .into_iter()
                    .map(|(name, _)| name)
                    .filter(|name| !name.starts_with("x-"))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The client credentials flow: 3.x's `flows.clientCredentials`, 2.0's scheme itself when its
    /// `flow` is `application`. `None` without one; `None`, with a warning, when it has no token
    /// URL or one that is no URL reference. Scopes it lacks are a warning, and none.
    pub(super) fn client_credentials(
        &self,
        name: &str,
        at: At,
        diags: &mut Diagnostics,
    ) -> Option<TokenFlow> {
        let v = self.version;
        let flow = if v == Version::V20 {
            (self.str(at, "flow").as_deref() == Some("application")).then_some(at)?
        } else {
            self.get(at, "flows")
                .and_then(|f| self.get(f, "clientCredentials"))?
        };
        let Some(token_url) = self.str(flow, "tokenUrl") else {
            let code = oas!(v, "swagger.security", "oauth-flow-object.tokenUrl.required");
            let message = format!(
                "security scheme `{name}`: the client credentials flow has no `tokenUrl`: the client requests no token for it"
            );
            self.warn(diags, code, flow, message);
            return None;
        };
        if !is_url_reference(&token_url) {
            let code = oas!(
                v,
                "swagger.security",
                "oauth-flow-object.tokenUrl.must-form-url"
            );
            let message = format!(
                "security scheme `{name}`: the `tokenUrl` {token_url:?} is not in the form of a URL: the client requests no token for it"
            );
            self.warn(diags, code, flow, message);
            return None;
        }
        let scopes = match self.get(flow, "scopes") {
            Some(s) if self.is(s, false) => self
                .members(s)
                .into_iter()
                .map(|(scope, _)| scope)
                .filter(|scope| v != Version::V20 || !scope.starts_with("x-"))
                .collect(),
            Some(_) | None => {
                let code = oas!(v, "swagger.security", "oauth-flow-object.scopes.required");
                let message = format!(
                    "security scheme `{name}`: the client credentials flow has no `scopes` map: read as none"
                );
                self.warn(diags, code, flow, message);
                Vec::new()
            }
        };
        Some(TokenFlow { token_url, scopes })
    }
}

/// Whether a string is an RFC 3986 URI reference: no space or control character, a `%` only
/// before two hex digits, only the characters a URI may hold, and a scheme, where it has one,
/// that opens with a letter.
pub(crate) fn is_url_reference(s: &str) -> bool {
    const OTHER: &str = "-._~:/?#[]@!$&'()*+,;=";
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let fits = b.is_ascii_alphanumeric()
            || OTHER.as_bytes().contains(&b)
            || (b == b'%'
                && bytes
                    .get(i + 1..i + 3)
                    .is_some_and(|h| h.iter().all(u8::is_ascii_hexdigit)));
        if !fits {
            return false;
        }
    }
    let first = s.split(['/', '?', '#']).next().unwrap_or("");
    match first.split_once(':') {
        Some((scheme, _)) => scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic()),
        None => true,
    }
}
