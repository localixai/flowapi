//! The commands about an SDK's config, `flowapi_sdk`'s: `summary`, `draft`, `schema`, `spec` and
//! `check`, the `flowapi-sdk` functions the command's `summary` and `config draft|schema|check`
//! call (`crates/flowapi-cli/src/sdk.rs`), over the documents and the config the host read. The
//! host keeps what is I/O, as the command does: the files, a draft's `spec` path relative to where
//! it is written, a config's `spec` relative to the config.
//!
//! The request names its command and what it reads:
//!
//! ```json
//! { "command": "summary", "documents": [{ "uri": "openapi.yaml", "text": "…" }],
//!   "filter": { "tag": "pets", "path": "/pets", "maxBytes": 60000 } }
//! { "command": "draft", "documents": […], "spec": "../openapi.yaml" }
//! { "command": "schema" }
//! { "command": "spec", "config": { "uri": "flowapi-sdk.yml", "text": "…" } }
//! { "command": "check", "documents": […], "config": { "uri": "flowapi-sdk.yml", "text": "…" } }
//! ```
//!
//! The answer: `text` the command's output (`null` when the documents are not an OpenAPI or
//! Swagger description, or the config names no `spec`); for `check`, `text` the issues as the
//! command prints them and `json` as `--json` prints them; `diagnostics` the document or config
//! that could not be read, as `generate` gives them, with no output:
//!
//! ```json
//! { "text": "…", "json": null, "diagnostics": [] }
//! ```

use crate::answer;
use crate::request::{self, Doc};
use crate::{Source, found, read_all, unread};
use flowapi::id::NodeId;
use flowapi::json::{self, Tree, Value};
use flowapi::{Diagnostic, Documents};
use flowapi_sdk::Filter;
use flowapi_yaml::Text;
use std::borrow::Cow;

/// What a request asks for.
enum Command<'t> {
    Summary(Vec<Doc<'t>>, Filter),
    Draft(Vec<Doc<'t>>, Option<Cow<'t, str>>),
    Schema,
    Spec(Doc<'t>),
    Check(Vec<Doc<'t>>, Doc<'t>),
}

/// Runs the command a request names (the module's documentation shows them) and returns the
/// answer as JSON. Never fails: a request that cannot be read is an answer with one error.
#[must_use]
pub fn sdk(request: &[u8]) -> String {
    let Ok(text) = std::str::from_utf8(request) else {
        return refused("the request is not UTF-8");
    };
    let tree = match json::parse(text) {
        Ok(tree) => tree,
        Err(e) => return refused(&format!("the request is not JSON: {}", e.message)),
    };
    match read(&tree) {
        Ok(command) => run(command),
        Err(message) => refused(&message),
    }
}

fn run(command: Command<'_>) -> String {
    match command {
        Command::Schema => answer::sdk(Some(&flowapi_sdk::config_schema()), None, &[]),
        Command::Spec(config) => match Text::read(&config.uri, config.text) {
            Ok(text) => answer::sdk(flowapi_sdk::spec_of(text.json()).as_deref(), None, &[]),
            Err(e) => answer::sdk(None, None, &[unread(&config.uri, &e)]),
        },
        Command::Summary(documents, filter) => {
            with_documents(documents, |docs| flowapi_sdk::summary(docs, &filter))
        }
        Command::Draft(documents, spec) => {
            let spec = spec.unwrap_or_else(|| {
                documents
                    .first()
                    .map_or(Cow::Borrowed(""), |d| d.uri.clone())
            });
            with_documents(documents, |docs| flowapi_sdk::draft(docs, &spec))
        }
        Command::Check(documents, config) => check(documents, &config),
    }
}

/// The text a function gives for the documents read, or the diagnostic of the first that
/// cannot be.
fn with_documents(
    documents: Vec<Doc<'_>>,
    f: impl FnOnce(&Documents<'_>) -> Option<String>,
) -> String {
    let sources = match read_all(documents) {
        Ok(s) => s,
        Err((uri, e)) => return answer::sdk(None, None, &[unread(&uri, &e)]),
    };
    match add(&sources) {
        Ok(docs) => answer::sdk(f(&docs).as_deref(), None, &[]),
        Err(d) => answer::sdk(None, None, &[found(&sources, &d)]),
    }
}

/// The documents given to the core, as `generate` gives them; the diagnostic of one refused.
fn add<'s>(sources: &'s [Source<'_>]) -> Result<Documents<'s>, Diagnostic> {
    let mut docs = Documents::new();
    for source in sources {
        docs.add(source.uri.as_ref(), source.text.json())?;
    }
    Ok(docs)
}

/// `check`: the config read first, as the command reads it before its description; the issues
/// as text (the config's URI naming it) and as JSON, at the config's own lines.
fn check(documents: Vec<Doc<'_>>, config: &Doc<'_>) -> String {
    let text = match Text::read(&config.uri, config.text.clone()) {
        Ok(text) => text,
        Err(e) => return answer::sdk(None, None, &[unread(&config.uri, &e)]),
    };
    let sources = match read_all(documents) {
        Ok(s) => s,
        Err((uri, e)) => return answer::sdk(None, None, &[unread(&uri, &e)]),
    };
    let docs = match add(&sources) {
        Ok(docs) => docs,
        Err(d) => return answer::sdk(None, None, &[found(&sources, &d)]),
    };
    let issues = flowapi_sdk::check(&docs, text.json());
    let place = |offset: u32| text.place(offset);
    let printed = flowapi_sdk::issues_text(&issues, &config.uri, &place);
    let json = flowapi_sdk::issues_json(&issues, &place);
    answer::sdk(Some(&printed), Some(&json), &[])
}

fn refused(message: &str) -> String {
    answer::sdk(None, None, &[answer::refusal(message)])
}

/// Reads a request; the error says what it lacks.
fn read<'t>(tree: &Tree<'t>) -> Result<Command<'t>, String> {
    let root = tree.root();
    let command = request::string(tree, root, "command").ok_or("the request has no `command`")?;
    let described = || -> Result<Vec<Doc<'t>>, String> {
        let documents = request::documents(tree, root)?;
        if documents.is_empty() {
            return Err(String::from("the request has no document"));
        }
        Ok(documents)
    };
    let config = || -> Result<Doc<'t>, String> {
        let node = tree
            .get(root, "config")
            .ok_or("the request has no `config`")?;
        request::document(tree, node).map_err(|lacks| format!("the config {lacks}"))
    };
    Ok(match command.as_ref() {
        "summary" => Command::Summary(described()?, filter(tree, root)?),
        "draft" => Command::Draft(described()?, request::string(tree, root, "spec")),
        "schema" => Command::Schema,
        "spec" => Command::Spec(config()?),
        "check" => {
            // No description is a config that names none: checked against nothing, as the
            // command checks it.
            let documents = match tree.get(root, "documents") {
                Some(_) => request::documents(tree, root)?,
                None => Vec::new(),
            };
            Command::Check(documents, config()?)
        }
        other => {
            return Err(format!(
                "`{other}` is not a command: `summary`, `draft`, `schema`, `spec` or `check`"
            ));
        }
    })
}

/// `summary`'s filter: `tag`, `path` and `maxBytes` (a whole number, written as a number or a
/// string of digits; past what the module can address, no bound), each optional.
fn filter(tree: &Tree<'_>, root: NodeId) -> Result<Filter, String> {
    let mut filter = Filter::default();
    let Some(node) = tree.get(root, "filter") else {
        return Ok(filter);
    };
    filter.tag = request::string(tree, node, "tag").map(Cow::into_owned);
    filter.path = request::string(tree, node, "path").map(Cow::into_owned);
    if let Some(n) = tree.get(node, "maxBytes") {
        let digits = match tree.value(n) {
            Value::Number(span) => Cow::Borrowed(tree.raw(span)),
            Value::String(_) => tree.as_str(n).unwrap_or_default(),
            Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => Cow::Borrowed(""),
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!(
                "`filter.maxBytes` is a whole number of bytes, not `{digits}`"
            ));
        }
        filter.max_bytes = digits.parse().unwrap_or(usize::MAX);
    }
    Ok(filter)
}
