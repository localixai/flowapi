//! The pagination schemes a `GET` operation looks like it has, each with its reason and how sure
//! the guess is (study 02, section 1.4): signals in the parameters' names (`cursor`, `page_token`,
//! `offset`, `page`, `limit`), in the 2xx JSON body (an array of items, `next_cursor`,
//! `has_more`, `total`) and in its headers (`Link`). A signal is a point; two make a suggestion,
//! three a sure one. Names are compared by role, not by spelling: the request's and the
//! response's names need not match.

use super::schema::{self, Prop, Types};
use super::{Paging, fit};
use crate::doc::{At, Doc};
use flowapi::spec::{Location, Operation};

const LIMIT: &[&str] = &[
    "limit",
    "perpage",
    "pagesize",
    "size",
    "count",
    "maxresults",
    "top",
    "maxitems",
];
const CURSOR: &[&str] = &[
    "cursor",
    "after",
    "startingafter",
    "pagetoken",
    "nexttoken",
    "continuation",
    "continuationtoken",
    "afterid",
    "startcursor",
];
const OFFSET: &[&str] = &["offset", "skip"];
const PAGE: &[&str] = &["page", "pagenumber", "pageno"];
const NEXT_TOKEN: &[&str] = &[
    "nextcursor",
    "nextpagetoken",
    "nexttoken",
    "endcursor",
    "continuationtoken",
    "cursor",
];
const NEXT_URL: &[&str] = &[
    "next",
    "nextlink",
    "odatanextlink",
    "nexturl",
    "nextpageurl",
];
const HAS_MORE: &[&str] = &["hasmore", "hasnext", "hasnextpage"];
const TOTAL: &[&str] = &["total", "totalcount", "totalresults", "totalitems"];
const TOTAL_PAGES: &[&str] = &["totalpages", "pagecount"];
const ITEMS: &[&str] = &[
    "data", "items", "results", "result", "records", "entries", "values", "nodes",
];

/// A suggested scheme: its members, its points, and why.
#[derive(Debug)]
pub(crate) struct Suggestion {
    pub(crate) paging: Paging,
    pub(crate) score: u32,
    pub(crate) reason: String,
}

impl Suggestion {
    /// How sure the guess is: `high` with three signals or more, else `medium`.
    pub(crate) fn confidence(&self) -> &'static str {
        if self.score >= 3 { "high" } else { "medium" }
    }
}

/// A name as roles are matched: lower case, letters and digits only (`page_size`, `pageSize`
/// and `$top` are `pagesize` and `top`).
fn norm(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The body's members, at the top and one level down, each with its dotted path.
fn fields(doc: &Doc<'_, '_>, root: At) -> Vec<(String, String, At)> {
    let mut out = Vec::new();
    for (name, at) in schema::properties(doc, root).into_iter().take(200) {
        if schema::types(doc, at) == Types::Known(vec![String::from("object")]) {
            for (inner, at) in schema::properties(doc, at).into_iter().take(200) {
                out.push((format!("{name}.{inner}"), norm(&inner), at));
            }
        }
        out.push((name.clone(), norm(&name), at));
    }
    out
}

/// The array of items, its path with its schema: the body itself (no path), else a member of the top named as items are (by
/// the order of [`ITEMS`]) or the one array there, else one so named under `data` or `result`.
fn items(
    doc: &Doc<'_, '_>,
    root: At,
    fields: &[(String, String, At)],
) -> Option<(Option<String>, At)> {
    if schema::types(doc, root).allows(&["array"]) && !schema::types(doc, root).allows(&["object"])
    {
        return Some((None, root));
    }
    let mut arrays: Vec<&(String, String, At)> = fields
        .iter()
        .filter(|(_, _, at)| typed(doc, *at, &["array"]))
        .collect();
    arrays.sort_by(|a, b| a.0.cmp(&b.0));
    let named = |nested: bool| {
        ITEMS.iter().find_map(|n| {
            arrays
                .iter()
                .filter(|(p, _, _)| p.contains('.') == nested)
                .filter(|(p, _, _)| !nested || p.starts_with("data.") || p.starts_with("result."))
                .find(|(_, norm, _)| norm == n)
                .map(|f| (f.0.clone(), f.2))
        })
    };
    if let Some(found) = named(false) {
        return Some((Some(found.0), found.1));
    }
    let top: Vec<&&(String, String, At)> =
        arrays.iter().filter(|(p, _, _)| !p.contains('.')).collect();
    if let [one] = top.as_slice() {
        return Some((Some(one.0.clone()), one.2));
    }
    named(true).map(|(p, at)| (Some(p), at))
}

/// The parameter (not in the path, not required) of the first of `names` one has.
fn param<'o>(op: &'o Operation, names: &[&str]) -> Option<&'o str> {
    names.iter().find_map(|n| {
        op.params
            .iter()
            .filter(|p| p.location != Location::Path && !p.required)
            .find(|p| norm(&p.name) == *n)
            .map(|p| p.name.as_str())
    })
}

/// Whether a schema says it is one of `types`.
fn typed(doc: &Doc<'_, '_>, at: At, types: &[&str]) -> bool {
    matches!(schema::types(doc, at), Types::Known(t) if t.iter().any(|t| types.contains(&t.as_str())))
}

/// The body member of a type of `types` whose last name is the first of `names` one has; of
/// equals, the first by path: the same whatever order the schema writes its properties in.
fn field(
    doc: &Doc<'_, '_>,
    fields: &[(String, String, At)],
    names: &[&str],
    types: &[&str],
) -> Option<String> {
    names.iter().find_map(|n| {
        fields
            .iter()
            .filter(|(_, norm, at)| norm == n && typed(doc, *at, types))
            .map(|(p, _, _)| p.clone())
            .min()
    })
}

/// The schemes `op` looks like it has, the likeliest first.
pub(crate) fn suggest(doc: &Doc<'_, '_>, op: &Operation) -> Vec<Suggestion> {
    if op.method != "get" {
        return Vec::new();
    }
    let Some(root) = fit::body(op) else {
        return Vec::new();
    };
    let fields = fields(doc, root);
    let Some(list) = items(doc, root, &fields) else {
        return Vec::new();
    };
    let (items_path, items_at) = (list.0.as_deref(), list.1);
    let s = Signals {
        limit: param(op, LIMIT),
        cursor: param(op, CURSOR),
        offset: param(op, OFFSET),
        page: param(op, PAGE),
        next_token: field(doc, &fields, NEXT_TOKEN, &["string", "integer"]),
        next_url: field(doc, &fields, NEXT_URL, &["string"]),
        has_more: field(doc, &fields, HAS_MORE, &["boolean"]),
        total: field(doc, &fields, TOTAL, &["integer", "number"]),
        total_pages: field(doc, &fields, TOTAL_PAGES, &["integer", "number"]),
        link: op
            .responses
            .iter()
            .filter(|r| r.status.starts_with('2'))
            .any(|r| {
                r.headers
                    .iter()
                    .any(|h| h.name.eq_ignore_ascii_case("link"))
            }),
        has_id: schema::items(doc, items_at)
            .is_some_and(|i| matches!(schema::property(doc, i, "id"), Prop::Found(_))),
        items: items_path,
    };
    let mut out: Vec<Suggestion> = s.verdicts().into_iter().filter(|v| v.score >= 2).collect();
    out.retain(|v| fit::fit(doc, op, &v.paging).iter().all(|p| p.warning));
    out.sort_by_key(|a| std::cmp::Reverse(a.score));
    out
}

/// What an operation's names say.
struct Signals<'o> {
    limit: Option<&'o str>,
    cursor: Option<&'o str>,
    offset: Option<&'o str>,
    page: Option<&'o str>,
    next_token: Option<String>,
    next_url: Option<String>,
    has_more: Option<String>,
    total: Option<String>,
    total_pages: Option<String>,
    link: bool,
    has_id: bool,
    items: Option<&'o str>,
}

impl Signals<'_> {
    /// A suggestion of `kind`: the items and the limit added, a point each for the items and the
    /// limit.
    fn make(
        &self,
        kind: &'static str,
        req: &[(&str, &str)],
        res: &[(&str, &str)],
        score: u32,
        why: &str,
    ) -> Suggestion {
        let mut res = res.to_vec();
        if let Some(p) = self.items {
            res.push(("items", p));
        }
        let mut req = req.to_vec();
        if let Some(l) = self.limit {
            req.push((if kind == "page" { "per_page" } else { "limit" }, l));
        }
        let items = self.items.map_or_else(
            || String::from("the body is the array of items"),
            |p| format!("items in `{p}`"),
        );
        Suggestion {
            paging: Paging::new(kind, &req, &res),
            score: score + 1 + u32::from(self.limit.is_some()),
            reason: format!("{why}; {items}"),
        }
    }

    fn verdicts(&self) -> Vec<Suggestion> {
        let mut out = Vec::new();
        if let (Some(c), Some(n)) = (self.cursor, &self.next_token) {
            let mut res = vec![("next", n.as_str())];
            if let Some(h) = &self.has_more {
                res.push(("has_more", h.as_str()));
            }
            out.push(self.make(
                "cursor",
                &[("cursor", c)],
                &res,
                2,
                &format!("the parameter `{c}` and `{n}` in the response"),
            ));
        } else if let (Some(c), Some(h), true) = (self.cursor, &self.has_more, self.has_id) {
            let why = format!("the parameter `{c}`, `{h}` in the response and items with an `id`");
            out.push(self.make(
                "last_item_id",
                &[("cursor", c)],
                &[("has_more", h.as_str())],
                2,
                &why,
            ));
        }
        if let Some(o) = self.offset {
            let res = self
                .total
                .as_deref()
                .map(|t| vec![("total", t)])
                .unwrap_or_default();
            let score = 1 + u32::from(self.total.is_some());
            out.push(self.make(
                "offset",
                &[("offset", o)],
                &res,
                score,
                &format!("the parameter `{o}`"),
            ));
        }
        if let Some(p) = self.page {
            let mut res = Vec::new();
            if let Some(t) = &self.total_pages {
                res.push(("total_pages", t.as_str()));
            }
            if let Some(h) = &self.has_more {
                res.push(("has_more", h.as_str()));
            }
            let score = 1 + u32::from(!res.is_empty());
            out.push(self.make(
                "page",
                &[("page", p)],
                &res,
                score,
                &format!("the parameter `{p}`"),
            ));
        }
        if self.link {
            out.push(self.make(
                "link",
                &[],
                &[("header", "Link")],
                2,
                "a `Link` header in the response",
            ));
        } else if let (Some(n), None) = (&self.next_url, self.cursor) {
            out.push(self.make(
                "link",
                &[],
                &[("next", n.as_str())],
                1,
                &format!("`{n}` in the response"),
            ));
        }
        out
    }
}
