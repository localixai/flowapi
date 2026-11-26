//! `security`: a list of alternatives, each naming the schemes it needs together with their
//! scopes (role names for a scheme that is not OAuth 2.0 or OpenID Connect, kept as written). A
//! name is looked up in the entry document's schemes wherever the requirement is written, a
//! document a Path Item's `$ref` reaches included: the resolution Appendix F recommends. 3.2 may
//! name a scheme by URI; one that names a scheme of the entry document's components
//! (`#/components/securitySchemes/<name>`, relative to it or absolute) is that scheme.

use crate::doc::{At, Doc, ROOT, oas};
use crate::model::Requirement;
use flowapi::{Diagnostics, Version};

/// Where 3.2's URI names a scheme of the entry document's components.
const COMPONENT: &str = "#/components/securitySchemes/";

impl Doc<'_, '_> {
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
            for (written, scopes) in self.members(item) {
                let name = self.scheme_name(&written, &declared);
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

    /// The scheme a requirement's key names: the key, a declared name; in 3.2, a URI of a scheme
    /// of the entry document's components, that scheme's name. Else the key as written.
    fn scheme_name(&self, written: &str, declared: &[String]) -> String {
        if self.version != Version::V32 || declared.iter().any(|d| d == written) {
            return written.to_owned();
        }
        let (resource, fragment) = flowapi::uri::split_fragment(written);
        let here = resource.is_empty()
            || flowapi::uri::decoded(&flowapi::uri::resolve(&self.base, resource))
                == flowapi::uri::decoded(flowapi::uri::split_fragment(&self.base).0);
        let pointer = flowapi::uri::decoded(fragment);
        let name = format!("#{pointer}")
            .strip_prefix(COMPONENT)
            .filter(|n| here && !n.contains('/'))
            .map(|n| n.replace("~1", "/").replace("~0", "~"));
        match name {
            Some(n) if declared.contains(&n) => n,
            Some(_) | None => written.to_owned(),
        }
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
