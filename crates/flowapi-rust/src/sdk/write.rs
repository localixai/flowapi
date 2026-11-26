//! What one parameter's expansion writes, piece by piece: the places a value can sit
//! ([`At`]) and the state of one expansion ([`W`]), RFC 6570's rules for each (§3.2.1, 3.2
//! Appendix C); a form member its Encoding Object names, by its [`Part`]. Printed inside the
//! generated `sdk` module beside `expand`, its serde half.

use super::expand::{Error, Ser};
use super::{Enc, Part, Shape, Style, encode};
use serde::Serialize;

/// Where a value sits, which says what each of its kinds writes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum At {
    /// The parameter itself.
    Top,
    /// An item of the parameter's list.
    Item,
    /// A member of the parameter's associative array; its key is in `W::key`.
    Member,
    /// A member of a `deepObject`, its key already in `W::path`.
    Deep,
    /// An item of a list inside a `deepObject`.
    DeepItem,
    /// A map's key, written raw into `W::key`.
    Key,
    /// The whole value of a form: an associative array, or else one encoded string.
    Form,
    /// A member of a form, its key in `W::key`; `null` is `key=`.
    FormMember,
    /// An item of a form member's list: a pair of the member's key.
    FormItem,
    /// What writes nothing: a `deepObject` that is not an associative array.
    Ignore,
    /// An item of a list inside the parameter's list, that many levels down from the first
    /// (Swagger 2.0's Items Object): joined by `Style::nest` at its level.
    Nest(usize),
}

/// The state of one expansion.
pub(super) struct W<'a> {
    pub(super) out: &'a mut String,
    /// What goes before the first piece written: the separator from what `out` holds already.
    pub(super) lead: &'a str,
    /// The parameter's name, percent-encoded where it goes into a URI.
    pub(super) name: &'a str,
    pub(super) s: Style,
    /// Pieces written so far.
    pub(super) n: usize,
    pub(super) key: String,
    pub(super) path: String,
    /// Whether the list inside a list being written has no item yet.
    pub(super) fresh: bool,
    /// A form's members its Encoding Objects say how to write, by key.
    pub(super) parts: &'a [(&'a str, Part)],
    /// Whether the form member being written is JSON: a string is then its JSON text.
    pub(super) json: bool,
}

impl<'a> W<'a> {
    pub(super) fn new(out: &'a mut String, lead: &'a str, name: &'a str, s: Style) -> Self {
        Self {
            out,
            lead,
            name,
            s,
            n: 0,
            key: String::new(),
            path: String::new(),
            fresh: false,
            parts: &[],
            json: false,
        }
    }

    /// The [`Part`] of the form member whose key is in `key`, where its Encoding Object says.
    pub(super) fn part(&self) -> Option<Part> {
        let key = self.key.as_str();
        self.parts.iter().find(|(k, _)| *k == key).map(|&(_, p)| p)
    }

    /// A form member written by style `s`, as a query parameter of that style named by its key
    /// percent-encoded (RFC 3986, as a query's): after `&` unless it is the form's first piece.
    pub(super) fn styled<T: Serialize + ?Sized>(&mut self, s: Style, v: &T) -> Result<(), Error> {
        let mut name = String::new();
        encode(&mut name, &self.key, Enc::Plain);
        let lead = if self.n == 0 { self.lead } else { "&" };
        let mut w = W::new(&mut *self.out, lead, &name, s);
        match v.serialize(Ser {
            w: &mut w,
            at: At::Top,
        }) {
            Ok(wrote) => {
                self.n += usize::from(wrote);
                Ok(())
            }
            Err(Error::NonFinite) => Err(Error::NonFinite),
            Err(Error::Compound | Error::Custom(_)) => Ok(()),
        }
    }

    /// A string's text; in a form member written as JSON, its JSON text.
    pub(super) fn string(&mut self, at: At, t: &str) -> bool {
        if self.json && matches!(at, At::FormMember | At::FormItem) {
            return self.text(at, &serde_json::to_string(t).unwrap_or_default());
        }
        self.text(at, t)
    }

    fn enc(&mut self, t: &str) {
        encode(self.out, t, self.s.enc);
    }

    /// Before the first piece: the lead, the operator's prefix, `name=` where it opens the list.
    pub(super) fn open(&mut self, list_named: bool) {
        if self.n == 0 {
            self.out.push_str(self.lead);
            self.out.push_str(self.s.first);
            if list_named && self.s.named && !self.s.explode {
                self.out.push_str(self.name);
                self.out.push('=');
            }
        } else {
            self.out.push_str(if self.s.explode {
                self.s.sep
            } else {
                self.s.join
            });
        }
        self.n += 1;
    }

    /// A list inside a list opened: as an item of the parameter's list (`d` none, `name=` before
    /// it where each item is a pair), or of the list at level `d`. Its items are a level down.
    pub(super) fn nest(&mut self, d: Option<usize>) -> At {
        match d {
            None => {
                self.open(true);
                if self.s.explode && self.s.named {
                    self.out.push_str(self.name);
                    self.out.push('=');
                }
            }
            Some(d) => self.join_nested(d),
        }
        self.fresh = true;
        At::Nest(d.map_or(0, |d| d + 1))
    }

    /// Before an item of the list at level `d`: its join, unless it is the list's first.
    fn join_nested(&mut self, d: usize) {
        if !self.fresh {
            let join = self.s.nest.get(d).copied().unwrap_or(",");
            self.out.push_str(join);
        }
        self.fresh = false;
    }

    /// `name`, then `=` and the value, or for an empty value what the operator writes then.
    fn named(&mut self, key_is_name: bool, t: &str, empty_ok: bool) {
        if key_is_name {
            self.out.push_str(self.name);
        }
        if t.is_empty() && empty_ok {
            if self.s.ifemp {
                self.out.push('=');
            }
        } else {
            self.out.push('=');
            self.enc(t);
        }
    }

    pub(super) fn text(&mut self, at: At, t: &str) -> bool {
        match at {
            At::Top if self.s.shape == Shape::Deep => false,
            At::Top => {
                self.out.push_str(self.lead);
                self.out.push_str(self.s.first);
                if self.s.named {
                    self.named(true, t, true)
                } else {
                    self.enc(t)
                }
                true
            }
            At::Item => {
                self.open(true);
                if self.s.explode && self.s.named {
                    self.named(true, t, true)
                } else {
                    self.enc(t)
                }
                true
            }
            At::Member => {
                self.open(true);
                let key = std::mem::take(&mut self.key);
                self.enc(&key);
                self.key = key;
                if self.s.explode {
                    let named = self.s.named;
                    self.named(false, t, named);
                } else {
                    self.out.push_str(self.s.join);
                    self.enc(t);
                }
                true
            }
            At::Deep | At::DeepItem => {
                self.out.push_str(if self.n == 0 { self.lead } else { "&" });
                self.n += 1;
                self.out.push_str(&self.path);
                self.out.push('=');
                self.enc(t);
                true
            }
            At::FormMember | At::FormItem => {
                self.out.push_str(if self.n == 0 { self.lead } else { "&" });
                self.n += 1;
                let key = std::mem::take(&mut self.key);
                self.enc(&key);
                self.key = key;
                self.out.push('=');
                self.enc(t);
                true
            }
            At::Form => {
                self.out.push_str(self.lead);
                self.enc(t);
                self.n += 1;
                true
            }
            At::Key => {
                self.key.push_str(t);
                true
            }
            At::Nest(d) => {
                self.join_nested(d);
                self.enc(t);
                true
            }
            At::Ignore => false,
        }
    }

    /// `null`: `name=` for the parameter where the operator writes `=` for an empty value (the
    /// "undefined" column of the spec's Style Examples), `key=` in a form, else nothing.
    pub(super) fn null(&mut self, at: At) -> bool {
        match at {
            At::Top if self.s.shape == Shape::Deep => false,
            At::Top => {
                if self.s.ifemp {
                    self.out.push_str(self.lead);
                    self.out.push_str(self.name);
                    self.out.push('=');
                }
                true
            }
            At::FormMember | At::FormItem => self.text(at, ""),
            At::Item
            | At::Member
            | At::Deep
            | At::DeepItem
            | At::Key
            | At::Form
            | At::Ignore
            | At::Nest(_) => false,
        }
    }
}

/// A number's text, written on the stack: no allocation for it.
pub(super) struct Digits {
    buf: [u8; 64],
    len: usize,
}

impl Default for Digits {
    fn default() -> Self {
        Self {
            buf: [0; 64],
            len: 0,
        }
    }
}

impl Digits {
    pub(super) fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| std::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

impl std::fmt::Write for Digits {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let end = self.len + s.len();
        let slot = self.buf.get_mut(self.len..end).ok_or(std::fmt::Error)?;
        slot.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
