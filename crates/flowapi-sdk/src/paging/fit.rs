//! A pagination scheme held to one operation: each request role names a parameter (or a member
//! of the JSON body) of a type that can hold it, and each response role a path in the 2xx JSON
//! body's schema of a type that fits (an array for `items`, a string or an integer for a token, a
//! boolean for `has_more`). A method whose response cannot be paged so is an error of the config,
//! not an SDK that pages nothing (study 03, pitfall 9).

use super::Paging;
use super::schema::{self, Types};
use crate::config::issue::closest;
use crate::doc::{At, Doc};
use flowapi::spec::{Location, Operation, Payload};

/// What does not fit: the code, the part and role it is about, what is wrong, a hint, and
/// whether it is only a warning.
#[derive(Debug)]
pub(crate) struct Problem {
    pub(crate) code: &'static str,
    pub(crate) part: &'static str,
    pub(crate) role: &'static str,
    pub(crate) message: String,
    pub(crate) hint: Option<String>,
    pub(crate) warning: bool,
}

fn problem(code: &'static str, part: &'static str, role: &'static str, message: String) -> Problem {
    Problem {
        code,
        part,
        role,
        message,
        hint: None,
        warning: false,
    }
}

/// The JSON types a role's value may have.
fn wanted(kind: &str, role: &str) -> &'static [&'static str] {
    match role {
        "items" => &["array"],
        "cursor" | "id" => &["string", "integer"],
        "next" if kind == "link" => &["string"],
        "next" => &["string", "integer"],
        "has_more" => &["boolean"],
        _ => &["integer", "number"],
    }
}

/// The JSON body schema of the lowest 2xx status (`200` before `201` before `2XX`), of its JSON
/// media types the first by name: the same whatever order the description writes them in.
pub(crate) fn body(op: &Operation) -> Option<At> {
    let mut found: Vec<(&str, &str, At)> = op
        .responses
        .iter()
        .filter(|r| r.status.starts_with('2'))
        .flat_map(|r| r.content.iter().map(move |m| (r.status.as_str(), m)))
        .filter(|(_, m)| flowapi_ts::ReadAs::of(&m.media_type) == flowapi_ts::ReadAs::Json)
        .filter_map(|(status, m)| match &m.payload {
            Payload::Schema(Some(s)) => Some((status, m.media_type.as_str(), (s.doc, s.node))),
            Payload::Schema(None) | Payload::Form(_) => None,
        })
        .collect();
    found.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    found.first().map(|f| f.2)
}

/// The JSON request body's schema.
fn request_body(op: &Operation) -> Option<At> {
    op.body
        .iter()
        .flat_map(|b| b.content.iter())
        .find_map(|m| match &m.payload {
            Payload::Schema(Some(s))
                if flowapi_ts::ReadAs::of(&m.media_type) == flowapi_ts::ReadAs::Json =>
            {
                Some((s.doc, s.node))
            }
            Payload::Schema(_) | Payload::Form(_) => None,
        })
}

/// Every way `p` does not fit `op`.
pub(crate) fn fit(doc: &Doc<'_, '_>, op: &Operation, p: &Paging) -> Vec<Problem> {
    let mut out = Vec::new();
    for (role, name) in &p.request {
        request(doc, op, p.kind, role, name, &mut out);
    }
    if p.kind == "link" && p.get("next").is_some() == p.get("header").is_some() {
        let message =
            String::from("a link scheme names one of `next` (a path in the body) or `header`");
        out.push(problem(
            "config.pagination.link",
            "response",
            "next",
            message,
        ));
    }
    if let Some(header) = p.get("header") {
        let declared = op
            .responses
            .iter()
            .filter(|r| r.status.starts_with('2'))
            .any(|r| {
                r.headers
                    .iter()
                    .any(|h| h.name.eq_ignore_ascii_case(header))
            });
        if !declared {
            let message = format!(
                "no 2xx response of `{}` declares the header `{header}`",
                op.path
            );
            let mut w = problem("config.pagination.header", "response", "header", message);
            w.warning = true;
            out.push(w);
        }
    }
    let Some(root) = body(op) else {
        let message = String::from("the operation has no 2xx JSON response to read the page from");
        out.push(problem(
            "config.pagination.response",
            "response",
            "items",
            message,
        ));
        return out;
    };
    let items = p.get("items");
    let list = response(doc, root, p.kind, "items", items, &mut out);
    for (role, path) in &p.response {
        match *role {
            "items" | "header" => {}
            "id" => {
                if let Some(item) = list.and_then(|l| schema::items(doc, l)) {
                    response(doc, item, p.kind, "id", Some(path.as_str()), &mut out);
                }
            }
            _ => {
                response(doc, root, p.kind, role, Some(path.as_str()), &mut out);
            }
        }
    }
    out
}

/// A request role: a parameter not in the path, else a member of the JSON body.
fn request(
    doc: &Doc<'_, '_>,
    op: &Operation,
    kind: &str,
    role: &'static str,
    name: &str,
    out: &mut Vec<Problem>,
) {
    let param = op
        .params
        .iter()
        .find(|p| p.name == name && p.location != Location::Path);
    let found = match param {
        Some(p) => Some(p.schema.as_ref().map(|s| (s.doc, s.node))),
        None => request_body(op).and_then(|b| match schema::property(doc, b, name) {
            schema::Prop::Found(at) => Some(Some(at)),
            schema::Prop::Open | schema::Prop::Missing(_) => None,
        }),
    };
    let Some(schema) = found else {
        let names: Vec<&str> = op
            .params
            .iter()
            .filter(|p| p.location != Location::Path)
            .map(|p| p.name.as_str())
            .collect();
        let message = format!(
            "`{}` has no parameter `{name}` (it has {})",
            op.path,
            list(&names)
        );
        let mut p = problem("config.pagination.missing", "request", role, message);
        p.hint = closest(name, names.iter().copied()).map(|n| format!("did you mean `{n}`?"));
        out.push(p);
        return;
    };
    let types = schema.map_or(Types::Unknown, |s| schema::types(doc, s));
    let want = wanted(kind, role);
    if !types.allows(want) {
        let message = format!(
            "the parameter `{name}` is of type {}; `{role}` needs {}",
            types.show(),
            want.join(" or ")
        );
        out.push(problem(
            "config.pagination.mismatch",
            "request",
            role,
            message,
        ));
    }
}

/// A response role's path under `root` (the body itself for `items` not given), its type held
/// to the role; the schema found there.
fn response(
    doc: &Doc<'_, '_>,
    root: At,
    kind: &str,
    role: &'static str,
    path: Option<&str>,
    out: &mut Vec<Problem>,
) -> Option<At> {
    let at = match path {
        None => Some(root),
        Some(p) => match schema::path(doc, root, p) {
            Ok(at) => at,
            Err((segment, named)) => {
                let way = if segment == p {
                    String::new()
                } else {
                    format!(" on the way to `{p}`")
                };
                let message = format!(
                    "the response has no `{segment}`{way} (it has {})",
                    list(&named)
                );
                let mut problem = problem("config.pagination.missing", "response", role, message);
                problem.hint = closest(&segment, named.iter().map(String::as_str))
                    .map(|n| format!("did you mean `{n}`?"));
                out.push(problem);
                return None;
            }
        },
    }?;
    let types = schema::types(doc, at);
    let want = wanted(kind, role);
    if !types.allows(want) {
        let place = path.map_or_else(|| String::from("the body"), |p| format!("`{p}`"));
        let message = format!(
            "{place} is of type {}; `{role}` needs {}",
            types.show(),
            want.join(" or ")
        );
        out.push(problem(
            "config.pagination.mismatch",
            "response",
            role,
            message,
        ));
    }
    Some(at)
}

fn list(names: &[impl AsRef<str>]) -> String {
    if names.is_empty() {
        return String::from("none");
    }
    let all: Vec<String> = names
        .iter()
        .take(12)
        .map(|n| format!("`{}`", n.as_ref()))
        .collect();
    let more = names.len().saturating_sub(12);
    if more > 0 {
        format!("{}, and {more} more", all.join(", "))
    } else {
        all.join(", ")
    }
}
