//! An item printed: a struct for an object, an enum of renamed unit variants for strings of a
//! few values, an untagged enum for a choice, a type alias (or, on a cycle of aliases, a
//! transparent newtype) for anything else.

use crate::doc::{self, Says};
use crate::emit::Emitter;
use crate::ident::{Taken, literal, pascal};
use crate::shape::{Choice, Part, Shape, Slot, expand, nullable};
use flowapi::graph::{Kind, Lit};
use flowapi::id::TypeId;
use std::fmt::Write as _;

const DERIVE: &str = "#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]\n";
const INDENT: &str = "    ";

/// A declaration: its doc comment, then the item its type reads as.
pub(crate) fn decl(e: &mut Emitter<'_, '_>, i: usize, id: TypeId, out: &mut String) {
    let p = e.p;
    let ty = p.get(id);
    let choice = expand(p, id);
    let widened = matches!(&choice, Choice::Parts(ps) if matches!(ps.as_slice(), [Part::Object(s, _)] if s.widened));
    let says = Says {
        text: true,
        lossy: true,
        widened,
    };
    doc::write(out, "", &doc::lines(ty, says));
    match choice {
        Choice::Any => alias(e, i, "serde_json::Value", out),
        Choice::Parts(mut parts) => {
            let nullable_pair = parts.len() == 2 && parts.iter().any(|x| matches!(x, Part::Null));
            if parts.len() > 1 && !nullable_pair {
                body_untagged(e, i, &parts, out);
                return;
            }
            match parts.pop() {
                Some(Part::Object(shape, _)) if parts.is_empty() => body_struct(e, i, &shape, out),
                Some(Part::Lits(lits)) if parts.is_empty() => body_lits(e, i, &lits, out),
                last => {
                    parts.extend(last);
                    let text = e.choice(i, id, parts, "", true);
                    alias(e, i, &text, out);
                }
            }
        }
    }
}

fn alias(e: &mut Emitter<'_, '_>, i: usize, text: &str, out: &mut String) {
    let name = e.name(i).to_owned();
    if e.plan.newtype.binary_search(&i).is_ok() {
        out.push_str(DERIVE);
        let _ = writeln!(out, "#[serde(transparent)]\npub struct {name}(pub {text});");
    } else {
        let _ = writeln!(out, "pub type {name} = {text};");
        if let Some(item) = e.items.get_mut(i) {
            item.alias = true;
        }
    }
}

/// An unnamed struct: only what its type does not check is said of it, the rest is said at
/// the field that holds it.
pub(crate) fn shape(
    e: &mut Emitter<'_, '_>,
    i: usize,
    s: &Shape<'_>,
    src: TypeId,
    out: &mut String,
) {
    let says = Says {
        text: false,
        lossy: true,
        widened: s.widened,
    };
    doc::write(out, "", &doc::lines(e.p.get(src), says));
    body_struct(e, i, s, out);
}

pub(crate) fn lits(e: &mut Emitter<'_, '_>, i: usize, lits: &[&Lit<'_>], out: &mut String) {
    body_lits(e, i, lits, out);
}

pub(crate) fn untagged<'p, 't>(
    e: &mut Emitter<'p, 't>,
    i: usize,
    parts: &[Part<'p, 't>],
    out: &mut String,
) {
    body_untagged(e, i, parts, out);
}

fn body_struct(e: &mut Emitter<'_, '_>, i: usize, s: &Shape<'_>, out: &mut String) {
    let p = e.p;
    out.push_str(DERIVE);
    if s.extra == Slot::Never {
        out.push_str("#[serde(deny_unknown_fields)]\n");
    }
    let _ = writeln!(out, "pub struct {} {{", e.name(i));
    let (names, mut idents) = crate::ident::fields(s.fields.iter().map(|f| f.name));
    for (f, ident) in s.fields.iter().zip(names) {
        if let Some(d) = f.doc {
            let says = Says {
                text: true,
                lossy: false,
                widened: false,
            };
            doc::write(out, INDENT, &doc::lines(p.get(d), says));
        }
        let mut attrs: Vec<String> = Vec::new();
        if ident.strip_prefix("r#").unwrap_or(&ident) != f.name {
            attrs.push(format!("rename = {}", literal(f.name)));
        }
        let ty = e.slot(i, f.ty, f.name, true);
        let null_ok = nullable(p, f.ty);
        let ty = if f.required {
            // A field whose type reads `null` is optional to serde unless it is read by hand.
            if null_ok && ty != "serde_json::Value" && ty != "()" {
                attrs.push(String::from(
                    "deserialize_with = \"Deserialize::deserialize\"",
                ));
            }
            ty
        } else {
            attrs.push(String::from(
                "default, skip_serializing_if = \"Option::is_none\"",
            ));
            if null_ok {
                // Absent is `None`, `null` is `Some(None)`: the two stay apart.
                attrs.push(String::from("deserialize_with = \"deserialize_some\""));
                e.uses.helpers.some = true;
            }
            format!("Option<{ty}>")
        };
        if !attrs.is_empty() {
            let _ = writeln!(out, "{INDENT}#[serde({})]", attrs.join(", "));
        }
        let _ = writeln!(out, "{INDENT}pub {ident}: {ty},");
    }
    if s.extra != Slot::Never {
        let value = e.slot(i, s.extra, "extra", false);
        e.uses.map = true;
        let ident = idents.take("extra");
        let _ = writeln!(out, "{INDENT}/// Every other property.");
        let _ = writeln!(out, "{INDENT}#[serde(flatten)]");
        let _ = writeln!(out, "{INDENT}pub {ident}: BTreeMap<String, {value}>,");
    }
    out.push_str("}\n");
}

fn body_lits(e: &Emitter<'_, '_>, i: usize, lits: &[&Lit<'_>], out: &mut String) {
    out.push_str(DERIVE);
    let _ = writeln!(out, "pub enum {} {{", e.name(i));
    let mut idents = Taken::default();
    for l in lits {
        let ident = idents.take(&pascal(&l.norm, "Empty", "V"));
        if ident != l.norm {
            let _ = writeln!(out, "{INDENT}#[serde(rename = {})]", literal(&l.norm));
        }
        let _ = writeln!(out, "{INDENT}{ident},");
    }
    out.push_str("}\n");
}

fn body_untagged<'p, 't>(
    e: &mut Emitter<'p, 't>,
    i: usize,
    parts: &[Part<'p, 't>],
    out: &mut String,
) {
    let p = e.p;
    out.push_str(DERIVE);
    out.push_str("#[serde(untagged)]\n");
    let mut body = String::new();
    let mut idents = Taken::default();
    for part in parts {
        let ident = idents.take(&variant(e, part));
        if matches!(part, Part::Null) {
            let _ = writeln!(body, "{INDENT}{ident},");
            continue;
        }
        if let Part::Ref(r) = part {
            let says = Says {
                text: p.name_of(*r).is_none(),
                lossy: false,
                widened: false,
            };
            doc::write(&mut body, INDENT, &doc::lines(p.get(*r), says));
        }
        let payload = e.part(i, (i, crate::emit::ITEM_LITS), part.clone(), &ident, true);
        let _ = writeln!(body, "{INDENT}{ident}({payload}),");
    }
    let _ = writeln!(out, "pub enum {} {{", e.name(i));
    out.push_str(&body);
    out.push_str("}\n");
}

/// A variant's name: the type it holds when that is named, else the JSON kind.
fn variant(e: &Emitter<'_, '_>, part: &Part<'_, '_>) -> String {
    let p = e.p;
    match part {
        Part::Null => String::from("Null"),
        Part::Bool => String::from("Bool"),
        Part::Int => String::from("Integer"),
        Part::Num => String::from("Number"),
        Part::Str | Part::Lits(_) => String::from("String"),
        Part::Array(_) => String::from("Array"),
        Part::Object(..) => String::from("Object"),
        Part::Ref(r) => match p.name_of(*r) {
            Some(name) => name.to_owned(),
            None => String::from(match &p.get(*r).kind {
                Kind::Intersection(_) => "AllOf",
                Kind::Union(_) => "AnyOf",
                Kind::Value(_) => "Value",
                Kind::Any | Kind::Never | Kind::Alias(_) => "Variant",
            }),
        },
    }
}
