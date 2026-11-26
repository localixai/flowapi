//! `api.md`: every method by resource, each with its TypeScript signature (the types the package
//! exports, so a caller can write them), its operation's route and summary, and whether the
//! description marks it deprecated. `tests/texts.mjs` compiles each signature against the client.

use super::super::{Call, Ctx, HEADER, Kind};
use super::counted;
use super::sections::in_order;
use crate::client::resources::{access_of, sse_only};
use crate::config::check::{effective, find};
use crate::prose;
use std::fmt::Write as _;

pub(super) fn api(ctx: &Ctx<'_>) -> String {
    let c = ctx.config;
    let mut out = format!(
        "<!-- {HEADER} -->\n\n# API\n\nEvery method of `{}`, by resource: {}. A method takes its operation's parameters (`params`) and the options of one call (`options`); the types are the package's:\n\n```ts\nimport type {{ ApiCall, Data, Input, ItemOf, PagePromise, RequestOptions, ServerSentEvent, Stream }} from '{}'\n```\n",
        ctx.client(),
        counted(ctx),
        ctx.package()
    );
    for i in in_order(ctx) {
        let calls: Vec<&Call> = ctx
            .calls
            .iter()
            .filter(|k| c.methods.get(k.method).is_some_and(|m| m.resource == i))
            .collect();
        if calls.is_empty() {
            continue;
        }
        let _ = writeln!(out, "\n## client.{}\n", access_of(c, i));
        for call in calls {
            method(ctx, call, &mut out);
        }
    }
    out
}

/// One method's line: its signature, then its route and what its operation says of itself.
fn method(ctx: &Ctx<'_>, call: &Call, out: &mut String) {
    let c = ctx.config;
    let Some(op) = ctx.ops.list.get(call.op) else {
        return;
    };
    let o = &op.operation;
    let optional =
        !o.params.iter().any(|p| p.required) && !o.body.as_ref().is_some_and(|b| b.required);
    let data = format!("Data<typeof {}Response>", call.base);
    let returns = match call.kind {
        Kind::Call => format!("ApiCall<{data}>"),
        Kind::Stream if sse_only(o) => String::from("ApiCall<Stream<ServerSentEvent>>"),
        Kind::Stream => String::from("ApiCall<Stream<unknown>>"),
        Kind::Page => {
            let items = c
                .methods
                .get(call.method)
                .and_then(|m| effective(c, m))
                .and_then(|n| find(c, &n.name))
                .and_then(|p| p.get("items"))
                .unwrap_or("");
            let tuple: Vec<String> = items
                .split('.')
                .filter(|m| !m.is_empty())
                .map(|m| format!("\"{m}\""))
                .collect();
            format!("PagePromise<ItemOf<{data}, [{}]>>", tuple.join(", "))
        }
    };
    let about = op
        .summary
        .as_deref()
        .map(prose::inline)
        .filter(|s| !s.is_empty())
        .or_else(|| op.description.as_deref().and_then(prose::sentence));
    let mut said = about.map(|s| format!(": {s}")).unwrap_or_default();
    if op.deprecated {
        said.push_str(" (deprecated)");
    }
    let _ = writeln!(
        out,
        "- `client.{}(params{}: Input<typeof {}Request>, options?: RequestOptions): {returns}`  \n  `{} {}`{said}",
        call.access,
        if optional { "?" } else { "" },
        call.base,
        op.method(),
        o.path
    );
}
