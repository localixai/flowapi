//! A config as the checks read it, each value with where it was written. Resources are a flat
//! list, each naming its parent, so that no walk over them recurses (G1).

use super::issue::At;
use crate::paging::Paging;

/// A name written as a key, with where.
#[derive(Clone, Debug)]
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
}

/// What the generated client is made with beside its resources: values no check is about.
#[derive(Debug, Default)]
pub(crate) struct Settings {
    /// Each environment's URL, by its name as written.
    pub(crate) urls: Vec<(String, String)>,
    /// `environments.env_var`: the variable that overrides the base URL.
    pub(crate) base_url_env: Option<String>,
    /// Each security scheme's `env_var`, by its name.
    pub(crate) auth_env: Vec<(String, String)>,
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
    /// `readme.intro`.
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
    pub(crate) auth: Vec<Named>,
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
