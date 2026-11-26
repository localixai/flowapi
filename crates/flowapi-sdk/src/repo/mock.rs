//! The mock server: `mock/server.mjs`, the same for every API (`files/server.mjs`), and
//! `mock/routes.mjs`, the operations the client calls as it reads them, printed from the
//! description: each one's method and path, the query parameters, headers and cookies it
//! requires, its body (whether required, the media types it takes), and its answer (the lowest
//! success status, its media type, an example value of its schema, `example.rs`).

use super::example::{self, Value};
use super::{Ctx, HEADER, RepoFile, generated_file};
use crate::out::{json_list, json_str};
use flowapi::spec::{Location, Operation, Response};
use std::fmt::Write as _;

const SERVER: &str = include_str!("files/server.mjs");

pub(crate) fn files(ctx: &Ctx<'_>) -> Vec<RepoFile> {
    vec![
        generated_file("mock/routes.mjs", routes(ctx)),
        generated_file("mock/server.mjs", SERVER.to_owned()),
    ]
}

fn routes(ctx: &Ctx<'_>) -> String {
    let mut out = format!(
        "// {HEADER}\n//\n// The operations the client calls, as the mock server answers them (server.mjs).\n\nexport default [\n"
    );
    let mut seen = Vec::new();
    for call in &ctx.calls {
        if seen.contains(&call.op) {
            continue;
        }
        seen.push(call.op);
        if let Some(op) = ctx.ops.list.get(call.op) {
            let _ = writeln!(out, "  {},", route(ctx, &op.operation));
        }
    }
    out.push_str("]\n");
    out
}

/// One route, as a JSON object on one line.
fn route(ctx: &Ctx<'_>, op: &Operation) -> String {
    let doc = &ctx.ops.doc;
    let required = |location: Location| -> Vec<String> {
        op.params
            .iter()
            .filter(|p| p.required && p.location == location)
            // An object's members may each be a parameter of their own (`form`, exploded):
            // only a scalar or a list is looked for by its name.
            .filter(|p| {
                !matches!(
                    example::of(doc, p.schema.as_ref().map(|r| (r.doc, r.node)), false),
                    Value::Obj(_)
                )
            })
            .map(|p| {
                if location == Location::Header {
                    p.name.to_ascii_lowercase()
                } else {
                    p.name.clone()
                }
            })
            .collect()
    };
    let method = op.method.to_ascii_uppercase();
    let body = op.body.as_ref().map_or_else(
        || String::from("null"),
        |b| {
            let media: Vec<&str> = b.content.iter().map(|m| m.media_type.as_str()).collect();
            format!(
                "{{ \"required\": {}, \"media\": {} }}",
                b.required,
                json_list(&media)
            )
        },
    );
    let (status, media, value) = success(ctx, op);
    format!(
        "{{ \"route\": {}, \"method\": {}, \"path\": {}, \"query\": {}, \"headers\": {}, \"cookies\": {}, \"body\": {body}, \"status\": {status}, \"media\": {}, \"example\": {} }}",
        json_str(&format!("{method} {}", op.path)),
        json_str(&method),
        json_str(&op.path),
        json_list(&required(Location::Query)),
        json_list(&required(Location::Header)),
        json_list(&required(Location::Cookie)),
        media
            .as_deref()
            .map_or_else(|| String::from("null"), json_str),
        value.json(),
    )
}

/// The status of a response as a number: `2XX` is 200.
fn status(r: &Response) -> Option<u16> {
    let s = r.status.to_ascii_uppercase().replace('X', "0");
    s.parse().ok().filter(|n| (200..300).contains(n))
}

/// The answer: the lowest success status, its media type (JSON first), an example of its schema.
fn success(ctx: &Ctx<'_>, op: &Operation) -> (u16, Option<String>, Value) {
    let Some((code, r)) = op
        .responses
        .iter()
        .filter_map(|r| Some((status(r)?, r)))
        .min_by_key(|(n, _)| *n)
    else {
        return (200, None, Value::Null);
    };
    let media = r
        .content
        .iter()
        .find(|m| m.media_type.eq_ignore_ascii_case("application/json"))
        .or_else(|| r.content.first());
    let Some(media) = media else {
        return (code, None, Value::Null);
    };
    let schema = media.schemas().first().map(|s| (s.doc, s.node));
    let value = example::of(&ctx.ops.doc, schema, true);
    (code, Some(media.media_type.clone()), value)
}
