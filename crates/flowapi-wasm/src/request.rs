//! A request read from its JSON: the documents, the description first, and the options.

use flowapi::id::NodeId;
use flowapi::json::Tree;
use std::borrow::Cow;

/// One document: the URI its references resolve against, and its text.
pub(crate) struct Doc<'t> {
    pub(crate) uri: Cow<'t, str>,
    pub(crate) text: Cow<'t, str>,
}

/// What to generate.
pub(crate) struct Request<'t> {
    /// The description first, then every document its references reach.
    pub(crate) documents: Vec<Doc<'t>>,
    /// The path of the file written, as `-o` gives it to the CLI.
    pub(crate) output: Cow<'t, str>,
    /// Whether the SDK is written after the types.
    pub(crate) sdk: bool,
}

/// The path written when the request names none.
const OUTPUT: &str = "types.ts";

/// Reads a request; the error says what it lacks.
pub(crate) fn read<'t>(tree: &Tree<'t>) -> Result<Request<'t>, String> {
    let root = tree.root();
    let list = tree
        .get(root, "documents")
        .ok_or("the request has no `documents`")?;
    let mut documents = Vec::new();
    for (i, &node) in tree.items(list).iter().enumerate() {
        let uri = string(tree, node, "uri").ok_or_else(|| format!("document {i} has no `uri`"))?;
        let text =
            string(tree, node, "text").ok_or_else(|| format!("document {i} has no `text`"))?;
        documents.push(Doc { uri, text });
    }
    if documents.is_empty() {
        return Err(String::from("the request has no document"));
    }
    let options = tree.get(root, "options");
    let output = options
        .and_then(|options| string(tree, options, "output"))
        .unwrap_or(Cow::Borrowed(OUTPUT));
    // `sdk: true` adds the request and response functions after the types, as `--sdk` does.
    let sdk = options
        .and_then(|o| tree.get(o, "sdk"))
        .and_then(|n| tree.as_bool(n))
        == Some(true);
    Ok(Request {
        documents,
        output,
        sdk,
    })
}

/// An object's member `key`, when it is a string.
fn string<'t>(tree: &Tree<'t>, node: NodeId, key: &str) -> Option<Cow<'t, str>> {
    tree.get(node, key).and_then(|n| tree.as_str(n))
}
