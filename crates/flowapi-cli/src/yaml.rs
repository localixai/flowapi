//! A YAML description read into the JSON text the core reads, with the YAML line and column of
//! every JSON node it writes. JSON is the data model: a mapping is an object (its keys strings:
//! a scalar key that is not one is written as its JSON text, `200` as `"200"`), a sequence an
//! array, a scalar a value of the YAML 1.2 core schema. What JSON cannot hold is refused: `.inf`
//! and `.nan`, a mapping or sequence as a key, a key twice in a mapping, `<<` merge keys (YAML
//! 1.1, not 1.2), more than one document. Aliases are expanded, within a bound.

mod scalar;

use saphyr_parser::{Event, Marker, Parser, ScalarStyle, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The most nodes aliases may copy into one document, all copies together: an alias bomb (a
/// billion laughs) is refused, not expanded.
pub const MAX_ALIAS_NODES: usize = 1_000_000;
/// The most bytes of JSON aliases may copy into one document, all copies together.
pub const MAX_ALIAS_BYTES: usize = 64 << 20;

/// A YAML document as JSON.
#[derive(Debug)]
pub struct Json {
    /// The JSON text.
    pub text: String,
    /// Where in the YAML each JSON node came from.
    pub marks: Marks,
}

/// The YAML place of each JSON node, by the node's byte offset into the JSON text.
#[derive(Debug, Default)]
pub struct Marks {
    /// JSON offset, YAML line, YAML column (both from 1), in the order of the offsets.
    at: Vec<(u32, u32, u32)>,
}

impl Marks {
    /// The YAML line and column of the JSON node at `offset`, or of the node holding it; a node
    /// an alias copied is placed at the alias.
    #[must_use]
    pub fn place(&self, offset: u32) -> Option<(u32, u32)> {
        let i = self.at.partition_point(|&(o, _, _)| o <= offset);
        let &(_, line, column) = self.at.get(i.checked_sub(1)?)?;
        Some((line, column))
    }
}

/// Why a text is not a YAML document JSON can hold, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The line, from 1.
    pub line: usize,
    /// The column in characters, from 1.
    pub column: usize,
    /// What was found.
    pub message: String,
}

impl Error {
    fn at(mark: Marker, message: &str) -> Self {
        Self {
            line: mark.line(),
            column: mark.col() + 1,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

/// Reads one YAML document into JSON text.
///
/// # Errors
///
/// When the text is not YAML, or holds what JSON cannot (see the module's comment).
pub fn to_json(text: &str) -> Result<Json, Error> {
    let mut w = Writer::default();
    let mut last = Marker::default();
    for next in Parser::new_from_str(text) {
        let (event, span) = next.map_err(|e| Error::at(*e.marker(), e.info()))?;
        last = span.start;
        w.event(event, span)
            .map_err(|m| Error::at(span.start, &m))?;
    }
    if w.documents == 0 || w.out.is_empty() {
        return Err(Error::at(last, "no document"));
    }
    Ok(Json {
        text: w.out,
        marks: Marks { at: w.marks },
    })
}

/// A mapping or sequence being written.
enum Frame {
    Seq {
        first: bool,
        open: Open,
    },
    Map {
        first: bool,
        key_next: bool,
        keys: BTreeSet<String>,
        open: Open,
    },
}

/// Where a mapping or sequence began: its anchor (0 for none), its first JSON byte, the nodes
/// written before it.
struct Open {
    anchor: usize,
    start: usize,
    nodes: usize,
}

/// What an anchor names: a scalar, or the JSON a mapping or sequence was written as.
#[derive(Clone)]
enum Anchor {
    Scalar(scalar::Scalar),
    Tree {
        start: usize,
        end: usize,
        nodes: usize,
    },
}

#[derive(Default)]
struct Writer {
    out: String,
    marks: Vec<(u32, u32, u32)>,
    stack: Vec<Frame>,
    anchors: BTreeMap<usize, Anchor>,
    documents: usize,
    nodes: usize,
    copied_nodes: usize,
    copied_bytes: usize,
}

impl Writer {
    fn event(&mut self, event: Event<'_>, span: Span) -> Result<(), String> {
        match event {
            Event::Nothing | Event::StreamStart | Event::StreamEnd | Event::DocumentEnd => {}
            Event::DocumentStart(_) => {
                self.documents += 1;
                if self.documents > 1 {
                    return Err(String::from(
                        "a second document in the stream: a description is one document",
                    ));
                }
            }
            Event::Scalar(text, style, anchor, tag) => {
                let s = scalar::resolve(&text, style, tag.as_deref())?;
                if self.at_key() {
                    if matches!(style, ScalarStyle::Plain) && text == "<<" {
                        return Err(String::from(
                            "a `<<` merge key, which is YAML 1.1, not 1.2: write the keys out",
                        ));
                    }
                    self.key(&s.key, span)?;
                } else {
                    self.value(span)?;
                    self.out.push_str(&s.json);
                }
                if anchor != 0 {
                    self.anchors.insert(anchor, Anchor::Scalar(s));
                }
            }
            Event::SequenceStart(anchor, _) => self.open(anchor, span, false)?,
            Event::MappingStart(anchor, _) => self.open(anchor, span, true)?,
            Event::SequenceEnd | Event::MappingEnd => self.close()?,
            Event::Alias(id) => self.alias(id, span)?,
        }
        Ok(())
    }

    fn at_key(&self) -> bool {
        matches!(self.stack.last(), Some(Frame::Map { key_next: true, .. }))
    }

    /// Writes a mapping's key.
    fn key(&mut self, key: &str, span: Span) -> Result<(), String> {
        if let Some(Frame::Map {
            first,
            key_next,
            keys,
            ..
        }) = self.stack.last_mut()
        {
            if keys.contains(key) {
                return Err(format!("the key `{key}` twice in one mapping"));
            }
            keys.insert(key.to_owned());
            if !*first {
                self.out.push(',');
            }
            *first = false;
            *key_next = false;
        }
        mark(&self.out, &mut self.marks, span)?;
        scalar::push_string(&mut self.out, key);
        self.out.push(':');
        Ok(())
    }

    /// Makes room for a value: the comma before it, its mark.
    fn value(&mut self, span: Span) -> Result<(), String> {
        match self.stack.last_mut() {
            Some(Frame::Seq { first, .. }) => {
                if !*first {
                    self.out.push(',');
                }
                *first = false;
            }
            Some(Frame::Map { key_next, .. }) => {
                if *key_next {
                    return Err(String::from(
                        "a mapping or sequence as a key: JSON's keys are strings",
                    ));
                }
                *key_next = true;
            }
            None => {}
        }
        self.nodes += 1;
        mark(&self.out, &mut self.marks, span)
    }

    fn open(&mut self, anchor: usize, span: Span, map: bool) -> Result<(), String> {
        let nodes = self.nodes;
        self.value(span)?;
        let open = Open {
            anchor,
            start: self.out.len(),
            nodes,
        };
        if map {
            self.out.push('{');
            self.stack.push(Frame::Map {
                first: true,
                key_next: true,
                keys: BTreeSet::new(),
                open,
            });
        } else {
            self.out.push('[');
            self.stack.push(Frame::Seq { first: true, open });
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        let (open, end) = match self.stack.pop() {
            Some(Frame::Seq { open, .. }) => (open, ']'),
            Some(Frame::Map { open, .. }) => (open, '}'),
            None => return Err(String::from("the end of a node never opened")),
        };
        self.out.push(end);
        if open.anchor != 0 {
            let tree = Anchor::Tree {
                start: open.start,
                end: self.out.len(),
                nodes: self.nodes - open.nodes,
            };
            self.anchors.insert(open.anchor, tree);
        }
        Ok(())
    }

    /// Writes what an alias names again, counted against the bound.
    fn alias(&mut self, id: usize, span: Span) -> Result<(), String> {
        let Some(anchor) = self.anchors.get(&id).cloned() else {
            return Err(String::from("an alias inside the node it names"));
        };
        let (json, nodes) = match anchor {
            Anchor::Scalar(s) if self.at_key() => return self.key(&s.key, span),
            Anchor::Scalar(s) => (s.json, 1),
            Anchor::Tree { start, end, nodes } => {
                let json = self.out.get(start..end).unwrap_or_default().to_owned();
                (json, nodes)
            }
        };
        self.copied_nodes += nodes;
        self.copied_bytes += json.len();
        if self.copied_nodes > MAX_ALIAS_NODES || self.copied_bytes > MAX_ALIAS_BYTES {
            return Err(format!(
                "aliases that copy over {MAX_ALIAS_NODES} nodes or {} MiB: refused as an alias bomb",
                MAX_ALIAS_BYTES >> 20
            ));
        }
        self.value(span)?;
        self.nodes += nodes - 1;
        self.out.push_str(&json);
        Ok(())
    }
}

/// Records that the JSON written next comes from the YAML at `span`.
fn mark(out: &str, marks: &mut Vec<(u32, u32, u32)>, span: Span) -> Result<(), String> {
    let offset = u32::try_from(out.len()).map_err(|e| format!("over 4 GiB of JSON: {e}"))?;
    let line = u32::try_from(span.start.line()).unwrap_or(u32::MAX);
    let column = u32::try_from(span.start.col() + 1).unwrap_or(u32::MAX);
    marks.push((offset, line, column));
    Ok(())
}
