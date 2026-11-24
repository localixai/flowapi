//! The tree a document is read into: one arena of nodes, arrays' items and objects' members in
//! two flat lists, each node naming its range in them.

use super::text;
use crate::id::NodeId;
use std::borrow::Cow;

/// A range of the document's text, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// The first byte.
    pub start: u32,
    /// One past the last byte.
    pub end: u32,
}

/// A node's value. Strings and numbers are their raw tokens; arrays and objects are ranges of
/// the tree's item and member lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number's token, as written.
    Number(Span),
    /// A string's token, its quotes included.
    String(Span),
    /// An array's items: a range of the item list.
    Array(Span),
    /// An object's members: a range of the member list.
    Object(Span),
}

/// An object's member: its key's token (quotes included) and its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    /// The key's token.
    pub key: Span,
    /// The value.
    pub value: NodeId,
}

/// A read document.
#[derive(Debug)]
pub struct Tree<'t> {
    pub(super) text: &'t str,
    pub(super) values: Vec<Value>,
    pub(super) offsets: Vec<u32>,
    pub(super) parents: Vec<Option<NodeId>>,
    pub(super) items: Vec<NodeId>,
    pub(super) members: Vec<Member>,
}

impl<'t> Tree<'t> {
    /// The root node.
    #[must_use]
    pub fn root(&self) -> NodeId {
        NodeId::FIRST
    }

    /// The document's text.
    #[must_use]
    pub fn text(&self) -> &'t str {
        self.text
    }

    /// How many nodes the tree holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the tree is empty (never, once read).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// A node's value; `Null` for an id of another tree.
    #[must_use]
    pub fn value(&self, node: NodeId) -> Value {
        self.values
            .get(node.index())
            .copied()
            .unwrap_or(Value::Null)
    }

    /// The byte offset of a node's first byte.
    #[must_use]
    pub fn offset(&self, node: NodeId) -> u32 {
        self.offsets.get(node.index()).copied().unwrap_or(0)
    }

    /// The node an array item or object member belongs to; `None` for the root.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.parents.get(node.index()).copied().flatten()
    }

    /// An array's items; empty for any other node.
    #[must_use]
    pub fn items(&self, node: NodeId) -> &[NodeId] {
        match self.value(node) {
            Value::Array(range) => slice(&self.items, range),
            Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Object(_) => &[],
        }
    }

    /// An object's members, in the order written; empty for any other node.
    #[must_use]
    pub fn members(&self, node: NodeId) -> &[Member] {
        match self.value(node) {
            Value::Object(range) => slice(&self.members, range),
            Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Array(_) => &[],
        }
    }

    /// The value of an object's member `key`; the last one when a key is written twice.
    #[must_use]
    pub fn get(&self, node: NodeId, key: &str) -> Option<NodeId> {
        self.members(node)
            .iter()
            .rev()
            .find(|m| self.key_is(m.key, key))
            .map(|m| m.value)
    }

    /// Whether a key's token decodes to `key`.
    #[must_use]
    pub fn key_is(&self, token: Span, key: &str) -> bool {
        let raw = self.raw(token);
        let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or("");
        if inner.contains('\\') {
            text::decode(inner).as_deref() == Some(key)
        } else {
            inner == key
        }
    }

    /// A token's text, as written.
    #[must_use]
    pub fn raw(&self, span: Span) -> &'t str {
        self.text
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
    }

    /// A string token's value, decoded; `None` when it holds a lone surrogate (`"\ud800"`), which
    /// JSON allows and a Rust string cannot hold.
    #[must_use]
    pub fn string(&self, span: Span) -> Option<Cow<'t, str>> {
        let raw = self.raw(span);
        let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or("");
        if inner.contains('\\') {
            text::decode(inner).map(Cow::Owned)
        } else {
            Some(Cow::Borrowed(inner))
        }
    }

    /// A node's string value, if it is a string.
    #[must_use]
    pub fn as_str(&self, node: NodeId) -> Option<Cow<'t, str>> {
        match self.value(node) {
            Value::String(span) => self.string(span),
            Value::Null
            | Value::Bool(_)
            | Value::Number(_)
            | Value::Array(_)
            | Value::Object(_) => None,
        }
    }

    /// A node's boolean value, if it is a boolean.
    #[must_use]
    pub fn as_bool(&self, node: NodeId) -> Option<bool> {
        match self.value(node) {
            Value::Bool(b) => Some(b),
            Value::Null
            | Value::Number(_)
            | Value::String(_)
            | Value::Array(_)
            | Value::Object(_) => None,
        }
    }
}

fn slice<T>(list: &[T], range: Span) -> &[T] {
    list.get(range.start as usize..range.end as usize)
        .unwrap_or(&[])
}
