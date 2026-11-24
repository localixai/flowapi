//! The SDK, printed after the types when asked for ([`crate::Rust::with_sdk`]): for each
//! operation, `<name>_request(&input)`, which writes the request the input makes (method, path,
//! query, headers, body) and sends nothing, and `<name>_response(status, headers, body)`, which
//! reads an answer back into an enum of the responses the description names. A runtime of a
//! few functions is printed once, as `pub mod sdk` (`sdk/runtime.rs` and `sdk/errors.rs`, with
//! `sdk/write.rs`, `sdk/expand.rs` and `sdk/finite.rs` as its modules); it needs serde and
//! serde_json, as the types do. What a call does not
//! depend on is decided here, once: path templates split, names percent-encoded, each
//! parameter's style a constant, each status a `match` arm.

mod param;
mod request;
mod response;

use crate::emit::Emitter;
use crate::ident::{Taken, snake};
use crate::shape::Slot;
use flowapi::Program;
use flowapi::graph::{Kind, Object};
use flowapi::id::TypeId;

const RUNTIME: &str = include_str!("sdk/runtime.rs");
const WRITE: &str = include_str!("sdk/write.rs");
const EXPAND: &str = include_str!("sdk/expand.rs");
const ERRORS: &str = include_str!("sdk/errors.rs");
const FINITE: &str = include_str!("sdk/finite.rs");

/// A field of a struct the types printed: its identifier, key, type and whether it is there.
pub(crate) struct Field<'p> {
    pub(crate) ident: String,
    pub(crate) name: &'p str,
    pub(crate) ty: TypeId,
    pub(crate) required: bool,
}

/// What an operation's functions are printed with.
pub(crate) struct Op<'o> {
    pub(crate) endpoint: &'o flowapi::graph::Endpoint,
    /// The item of the operation's type, where its types are named from.
    pub(crate) item: usize,
    /// The functions' name before `_request` and `_response`.
    pub(crate) base: String,
    /// The operation's type's fields: `parameters`, `requestBody`, `responses`.
    pub(crate) fields: Vec<Field<'o>>,
}

pub(crate) fn write(e: &mut Emitter<'_, '_>, out: &mut String) {
    let p = e.p;
    let endpoints = p.endpoints();
    if endpoints.is_empty() {
        return;
    }
    out.push_str("\npub mod sdk {\n");
    out.push_str(RUNTIME);
    out.push('\n');
    out.push_str(crate::print::printed(ERRORS));
    out.push_str("\nmod write {\n");
    out.push_str(WRITE);
    out.push_str("}\n\nmod expand {\n");
    out.push_str(EXPAND);
    out.push_str("}\n\nmod finite {\n");
    out.push_str(FINITE);
    out.push_str("}\n}\n");
    let mut functions = Taken::of(["deserialize_some"]);
    for endpoint in endpoints {
        let Some(item) = e.decl_item(endpoint.ty) else {
            continue;
        };
        let Some(fields) = fields(p, endpoint.ty) else {
            continue;
        };
        let base = functions.take(&snake(e.name(item)));
        let op = Op {
            endpoint,
            item,
            base,
            fields,
        };
        request::write(e, out, &op);
        response::write(e, out, &op);
    }
}

/// The object a type is, when it is one.
pub(crate) fn object<'p>(p: &'p Program<'_>, id: TypeId) -> Option<&'p Object<'p>> {
    match &p.get(p.resolve(id)).kind {
        Kind::Value(b) => b.object.as_ref(),
        Kind::Any | Kind::Never | Kind::Alias(_) | Kind::Union(_) | Kind::Intersection(_) => None,
    }
}

/// An object type's fields, with the identifiers the struct printed for it gave them.
pub(crate) fn fields<'p>(p: &'p Program<'_>, id: TypeId) -> Option<Vec<Field<'p>>> {
    let o = object(p, id)?;
    let (idents, _) = crate::ident::fields(o.props.iter().map(|q| q.name.as_ref()));
    Some(
        o.props
            .iter()
            .zip(idents)
            .map(|(q, ident)| Field {
                ident,
                name: q.name.as_ref(),
                ty: q.ty,
                required: q.required,
            })
            .collect(),
    )
}

/// The field named `name`.
pub(crate) fn field<'f, 'p>(fields: &'f [Field<'p>], name: &str) -> Option<&'f Field<'p>> {
    fields.iter().find(|f| f.name == name)
}

/// The Rust type of a field's value, as the types named it.
pub(crate) fn type_of(e: &mut Emitter<'_, '_>, item: usize, ty: TypeId) -> String {
    let slot = Slot::of(e.p, ty);
    e.slot(item, slot, "", false)
}

/// RFC 3986 percent-encoding of every character but the unreserved ones, as UTF-8: a parameter
/// name in a URI (3.2, Appendix C, "Non-RFC6570 Field Values and Combinations").
pub(crate) fn percent(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            out.push('%');
            out.push(char::from(HEX[usize::from(b >> 4)]));
            out.push(char::from(HEX[usize::from(b & 15)]));
        }
    }
    out
}

/// A media type without its parameters, in lower case.
pub(crate) fn bare(media_type: &str) -> String {
    media_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

/// Whether a media type is JSON: `application/json` or a `+json` type (RFC 6839).
pub(crate) fn is_json(media_type: &str) -> bool {
    let bare = bare(media_type);
    bare == "application/json" || bare.ends_with("+json")
}
