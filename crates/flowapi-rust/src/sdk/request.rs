//! `<name>_request(&input)`: the request an operation's input makes. The input is a struct of
//! the operation's `parameters` (the type the types printed for them) and, when it has a body,
//! `body`, an enum of its media types. Each parameter is one call of the runtime with its style
//! a constant: RFC 6570's operator (3.2, Appendix C), the encoder (none for headers and `style:
//! cookie`, reserved expansion for `allowReserved`) and its name percent-encoded where it goes
//! into a URI.

use super::param::{access, call, path};
use super::{Field, Op, bare, field, fields, is_json, type_of};
use crate::emit::Emitter;
use crate::ident::{Taken, literal, pascal};
use flowapi::spec::Location;
use std::fmt::Write;

/// The methods a Path Item names by a field, written in lower case; 3.2's
/// `additionalOperations` are sent as written.
const FIELDS: [&str; 9] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace", "query",
];

/// What a body's media type takes and how it is written.
enum Kind {
    /// The media type's type, as JSON.
    Json,
    /// The media type's type, as `application/x-www-form-urlencoded`.
    Form,
    /// Bytes as given.
    Bytes,
    /// A content type (a multipart boundary, a range made exact) and bytes as given.
    Typed,
}

fn kind(media_type: &str) -> Kind {
    let b = bare(media_type);
    if b.contains('*') || b.starts_with("multipart/") {
        Kind::Typed
    } else if is_json(&b) {
        Kind::Json
    } else if b == "application/x-www-form-urlencoded" {
        Kind::Form
    } else {
        Kind::Bytes
    }
}

pub(super) fn write(e: &mut Emitter<'_, '_>, out: &mut String, op: &Op<'_>) {
    let o = &op.endpoint.operation;
    let decl = e.name(op.item).to_owned();
    let params = field(&op.fields, "parameters").filter(|_| !o.params.is_empty());
    let body = o.body.as_ref().filter(|b| !b.content.is_empty());
    let body_field = field(&op.fields, "requestBody");
    let has_input = params.is_some() || body.is_some();
    let input = if has_input {
        e.fresh(&format!("{decl}Input"))
    } else {
        String::new()
    };
    let body_enum = if body.is_some() {
        e.fresh(&format!("{decl}Body"))
    } else {
        String::new()
    };
    if has_input {
        let _ = writeln!(
            out,
            "\n/// The input of `{}_request`.\n#[derive(Serialize, Deserialize, Debug, Clone, \
             PartialEq)]\npub struct {input} {{",
            op.base
        );
        if let Some(f) = params {
            let ty = type_of(e, op.item, f.ty);
            let _ = writeln!(out, "    pub parameters: {ty},");
        }
        if let Some(b) = body {
            if b.required {
                let _ = writeln!(out, "    pub body: {body_enum},");
            } else {
                let _ = writeln!(
                    out,
                    "    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    \
                     pub body: Option<{body_enum}>,"
                );
            }
        }
        out.push_str("}\n");
    }
    let mut arms = String::new();
    if let Some(b) = body {
        let media = body_field
            .and_then(|f| fields(e.p, f.ty))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "\n/// The body of `{}_request`, by media type.\n#[derive(Serialize, Deserialize, \
             Debug, Clone, PartialEq)]\npub enum {body_enum} {{",
            op.base
        );
        let mut variants = Taken::default();
        for m in &b.content {
            let t = &m.media_type;
            let v = variants.take(&pascal(t, "Any", "V"));
            let value = match kind(t) {
                Kind::Json | Kind::Form => field(&media, t).map_or_else(
                    || String::from("serde_json::Value"),
                    |f| type_of(e, op.item, f.ty),
                ),
                Kind::Bytes => String::from("Vec<u8>"),
                Kind::Typed => String::from("String, Vec<u8>"),
            };
            let _ = writeln!(
                out,
                "    #[serde(rename = {})]\n    {v}({value}),",
                literal(t)
            );
            let ct = literal(t);
            let made = match kind(t) {
                Kind::Json => {
                    format!("(String::from({ct}), sdk::json_body(v).map_err(sdk::in_body)?)")
                }
                Kind::Form => {
                    format!("(String::from({ct}), sdk::form_body(v).map_err(sdk::in_body)?)")
                }
                Kind::Bytes => format!("(String::from({ct}), v.clone())"),
                Kind::Typed => String::from("(t.clone(), v.clone())"),
            };
            let bind = if matches!(kind(t), Kind::Typed) {
                "t, v"
            } else {
                "v"
            };
            let _ = writeln!(arms, "        {body_enum}::{v}({bind}) => {made},");
        }
        out.push_str("}\n");
    }
    function(
        e,
        out,
        op,
        params,
        &input,
        body.map(|b| (b.required, arms)).as_ref(),
    );
}

fn function(
    e: &mut Emitter<'_, '_>,
    out: &mut String,
    op: &Op<'_>,
    params: Option<&Field<'_>>,
    input: &str,
    body: Option<&(bool, String)>,
) {
    let o = &op.endpoint.operation;
    let method = signature(out, op, params.is_some() || body.is_some(), input);
    let locs: Vec<(Field<'_>, Vec<Field<'_>>)> = params
        .and_then(|f| fields(e.p, f.ty))
        .unwrap_or_default()
        .into_iter()
        .map(|l| {
            let inner = fields(e.p, l.ty).unwrap_or_default();
            (l, inner)
        })
        .collect();
    let has = |l: Location| o.params.iter().any(|p| p.location == l);
    let path = path(&o.path, &o.params, &locs);
    let query = has(Location::Query) || has(Location::QueryString);
    let headers = has(Location::Header) || has(Location::Cookie);
    if params.is_some() {
        out.push_str("    let p = &input.parameters;\n");
    }
    if let Some(text) = &path {
        let _ = writeln!(
            out,
            "    let mut path = String::with_capacity({});",
            o.path.len() + 32
        );
        out.push_str(text);
    }
    if query {
        let _ = writeln!(
            out,
            "    let mut query = String::with_capacity({});",
            room(o, &[Location::Query, Location::QueryString])
        );
    }
    if headers {
        let n = o
            .params
            .iter()
            .filter(|p| p.location == Location::Header)
            .count();
        let _ = writeln!(
            out,
            "    let mut headers: Vec<(String, String)> = Vec::with_capacity({});",
            n + usize::from(has(Location::Cookie))
        );
    }
    if has(Location::Cookie) {
        let _ = writeln!(
            out,
            "    let mut cookie = String::with_capacity({});",
            room(o, &[Location::Cookie])
        );
    }
    for location in [
        Location::Query,
        Location::QueryString,
        Location::Header,
        Location::Cookie,
    ] {
        for p in o.params.iter().filter(|p| p.location == location) {
            let Some(at) = access(&locs, p) else { continue };
            let _ = writeln!(out, "    {}", at.replace("STMT", &call(p, "v")));
        }
    }
    if has(Location::Cookie) {
        out.push_str(
            "    if !cookie.is_empty() {\n        headers.push((String::from(\"cookie\"), \
             cookie));\n    }\n",
        );
    }
    if let Some((required, arms)) = body {
        if *required {
            let _ = write!(
                out,
                "    let body = Some(match &input.body {{\n{arms}    }});\n"
            );
        } else {
            let _ = write!(
                out,
                "    let body = match &input.body {{\n        None => None,\n        Some(b) => \
                 Some(match b {{\n{arms}        }}),\n    }};\n"
            );
        }
    }
    let path_text = if path.is_some() {
        String::from("path")
    } else {
        format!("path: String::from({})", literal(&o.path))
    };
    let pick = |used: bool, name: &'static str, none: &'static str| if used { name } else { none };
    let _ = writeln!(
        out,
        "    Ok(sdk::Request {{\n        method: {},\n        {path_text},\n        {},\n        \
         {},\n        {},\n    }})\n}}",
        literal(&method),
        pick(query, "query", "query: String::new()"),
        pick(headers, "headers", "headers: Vec::new()"),
        pick(body.is_some(), "body", "body: None"),
    );
}

/// The function's doc comment and signature; the method as sent.
fn signature(out: &mut String, op: &Op<'_>, takes: bool, input: &str) -> String {
    let o = &op.endpoint.operation;
    let method = if FIELDS.contains(&o.method.as_str()) {
        o.method.to_ascii_uppercase()
    } else {
        o.method.clone()
    };
    let _ = write!(
        out,
        "\n/// `{method} {}`: the request",
        o.path.replace('`', "'")
    );
    if takes {
        let _ = write!(
            out,
            " `input` makes; it sends nothing, and refuses a value that would change what the \
             request means (`sdk::RequestError`).\npub fn {}_request(input: &{input}) -> \
             Result<sdk::Request, sdk::RequestError> {{\n",
            op.base
        );
    } else {
        let _ = write!(
            out,
            "; it sends nothing.\npub fn {}_request() -> Result<sdk::Request, \
             sdk::RequestError> {{\n",
            op.base
        );
    }
    method
}

/// Room for what the parameters of `locations` usually write: each name, its `=`, a short
/// value and a separator; a longer value grows the string once or twice.
fn room(o: &flowapi::spec::Operation, locations: &[Location]) -> usize {
    o.params
        .iter()
        .filter(|p| locations.contains(&p.location))
        .map(|p| p.name.len() + 24)
        .sum()
}
