//! Doc comments from a schema's title and description. Rustdoc reads them as Markdown and runs
//! an unmarked code block as a Rust test, so a fence without a language is marked `text` and a
//! line is never indented into a code block.

use flowapi::graph::Type;

/// What a doc comment says of a type.
#[derive(Clone, Copy)]
pub(crate) struct Says {
    /// The schema's title and description.
    pub(crate) text: bool,
    /// The keywords the type does not check.
    pub(crate) lossy: bool,
    /// An intersection that was widened.
    pub(crate) widened: bool,
}

/// The lines of a type's documentation.
pub(crate) fn lines(ty: &Type<'_>, says: Says) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    let texts = if says.text {
        [&ty.title, &ty.description]
    } else {
        [&None, &None]
    };
    for text in texts.into_iter().flatten() {
        if !out.is_empty() {
            out.push(String::new());
        }
        for line in text.lines() {
            out.push(clean(line, &mut fenced));
        }
    }
    if fenced {
        out.push(String::from("```"));
    }
    if says.text && ty.deprecated {
        out.push(String::from("Deprecated."));
    }
    let mut unchecked: Vec<&str> = if says.lossy {
        ty.lossy.iter().map(|k| k.name()).collect()
    } else {
        Vec::new()
    };
    if says.widened {
        unchecked.push("allOf");
    }
    if !unchecked.is_empty() {
        out.push(format!(
            "Not checked by this type: {}.",
            unchecked.join(", ")
        ));
    }
    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    out
}

fn clean(line: &str, fenced: &mut bool) -> String {
    // A bare carriage return is an error in a doc comment; other controls are spaces.
    let text: String = line
        .chars()
        .filter(|&c| c != '\r')
        .map(|c| if c.is_control() && c != '\t' { ' ' } else { c })
        .collect();
    let trimmed = text.trim();
    if let Some(info) = trimmed.strip_prefix("```") {
        let opening = !*fenced;
        *fenced = opening;
        return if opening && info.trim().is_empty() {
            String::from("```text")
        } else {
            trimmed.to_owned()
        };
    }
    if *fenced {
        text.trim_end().to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// The lines written as `///` comments, each indented by `indent`.
pub(crate) fn write(out: &mut String, indent: &str, lines: &[String]) {
    for line in lines {
        out.push_str(indent);
        out.push_str("///");
        if !line.is_empty() {
            out.push(' ');
            out.push_str(line);
        }
        out.push('\n');
    }
}
