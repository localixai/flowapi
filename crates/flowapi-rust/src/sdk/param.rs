//! One parameter's statement in `<name>_request`: where its value is read (through the
//! `Option`s of its location and itself), the runtime call that writes it with its style a
//! constant, and the path template filled.

use super::{Field, bare, field, is_json, percent};
use crate::ident::literal;
use flowapi::spec::{Location, Param, Serialization, Style};
use std::fmt::Write;

/// The statements that write the path template filled, each `{name}` its parameter's value;
/// `None` when the template has no parameter in it (it is then a literal). An expression no
/// parameter names is left as written. A segment a parameter writes into is held whole to
/// `sdk::segment` once written: it must not be empty, `.` or `..`.
pub(super) fn path(
    template: &str,
    params: &[Param],
    locs: &[(Field<'_>, Vec<Field<'_>>)],
) -> Option<String> {
    let mut text = String::new();
    let mut lit = String::new();
    let mut any = false;
    // The first parameter of the segment being written, and the bytes of it already in `lit`.
    let mut open: Option<(&str, bool)> = None;
    let mut rest = template;
    loop {
        let (before, expr) = match rest.split_once('{') {
            Some((before, after)) => match after.split_once('}') {
                Some((name, tail)) => {
                    rest = tail;
                    (before, Some(name))
                }
                None => (std::mem::take(&mut rest), None),
            },
            None => (std::mem::take(&mut rest), None),
        };
        // A `/` ends the open segment: it is checked before the literal goes on.
        let (head, tail) = before
            .split_once('/')
            .map_or((before, None), |(h, t)| (h, Some(t)));
        if let (Some(seg), Some(_)) = (open, tail) {
            lit.push_str(head);
            flush(&mut text, &mut lit);
            check(&mut text, seg);
            open = None;
            lit.push('/');
            lit.push_str(tail.unwrap_or(""));
        } else {
            lit.push_str(before);
        }
        let Some(name) = expr else {
            break;
        };
        let found = params
            .iter()
            .find(|p| p.location == Location::Path && p.name == name);
        match found.and_then(|p| access(locs, p).map(|a| (p, a))) {
            Some((p, at)) => {
                if open.is_none() {
                    // The segment starts after the last `/` written.
                    let k = lit
                        .rsplit_once('/')
                        .map_or(lit.len(), |(_, after)| after.len());
                    flush(&mut text, &mut lit);
                    let start = if k == 0 {
                        String::from("path.len()")
                    } else {
                        format!("path.len() - {k}")
                    };
                    let _ = writeln!(text, "    let s = {start};");
                    open = Some((&p.name, false));
                }
                if let Some(seg) = &mut open {
                    seg.1 |= p.serialization.allow_reserved;
                }
                flush(&mut text, &mut lit);
                let _ = writeln!(text, "    {}", at.replace("STMT", &call(p, "v")));
                any = true;
            }
            None => {
                lit.push('{');
                lit.push_str(name);
                lit.push('}');
            }
        }
    }
    flush(&mut text, &mut lit);
    if let Some(seg) = open {
        check(&mut text, seg);
    }
    any.then_some(text)
}

/// The check of a segment once written: its first parameter, and whether one keeps `/`.
fn check(text: &mut String, (name, pieces): (&str, bool)) {
    let _ = writeln!(
        text,
        "    sdk::segment(&path, s, {pieces}).map_err(sdk::at(\"path\", {}))?;",
        literal(name)
    );
}

/// The literal text so far, pushed onto the path.
fn flush(text: &mut String, lit: &mut String) {
    if !lit.is_empty() {
        let _ = writeln!(text, "    path.push_str({});", literal(lit));
        lit.clear();
    }
}

/// The statement that reads a parameter as `v` (a reference) and runs `STMT` on it: through
/// `Option`s where its location or itself may be absent.
pub(super) fn access(locs: &[(Field<'_>, Vec<Field<'_>>)], p: &Param) -> Option<String> {
    let (loc, inner) = locs.iter().find(|(l, _)| l.name == p.location.name())?;
    let f = field(inner, &p.name)?;
    let (l, i) = (&loc.ident, &f.ident);
    Some(match (loc.required, f.required) {
        (true, true) => format!("{{\n        let v = &p.{l}.{i};\n        STMT\n    }}"),
        (true, false) => format!("if let Some(v) = &p.{l}.{i} {{\n        STMT\n    }}"),
        (false, true) => {
            format!("if let Some(l) = &p.{l} {{\n        let v = &l.{i};\n        STMT\n    }}")
        }
        (false, false) => format!(
            "if let Some(v) = p.{l}.as_ref().and_then(|l| l.{i}.as_ref()) {{\n        STMT\n    }}"
        ),
    })
}

/// A form's members written by their Encoding Objects, as the runtime takes them: by a style
/// as a query parameter of that style, or as JSON for a JSON `contentType` alone; a member whose
/// `contentType` is another is written by the form's defaults, as one not listed.
pub(super) fn parts(encoding: &[(String, Serialization)]) -> String {
    let mut out = String::from("&[");
    for (name, s) in encoding {
        let part = match &s.media_type {
            Some(t) if is_json(t) => String::from("sdk::Part::Json"),
            Some(_) => continue,
            None => format!("sdk::Part::Style({})", style(s, Location::Query)),
        };
        let _ = write!(out, "({}, {part}), ", literal(name));
    }
    out.truncate(out.trim_end_matches(", ").len());
    out.push(']');
    out
}

/// The runtime call that writes one parameter's value `v`.
pub(super) fn call(p: &Param, v: &str) -> String {
    let s = &p.serialization;
    let raw = p.location == Location::Header || s.style == Style::Cookie;
    let name = literal(&if raw {
        p.name.clone()
    } else {
        percent(&p.name)
    });
    let style = style(s, p.location);
    let at = format!(
        ".map_err(sdk::at({}, {}))?",
        literal(p.location.name()),
        literal(&p.name)
    );
    let value = match &s.media_type {
        Some(t) => format!("sdk::content({v}, {}){at}.as_str()", is_json(t)),
        None => v.to_owned(),
    };
    match p.location {
        Location::Path => format!("sdk::put(&mut path, \"\", {name}, {style}, {value}){at};"),
        Location::Query => format!("sdk::put(&mut query, \"&\", {name}, {style}, {value}){at};"),
        Location::Header => format!("sdk::header(&mut headers, {name}, {style}, {value}){at};"),
        Location::Cookie => {
            format!("sdk::put(&mut cookie, \"; \", {name}, {style}, {value}){at};")
        }
        Location::QueryString => {
            let t = s.media_type.as_deref().unwrap_or("text/plain");
            if bare(t) == "application/x-www-form-urlencoded" {
                format!("sdk::form(&mut query, \"&\", {v}, &[]){at};")
            } else {
                format!(
                    "sdk::query_string(&mut query, &sdk::content({v}, {}){at});",
                    is_json(t)
                )
            }
        }
    }
}

/// The constant of a value's style in `location`: its shape, `explode` and encoder (none for a
/// header and `style: cookie`, reserved expansion for `allowReserved`). A delimited style outside
/// a query (Swagger 2.0's path and header) is `simple` joined by its delimiter; the arrays inside
/// a 2.0 value are joined by their own, outermost first.
fn style(s: &Serialization, location: Location) -> String {
    let raw = location == Location::Header || s.style == Style::Cookie;
    let query = location == Location::Query;
    let mut more = String::new();
    let shape = if s.media_type.is_some() {
        match location {
            Location::Query | Location::Cookie => "Form",
            Location::Path | Location::Header | Location::QueryString => "Simple",
        }
    } else {
        match s.style {
            Style::Simple => "Simple",
            Style::Label => "Label",
            Style::Matrix => "Matrix",
            Style::Form => "Form",
            Style::SpaceDelimited if query => "SpaceDelimited",
            Style::PipeDelimited if query => "PipeDelimited",
            Style::TabDelimited if query => "TabDelimited",
            Style::SpaceDelimited | Style::PipeDelimited | Style::TabDelimited => {
                let _ = write!(more, ".joined({})", literal(delimiter(s.style, raw)));
                "Simple"
            }
            Style::DeepObject => "Deep",
            Style::Cookie => "Cookie",
        }
    };
    if !s.nested.is_empty() {
        let joins: Vec<String> = s
            .nested
            .iter()
            .map(|&n| literal(delimiter(n, raw)))
            .collect();
        let _ = write!(more, ".nested(&[{}])", joins.join(", "));
    }
    let enc = if raw {
        "Raw"
    } else if s.allow_reserved {
        "Reserved"
    } else {
        "Plain"
    };
    format!(
        "const {{ sdk::Style::new(sdk::Shape::{shape}, {}, sdk::Enc::{enc}){more} }}",
        s.explode
    )
}

/// What joins an unexploded array's items in a style: a comma, or a delimited style's
/// delimiter, percent-encoded unless the value is written raw (a header).
fn delimiter(style: Style, raw: bool) -> &'static str {
    match (style, raw) {
        (Style::SpaceDelimited, true) => " ",
        (Style::SpaceDelimited, false) => "%20",
        (Style::PipeDelimited, true) => "|",
        (Style::PipeDelimited, false) => "%7C",
        (Style::TabDelimited, true) => "\t",
        (Style::TabDelimited, false) => "%09",
        (
            Style::Simple
            | Style::Label
            | Style::Matrix
            | Style::Form
            | Style::DeepObject
            | Style::Cookie,
            _,
        ) => ",",
    }
}
