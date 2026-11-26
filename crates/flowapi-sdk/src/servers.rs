//! Where the API is served. 3.x: `servers` of the description, a Path Item and an operation, the
//! nearest non-empty one winning; the description's default is `/` (3.x, "OpenAPI Object",
//! `servers`). Swagger 2.0: `host`, `basePath` and `schemes` lifted into servers, an operation's
//! own `schemes` into servers of its own. A server that cannot be used as written (no `url`, a
//! variable with no `default` or that the URL names but `variables` does not) is left out, with a
//! warning; what only breaks a SHOULD or a MUST of the author is a warning and kept. A relative
//! URL is relative to the document that holds its Server Object (3.x, `url`; 3.1 and 3.2,
//! "Relative References in API URLs"): one of another document is printed relative to the first,
//! which the client resolves against where the description is served (`descriptionURL`).

use crate::doc::{At, Doc, ROOT, oas};
use crate::model::{Server, Variable};
use flowapi::id::DocId;
use flowapi::{Diagnostics, Version};

/// The schemes a 2.0 `schemes` list may name (2.0, "Swagger Object", `schemes`).
const SCHEMES: [&str; 4] = ["http", "https", "ws", "wss"];

impl Doc<'_, '_> {
    /// The description's servers: 3.x's `servers`, `/` when it has none; 2.0's lifted.
    pub(crate) fn servers(&self, diags: &mut Diagnostics) -> Vec<Server> {
        if self.version == Version::V20 {
            return self.lifted(ROOT, diags).unwrap_or_default();
        }
        match self
            .get(ROOT, "servers")
            .filter(|&l| !self.items(l).is_empty())
        {
            Some(list) => self.server_list(list, diags),
            None => vec![Server {
                url: String::from("/"),
                name: None,
                description: None,
                variables: Vec::new(),
            }],
        }
    }

    /// The servers that serve an operation instead of the description's: its own, else its Path
    /// Item's (the first item that has them); 2.0's from its own `schemes`. Empty for none.
    pub(crate) fn own_servers(
        &self,
        items: &[At],
        op: Option<At>,
        diags: &mut Diagnostics,
    ) -> Vec<Server> {
        if self.version == Version::V20 {
            return op.and_then(|op| self.lifted(op, diags)).unwrap_or_default();
        }
        let given = |at: At| {
            self.get(at, "servers")
                .filter(|&l| !self.items(l).is_empty())
        };
        op.and_then(given)
            .or_else(|| items.iter().find_map(|&item| given(item)))
            .map(|list| self.server_list(list, diags))
            .unwrap_or_default()
    }

    fn server_list(&self, list: At, diags: &mut Diagnostics) -> Vec<Server> {
        self.items(list)
            .into_iter()
            .filter_map(|s| self.server(s, diags))
            .collect()
    }

    fn server(&self, at: At, diags: &mut Diagnostics) -> Option<Server> {
        let Some(url) = self.str(at, "url") else {
            let code = oas!(self.version, "swagger.server", "server-object.url.required");
            self.warn(
                diags,
                code,
                at,
                String::from("a server has no `url` string: left out"),
            );
            return None;
        };
        if url.contains(['?', '#']) && self.version != Version::V30 {
            let code = oas!(
                self.version,
                "swagger.server",
                "server-object.url.must-not-query-fragment-part-url"
            );
            let message =
                format!("server URL {url:?} has a query or a fragment, which it MUST NOT");
            self.warn(diags, code, at, message);
        }
        let mut variables = Vec::new();
        for (name, v) in self
            .get(at, "variables")
            .map(|n| self.members(n))
            .unwrap_or_default()
        {
            variables.push(self.variable(name, v, &url, diags)?);
        }
        for name in template_names(&url) {
            if variables.iter().all(|v| v.name != name) {
                let message = format!(
                    "server URL {url:?} names `{{{name}}}`, which `variables` does not define: left out"
                );
                self.warn(diags, "server.variable", at, message);
                return None;
            }
        }
        let url = if at.0 == DocId::FIRST {
            url
        } else {
            let uri = |d: DocId| self.docs.get(d).map_or("", |d| d.uri.as_str());
            rebased(uri(DocId::FIRST), uri(at.0), url)
        };
        Some(Server {
            url,
            name: self
                .str(at, "name")
                .filter(|_| self.version == Version::V32),
            description: self.str(at, "description"),
            variables,
        })
    }

    /// A Server Variable Object; `None` (the server left out) when it has no `default`.
    fn variable(
        &self,
        name: String,
        at: At,
        url: &str,
        diags: &mut Diagnostics,
    ) -> Option<Variable> {
        let Some(default) = self.str(at, "default") else {
            let code = oas!(
                self.version,
                "swagger.server",
                "server-variable-object.default.required"
            );
            let message = format!(
                "variable `{name}` of server {url:?} has no `default` string: the server is left out"
            );
            self.warn(diags, code, at, message);
            return None;
        };
        let mut values: Vec<String> = Vec::new();
        if let Some(list) = self.get(at, "enum") {
            for item in self.items(list) {
                let value = self.tree(item.0).and_then(|t| t.as_str(item.1));
                if let Some(value) = value.filter(|v| !values.iter().any(|w| w == v)) {
                    values.push(value.into_owned());
                }
            }
            if values.is_empty() {
                let code = match self.version {
                    Version::V30 => "oas-3.0.server-variable-object.enum.should-not-array-empty",
                    Version::V20 | Version::V31 | Version::V32 => {
                        oas!(
                            self.version,
                            "swagger.server",
                            "server-variable-object.enum.must-not-array-empty"
                        )
                    }
                };
                let message =
                    format!("variable `{name}` has an `enum` of no string: any value is allowed");
                self.warn(diags, code, list, message);
            }
        }
        if !values.is_empty() && !values.contains(&default) {
            let code = match self.version {
                Version::V30 => {
                    "oas-3.0.server-variable-object.default.should-enum-defined-value-exist-enum-s"
                }
                Version::V20 | Version::V31 | Version::V32 => {
                    oas!(
                        self.version,
                        "swagger.server",
                        "server-variable-object.default.must-enum-defined-value-exist-enum-s"
                    )
                }
            };
            let message =
                format!("the `default` {default:?} of variable `{name}` is not in its `enum`");
            self.warn(diags, code, at, message);
        }
        Some(Variable {
            name,
            default,
            values,
            description: self.str(at, "description"),
        })
    }

    /// Swagger 2.0's servers at `at` (the description, or an operation with its own `schemes`):
    /// one per scheme, `<scheme>://<host><basePath>`. With no `host` the URL is the base path,
    /// relative to where the description is served; with a host and no `schemes`, `//<host>...`,
    /// the scheme the description is served with (2.0, `schemes`). `None` for an operation with
    /// no `schemes` of its own.
    fn lifted(&self, at: At, diags: &mut Diagnostics) -> Option<Vec<Server>> {
        let root = ROOT;
        let own = self
            .get(at, "schemes")
            .filter(|&l| !self.items(l).is_empty());
        let host = self.str(root, "host");
        // An operation's `schemes` change nothing where there is no host: its URL is relative.
        if at != root && (own.is_none() || host.is_none()) {
            return None;
        }
        let mut base = self
            .str(root, "basePath")
            .unwrap_or_else(|| String::from("/"));
        if !base.starts_with('/') {
            let message = format!("`basePath` {base:?} does not start with `/`: read as `/{base}`");
            self.warn(diags, "swagger.server", root, message);
            base.insert(0, '/');
        }
        let Some(host) = host else {
            return Some(vec![plain(base)]);
        };
        let Some(own) = own else {
            return Some(vec![plain(format!("//{host}{base}"))]);
        };
        let mut schemes: Vec<String> = Vec::new();
        for item in self.items(own) {
            let s = self.tree(item.0).and_then(|t| t.as_str(item.1));
            match s.filter(|s| SCHEMES.contains(&s.as_ref())) {
                Some(s) if !schemes.iter().any(|t| *t == s) => schemes.push(s.into_owned()),
                Some(_) => {}
                None => {
                    let message =
                        String::from("a scheme is none of http, https, ws, wss: left out");
                    self.warn(diags, "swagger.server", item, message);
                }
            }
        }
        Some(
            schemes
                .iter()
                .map(|s| plain(format!("{s}://{host}{base}")))
                .collect(),
        )
    }
}

fn plain(url: String) -> Server {
    Server {
        url,
        name: None,
        description: None,
        variables: Vec::new(),
    }
}

/// A server URL written in the document at `own`, as a reference from the document at `first`:
/// a relative path (`./v2`, `../v2`, `v2`) joined to `own`'s folder, then written from `first`'s.
/// A URL with a scheme, a host (`//`), an absolute path or a variable before its first `/` (a
/// templated host) is the same from either document.
fn rebased(first: &str, own: &str, url: String) -> String {
    let head = url.split('/').next().unwrap_or("");
    if url.starts_with('/') || head.contains([':', '{']) {
        return url;
    }
    let folder = |uri: &str| -> Vec<String> {
        let mut parts: Vec<String> = uri.split('/').map(str::to_owned).collect();
        parts.pop();
        parts
    };
    let mut path = folder(own);
    for part in url.split('/') {
        match part {
            "." | "" => {}
            ".." if path.last().is_some_and(|p| p != "..") => {
                path.pop();
            }
            _ => path.push(part.to_owned()),
        }
    }
    let base = folder(first);
    let same = base.iter().zip(&path).take_while(|(a, b)| a == b).count();
    let mut out: Vec<String> = base
        .get(same..)
        .unwrap_or_default()
        .iter()
        .map(|_| String::from(".."))
        .collect();
    out.extend(path.get(same..).unwrap_or_default().iter().cloned());
    if out.is_empty() {
        String::from(".")
    } else {
        out.join("/")
    }
}

/// The names of a URL's `{name}` expressions, in order.
fn template_names(url: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = url;
    while let Some((_, after)) = rest.split_once('{') {
        let Some((name, tail)) = after.split_once('}') else {
            break;
        };
        out.push(name);
        rest = tail;
    }
    out
}
