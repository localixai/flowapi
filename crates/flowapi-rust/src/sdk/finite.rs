//! A walk of a value that stops at NaN or an infinity: serde_json writes them as `null`, so a
//! JSON body and a value's JSON text are walked first. It writes nothing and allocates nothing.
//! Printed once inside the generated `sdk` module, never compiled here.

use serde::Serialize;
use serde::ser;
use std::fmt::Display;

/// Why the walk stopped: a number JSON has none for, or the value's own serde error (left to
/// serde_json to report as it does).
#[derive(Debug)]
pub(super) enum Stop {
    NonFinite,
    Other,
}

impl Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "NaN or an infinity",
            Self::Other => "the value's own error",
        })
    }
}

impl std::error::Error for Stop {}

impl ser::Error for Stop {
    fn custom<T: Display>(_: T) -> Self {
        Self::Other
    }
}

/// Whether a value holds no NaN and no infinity.
pub(super) fn finite<T: Serialize + ?Sized>(v: &T) -> bool {
    !matches!(v.serialize(Walk), Err(Stop::NonFinite))
}

pub(super) struct Walk;

macro_rules! scalars {
    ($($f:ident: $t:ty),*) => {
        $(fn $f(self, _: $t) -> Result<(), Stop> { Ok(()) })*
    };
}

impl ser::Serializer for Walk {
    type Ok = ();
    type Error = Stop;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    scalars!(serialize_bool: bool, serialize_i8: i8, serialize_i16: i16, serialize_i32: i32,
        serialize_i64: i64, serialize_i128: i128, serialize_u8: u8, serialize_u16: u16,
        serialize_u32: u32, serialize_u64: u64, serialize_u128: u128, serialize_char: char,
        serialize_str: &str, serialize_bytes: &[u8], serialize_unit_struct: &'static str);

    fn serialize_f32(self, v: f32) -> Result<(), Stop> {
        self.serialize_f64(f64::from(v))
    }
    fn serialize_f64(self, v: f64) -> Result<(), Stop> {
        if v.is_finite() { Ok(()) } else { Err(Stop::NonFinite) }
    }
    fn serialize_none(self) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Stop> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Stop> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<(), Stop> {
        v.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Stop> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self, Stop> {
        Ok(self)
    }
}

macro_rules! items {
    ($($t:ident: $f:ident),*) => {$(
        impl ser::$t for Walk {
            type Ok = ();
            type Error = Stop;
            fn $f<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Stop> { v.serialize(Walk) }
            fn end(self) -> Result<(), Stop> { Ok(()) }
        }
    )*};
}

items!(SerializeSeq: serialize_element, SerializeTuple: serialize_element,
    SerializeTupleStruct: serialize_field, SerializeTupleVariant: serialize_field);

macro_rules! fields {
    ($($t:ident),*) => {$(
        impl ser::$t for Walk {
            type Ok = ();
            type Error = Stop;
            fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, v: &T) -> Result<(), Stop> {
                v.serialize(Walk)
            }
            fn end(self) -> Result<(), Stop> { Ok(()) }
        }
    )*};
}

fields!(SerializeStruct, SerializeStructVariant);

impl ser::SerializeMap for Walk {
    type Ok = ();
    type Error = Stop;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, _: &T) -> Result<(), Stop> {
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Stop> {
        v.serialize(Walk)
    }
    fn end(self) -> Result<(), Stop> {
        Ok(())
    }
}
