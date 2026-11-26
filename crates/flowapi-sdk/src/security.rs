//! How a caller proves who it is. The schemes are 3.x's `components.securitySchemes` (Reference
//! Objects followed) or 2.0's `securityDefinitions`, lifted into the 3.x model (`basic` is HTTP
//! basic; 2.0's OAuth flows take their 3.x names). A scheme that cannot be used as written (no
//! `type`, or one unknown; an API key with no `in` or `name`; HTTP with no `scheme`; OAuth 2.0
//! with no flows; OpenID Connect with no URL) is left out, with a warning. `security` is a list
//! of alternatives, each naming the schemes it needs together; an operation's replaces the
//! description's, `[]` meaning none. A name no scheme is declared under is a warning, and kept: a
//! client cannot meet that alternative, and says so where it is asked to. The flows are read in
//! `security/flows.rs`, the requirements in `security/requirements.rs`. A scheme's
//! Specification Extensions (`x-` members) are read by nothing.

mod flows;
mod requirements;

use crate::doc::{At, Doc, ROOT, oas};
use crate::model::{Auth, KeyLocation, Requirement, Scheme};
use flowapi::spec::Operation;
use flowapi::{Diagnostics, Version};

impl Doc<'_, '_> {
    /// Where the schemes are declared: 3.x's `components.securitySchemes`, 2.0's
    /// `securityDefinitions`.
    fn declared(&self) -> Option<At> {
        match self.version {
            Version::V20 => self.get(ROOT, "securityDefinitions"),
            Version::V30 | Version::V31 | Version::V32 => self
                .get(ROOT, "components")
                .and_then(|c| self.get(c, "securitySchemes")),
        }
    }

    /// An operation's security: its own `security`, else the description's (`root`), with
    /// whether it is the description's.
    pub(crate) fn effective(
        &self,
        op: &Operation,
        root: &[Requirement],
        diags: &mut Diagnostics,
    ) -> (Vec<Requirement>, bool) {
        let items = self.path_items(op);
        let own = self
            .operation(&items, op)
            .and_then(|at| self.security(at, diags));
        let inherits = own.is_none();
        (own.unwrap_or_else(|| root.to_vec()), inherits)
    }

    /// The security schemes, in the order written.
    pub(crate) fn schemes(&self, diags: &mut Diagnostics) -> Vec<Scheme> {
        let mut out = Vec::new();
        for (name, at) in self.declared().map(|d| self.members(d)).unwrap_or_default() {
            let what = format!("security scheme `{name}`");
            let Some(at) = self.follow(at, &what, diags) else {
                continue;
            };
            if let Some(auth) = self.auth(&name, at, diags) {
                // A name written twice is the last one's, as the core reads a key.
                out.retain(|s: &Scheme| s.name != name);
                out.push(Scheme {
                    name,
                    auth,
                    description: self.str(at, "description"),
                    deprecated: self.version == Version::V32 && self.flag(at, "deprecated"),
                });
            }
        }
        out
    }

    fn flag(&self, at: At, key: &str) -> bool {
        self.get(at, key)
            .and_then(|(d, n)| self.tree(d)?.as_bool(n))
            == Some(true)
    }

    /// What a scheme is; `None`, with a warning, when it cannot be used as written.
    fn auth(&self, name: &str, at: At, diags: &mut Diagnostics) -> Option<Auth> {
        let v = self.version;
        let ty = self.need(name, at, "type", diags)?;
        let auth = match ty.as_str() {
            "apiKey" => {
                let location = match self.str(at, "in").as_deref() {
                    Some("header") => KeyLocation::Header,
                    Some("query") => KeyLocation::Query,
                    Some("cookie") if v != Version::V20 => KeyLocation::Cookie,
                    Some(_) | None => {
                        self.missing(name, at, "in", diags);
                        return None;
                    }
                };
                let key = self.need(name, at, "name", diags)?;
                Auth::ApiKey {
                    location,
                    name: key,
                }
            }
            "basic" if v == Version::V20 => Auth::Http {
                scheme: String::from("basic"),
                format: None,
            },
            "http" if v != Version::V20 => Auth::Http {
                scheme: self.need(name, at, "scheme", diags)?.to_ascii_lowercase(),
                format: self.str(at, "bearerFormat"),
            },
            "oauth2" => {
                let flows = self.flows(at);
                if flows.is_empty() {
                    self.missing(name, at, "flows", diags);
                    return None;
                }
                Auth::OAuth2 {
                    client_credentials: self.client_credentials(name, at, diags),
                    flows,
                }
            }
            "openIdConnect" if v != Version::V20 => Auth::OpenIdConnect {
                url: self.need(name, at, "openIdConnectUrl", diags)?,
            },
            "mutualTLS" if matches!(v, Version::V31 | Version::V32) => Auth::MutualTls,
            other => {
                let message = format!(
                    "security scheme `{name}` has a `type` {other:?} this version does not \
                     define: left out"
                );
                let code = oas!(v, "swagger.security", "security-scheme-object.type");
                self.warn(diags, code, at, message);
                return None;
            }
        };
        Some(auth)
    }

    /// A string field a scheme requires; `None`, with a warning, when it has none.
    fn need(&self, name: &str, at: At, field: &str, diags: &mut Diagnostics) -> Option<String> {
        let found = self.str(at, field);
        if found.is_none() {
            self.missing(name, at, field, diags);
        }
        found
    }

    /// The warning for a scheme left out for want of a field it requires.
    fn missing(&self, name: &str, at: At, field: &str, diags: &mut Diagnostics) {
        let v = self.version;
        let code = match field {
            "type" => oas!(
                v,
                "swagger.security",
                "security-scheme-object.type.required"
            ),
            "in" => oas!(v, "swagger.security", "security-scheme-object.in.required"),
            "name" => oas!(
                v,
                "swagger.security",
                "security-scheme-object.name.required"
            ),
            "scheme" => oas!(
                v,
                "swagger.security",
                "security-scheme-object.scheme.required"
            ),
            "flows" => oas!(
                v,
                "swagger.security",
                "security-scheme-object.flows.required"
            ),
            _ => oas!(
                v,
                "swagger.security",
                "security-scheme-object.openIdConnectUrl.required"
            ),
        };
        let message =
            format!("security scheme `{name}` has no `{field}` it can be used by: left out");
        self.warn(diags, code, at, message);
    }
}
