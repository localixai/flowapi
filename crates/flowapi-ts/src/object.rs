//! An object type, one member a line: each property with its schema's description above it,
//! then the index signature when the object takes other properties. Nested shapes are printed
//! first and indented as they are placed.

use crate::pattern::template;
use crate::print::{Prec, reference};
use flowapi::Program;
use flowapi::graph::{Kind, Object, Type};
use flowapi::id::TypeId;

const INDENT: &str = "  ";

/// The members every object inherits, as TypeScript's `Object` declares them.
const INHERITED: &[&str] = &[
    "constructor",
    "toString",
    "toLocaleString",
    "valueOf",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
];

pub(crate) fn object(out: &mut String, p: &Program<'_>, o: &Object<'_>) {
    if o.props.is_empty() && o.patterns.is_empty() && p.is_never(o.extra) {
        // An object that holds no property: TypeScript's own way of saying so.
        out.push_str("Record<string, never>");
        return;
    }
    out.push_str("{\n");
    for prop in &o.props {
        member_doc(out, p.get(prop.ty));
        out.push_str(INDENT);
        key(out, &prop.raw, &prop.name);
        if !prop.required {
            out.push('?');
        }
        out.push_str(": ");
        let mut ty = String::new();
        reference(&mut ty, p, prop.ty, Prec::Union);
        if !prop.required && INHERITED.contains(&prop.name.as_ref()) {
            // TypeScript reads an object literal's missing `constructor` (and the other members
            // every object inherits) as the inherited one: the type takes it too, so `{}` passes.
            ty.push_str(" | Object[\"");
            ty.push_str(&prop.name);
            ty.push_str("\"]");
        }
        indented(out, &ty);
        out.push_str(";\n");
    }
    // Each pattern a template literal type can state has its own signature; the others fall
    // under the string signature, whose type allows their values too.
    let templates: Vec<Option<String>> = o
        .patterns
        .iter()
        .map(|pat| pat.affix.as_ref().and_then(template))
        .collect();
    for (i, (t, pat)) in templates.iter().zip(&o.patterns).enumerate() {
        let (Some(t), Some(affix)) = (t, &pat.affix) else {
            continue;
        };
        if templates.iter().take(i).any(|u| u.as_ref() == Some(t)) {
            continue;
        }
        out.push_str(INDENT);
        out.push_str("[key: ");
        out.push_str(t);
        out.push_str("]: ");
        // Patterns of one template share its signature (TypeScript allows one), which takes the
        // values of each; a named property it matches must fit the signature too.
        let same = templates
            .iter()
            .zip(&o.patterns)
            .filter(|(u, _)| u.as_ref() == Some(t));
        let mut members: Vec<TypeId> = Vec::new();
        for (_, q) in same {
            let r = p.resolve(q.ty);
            if !members.contains(&r) {
                members.push(r);
            }
        }
        for q in o.props.iter().filter(|q| affix.matches(&q.name)) {
            let r = p.resolve(q.ty);
            if !members.contains(&r) {
                members.push(r);
            }
        }
        let mut ty = String::new();
        union_of(&mut ty, p, &members);
        let matching = o.props.iter().filter(|q| affix.matches(&q.name));
        inherited(&mut ty, matching.map(|q| (q.name.as_ref(), q.required)));
        indented(out, &ty);
        out.push_str(";\n");
    }
    let closed =
        !templates.is_empty() && templates.iter().all(Option::is_some) && p.is_never(o.rest);
    if !closed && (!p.is_never(o.extra) || o.props.is_empty()) {
        out.push_str(INDENT);
        out.push_str("[key: string]: ");
        let mut ty = String::new();
        index(&mut ty, p, o);
        indented(out, &ty);
        out.push_str(";\n");
    }
    out.push('}');
}

/// The index signature's value: any other property's, and every named property's too, since a
/// named property must fit the signature as well.
fn index(out: &mut String, p: &Program<'_>, o: &Object<'_>) {
    if p.is_any(o.extra) || o.props.iter().any(|q| p.is_any(q.ty)) {
        out.push_str("unknown");
        return;
    }
    let mut members: Vec<TypeId> = Vec::with_capacity(o.props.len() + 1);
    for m in std::iter::once(o.extra).chain(o.props.iter().map(|q| q.ty)) {
        let r = p.resolve(m);
        if !p.is_never(r) && !members.contains(&r) {
            members.push(r);
        }
    }
    union_of(out, p, &members);
    inherited(out, o.props.iter().map(|q| (q.name.as_ref(), q.required)));
}

/// For each optional property named like a member every object inherits, the inherited member's
/// type, which a signature it falls under must take too (TS2411 otherwise).
fn inherited<'n>(out: &mut String, props: impl Iterator<Item = (&'n str, bool)>) {
    for (name, required) in props {
        if !required && INHERITED.contains(&name) {
            out.push_str(" | Object[\"");
            out.push_str(name);
            out.push_str("\"]");
        }
    }
}

/// Types joined by `|`; `unknown` when one allows every value, `never` for none.
fn union_of(out: &mut String, p: &Program<'_>, members: &[TypeId]) {
    // An unnamed union among the members is its own members, so each prints once.
    let mut flat: Vec<TypeId> = Vec::with_capacity(members.len());
    let mut stack: Vec<TypeId> = members.iter().rev().copied().collect();
    while let Some(m) = stack.pop() {
        let r = p.resolve(m);
        match &p.get(r).kind {
            Kind::Union(inner) if p.name_of(r).is_none() && flat.len() + stack.len() < 256 => {
                stack.extend(inner.iter().rev());
            }
            Kind::Union(_)
            | Kind::Any
            | Kind::Never
            | Kind::Value(_)
            | Kind::Intersection(_)
            | Kind::Alias(_) => {
                flat.push(r);
            }
        }
    }
    let members: Vec<TypeId> = flat.into_iter().filter(|&m| !p.is_never(m)).collect();
    if members.iter().any(|&m| p.is_any(m)) {
        out.push_str("unknown");
        return;
    }
    // Two types can print alike (two schemas of one `type`): each text once.
    let mut printed: Vec<String> = Vec::with_capacity(members.len());
    for &m in &members {
        let mut text = String::new();
        reference(&mut text, p, m, Prec::Union);
        if !printed.contains(&text) {
            printed.push(text);
        }
    }
    out.push_str(if printed.is_empty() { "never" } else { "" });
    out.push_str(&printed.join(" | "));
}

/// Text printed on its own, placed one level in: every line after the first indented.
fn indented(out: &mut String, text: &str) {
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
            if !line.is_empty() {
                out.push_str(INDENT);
            }
        }
        out.push_str(line);
    }
}

/// A property's description and title, and whether it is deprecated, as a comment above it.
fn member_doc(out: &mut String, ty: &Type<'_>) {
    let mut lines: Vec<String> = Vec::new();
    for text in [&ty.title, &ty.description].into_iter().flatten() {
        lines.extend(text.lines().map(crate::print::comment_line));
    }
    if ty.deprecated {
        lines.push(String::from("@deprecated"));
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    match lines.as_slice() {
        [] => {}
        [one] => {
            out.push_str(INDENT);
            out.push_str("/** ");
            out.push_str(one);
            out.push_str(" */\n");
        }
        many => {
            out.push_str(INDENT);
            out.push_str("/**\n");
            for line in many {
                out.push_str(INDENT);
                out.push_str(" *");
                if !line.is_empty() {
                    out.push(' ');
                    out.push_str(line);
                }
                out.push('\n');
            }
            out.push_str(INDENT);
            out.push_str(" */\n");
        }
    }
}

/// A property name: bare when it is an identifier as written, its JSON token otherwise (a valid
/// TypeScript string literal).
fn key(out: &mut String, raw: &str, name: &str) {
    let bare = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        && raw.len() == name.len() + 2;
    out.push_str(if bare { name } else { raw });
}
