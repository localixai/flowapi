//! A serde `Serializer` that writes one parameter's value as its style says, straight into the
//! request's string: printed once inside the generated `sdk` module, never compiled here (this
//! crate has no serde). A value is one of RFC 6570's four (§2.3): `null`, a string (a number or
//! boolean as its text), a list (a sequence), or an associative array (a map or a struct, its
//! members in the order they are serialized). An item or member that is itself a list or an
//! associative array is its JSON text, but for a list in a list that Swagger 2.0's Items Object
//! joins (`Style::nest`). The `form` modes write `application/x-www-form-urlencoded`.

use super::write::{At, Digits, W};
use super::{Part, Shape, encode};
use serde::Serialize;
use serde::ser;
use std::fmt::{Display, Write as _};

/// The serializer of one value at one place.
pub(super) struct Ser<'w, 'a> {
    pub(super) w: &'w mut W<'a>,
    pub(super) at: At,
}

/// Why a value was not written: it is a list or an associative array where only its JSON text
/// goes (caught by the caller, who writes the JSON), NaN or an infinity (refused), or serde's
/// own error.
#[derive(Debug)]
pub enum Error {
    Compound,
    NonFinite,
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compound => f.write_str("a list or an object where a value goes"),
            Self::NonFinite => f.write_str("NaN or an infinity, which JSON has no number for"),
            Self::Custom(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl<'w, 'a> Ser<'w, 'a> {
    fn number(self, v: impl Display) -> Result<bool, Error> {
        let mut d = Digits::default();
        let _ = write!(d, "{v}");
        Ok(self.w.text(self.at, d.as_str()))
    }

    /// A number as JavaScript's `String(n)` writes it, as the TypeScript SDK does: the
    /// shortest digits that read back, in decimal from 1e-6 up to 1e21 and with an exponent
    /// outside (`1e+21`, `1e-7`); `0` for both zeros. NaN and the infinities are refused: JSON
    /// has no number for them, and their text would reach the server as a string.
    fn float(self, v: f64) -> Result<bool, Error> {
        if !v.is_finite() {
            return Err(Error::NonFinite);
        }
        let mut d = Digits::default();
        let a = v.abs();
        let _ = if v == 0.0 {
            d.write_str("0")
        } else if (1e-6..1e21).contains(&a) {
            write!(d, "{v}")
        } else {
            let mut e = Digits::default();
            let _ = write!(e, "{v:e}");
            let (m, x) = e.as_str().split_once('e').unwrap_or((e.as_str(), "0"));
            let sign = if x.starts_with('-') { "" } else { "+" };
            write!(d, "{m}e{sign}{x}")
        };
        Ok(self.w.text(self.at, d.as_str()))
    }

    /// What a list's items or an associative array's members are, or `Compound` where the
    /// value must be its JSON text.
    fn inner(self, map: bool) -> Result<Many<'w, 'a>, Error> {
        let deep = self.w.s.shape == Shape::Deep;
        let at = match (self.at, map) {
            (At::Top, _) if deep && !map => At::Ignore,
            (At::Top, true) if deep => At::Deep,
            (At::Top, true) => At::Member,
            (At::Top, false) => At::Item,
            (At::Deep, false) => At::DeepItem,
            (At::Deep, true) => At::Deep,
            (At::Form, true) => At::FormMember,
            (At::FormMember, false) => At::FormItem,
            (At::Item, false) if !self.w.s.nest.is_empty() => self.w.nest(None),
            (At::Nest(d), false) if d + 1 < self.w.s.nest.len() => self.w.nest(Some(d)),
            (At::Ignore, _) => At::Ignore,
            (At::Key, _) => return Err(Error::Custom(String::from("a key that is not a string"))),
            _ => return Err(Error::Compound),
        };
        let base = self.w.path.len();
        if self.at == At::Top && deep {
            self.w.path.clear();
            self.w.path.push_str(self.w.name);
        }
        Ok(Many {
            w: self.w,
            at,
            base,
        })
    }
}

/// A list or an associative array being written; `base` is the length of `W::path` to return to.
pub(super) struct Many<'w, 'a> {
    w: &'w mut W<'a>,
    at: At,
    base: usize,
}

impl Many<'_, '_> {
    /// One item, or one member's value once its key is in `W::key` (or `W::path`).
    fn one<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        match v.serialize(Ser {
            w: self.w,
            at: self.at,
        }) {
            Ok(_) => Ok(()),
            Err(Error::Compound) => {
                let json = serde_json::to_string(v).map_err(|e| Error::Custom(e.to_string()))?;
                // A non-finite number is `null` in the text: walked only when there is one.
                if json.contains("null") && !super::finite::finite(v) {
                    return Err(Error::NonFinite);
                }
                self.w.text(self.at, &json);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn member<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        if self.at == At::FormMember {
            match self.w.part() {
                Some(Part::Style(s)) => return self.w.styled(s, v),
                Some(Part::Json) => self.w.json = true,
                None => {}
            }
            let r = self.one(v);
            self.w.json = false;
            return r;
        }
        if self.at != At::Deep {
            return self.one(v);
        }
        let len = self.w.path.len();
        self.w.path.push_str("%5B");
        encode(&mut self.w.path, &self.w.key, self.w.s.enc);
        self.w.path.push_str("%5D");
        let r = self.one(v);
        self.w.path.truncate(len);
        r
    }

    fn key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Error> {
        self.w.key.clear();
        k.serialize(Ser {
            w: self.w,
            at: At::Key,
        })
        .map(|_| ())
    }

    fn end(self) -> bool {
        // A list inside a list ended: the next item of the list around it is joined to it.
        if matches!(self.at, At::Nest(_)) {
            self.w.fresh = false;
        }
        self.w.path.truncate(self.base);
        self.w.n > 0
    }
}

macro_rules! numbers {
    ($($f:ident: $t:ty),*) => {
        $(fn $f(self, v: $t) -> Result<bool, Error> { self.number(v) })*
    };
}

impl<'w, 'a> ser::Serializer for Ser<'w, 'a> {
    type Ok = bool;
    type Error = Error;
    type SerializeSeq = Many<'w, 'a>;
    type SerializeTuple = Many<'w, 'a>;
    type SerializeTupleStruct = Many<'w, 'a>;
    type SerializeTupleVariant = Many<'w, 'a>;
    type SerializeMap = Many<'w, 'a>;
    type SerializeStruct = Many<'w, 'a>;
    type SerializeStructVariant = Many<'w, 'a>;

    numbers!(serialize_i8: i8, serialize_i16: i16, serialize_i32: i32, serialize_i64: i64,
        serialize_i128: i128, serialize_u8: u8, serialize_u16: u16, serialize_u32: u32,
        serialize_u64: u64, serialize_u128: u128);

    fn serialize_f32(self, v: f32) -> Result<bool, Error> {
        self.float(f64::from(v))
    }
    fn serialize_f64(self, v: f64) -> Result<bool, Error> {
        self.float(v)
    }

    fn serialize_bool(self, v: bool) -> Result<bool, Error> {
        Ok(self.w.text(self.at, if v { "true" } else { "false" }))
    }
    fn serialize_char(self, v: char) -> Result<bool, Error> {
        Ok(self.w.string(self.at, v.encode_utf8(&mut [0; 4])))
    }
    fn serialize_str(self, v: &str) -> Result<bool, Error> {
        Ok(self.w.string(self.at, v))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<bool, Error> {
        Ok(self.w.text(self.at, &String::from_utf8_lossy(v)))
    }
    fn serialize_none(self) -> Result<bool, Error> {
        Ok(self.w.null(self.at))
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<bool, Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<bool, Error> {
        Ok(self.w.null(self.at))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<bool, Error> {
        Ok(self.w.null(self.at))
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<bool, Error> {
        Ok(self.w.string(self.at, v))
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<bool, Error> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<bool, Error> {
        v.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Many<'w, 'a>, Error> {
        self.inner(false)
    }
    fn serialize_tuple(self, _: usize) -> Result<Many<'w, 'a>, Error> {
        self.inner(false)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Many<'w, 'a>, Error> {
        self.inner(false)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Many<'w, 'a>, Error> {
        self.inner(false)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Many<'w, 'a>, Error> {
        self.inner(true)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Many<'w, 'a>, Error> {
        self.inner(true)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Many<'w, 'a>, Error> {
        self.inner(true)
    }
}

macro_rules! items {
    ($($t:ident: $f:ident),*) => {$(
        impl ser::$t for Many<'_, '_> {
            type Ok = bool;
            type Error = Error;
            fn $f<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> { self.one(v) }
            fn end(self) -> Result<bool, Error> { Ok(Many::end(self)) }
        }
    )*};
}

items!(SerializeSeq: serialize_element, SerializeTuple: serialize_element,
    SerializeTupleStruct: serialize_field, SerializeTupleVariant: serialize_field);

macro_rules! fields {
    ($($t:ident),*) => {$(
        impl ser::$t for Many<'_, '_> {
            type Ok = bool;
            type Error = Error;
            fn serialize_field<T: Serialize + ?Sized>(&mut self, k: &'static str, v: &T) -> Result<(), Error> {
                self.w.key.clear();
                self.w.key.push_str(k);
                self.member(v)
            }
            fn end(self) -> Result<bool, Error> { Ok(Many::end(self)) }
        }
    )*};
}

fields!(SerializeStruct, SerializeStructVariant);

impl ser::SerializeMap for Many<'_, '_> {
    type Ok = bool;
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Error> {
        self.key(k)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        self.member(v)
    }
    fn serialize_entry<K: Serialize + ?Sized, V: Serialize + ?Sized>(
        &mut self,
        k: &K,
        v: &V,
    ) -> Result<(), Error> {
        self.key(k)?;
        self.member(v)
    }
    fn end(self) -> Result<bool, Error> {
        Ok(Many::end(self))
    }
}
