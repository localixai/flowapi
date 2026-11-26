//! A config as the checks read it, each value with where it was written. Resources are a flat
//! list, each naming its parent, so that no walk over them recurses (G1).

use super::issue::At;
use crate::paging::Paging;

/// A name written as a key, with where.
#[derive(Clone, Debug, Default)]
pub(crate) struct Named {
    pub(crate) name: String,
    pub(crate) at: At,
}

/// A method's pagination.
#[derive(Clone, Debug)]
pub(crate) enum PagingRef {
    /// Not said: the default for a method named `list`.
    Unset,
    /// `false`: none.
    Off,
    /// A scheme, by name.
    Scheme(Named),
}

/// A resource.
#[derive(Debug)]
pub(crate) struct Resource {
    pub(crate) name: Named,
    /// The resource it is under; `None` at the top.
    pub(crate) parent: Option<usize>,
}

/// A method of a resource.
#[derive(Debug)]
pub(crate) struct Method {
    pub(crate) name: Named,
    pub(crate) resource: usize,
    /// The operation it calls, as written.
    pub(crate) op: Named,
    pub(crate) paging: PagingRef,
    /// Where `stream: true` is written, when it is.
    pub(crate) stream: Option<At>,
    /// Its own `retries`.
    pub(crate) retries: Option<i64>,
    /// Its own `security`.
    pub(crate) security: Option<Security>,
}

/// A `security` of the config: OpenAPI's Security Requirement Objects as written.
#[derive(Clone, Debug, Default)]
pub(crate) struct Security {
    /// Each alternative: each scheme it needs, by name with where, and its scopes.
    pub(crate) alternatives: Vec<Vec<(Named, Vec<String>)>>,
}

/// A credential of `auth`: the options its scheme is read by and, for a scheme the description
/// lacks, what that scheme is.
#[derive(Debug, Default)]
pub(crate) struct Credential {
    /// The scheme's name, the key.
    pub(crate) name: Named,
    /// `type`, `scheme`, `in`, `name` and `token_url`: a declaration.
    pub(crate) kind: Option<Named>,
    pub(crate) scheme: Option<Named>,
    pub(crate) location: Option<Named>,
    pub(crate) key: Option<Named>,
    pub(crate) token_url: Option<Named>,
    pub(crate) description: Option<String>,
    /// Its environment variables, by member (`env_var`, `username_env_var`, ...).
    pub(crate) env: Vec<(&'static str, Named)>,
    /// `scopes`, with where.
    pub(crate) scopes: Option<(Vec<String>, At)>,
}

impl Credential {
    /// The variable of a member (`env_var`, ...), where one is given.
    pub(crate) fn var(&self, member: &str) -> Option<&str> {
        self.env
            .iter()
            .find(|(m, _)| *m == member)
            .map(|(_, n)| n.name.as_str())
    }

    /// Whether it declares a scheme (gives any of `type`, `scheme`, `in`, `name`, `token_url`).
    pub(crate) fn declares(&self) -> Option<&Named> {
        [
            &self.kind,
            &self.scheme,
            &self.location,
            &self.key,
            &self.token_url,
        ]
        .into_iter()
        .find_map(Option::as_ref)
    }
}

/// What the generated client is made with beside its resources: values no check is about.
#[derive(Debug, Default)]
pub(crate) struct Settings {
    /// Each environment's URL, by its name as written.
    pub(crate) urls: Vec<(String, String)>,
    /// `environments.env_var`: the variable that overrides the base URL.
    pub(crate) base_url_env: Option<String>,
    /// `defaults.retries`.
    pub(crate) retries: Option<i64>,
    /// `defaults.timeout`, in seconds.
    pub(crate) timeout: Option<f64>,
    /// `defaults.max_retry_delay`, in seconds.
    pub(crate) max_retry_delay: Option<f64>,
    /// `package.name`, `package.version` and `package.repository`.
    pub(crate) package: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) repository: Option<String>,
    /// `package.description` and `package.keywords`.
    pub(crate) description: Option<String>,
    pub(crate) keywords: Vec<String>,
    /// `readme.title` and `readme.intro`.
    pub(crate) title: Option<String>,
    pub(crate) intro: Option<String>,
}

/// A pagination scheme defined under `pagination`.
#[derive(Debug)]
pub(crate) struct Scheme {
    pub(crate) name: Named,
    pub(crate) paging: Paging,
}

/// The config.
#[derive(Debug, Default)]
pub(crate) struct Config {
    pub(crate) spec: Option<String>,
    pub(crate) client: Option<Named>,
    /// `package.license`, with where: the repository refuses one it has no text for.
    pub(crate) license: Option<Named>,
    pub(crate) generator: Option<Named>,
    pub(crate) environments: Vec<Named>,
    pub(crate) default_environment: Option<Named>,
    pub(crate) auth: Vec<Credential>,
    /// The top-level `security`.
    pub(crate) security: Option<Security>,
    pub(crate) default_paging: Option<Named>,
    pub(crate) schemes: Vec<Scheme>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) methods: Vec<Method>,
    pub(crate) skip: Vec<Named>,
    /// `readme`'s methods, by member.
    pub(crate) readme: Vec<(&'static str, Named)>,
    /// Where `resources` is.
    pub(crate) resources_at: At,
    pub(crate) settings: Settings,
}

impl Config {
    /// A resource's names from the top, joined by `.` (`projects.members`).
    pub(crate) fn path(&self, resource: usize) -> String {
        let mut names = Vec::new();
        let mut next = Some(resource);
        while let Some(i) = next {
            let Some(r) = self.resources.get(i) else {
                break;
            };
            names.push(r.name.name.as_str());
            next = r.parent;
            if names.len() > self.resources.len() {
                break;
            }
        }
        names.reverse();
        names.join(".")
    }
}
