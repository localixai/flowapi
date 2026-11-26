//! `flowapi summary`: a description told to an agent in a few kilobytes of JSON, so that it can
//! write a config without reading the description: its title and version, servers, security
//! schemes, the description's `security` (each alternative the schemes it needs), counts, tags with their operations' counts, and each operation (how a config names
//! it, method, path, tags, summary, its own `security` where it has one, parameters, the media types of its body and of each response,
//! and the pagination it looks like it has, with why and how sure). Over the budget, the tags
//! alone, and how to read on: by `--tag` or `--path`.

use crate::model::{Auth, Requirement, Scheme};
use crate::ops::{Op, Ops};
use crate::out::{json_list, json_str, short};
use crate::paging::suggest;
use crate::words;
use flowapi::{Diagnostics, Documents};
use std::fmt::Write as _;

/// Which operations a summary shows, and its budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    /// Only operations with this tag (equal as words).
    pub tag: Option<String>,
    /// Only operations whose path opens with this.
    pub path: Option<String>,
    /// The most bytes the summary takes; over it, the tags alone.
    pub max_bytes: usize,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            tag: None,
            path: None,
            max_bytes: 60_000,
        }
    }
}

impl Filter {
    fn shows(&self, op: &Op) -> bool {
        let tag = self
            .tag
            .as_ref()
            .is_none_or(|t| op.tags.iter().any(|o| o == t || words::same(o, t)));
        let path = self
            .path
            .as_ref()
            .is_none_or(|p| op.operation.path.starts_with(p.as_str()));
        tag && path
    }
}

/// The summary of the description in `docs`, as JSON; `None` when the first document is not an
/// OpenAPI or Swagger description.
#[must_use]
pub fn summary(docs: &Documents<'_>, filter: &Filter) -> Option<String> {
    let (ops, _) = Ops::read(docs);
    let ops = ops?;
    let shown: Vec<usize> = (0..ops.list.len())
        .filter(|&i| ops.list.get(i).is_some_and(|op| filter.shows(op)))
        .collect();
    let mut head = format!(
        "{{\"info\":{{\"title\":{},\"version\":{}}}",
        json_str(&ops.info("title").unwrap_or_default()),
        json_str(&ops.info("version").unwrap_or_default())
    );
    let servers: Vec<String> = ops
        .doc
        .servers(&mut Diagnostics::default())
        .into_iter()
        .map(|s| s.url)
        .collect();
    let _ = write!(head, ",\n\"servers\":{}", json_list(&servers));
    let schemes: Vec<String> = ops
        .doc
        .schemes(&mut Diagnostics::default())
        .iter()
        .map(scheme)
        .collect();
    let _ = write!(head, ",\n\"securitySchemes\":{{{}}}", schemes.join(","));
    let root = ops
        .doc
        .security(crate::doc::ROOT, &mut Diagnostics::default())
        .unwrap_or_default();
    let _ = write!(head, ",\n\"security\":{}", requirements(&root));
    let tags = tags(&ops);
    let _ = write!(
        head,
        ",\n\"counts\":{{\"operations\":{},\"shown\":{},\"tags\":{},\"schemas\":{}}}",
        ops.list.len(),
        shown.len(),
        tags.len(),
        ops.schema_names.len()
    );
    if filter.tag.is_some() || filter.path.is_some() {
        let f = |v: &Option<String>| v.as_deref().map_or_else(|| String::from("null"), json_str);
        let _ = write!(
            head,
            ",\n\"filter\":{{\"tag\":{},\"path\":{}}}",
            f(&filter.tag),
            f(&filter.path)
        );
    }
    let index: Vec<String> = tags
        .iter()
        .map(|(t, n)| format!("{{\"name\":{},\"operations\":{n}}}", json_str(t)))
        .collect();
    let _ = write!(head, ",\n\"tags\":[{}]", index.join(","));
    let lines: Vec<String> = shown.iter().map(|&i| operation(&ops, i)).collect();
    let body = format!(",\n\"operations\":[\n{}\n]}}\n", lines.join(",\n"));
    if head.len() + body.len() <= filter.max_bytes {
        return Some(head + &body);
    }
    let _ = write!(
        head,
        ",\n\"truncated\":{{\"bytes\":{},\"max\":{},\"next\":{}}}}}\n",
        head.len() + body.len(),
        filter.max_bytes,
        json_str(
            "the operations are over the budget and left out: read one tag at a time with --tag <name>, or a part of the API with --path <prefix>, or raise --max-bytes"
        )
    );
    Some(head)
}

/// Each tag the operations have, with how many have it, in the order first met; `""` counts the
/// operations with none.
fn tags(ops: &Ops<'_, '_>) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for op in &ops.list {
        let own: Vec<String> = if op.tags.is_empty() {
            vec![String::new()]
        } else {
            op.tags.clone()
        };
        for t in own {
            match out.iter_mut().find(|(n, _)| *n == t) {
                Some((_, count)) => *count += 1,
                None => out.push((t, 1)),
            }
        }
    }
    out
}

/// A security scheme as `"name": {"type": …}`.
fn scheme(s: &Scheme) -> String {
    let fields = match &s.auth {
        Auth::ApiKey { location, name } => {
            format!(
                "\"type\":\"apiKey\",\"in\":{},\"name\":{}",
                json_str(location.name()),
                json_str(name)
            )
        }
        Auth::Http { scheme, .. } => format!("\"type\":\"http\",\"scheme\":{}", json_str(scheme)),
        Auth::OAuth2 {
            flows,
            client_credentials,
        } => {
            let token = client_credentials
                .as_ref()
                .map(|f| format!(",\"tokenUrl\":{}", json_str(&f.token_url)))
                .unwrap_or_default();
            format!("\"type\":\"oauth2\",\"flows\":{}{token}", json_list(flows))
        }
        Auth::OpenIdConnect { .. } => String::from("\"type\":\"openIdConnect\""),
        Auth::MutualTls => String::from("\"type\":\"mutualTLS\""),
    };
    format!("{}:{{{fields}}}", json_str(&s.name))
}

/// Security requirements compactly: each alternative the names of the schemes it needs
/// together, `[]` for one that needs none.
fn requirements(list: &[Requirement]) -> String {
    let alternatives: Vec<String> = list
        .iter()
        .map(|r| {
            let names: Vec<String> = r.schemes.iter().map(|(n, _)| n.clone()).collect();
            json_list(&names)
        })
        .collect();
    format!("[{}]", alternatives.join(","))
}

/// One operation, on one line.
fn operation(ops: &Ops<'_, '_>, i: usize) -> String {
    let Some(op) = ops.list.get(i) else {
        return String::from("{}");
    };
    let o = &op.operation;
    let mut out = format!(
        "{{\"id\":{},\"method\":{},\"path\":{}",
        json_str(&ops.id(i)),
        json_str(&op.method()),
        json_str(&o.path)
    );
    if !op.tags.is_empty() {
        let _ = write!(out, ",\"tags\":{}", json_list(&op.tags));
    }
    if let Some(s) = &op.summary {
        let _ = write!(out, ",\"summary\":{}", json_str(&short(s, 120)));
    }
    if o.deprecated {
        out.push_str(",\"deprecated\":true");
    }
    let root = ops
        .doc
        .security(crate::doc::ROOT, &mut Diagnostics::default())
        .unwrap_or_default();
    let (security, inherits) = ops.doc.effective(o, &root, &mut Diagnostics::default());
    if !inherits {
        let _ = write!(out, ",\"security\":{}", requirements(&security));
    }
    let params: Vec<String> = o
        .params
        .iter()
        .map(|p| {
            format!(
                "{{\"name\":{},\"in\":{},\"required\":{}}}",
                json_str(&p.name),
                json_str(p.location.name()),
                p.required
            )
        })
        .collect();
    if !params.is_empty() {
        let _ = write!(out, ",\"params\":[{}]", params.join(","));
    }
    if let Some(b) = &o.body {
        let media: Vec<&str> = b.content.iter().map(|m| m.media_type.as_str()).collect();
        let _ = write!(
            out,
            ",\"body\":{{\"required\":{},\"content\":{}}}",
            b.required,
            json_list(&media)
        );
    }
    let responses: Vec<String> = o
        .responses
        .iter()
        .map(|r| {
            let media: Vec<&str> = r.content.iter().map(|m| m.media_type.as_str()).collect();
            format!("{}:{}", json_str(&r.status), json_list(&media))
        })
        .collect();
    let _ = write!(out, ",\"responses\":{{{}}}", responses.join(","));
    let pages: Vec<String> = suggest::suggest(&ops.doc, o)
        .iter()
        .map(|s| {
            format!(
                "{{\"scheme\":{},\"fields\":{},\"confidence\":{},\"reason\":{}}}",
                json_str(s.paging.kind),
                s.paging.json(),
                json_str(s.confidence()),
                json_str(&s.reason)
            )
        })
        .collect();
    if !pages.is_empty() {
        let _ = write!(out, ",\"pagination\":[{}]", pages.join(","));
    }
    out.push('}');
    out
}
