//! The reader: RFC 8259, strict. Recursive descent with a bound on nesting, so no input can
//! exhaust the stack (G1); held to JSONTestSuite (`conformance/`).

use super::tree::{Member, Span, Tree, Value};
use crate::id::NodeId;

/// How deep arrays and objects may nest. Real descriptions nest a few dozen levels.
const MAX_DEPTH: u32 = 512;

/// Why a text is not JSON: what was expected, and the byte offset where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// The byte offset.
    pub offset: u32,
    /// What was expected or found.
    pub message: &'static str,
}

/// Reads a JSON text. A leading byte order mark is skipped (RFC 8259 §8.1 allows it).
///
/// # Errors
///
/// When the text is not JSON, is over 4 GiB, or nests deeper than 512 levels.
pub fn parse(text: &str) -> Result<Tree<'_>, ParseError> {
    if u32::try_from(text.len()).is_err() {
        return Err(ParseError {
            offset: 0,
            message: "a document over 4 GiB",
        });
    }
    let start = if text.starts_with('\u{feff}') { 3 } else { 0 };
    let mut reader = Reader {
        bytes: text.as_bytes(),
        at: start,
        tree: Tree {
            text,
            values: Vec::new(),
            offsets: Vec::new(),
            parents: Vec::new(),
            items: Vec::new(),
            members: Vec::new(),
        },
        scratch_items: Vec::new(),
        scratch_members: Vec::new(),
    };
    reader.space();
    reader.value(None, 0)?;
    reader.space();
    if reader.at < reader.bytes.len() {
        return Err(reader.error("text after the value"));
    }
    Ok(reader.tree)
}

struct Reader<'t> {
    bytes: &'t [u8],
    at: usize,
    tree: Tree<'t>,
    scratch_items: Vec<NodeId>,
    scratch_members: Vec<Member>,
}

impl Reader<'_> {
    fn error(&self, message: &'static str) -> ParseError {
        ParseError {
            offset: offset(self.at),
            message,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn node(&mut self, parent: Option<NodeId>) -> Result<NodeId, ParseError> {
        let id = NodeId::new(self.tree.values.len()).ok_or_else(|| self.error("too many nodes"))?;
        self.tree.values.push(Value::Null);
        self.tree.offsets.push(offset(self.at));
        self.tree.parents.push(parent);
        Ok(id)
    }

    fn set(&mut self, id: NodeId, value: Value) {
        if let Some(slot) = self.tree.values.get_mut(id.index()) {
            *slot = value;
        }
    }

    fn value(&mut self, parent: Option<NodeId>, depth: u32) -> Result<NodeId, ParseError> {
        let id = self.node(parent)?;
        let value = match self.peek() {
            Some(b'{') => self.object(id, depth)?,
            Some(b'[') => self.array(id, depth)?,
            Some(b'"') => Value::String(self.string()?),
            Some(b't') => self.word(b"true", Value::Bool(true))?,
            Some(b'f') => self.word(b"false", Value::Bool(false))?,
            Some(b'n') => self.word(b"null", Value::Null)?,
            Some(b'-' | b'0'..=b'9') => Value::Number(self.number()?),
            Some(_) | None => return Err(self.error("a value")),
        };
        self.set(id, value);
        Ok(id)
    }

    fn word(&mut self, word: &[u8], value: Value) -> Result<Value, ParseError> {
        if self.bytes.get(self.at..self.at + word.len()) == Some(word) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(self.error("true, false or null"))
        }
    }

    fn array(&mut self, id: NodeId, depth: u32) -> Result<Value, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.error("nesting deeper than 512 levels"));
        }
        self.at += 1;
        let mark = self.scratch_items.len();
        self.space();
        if self.peek() == Some(b']') {
            self.at += 1;
        } else {
            loop {
                self.space();
                let item = self.value(Some(id), depth + 1)?;
                self.scratch_items.push(item);
                self.space();
                match self.peek() {
                    Some(b',') => self.at += 1,
                    Some(b']') => {
                        self.at += 1;
                        break;
                    }
                    Some(_) | None => return Err(self.error("`,` or `]`")),
                }
            }
        }
        let start = offset(self.tree.items.len());
        self.tree.items.extend(self.scratch_items.drain(mark..));
        Ok(Value::Array(Span {
            start,
            end: offset(self.tree.items.len()),
        }))
    }

    fn object(&mut self, id: NodeId, depth: u32) -> Result<Value, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.error("nesting deeper than 512 levels"));
        }
        self.at += 1;
        let mark = self.scratch_members.len();
        self.space();
        if self.peek() == Some(b'}') {
            self.at += 1;
        } else {
            loop {
                self.space();
                if self.peek() != Some(b'"') {
                    return Err(self.error("a key"));
                }
                let key = self.string()?;
                self.space();
                if self.peek() != Some(b':') {
                    return Err(self.error("`:`"));
                }
                self.at += 1;
                self.space();
                let value = self.value(Some(id), depth + 1)?;
                self.scratch_members.push(Member { key, value });
                self.space();
                match self.peek() {
                    Some(b',') => self.at += 1,
                    Some(b'}') => {
                        self.at += 1;
                        break;
                    }
                    Some(_) | None => return Err(self.error("`,` or `}`")),
                }
            }
        }
        let start = offset(self.tree.members.len());
        self.tree.members.extend(self.scratch_members.drain(mark..));
        Ok(Value::Object(Span {
            start,
            end: offset(self.tree.members.len()),
        }))
    }

    fn string(&mut self) -> Result<Span, ParseError> {
        let start = self.at;
        self.at += 1;
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.at += 1;
                    return Ok(Span {
                        start: offset(start),
                        end: offset(self.at),
                    });
                }
                Some(b'\\') => self.escape()?,
                Some(0..=0x1f) => return Err(self.error("a control character in a string")),
                Some(_) => self.at += 1,
                None => return Err(self.error("the string's closing quote")),
            }
        }
    }

    fn escape(&mut self) -> Result<(), ParseError> {
        self.at += 1;
        match self.peek() {
            Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => self.at += 1,
            Some(b'u') => {
                let hex = self.bytes.get(self.at + 1..self.at + 5).unwrap_or(&[]);
                if hex.len() != 4 || !hex.iter().all(u8::is_ascii_hexdigit) {
                    return Err(self.error("four hex digits after `\\u`"));
                }
                self.at += 5;
            }
            Some(_) | None => return Err(self.error("an escape")),
        }
        Ok(())
    }

    fn number(&mut self) -> Result<Span, ParseError> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => self.digits(),
            Some(_) | None => return Err(self.error("a digit")),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("a digit after `.`"));
            }
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("a digit in the exponent"));
            }
            self.digits();
        }
        Ok(Span {
            start: offset(start),
            end: offset(self.at),
        })
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
    }
}

/// A position as a `u32`; the reader refused texts over 4 GiB first, so it always fits.
fn offset(at: usize) -> u32 {
    u32::try_from(at).unwrap_or(u32::MAX)
}
