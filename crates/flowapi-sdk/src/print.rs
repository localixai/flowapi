//! The [`Service`] printed as TypeScript, after `flowapi-ts`'s SDK for the same program: three
//! `as const` values a client reads, each operation named, its method chosen and each body's
//! reading decided by that target (`sdk_bases`, `sdk_method`, `ReadAs`), so the table and the
//! functions it points at cannot disagree. The same service prints the same bytes (G4).

use crate::model::{Auth, Requirement, Scheme, Server, Service};
use flowapi::Program;
use flowapi_ts::{ReadAs, sdk_bases, sdk_method, string_literal as lit};
use std::fmt::Write;

const SERVERS: &str = "
/**
 * The servers the description names, in its order (`servers`; Swagger 2.0's `host`, `basePath`
 * and `schemes`). A URL may be relative to where the description is served (`/v1`, or `//host`
 * for its scheme); each `{name}` in it is a variable, its `default` unless the caller gives a
 * value (one of its `enum`, where it has one).
 */
export const flowapiServers = [
";

const SCHEMES: &str = "
/**
 * The security schemes, by name: `apiKey` a key sent `in` a header, a query parameter or a
 * cookie under `name`; `http` as `Authorization: <scheme> <credentials>`; `oauth2` and
 * `openIdConnect` a token sent as a bearer token, which the caller obtained or, for OAuth 2.0's
 * `clientCredentials` flow, the client requests at its `tokenUrl` (relative to the server's URL
 * where it is relative) for the scopes it needs. `mutualTLS` is a client certificate, which no
 * request carries: a client cannot send it, only its transport can.
 */
export const flowapiSecuritySchemes = {
";

const OPERATIONS: &str = "
/**
 * Each operation, by the name of its functions: its method and path; the servers that serve it
 * instead of `flowapiServers`, where it has its own; its security, alternatives of which any one
 * is enough, each naming the schemes it needs with their scopes (`[]`: none); how each response's
 * body is read, by media type (`json`, `text`, `bytes`, or `sent`: by the `Content-Type` the
 * answer sends, as `flowapiReadsBytes` says); by form and `multipart` media type, how the body's
 * values are written (`encoding`: the description's Encoding Objects by name and by position,
 * given as the body's `encoding`, `prefixEncoding` and `itemEncoding`); and its `<name>Request`
 * and `<name>Response`.
 */
export const flowapiOperations = {
";

pub(crate) fn table(program: &Program<'_>, service: &Service) -> String {
    let mut out = String::from(SERVERS);
    for s in &service.servers {
        let _ = writeln!(out, "  {},", server(s));
    }
    out.push_str("] as const;\n");
    out.push_str(SCHEMES);
    for s in &service.schemes {
        let _ = writeln!(out, "  {}: {{ {} }},", lit(&s.name), scheme(s));
    }
    out.push_str("} as const;\n");
    out.push_str(OPERATIONS);
    let bases = sdk_bases(program);
    for ((e, access), base) in program
        .endpoints()
        .iter()
        .zip(&service.operations)
        .zip(&bases)
    {
        let Some(base) = base else { continue };
        let op = &e.operation;
        let _ = writeln!(out, "  {base}: {{");
        let _ = writeln!(out, "    method: {},", lit(&sdk_method(op)));
        let _ = writeln!(out, "    path: {},", lit(&op.path));
        if !access.servers.is_empty() {
            let servers: Vec<String> = access.servers.iter().map(server).collect();
            let _ = writeln!(out, "    servers: [{}],", servers.join(", "));
        }
        let security: Vec<String> = access.security.iter().map(requirement).collect();
        let _ = writeln!(out, "    security: [{}],", security.join(", "));
        let responses: Vec<String> = op
            .responses
            .iter()
            .map(|r| {
                let content: Vec<String> = r
                    .content
                    .iter()
                    .map(|m| {
                        let read = ReadAs::of(&m.media_type).name();
                        format!("{}: {}", lit(&m.media_type), lit(read))
                    })
                    .collect();
                format!("{}: {{ content: {} }}", lit(&r.status), object(&content))
            })
            .collect();
        let _ = writeln!(out, "    responses: {},", object(&responses));
        if !access.encodings.is_empty() {
            let encodings = crate::encoding::literal(&access.encodings);
            let _ = writeln!(out, "    encoding: {encodings},");
        }
        let _ = writeln!(out, "    request: {base}Request,");
        let _ = writeln!(out, "    response: {base}Response,");
        out.push_str("  },\n");
    }
    out.push_str("} as const;\n");
    out
}

/// A scheme's members on one line.
fn scheme(s: &Scheme) -> String {
    let mut fields = match &s.auth {
        Auth::ApiKey { location, name } => format!(
            "type: \"apiKey\", in: {}, name: {}",
            lit(location.name()),
            lit(name)
        ),
        Auth::Http { scheme, format } => {
            let format = format
                .as_ref()
                .map(|f| format!(", bearerFormat: {}", lit(f)))
                .unwrap_or_default();
            format!("type: \"http\", scheme: {}{format}", lit(scheme))
        }
        Auth::OAuth2 {
            flows,
            client_credentials,
        } => {
            let mut out = format!("type: \"oauth2\", flows: {}", list(flows));
            if let Some(f) = client_credentials {
                let _ = write!(
                    out,
                    ", clientCredentials: {{ tokenUrl: {}, scopes: {} }}",
                    lit(&f.token_url),
                    list(&f.scopes)
                );
            }
            out
        }
        Auth::OpenIdConnect { url } => {
            format!("type: \"openIdConnect\", openIdConnectUrl: {}", lit(url))
        }
        Auth::MutualTls => String::from("type: \"mutualTLS\""),
    };
    if let Some(d) = &s.description {
        let _ = write!(fields, ", description: {}", lit(d));
    }
    if s.deprecated {
        fields.push_str(", deprecated: true");
    }
    fields
}

/// A server as an object literal on one line.
fn server(s: &Server) -> String {
    let mut out = format!("{{ url: {}", lit(&s.url));
    if let Some(name) = &s.name {
        let _ = write!(out, ", name: {}", lit(name));
    }
    if let Some(d) = &s.description {
        let _ = write!(out, ", description: {}", lit(d));
    }
    if !s.variables.is_empty() {
        let vars: Vec<String> = s
            .variables
            .iter()
            .map(|v| {
                let mut fields = format!("default: {}", lit(&v.default));
                if !v.values.is_empty() {
                    let _ = write!(fields, ", enum: {}", list(&v.values));
                }
                if let Some(d) = &v.description {
                    let _ = write!(fields, ", description: {}", lit(d));
                }
                format!("{}: {{ {fields} }}", lit(&v.name))
            })
            .collect();
        let _ = write!(out, ", variables: {{ {} }}", vars.join(", "));
    }
    out.push_str(" }");
    out
}

/// A requirement: each scheme's name with its scopes.
fn requirement(r: &Requirement) -> String {
    let schemes: Vec<String> = r
        .schemes
        .iter()
        .map(|(name, scopes)| format!("{}: {}", lit(name), list(scopes)))
        .collect();
    object(&schemes)
}

/// Members as an object literal: `{}` for none.
pub(crate) fn object(members: &[String]) -> String {
    if members.is_empty() {
        return String::from("{}");
    }
    format!("{{ {} }}", members.join(", "))
}

/// Strings as an array literal.
fn list(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|s| lit(s)).collect();
    format!("[{}]", items.join(", "))
}
