//! One parameter's statement in `<name>_request`: where its value is read (through the
//! `Option`s of its location and itself), the runtime call that writes it with its style a
//! constant, and the path template filled.

use super::{Field, bare, field, is_json, percent};
use crate::ident::literal;
use flowapi::spec::{Location, Param, Style};
use std::fmt::Write;

/// The statements that write the path template filled, each `{name}` its parameter's value;
/// `None` when the template has no parameter in it (it is then a literal). An expression no
/// parameter names is left as written.
pub(super) fn path(
    template: &str,
    params: &[Param],
    locs: &[(Field<'_>, Vec<Field<'_>>)],
) -> Option<String> {
    let mut text = String::new();
    let mut lit = String::new();
    let mut any = false;
    let mut rest = template;
    while let Some((before, after)) = rest.split_once('{') {
        let Some((name, tail)) = after.split_once('}') else {
            break;
        };
        lit.push_str(before);
        let found = params
            .iter()
            .find(|p| p.location == Location::Path && p.name == name);
        match found.and_then(|p| access(locs, p).map(|a| (p, a))) {
            Some((p, at)) => {
                if !lit.is_empty() {
                    let _ = writeln!(text, "    path.push_str({});", literal(&lit));
                    lit.clear();
                }
                let _ = writeln!(text, "    {}", at.replace("STMT", &call(p, "v")));
                any = true;
            }
            None => {
                lit.push('{');
                lit.push_str(name);
                lit.push('}');
            }
        }
        rest = tail;
    }
    lit.push_str(rest);
    if !lit.is_empty() {
        let _ = writeln!(text, "    path.push_str({});", literal(&lit));
    }
    any.then_some(text)
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

/// The runtime call that writes one parameter's value `v`.
pub(super) fn call(p: &Param, v: &str) -> String {
    let s = &p.serialization;
    let raw = p.location == Location::Header || s.style == Style::Cookie;
    let shape = if s.media_type.is_some() {
        match p.location {
            Location::Query | Location::Cookie => "Form",
            Location::Path | Location::Header | Location::QueryString => "Simple",
        }
    } else {
        match s.style {
            Style::Simple => "Simple",
            Style::Label => "Label",
            Style::Matrix => "Matrix",
            Style::Form => "Form",
            Style::SpaceDelimited => "SpaceDelimited",
            Style::PipeDelimited => "PipeDelimited",
            Style::DeepObject => "Deep",
            Style::Cookie => "Cookie",
        }
    };
    let enc = if raw {
        "Raw"
    } else if s.allow_reserved {
        "Reserved"
    } else {
        "Plain"
    };
    let name = literal(&if raw {
        p.name.clone()
    } else {
        percent(&p.name)
    });
    let style = format!(
        "const {{ sdk::Style::new(sdk::Shape::{shape}, {}, sdk::Enc::{enc}) }}",
        s.explode
    );
    let value = match &s.media_type {
        Some(t) => format!("sdk::content({v}, {}).as_str()", is_json(t)),
        None => v.to_owned(),
    };
    match p.location {
        Location::Path => format!("sdk::put(&mut path, \"\", {name}, {style}, {value});"),
        Location::Query => format!("sdk::put(&mut query, \"&\", {name}, {style}, {value});"),
        Location::Header => format!("sdk::header(&mut headers, {name}, {style}, {value});"),
        Location::Cookie => format!("sdk::put(&mut cookie, \"; \", {name}, {style}, {value});"),
        Location::QueryString => {
            let t = s.media_type.as_deref().unwrap_or("text/plain");
            if bare(t) == "application/x-www-form-urlencoded" {
                format!("sdk::form(&mut query, \"&\", {v});")
            } else {
                format!(
                    "sdk::query_string(&mut query, &sdk::content({v}, {}));",
                    is_json(t)
                )
            }
        }
    }
}
