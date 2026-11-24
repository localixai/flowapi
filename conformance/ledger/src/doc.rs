//! The common shape both readers produce: a text's headings and the units of text under them.

/// A heading (OpenAPI) or a section (JSON Schema).
#[derive(Debug, Clone)]
pub struct Heading {
    /// The heading's text, markup removed.
    pub title: String,
    /// What this heading contributes to an id: a slug of the title, a keyword or an anchor.
    pub component: String,
    /// A base starts an id path of its own: an OpenAPI Object, a JSON Schema keyword section or a
    /// section with an explicit anchor.
    pub base: bool,
    /// The anchor of the heading in the rendered specification.
    pub anchor: String,
    /// The enclosing heading.
    pub parent: Option<usize>,
    /// The line of the heading in the pinned text (1-based).
    pub line: usize,
}

/// What a unit of text is.
#[derive(Debug, Clone)]
pub enum UnitKind {
    /// A paragraph, a list item or a table cell.
    Prose,
    /// A row of an OpenAPI "Field Name" or "Field Pattern" table; the text is its description.
    Field {
        /// The field's name as written (`style`, `/{path}`, `^x-`).
        name: String,
        /// Whether the table names patterns rather than fixed names.
        patterned: bool,
        /// The row's own anchor (`<a name="parameter-style">`), if it has one.
        anchor: Option<String>,
    },
    /// A JSON Schema keyword, or a keyword listed in OpenAPI's "JSON Schema Keywords".
    Keyword {
        /// The keyword (`$ref`, `minLength`).
        name: String,
        /// The keyword's section is its own: its id is the section's id.
        own_section: bool,
    },
}

/// A unit of text under a heading.
#[derive(Debug, Clone)]
pub struct Unit {
    /// The heading it is under; `None` before the first heading.
    pub heading: Option<usize>,
    /// The line where the unit starts in the pinned text (1-based).
    pub line: usize,
    /// What the unit is.
    pub kind: UnitKind,
    /// The unit's text, inline markup removed; `\n` separates hard breaks (list items in a cell).
    pub text: String,
}

/// A whole text: its headings in order and its units in order.
#[derive(Debug, Clone, Default)]
pub struct Doc {
    /// Every heading, in document order.
    pub headings: Vec<Heading>,
    /// Every unit, in document order.
    pub units: Vec<Unit>,
}

impl Doc {
    /// The heading path from the outermost heading down to `heading`, titles joined by ` > `.
    pub fn section_path(&self, heading: Option<usize>) -> String {
        let mut titles = Vec::new();
        let mut at = heading;
        while let Some(index) = at {
            let Some(found) = self.headings.get(index) else {
                break;
            };
            titles.push(found.title.as_str());
            at = found.parent;
        }
        titles.reverse();
        titles.join(" > ")
    }
}
