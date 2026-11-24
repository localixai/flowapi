//! A minimal XML reader, enough for the xml2rfc sources of JSON Schema: elements, attributes,
//! text, CDATA and the five predefined entities and character references. Comments, processing
//! instructions and the DOCTYPE are skipped; an entity the DOCTYPE declares is kept as written.

/// A node of the tree.
#[derive(Debug, Clone)]
pub enum Node {
    /// An element.
    Element(Element),
    /// Text, entities decoded.
    Text(String),
}

/// An element with its attributes and children.
#[derive(Debug, Clone)]
pub struct Element {
    /// The tag name.
    pub name: String,
    /// Attributes in order, values decoded.
    pub attrs: Vec<(String, String)>,
    /// Children in order.
    pub children: Vec<Node>,
    /// The line of the start tag (1-based).
    pub line: usize,
}

impl Element {
    /// The value of an attribute.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The child elements.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|node| match node {
            Node::Element(element) => Some(element),
            Node::Text(_) => None,
        })
    }

    /// All the text inside, in order.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for node in &self.children {
            match node {
                Node::Text(text) => out.push_str(text),
                Node::Element(element) => out.push_str(&element.text()),
            }
        }
        out
    }
}

/// Parses a document; returns its root element.
pub fn parse(source: &str) -> Result<Element, String> {
    let mut stack: Vec<Element> = vec![Element {
        name: String::new(),
        attrs: Vec::new(),
        children: Vec::new(),
        line: 0,
    }];
    let mut rest = source;
    let line_of = |rest: &str| {
        source
            .get(..source.len() - rest.len())
            .unwrap_or_default()
            .matches('\n')
            .count()
            + 1
    };
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("<!--") {
            rest = skip_past(after, "-->")?;
        } else if let Some(after) = rest.strip_prefix("<![CDATA[") {
            let end = after.find("]]>").ok_or("unterminated CDATA")?;
            push_text(&mut stack, after.get(..end).unwrap_or_default().to_owned());
            rest = after.get(end + 3..).unwrap_or_default();
        } else if let Some(after) = rest.strip_prefix("<!DOCTYPE") {
            rest = skip_doctype(after)?;
        } else if let Some(after) = rest.strip_prefix("<?") {
            rest = skip_past(after, "?>")?;
        } else if let Some(after) = rest.strip_prefix("</") {
            let end = after.find('>').ok_or("unterminated end tag")?;
            let name = after.get(..end).unwrap_or_default().trim();
            let closed = stack.pop().ok_or("end tag without start")?;
            if closed.name != name {
                return Err(format!(
                    "line {}: </{name}> closes <{}>",
                    line_of(rest),
                    closed.name
                ));
            }
            let parent = stack.last_mut().ok_or("end tag closes the document")?;
            parent.children.push(Node::Element(closed));
            rest = after.get(end + 1..).unwrap_or_default();
        } else if rest.starts_with('<') {
            let line = line_of(rest);
            let end = tag_end(rest).ok_or_else(|| format!("line {line}: unterminated tag"))?;
            let body = rest.get(1..end).unwrap_or_default();
            let (body, empty) = match body.strip_suffix('/') {
                Some(inner) => (inner, true),
                None => (body, false),
            };
            let element = start_tag(body, line);
            if empty {
                let parent = stack.last_mut().ok_or("no parent")?;
                parent.children.push(Node::Element(element));
            } else {
                stack.push(element);
            }
            rest = rest.get(end + 1..).unwrap_or_default();
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            push_text(&mut stack, decode(rest.get(..end).unwrap_or_default()));
            rest = rest.get(end..).unwrap_or_default();
        }
    }
    let document = stack.pop().ok_or("empty document")?;
    if !stack.is_empty() {
        return Err(format!("<{}> is never closed", document.name));
    }
    document
        .elements()
        .next()
        .cloned()
        .ok_or_else(|| "no root element".to_owned())
}

fn push_text(stack: &mut [Element], text: String) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Node::Text(text));
    }
}

fn skip_past<'a>(rest: &'a str, marker: &str) -> Result<&'a str, String> {
    let end = rest
        .find(marker)
        .ok_or_else(|| format!("missing {marker}"))?;
    Ok(rest.get(end + marker.len()..).unwrap_or_default())
}

fn skip_doctype(rest: &str) -> Result<&str, String> {
    let bracket = rest.find('[');
    let close = rest.find('>').ok_or("unterminated DOCTYPE")?;
    match bracket {
        Some(open) if open < close => skip_past(rest, "]>"),
        _ => Ok(rest.get(close + 1..).unwrap_or_default()),
    }
}

/// The index of the `>` that ends the tag at the start of `rest`, quotes respected.
fn tag_end(rest: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (index, ch) in rest.char_indices() {
        match (quote, ch) {
            (None, '"' | '\'') => quote = Some(ch),
            (Some(open), _) if open == ch => quote = None,
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

fn start_tag(body: &str, line: usize) -> Element {
    let name_end = body.find(char::is_whitespace).unwrap_or(body.len());
    let name = body.get(..name_end).unwrap_or_default().to_owned();
    let mut attrs = Vec::new();
    let mut rest = body.get(name_end..).unwrap_or_default().trim_start();
    while let Some(equals) = rest.find('=') {
        let key = rest.get(..equals).unwrap_or_default().trim().to_owned();
        let after = rest.get(equals + 1..).unwrap_or_default().trim_start();
        let Some(quote) = after.chars().next() else {
            break;
        };
        let Some(length) = after.get(1..).and_then(|v| v.find(quote)) else {
            break;
        };
        attrs.push((key, decode(after.get(1..1 + length).unwrap_or_default())));
        rest = after.get(length + 2..).unwrap_or_default().trim_start();
    }
    Element {
        name,
        attrs,
        children: Vec::new(),
        line,
    }
}

/// Decodes the predefined entities and character references; other entities stay as written.
pub fn decode(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(rest.get(..start).unwrap_or_default());
        let after = rest.get(start..).unwrap_or_default();
        let Some(end) = after.find(';').filter(|end| *end < 12) else {
            out.push('&');
            rest = after.get(1..).unwrap_or_default();
            continue;
        };
        let name = after.get(1..end).unwrap_or_default();
        match entity(name) {
            Some(ch) => out.push(ch),
            None => out.push_str(after.get(..=end).unwrap_or_default()),
        }
        rest = after.get(end + 1..).unwrap_or_default();
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    match name {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let code = match name.strip_prefix("#x") {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => name.strip_prefix('#')?.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}
