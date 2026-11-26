//! What a client needs of a description beyond its types, in no language: where the API is
//! served (`servers`, Swagger 2.0's `host`, `basePath` and `schemes`), how a caller proves who it
//! is (`securitySchemes`, 2.0's `securityDefinitions`), and which of those each operation asks
//! for (`security`).

/// A variable of a server's URL: `{name}` in it is replaced by a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    /// Its name, as `{name}` in the URL.
    pub name: String,
    /// The value used when the caller gives none.
    pub default: String,
    /// The values it allows (`enum`); empty for any.
    pub values: Vec<String>,
    /// Its `description`.
    pub description: Option<String>,
}

/// A server: a base URL that the operations' paths are appended to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    /// The URL as written, `{name}` for a variable; it may be relative to where the description
    /// is served (`/v1`, `//api.example.com`).
    pub url: String,
    /// Its `name` (3.2).
    pub name: Option<String>,
    /// Its `description`.
    pub description: Option<String>,
    /// Its variables, in the order written.
    pub variables: Vec<Variable>,
}

/// Where an API key goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyLocation {
    /// A header.
    Header,
    /// A query parameter.
    Query,
    /// A cookie.
    Cookie,
}

impl KeyLocation {
    /// The `in` value, as the description writes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Query => "query",
            Self::Cookie => "cookie",
        }
    }
}

/// How a scheme proves who the caller is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Auth {
    /// A key sent under a name in a header, a query parameter or a cookie.
    ApiKey {
        /// Where it goes.
        location: KeyLocation,
        /// The header's, parameter's or cookie's name.
        name: String,
    },
    /// HTTP authentication (RFC 9110, 11): `Authorization: <scheme> <credentials>`.
    Http {
        /// The scheme, in lower case (`bearer`, `basic`, or another).
        scheme: String,
        /// How a bearer token is formatted, a hint only (`JWT`).
        format: Option<String>,
    },
    /// OAuth 2.0: a token sent as a bearer token, which the caller obtained, or which the client
    /// requests itself by the client credentials flow, the one flow no person takes part in.
    OAuth2 {
        /// The flows' names, as written (2.0's lifted to 3.x names).
        flows: Vec<String>,
        /// The client credentials flow (2.0's `application`), where it has a token URL.
        client_credentials: Option<TokenFlow>,
    },
    /// OpenID Connect: a token the caller obtained, sent as a bearer token.
    OpenIdConnect {
        /// The discovery document's URL.
        url: String,
    },
    /// A client certificate (3.1): named so a requirement can say so; a client sends it through
    /// its transport, not in a request.
    MutualTls,
}

/// Where the client credentials flow asks for a token, and the scopes it may ask for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenFlow {
    /// The token endpoint as written: a URL, or a reference relative to the server's.
    pub token_url: String,
    /// The scopes the scheme defines, in the order written.
    pub scopes: Vec<String>,
}

/// A security scheme, by its name in the description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheme {
    /// Its name, the key under `securitySchemes`.
    pub name: String,
    /// What it is.
    pub auth: Auth,
    /// Its `description`.
    pub description: Option<String>,
    /// Whether it is deprecated (3.2).
    pub deprecated: bool,
}

/// One way to meet an operation's security: every scheme named, each with the scopes it needs.
/// An empty one means a call may be anonymous.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requirement {
    /// The schemes, each with its scopes, in the order written.
    pub schemes: Vec<(String, Vec<String>)>,
}

/// The types a `multipart/form-data` body's parts are sent as, by the description's Encoding
/// Objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Encoding {
    /// The media type, as written (`multipart/form-data`).
    pub media_type: String,
    /// Each part's name with its one `contentType`, in the order written.
    pub parts: Vec<(String, String)>,
}

/// What one operation asks for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Access {
    /// The servers that serve it instead of the description's: its own, else its Path Item's (or,
    /// in 2.0, its own `schemes`); empty when the description's serve it.
    pub servers: Vec<Server>,
    /// Its security: the alternatives, any one of which is enough; its own `security`, else the
    /// description's. Empty means no authentication.
    pub security: Vec<Requirement>,
    /// Whether `security` is the description's, the operation having none of its own: what a
    /// config's `security` replaces.
    pub inherits: bool,
    /// Its request body's encodings, one per `multipart/form-data` media type that has one.
    pub encodings: Vec<Encoding>,
}

/// What a description says about serving and securing its API.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Service {
    /// The description's servers; `/` when it names none (3.x), empty when every one it names
    /// was left out with a warning.
    pub servers: Vec<Server>,
    /// Its security schemes, in the order written; one that cannot be read is left out, with a
    /// warning.
    pub schemes: Vec<Scheme>,
    /// For each of the program's endpoints, in its order, what it asks for.
    pub operations: Vec<Access>,
}
