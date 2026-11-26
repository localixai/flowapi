//! How a caller proves who it is. The schemes are 3.x's `components.securitySchemes` (Reference
//! Objects followed) or 2.0's `securityDefinitions`, lifted into the 3.x model (`basic` is HTTP
//! basic; 2.0's OAuth flows take their 3.x names). A scheme that cannot be used as written (no
//! `type`, or one unknown; an API key with no `in` or `name`; HTTP with no `scheme`; OAuth 2.0
//! with no flows; OpenID Connect with no URL) is left out, with a warning. `security` is a list
//! of alternatives, each naming the schemes it needs together; an operation's replaces the
//! description's, `[]` meaning none. A name no scheme is declared under is a warning, and kept: a
//! client cannot meet that alternative, and says so where it is asked to.

use crate::doc::{At, Doc, ROOT, oas};
use crate::model::{Auth, KeyLocation, Requirement, Scheme};
use flowapi::{Diagnostics, Version};

/// 2.0's OAuth 2.0 flows and their 3.x names (2.0, "Security Scheme Object", `flow`).
const FLOWS_20: [(&str, &str); 4] = [
    ("implicit", "implicit"),
    ("password", "password"),
    ("application", "clientCredentials"),
    ("accessCode", "authorizationCode"),
];

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
                Auth::OAuth2 { flows }
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

    /// An OAuth 2.0 scheme's flows by name: 3.x's `flows` keys, 2.0's `flow` lifted.
    fn flows(&self, at: At) -> Vec<String> {
        if self.version == Version::V20 {
            let flow = self.str(at, "flow");
            return FLOWS_20
                .iter()
                .filter(|(old, _)| flow.as_deref() == Some(*old))
                .map(|(_, new)| (*new).to_owned())
                .collect();
        }
        self.get(at, "flows")
            .map(|f| self.members(f).into_iter().map(|(name, _)| name).collect())
            .unwrap_or_default()
    }

    /// The description's `security`: `None` when it has none or it is not a list.
    pub(crate) fn security(&self, owner: At, diags: &mut Diagnostics) -> Option<Vec<Requirement>> {
        let list = self.get(owner, "security")?;
        let v = self.version;
        if !self.is(list, true) {
            let code = if owner == ROOT {
                oas!(v, "swagger.security", "openapi-object.security")
            } else {
                oas!(v, "swagger.security", "operation-object.security")
            };
            self.warn(
                diags,
                code,
                list,
                String::from("`security` is not a list: left out"),
            );
            return None;
        }
        let declared: Vec<String> = self
            .declared()
            .map(|d| self.members(d).into_iter().map(|(n, _)| n).collect())
            .unwrap_or_default();
        let mut out = Vec::new();
        for item in self.items(list) {
            if !self.is(item, false) {
                let code = oas!(
                    v,
                    "swagger.security",
                    "security-requirement-object.pattern-name"
                );
                let message = String::from("a security requirement is not an object: left out");
                self.warn(diags, code, item, message);
                continue;
            }
            let mut requirement = Requirement::default();
            for (name, scopes) in self.members(item) {
                if !declared.contains(&name) {
                    let code = match v {
                        Version::V20 => "swagger.security",
                        Version::V30 => {
                            "oas-3.0.security-requirement-object.pattern-name.must-each-name-correspond-security-scheme-declared"
                        }
                        Version::V31 => {
                            "oas-3.1.security-requirement-object.pattern-name.must-each-name-correspond-security-scheme-declared"
                        }
                        Version::V32 => {
                            "oas-3.2.security-requirement-object.pattern-name.must-each-name-uri-correspond-security-scheme"
                        }
                    };
                    let message = format!(
                        "a security requirement names `{name}`, which no security scheme is declared under"
                    );
                    self.warn(diags, code, item, message);
                }
                requirement.schemes.retain(|(n, _)| *n != name);
                requirement.schemes.push((name, self.scopes(scopes, diags)));
            }
            out.push(requirement);
        }
        Some(out)
    }

    /// A requirement's scopes: a list of strings; anything else is a warning and no scope.
    fn scopes(&self, at: At, diags: &mut Diagnostics) -> Vec<String> {
        let strings: Vec<String> = self
            .items(at)
            .into_iter()
            .filter_map(|(d, n)| Some(self.tree(d)?.as_str(n)?.into_owned()))
            .collect();
        if !self.is(at, true) || strings.len() != self.items(at).len() {
            let code = oas!(
                self.version,
                "swagger.security",
                "security-requirement-object.pattern-name"
            );
            let message = String::from(
                "a requirement's scopes are not a list of strings: read as the strings it holds",
            );
            self.warn(diags, code, at, message);
        }
        strings
    }
}
