//! The README's section on servers: the variables of the description's servers and the
//! operations' own, with a default and the values each takes, and how a client reaches a server
//! whose URL is relative to where the description is served (`descriptionURL`). Its examples are
//! compiled against the client by `tests/texts.mjs`.

use super::super::Ctx;
use std::fmt::Write as _;

/// The servers' variables, when they have any, and how to reach a server relative to where the
/// description is served, when one is.
pub(super) fn servers(ctx: &Ctx<'_>, out: &mut String) {
    let variables = crate::client::entry::server_variables(ctx.service);
    let service = ctx.service;
    let all = || {
        service
            .servers
            .iter()
            .chain(service.operations.iter().flat_map(|a| a.servers.iter()))
    };
    let relative = all().find(|s| !s.url.contains(':') && !s.url.starts_with('{'));
    if variables.is_empty() && relative.is_none() {
        return;
    }
    out.push_str("\n## Servers\n");
    if !variables.is_empty() {
        out.push_str("\nThe servers' URLs have variables. The client sends each one's default unless `serverVariables` gives another value:\n\n| Variable | Default | Values |\n| --- | --- | --- |\n");
        for (name, default, values) in &variables {
            let values = if values.is_empty() {
                String::from("any")
            } else {
                values
                    .iter()
                    .map(|v| format!("`{v}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let _ = writeln!(out, "| `{name}` | `{default}` | {values} |");
        }
        let example = variables
            .iter()
            .find_map(|(n, d, vs)| vs.iter().find(|v| *v != d).map(|v| (n, v)))
            .or_else(|| variables.first().map(|(n, d, _)| (n, d)));
        if let Some((name, value)) = example {
            let _ = writeln!(
                out,
                "\n```ts\nconst other = new {}({{ serverVariables: {{ {}: '{value}' }} }})\n```",
                ctx.client(),
                js_key(name)
            );
        }
        if variables.iter().any(|(.., vs)| !vs.is_empty()) {
            out.push_str(
                "\nA value that is not one of a variable's values throws a `RangeError`.\n",
            );
        }
    }
    if let Some(s) = relative {
        let _ = writeln!(
            out,
            "\nA server URL may be relative to where the description is served (`{}`): give that address as `descriptionURL` to call it.\n\n```ts\nconst near = new {}({{ descriptionURL: 'https://api.example.com/openapi.json' }})\n```",
            s.url,
            ctx.client()
        );
    }
}

/// A member's name as a JavaScript object key: as it is when it is an identifier, else quoted.
fn js_key(name: &str) -> String {
    let ident = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if ident {
        name.to_owned()
    } else {
        format!("'{}'", name.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}
