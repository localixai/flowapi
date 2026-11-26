//! The descriptions and the calls, built as `conformance/sdk/run.mjs` builds them for
//! TypeScript: one description per version; each row of the Style Examples table an operation
//! `s<line><where>` with one parameter `color`, each case of cases.json an operation `c<i>` in
//! each of its versions. Each call is the input as the Rust SDK takes it and what it must give.

use serde_json::{Map, Value, json};

/// What a call is checked against.
pub(crate) enum Check {
    /// A cell of the Style Examples table: the part of the request `location` writes.
    Table {
        id: String,
        location: String,
        serialized: String,
    },
    /// A request of a case; `content_type` is the one the case gives for its body.
    Request {
        /// The operation's id, which its path starts with.
        id: String,
        want: Value,
        content_type: Option<String>,
    },
    /// A response of a case.
    Response { want: Value, status: u64 },
}

/// One call: its version, its operation's position there, what the generated crate is given.
pub(crate) struct Call {
    pub(crate) version: String,
    pub(crate) op: usize,
    /// The operation takes no input: `<name>_request()`.
    pub(crate) bare: bool,
    pub(crate) response: bool,
    pub(crate) data: String,
    pub(crate) what: String,
    pub(crate) check: Check,
}

/// The descriptions, by version in the order made, with each one's operation ids in order.
#[derive(Default)]
pub(crate) struct Docs {
    pub(crate) docs: Vec<(String, Value, Vec<String>)>,
}

impl Docs {
    fn add(&mut self, version: &str, id: &str, path: &str, method: &str, op: Value) -> usize {
        if !self.docs.iter().any(|d| d.0 == version) {
            let doc = if version == "2.0" {
                json!({ "swagger": "2.0", "info": { "title": "sdk", "version": "1" }, "paths": {} })
            } else {
                json!({ "openapi": version, "info": { "title": "sdk", "version": "1" }, "paths": {} })
            };
            self.docs.push((version.to_owned(), doc, Vec::new()));
        }
        let Some(entry) = self.docs.iter_mut().find(|d| d.0 == version) else {
            return 0;
        };
        let mut full = Map::new();
        full.insert("operationId".into(), json!(id));
        full.insert(
            "responses".into(),
            json!({ "default": { "description": "any" } }),
        );
        if let Value::Object(o) = op {
            full.extend(o);
        }
        let paths = &mut entry.1["paths"];
        let item = paths
            .as_object_mut()
            .map(|p| p.entry(path.to_owned()).or_insert_with(|| json!({})));
        if let Some(Value::Object(item)) = item {
            if method.bytes().all(|b| b.is_ascii_lowercase()) {
                item.insert(method.to_owned(), Value::Object(full));
            } else {
                let extra = item
                    .entry("additionalOperations")
                    .or_insert_with(|| json!({}));
                if let Value::Object(extra) = extra {
                    extra.insert(method.to_owned(), Value::Object(full));
                }
            }
        }
        entry.2.push(id.to_owned());
        entry.2.len() - 1
    }
}

const VERSIONS: [&str; 3] = ["3.0.4", "3.1.2", "3.2.1"];

/// Where each style goes.
fn locations(style: &str) -> &'static [&'static str] {
    match style {
        "matrix" | "label" => &["path"],
        "simple" => &["path", "header"],
        "cookie" => &["cookie"],
        _ => &["query"],
    }
}

pub(crate) fn table(docs: &mut Docs, table: &Value, calls: &mut Vec<Call>) {
    let rows = table["rows"].as_array().cloned().unwrap_or_default();
    for row in &rows {
        let style = row["style"].as_str().unwrap_or("");
        for &location in locations(style) {
            let id = format!("s{}{}", row["line"], &location[..1]);
            let mut param =
                json!({ "name": "color", "in": location, "style": style, "schema": {} });
            if location == "path" {
                param["required"] = json!(true);
            }
            if !row["explode"].is_null() {
                param["explode"] = row["explode"].clone();
            }
            // After a literal `x`: the table is about the expansion, and a whole segment left
            // empty or `.` (as it shows for undefined) is refused, by the cases.
            let path = if location == "path" {
                format!("/{id}/x{{color}}")
            } else {
                format!("/{id}")
            };
            let op = docs.add("3.2.1", &id, &path, "get", json!({ "parameters": [param] }));
            for ex in row["examples"].as_array().into_iter().flatten() {
                let value = &table["values"][ex["value"].as_str().unwrap_or("")];
                calls.push(Call {
                    version: String::from("3.2.1"),
                    op,
                    bare: false,
                    response: false,
                    data: json!({ "parameters": { location: { "color": value } } }).to_string(),
                    what: format!(
                        "Style Examples line {} ({style}, explode {}, {location}, {})",
                        row["line"],
                        row["explode"],
                        ex["value"].as_str().unwrap_or("")
                    ),
                    check: Check::Table {
                        id: id.clone(),
                        location: location.to_owned(),
                        serialized: ex["serialized"].as_str().unwrap_or("").to_owned(),
                    },
                });
            }
        }
    }
}

/// The cases' operations and calls. `order` names the calls whose expectation the Rust types'
/// maps change (`order.json`): its `expect` is merged over theirs; the count merged is returned.
pub(crate) fn cases(
    docs: &mut Docs,
    prefix: &str,
    cases: &[Value],
    order: &[Value],
    calls: &mut Vec<Call>,
) -> usize {
    let mut merged = 0;
    for (i, c) in cases.iter().enumerate() {
        // A case for other targets alone (a body given as bytes that are not UTF-8, which a
        // call here cannot carry: it gives the body as text).
        if c["targets"]
            .as_array()
            .is_some_and(|t| !t.iter().any(|x| x.as_str() == Some("rust")))
        {
            continue;
        }
        let versions: Vec<String> = c["versions"].as_array().map_or_else(
            || VERSIONS.iter().map(|v| (*v).to_owned()).collect(),
            |vs| {
                vs.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            },
        );
        // Swagger 2.0's `formData` parameters are a form body, given as one.
        let params = c["parameters"].as_array().map_or(&[][..], Vec::as_slice);
        let has_params = params
            .iter()
            .any(|p| p["in"] != "formData" && p["in"] != "body");
        let lifted = json!({ "content": { FORM: {} } });
        let form_data = params.iter().any(|p| p["in"] == "formData");
        let body = c.get("requestBody").or(form_data.then_some(&lifted));
        for version in &versions {
            let mut op = Map::new();
            for k in ["parameters", "requestBody", "responses"] {
                if let Some(v) = c.get(k) {
                    op.insert(k.into(), v.clone());
                }
            }
            let id = format!("{prefix}{i}");
            let path = format!("/{id}{}", c["path"].as_str().unwrap_or(""));
            let method = c["method"].as_str().unwrap_or("get");
            let at = docs.add(version, &id, &path, method, Value::Object(op.clone()));
            let name = c["name"].as_str().unwrap_or("");
            for (j, call) in c["calls"].as_array().into_iter().flatten().enumerate() {
                let what = format!("{name} ({version})");
                let mut want = call["expect"].clone();
                for o in order.iter().filter(|o| o["case"] == name && o["call"] == j) {
                    for (k, v) in o["expect"].as_object().into_iter().flatten() {
                        want[k] = v.clone();
                    }
                    merged += 1;
                }
                if let Some(status) = call["status"].as_u64() {
                    let headers: Vec<(String, Value)> = call["headers"]
                        .as_object()
                        .map(|h| h.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default();
                    let data = json!({ "status": status, "headers": headers, "body": call["body"].as_str().unwrap_or("") });
                    calls.push(Call {
                        version: version.clone(),
                        op: at,
                        bare: false,
                        response: true,
                        data: data.to_string(),
                        what,
                        check: Check::Response { want, status },
                    });
                    continue;
                }
                let (data, content_type) = input(&call["input"], has_params, body);
                let (at, id) = match encoded(&op, &call["input"]["body"]) {
                    Some(op) => {
                        let id = format!("{id}e{j}");
                        let path = format!("/{id}{}", c["path"].as_str().unwrap_or(""));
                        (docs.add(version, &id, &path, method, op), id)
                    }
                    None => (at, id.clone()),
                };
                calls.push(Call {
                    version: version.clone(),
                    op: at,
                    bare: !has_params && body.is_none(),
                    response: false,
                    data,
                    what,
                    check: Check::Request {
                        id,
                        want,
                        content_type,
                    },
                });
            }
        }
    }
    merged
}

/// `application/x-www-form-urlencoded`.
const FORM: &str = "application/x-www-form-urlencoded";

/// The operation with its form body's Encoding Objects as a call gives them, where they differ
/// from the description's: the TypeScript SDK takes them with the body (its client passes the
/// description's), the Rust SDK prints them from the description, so the call gets an operation
/// of its own, `c<i>e<j>`.
fn encoded(op: &Map<String, Value>, body: &Value) -> Option<Value> {
    let written = op.get("requestBody")?["content"].get(FORM)?;
    if body["contentType"] != FORM || written.get("encoding") == body.get("encoding") {
        return None;
    }
    let mut op = Value::Object(op.clone());
    let media = op["requestBody"]["content"][FORM].as_object_mut()?;
    match body.get("encoding") {
        Some(e) => media.insert("encoding".into(), e.clone()),
        None => media.remove("encoding"),
    };
    Some(op)
}

/// A case's input as the Rust SDK takes it: `parameters` by location, and `body` an enum
/// tagged by the media type as the description writes it.
fn input(input: &Value, has_params: bool, body: Option<&Value>) -> (String, Option<String>) {
    let mut out = Map::new();
    let mut params = Map::new();
    for (k, v) in input.as_object().into_iter().flatten() {
        if k != "body" {
            params.insert(k.clone(), v.clone());
        }
    }
    if has_params {
        out.insert("parameters".into(), Value::Object(params));
    }
    let mut given = None;
    if let (Some(b), Some(content)) = (
        input.get("body"),
        body.and_then(|b| b["content"].as_object()),
    ) {
        let ct = b["contentType"].as_str().unwrap_or("");
        let key = content
            .keys()
            .find(|k| k.as_str() == ct)
            .or_else(|| content.keys().find(|k| bare(k) == bare(ct)));
        if let Some(key) = key {
            let value = &b["value"];
            let text = || {
                value
                    .as_str()
                    .map_or_else(|| value.to_string(), str::to_owned)
            };
            let k = bare(key);
            let v = if k.contains('*') || k.starts_with("multipart/") {
                json!([ct, text().into_bytes()])
            } else if k == "application/json"
                || k.ends_with("+json")
                || k == "application/x-www-form-urlencoded"
            {
                value.clone()
            } else {
                json!(text().into_bytes())
            };
            out.insert("body".into(), json!({ key.as_str(): v }));
            given = Some(ct.to_owned());
        }
    }
    (Value::Object(out).to_string(), given)
}

pub(crate) fn bare(t: &str) -> String {
    t.split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}
