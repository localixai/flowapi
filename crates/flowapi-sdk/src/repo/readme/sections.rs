//! The README's sections after its usage: the resources, the environments, errors, retries and
//! timeouts, pagination, streaming, logging, a request no operation describes, and how the
//! package is built. Each example calls a method the client has with the input its operation
//! requires, so that `tests/texts.mjs` can compile it; a section is there only when the API has
//! what it is about.

use super::super::{Call, Ctx, Kind, package, texts};
use super::input;
use crate::client::entry::log_env;
use crate::client::resources::access_of;
use std::fmt::Write as _;

/// Every resource with a method, as a table of `client.<resource>` and its count, each linked to
/// its section of `api.md`.
pub(super) fn resources(ctx: &Ctx<'_>, out: &mut String) {
    let c = ctx.config;
    let mut rows = Vec::new();
    for i in in_order(ctx) {
        let n = ctx
            .calls
            .iter()
            .filter(|k| c.methods.get(k.method).is_some_and(|m| m.resource == i))
            .count();
        if n > 0 {
            let access = format!("client.{}", access_of(c, i));
            rows.push(format!(
                "| [`{access}`](api.md#{}) | {n} |",
                anchor(&access)
            ));
        }
    }
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n## Resources\n\n| Resource | Methods |\n| --- | --- |\n{}",
        rows.join("\n")
    );
}

/// The resources in the order of their paths (`pets`, `pets.photos`, `users`), as the README and
/// `api.md` list them.
pub(super) fn in_order(ctx: &Ctx<'_>) -> Vec<usize> {
    let c = ctx.config;
    let mut order: Vec<(String, usize)> = (0..c.resources.len())
        .map(|i| (access_of(c, i), i))
        .collect();
    order.sort();
    order.into_iter().map(|(_, i)| i).collect()
}

/// The anchor GitHub gives a heading: lower case, spaces as `-`, other punctuation left out.
pub(super) fn anchor(heading: &str) -> String {
    heading
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            c if c.is_alphanumeric() || c == '-' || c == '_' => Some(c.to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

pub(super) fn environments(ctx: &Ctx<'_>, out: &mut String) {
    let s = &ctx.config.settings;
    if s.urls.is_empty() {
        return;
    }
    out.push_str("\n## Environments\n\n| Environment | Base URL |\n| --- | --- |\n");
    for (name, url) in &s.urls {
        let _ = writeln!(out, "| `{name}` | {url} |");
    }
    let default = ctx
        .config
        .default_environment
        .as_ref()
        .map(|n| n.name.as_str());
    let first = s.urls.first().map(|(n, _)| n.as_str());
    let chosen = default.or(first).unwrap_or("");
    let _ = write!(
        out,
        "\nThe client calls `{chosen}` unless it is given `environment` or `baseURL`"
    );
    match &s.base_url_env {
        Some(var) => {
            let _ = writeln!(out, ", or `{var}` is set.");
        }
        None => out.push_str(".\n"),
    }
    if let Some((other, _)) = s.urls.iter().find(|(n, _)| n != chosen) {
        let _ = writeln!(
            out,
            "\n```ts\nconst {} = new {}({{ environment: '{other}' }})\n```",
            camel_word(other),
            ctx.client()
        );
    }
}

/// An environment's name as a variable (`sandbox`, `eu_west` is `euWest`).
fn camel_word(name: &str) -> String {
    crate::client::camel(&crate::words::words(name))
}

/// The error classes by status, as `errors.ts` throws them.
const STATUSES: &[(&str, &str)] = &[
    ("400", "BadRequestError"),
    ("401", "AuthenticationError"),
    ("403", "PermissionDeniedError"),
    ("404", "NotFoundError"),
    ("408", "RequestTimeoutError"),
    ("409", "ConflictError"),
    ("422", "UnprocessableEntityError"),
    ("429", "RateLimitError"),
    ("500 and above", "InternalServerError"),
];

pub(super) fn errors(ctx: &Ctx<'_>, headline: Option<&Call>, out: &mut String) {
    out.push_str("\n## Errors\n\nA response whose status is not a success throws an `APIError` with its `status`, `headers`, `body` and `requestId`, of the class for its status (any other status is an `APIError` itself):\n\n| Status | Class |\n| --- | --- |\n");
    for (status, class) in STATUSES {
        let _ = writeln!(out, "| {status} | `{class}` |");
    }
    out.push_str("\nA request that gets no answer throws `APIConnectionError`: `APIConnectionTimeoutError` when an attempt took longer than its `timeout`, `APIUserAbortError` when its `signal` aborted it. These, and every other error the client throws itself, are `SdkError`s. A parameter the description does not allow throws `FlowapiRequestError` before anything is sent.\n");
    if let Some(call) = headline {
        let _ = writeln!(
            out,
            "\n```ts\nimport {{ APIError }} from '{}'\n\ntry {{\n  await client.{}({})\n}} catch (error) {{\n  if (error instanceof APIError) console.log(error.status, error.body)\n  else throw error\n}}\n```",
            ctx.package(),
            call.access,
            input(ctx, call)
        );
    }
}

/// Seconds as the README writes them: `60`, `0.5`.
fn secs(v: Option<f64>, default: f64) -> String {
    let v = v.unwrap_or(default);
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

pub(super) fn retries(ctx: &Ctx<'_>, headline: Option<&Call>, out: &mut String) {
    let s = &ctx.config.settings;
    // With no retry by default, the rules of when a request is retried are said for a caller who
    // sets `maxRetries`, so that no sentence claims a retry the client does not make.
    let n = s.retries.unwrap_or(1);
    let (lead, tries) = match n {
        0 => (
            "No request is retried by default (`maxRetries: 0`): each is sent once. With `maxRetries` set, a",
            String::from("is retried"),
        ),
        1 => ("A", String::from("is retried once by default")),
        n => ("A", format!("is retried {n} times by default")),
    };
    let again = match n {
        0 => "A retry that failed is tried again while `maxRetries` allows.",
        1 => "A retry that failed is not tried again: more retries are the caller's choice.",
        _ => "This SDK's config retries more than once, so a retry that failed is tried again.",
    };
    let others = match &s.idempotency_key {
        Some(h) => format!(
            "A call of any other method (POST, PATCH) sends a random `{h}` header, the same on each of its attempts, so that the server applies it once: it is retried like the others."
        ),
        None => String::from(
            "A POST or a PATCH that failed may have been applied already, so it is retried only when the server's `x-should-retry: true` asks.",
        ),
    };
    let _ = writeln!(
        out,
        "\n## Retries and timeouts\n\n{lead} request that cannot connect, or is answered 408, 409, 429 or 5xx, {tries} when sending it twice does what sending it once does: its method is GET, HEAD, OPTIONS, TRACE, PUT or DELETE. {others} The server's `x-should-retry` header decides first, where it sends one; a 403 is never retried. {again} Before each retry the client waits as long as the server asks in `retry-after-ms` or `Retry-After`, else 0.5 s doubled for each retry made, at most 8 s; when the server asks for more than {} s, the error is thrown at once. Each attempt may take {} s, its body read.\n\n`maxRetries`, `timeout` and `maxRetryDelay` change these for the client, the last two in milliseconds; `maxRetries` and `timeout` change them for one call, in its options:\n\n```ts\nconst patient = new {}({{ maxRetries: 5, timeout: 120_000 }})",
        secs(s.max_retry_delay, 60.0),
        secs(s.timeout, 60.0),
        ctx.client()
    );
    if let Some(call) = headline {
        let arg = input(ctx, call);
        let arg = if arg.is_empty() {
            String::from("{}")
        } else {
            arg
        };
        let _ = writeln!(
            out,
            "await client.{}({arg}, {{ maxRetries: 0, timeout: 5_000 }})",
            call.access
        );
    }
    out.push_str("```\n");
}

pub(super) fn paging(ctx: &Ctx<'_>, out: &mut String) {
    let c = ctx.config;
    let named = c
        .readme
        .iter()
        .find(|(k, _)| *k == "pagination_method")
        .map(|(_, n)| n.name.clone());
    let page = ctx
        .calls
        .iter()
        .find(|k| k.kind == Kind::Page && named.as_ref().is_some_and(|n| *n == k.name))
        .or_else(|| ctx.calls.iter().find(|k| k.kind == Kind::Page));
    if let Some(call) = page {
        let arg = input(ctx, call);
        let _ = writeln!(
            out,
            "\n## Pagination\n\nA list method is read a page at a time. `for await` over it reads every item, asking for the next page when it needs one; awaited, it gives the first page, with its `items`, `hasNextPage()` and `getNextPage()`. A list stops at 10,000 pages unless `maxPages` says otherwise.\n\n```ts\nfor await (const item of client.{0}({arg})) console.log(item)\n\nconst page = await client.{0}({arg})\nconsole.log(page.items, page.hasNextPage())\n```",
            call.access
        );
    }
    if let Some(call) = ctx.calls.iter().find(|k| k.kind == Kind::Stream) {
        let _ = writeln!(
            out,
            "\n## Streaming\n\nA streamed method gives its events as they come, read once; `abort()` stops the stream and closes its connection.\n\n```ts\nconst stream = await client.{}({})\nfor await (const event of stream) console.log(event)\n```",
            call.access,
            input(ctx, call)
        );
    }
}

pub(super) fn logging(ctx: &Ctx<'_>, out: &mut String) {
    let client = ctx.client();
    let _ = writeln!(
        out,
        "\n## Logging\n\nThe client logs to `console` at `warn` and above. Set `{}` to `off`, `error`, `warn`, `info` or `debug`, or give `logLevel`: at `info` it writes a line for each attempt, at `debug` the request's URL and headers too, with every credential written as `***`. `logger` sends the lines elsewhere.\n\n```ts\nconst verbose = new {client}({{ logLevel: 'debug', logger: console }})\n```",
        log_env(&client)
    );
}

pub(super) fn requests(out: &mut String) {
    let _ = writeln!(
        out,
        "\n## Requests no method makes\n\n`client.request(method, path, options)` sends a request no operation of the description has, with the client's base URL, credentials, retries and errors; `asResponse()` gives the raw `Response`:\n\n```ts\nconst response = await client.request('GET', '/').asResponse()\nconsole.log(response.status)\n```"
    );
}

pub(super) fn closing(out: &mut String) {
    let nodes = texts::NODES;
    let _ = writeln!(
        out,
        "\n## Requirements\n\nNode {} or later (CI runs {}), or another runtime with `fetch`. Built with TypeScript {}.\n\n## Development\n\n```sh\nnpm install\nscripts/check   # lint, build, the built types, the tests against the mock server\n```\n\nThe code is generated from `flowapi-sdk.yml` by flowapi: change that file and generate again, as [AGENTS.md](AGENTS.md) says.",
        nodes.first().copied().unwrap_or("20"),
        join_and(nodes),
        package::TYPESCRIPT,
    );
}

/// `20, 22 and 24`.
fn join_and(items: &[&str]) -> String {
    match items.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}
