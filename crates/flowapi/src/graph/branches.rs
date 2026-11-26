//! The values of each JSON kind a type allows. A branch that is `None` (or `false`) allows no
//! value of its kind; one that is there may narrow its kind further.

use crate::id::TypeId;
use std::borrow::Cow;

/// A set of JSON kinds, as bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Kinds(u8);

impl Kinds {
    /// `null`.
    pub const NULL: Self = Self(1);
    /// `true` and `false`.
    pub const BOOLEAN: Self = Self(2);
    /// Numbers, integers among them.
    pub const NUMBER: Self = Self(4);
    /// Strings.
    pub const STRING: Self = Self(8);
    /// Arrays.
    pub const ARRAY: Self = Self(16);
    /// Objects.
    pub const OBJECT: Self = Self(32);
    /// Every kind.
    pub const ALL: Self = Self(63);
    /// No kind.
    pub const NONE: Self = Self(0);

    /// Both sets' kinds.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The kinds of this set not in `other`.
    #[must_use]
    pub fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0 & Self::ALL.0)
    }

    /// The kinds in both sets.
    #[must_use]
    pub fn meet(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Whether every kind of `other` is in this set.
    #[must_use]
    pub fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Each kind of the set, alone.
    pub fn each(self) -> impl Iterator<Item = Self> {
        (0..6).map(|i| Self(1 << i)).filter(move |k| self.has(*k))
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// An `enum` or `const` value: its token as written, and the form two equal values share (a
/// string decoded, a number in its shortest form, `-0` as `0`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lit<'t> {
    /// The token, as written; a valid literal in TypeScript too.
    pub raw: &'t str,
    /// What equal values share.
    pub norm: Cow<'t, str>,
}

impl Lit<'_> {
    /// Whether it is an integer: a number with a zero fractional part, `1.0` among them.
    pub(crate) fn is_integer(&self) -> bool {
        self.raw
            .parse::<f64>()
            .is_ok_and(|v| v.is_finite() && v.fract() == 0.0)
    }

    /// The values both lists allow, in `a`'s order; `None` allows every value.
    pub(crate) fn common(a: Option<Vec<Self>>, b: Option<&[Self]>) -> Option<Vec<Self>> {
        match (a, b) {
            (Some(x), Some(y)) => Some(
                x.into_iter()
                    .filter(|l| y.iter().any(|e| e.norm == l.norm))
                    .collect(),
            ),
            (x, None) => x,
            (None, y) => y.map(<[Self]>::to_vec),
        }
    }
}

/// Which booleans a type allows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bools {
    /// `true`.
    pub allow_true: bool,
    /// `false`.
    pub allow_false: bool,
}

/// The numbers a type allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Numbers<'t> {
    /// Only integers (a number with a zero fractional part, `1.0` among them).
    pub integer: bool,
    /// Only these values; `None` for any.
    pub lits: Option<Vec<Lit<'t>>>,
}

/// The strings a type allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strings<'t> {
    /// The `format` named, an annotation the type does not check.
    pub format: Option<Cow<'t, str>>,
    /// Only these values; `None` for any.
    pub lits: Option<Vec<Lit<'t>>>,
}

/// The arrays a type allows: the first items by position, then the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Array {
    /// The type of each of the first items (`prefixItems`), any of which may be missing unless
    /// `min` says otherwise.
    pub prefix: Vec<TypeId>,
    /// The type of every item after them; `Never` for none.
    pub rest: TypeId,
    /// At least this many items (`minItems`).
    pub min: usize,
}

/// A property an object type names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prop<'t> {
    /// The key's token, quotes included: a valid TypeScript property name as written. A
    /// property the graph names itself (an operation's `parameters`) has a token made for it.
    pub raw: Cow<'t, str>,
    /// The key, decoded.
    pub name: Cow<'t, str>,
    /// Its value's type.
    pub ty: TypeId,
    /// Whether the key must be there.
    pub required: bool,
}

/// A `patternProperties` entry: the values of the properties whose names its regular
/// expression matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern<'t> {
    /// The regular expression, as written (ECMA-262, not anchored unless it says so).
    pub regex: Cow<'t, str>,
    /// The expression as a literal text, when it is one.
    pub affix: Option<super::Affix>,
    /// The type of a matching property's value.
    pub ty: TypeId,
}

/// The objects a type allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object<'t> {
    /// The properties named, in the order written.
    pub props: Vec<Prop<'t>>,
    /// The pattern properties; a target that can state a pattern narrows by them, one that
    /// cannot reads `extra`, which allows their values too.
    pub patterns: Vec<Pattern<'t>>,
    /// The type of a property neither named nor matching a pattern (`additionalProperties`).
    pub rest: TypeId,
    /// The type of any other property's value; `Never` when no other is allowed, `Any` when any is.
    pub extra: TypeId,
}

/// The values of each kind a type allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branches<'t> {
    /// `null`.
    pub null: bool,
    /// Booleans.
    pub boolean: Bools,
    /// Numbers.
    pub number: Option<Numbers<'t>>,
    /// Strings.
    pub string: Option<Strings<'t>>,
    /// Arrays.
    pub array: Option<Array>,
    /// Objects.
    pub object: Option<Object<'t>>,
}

impl<'t> Branches<'t> {
    /// Every value of the kinds given, each unnarrowed; `any` is the graph's `Any` type.
    #[must_use]
    pub fn of(kinds: Kinds, any: TypeId) -> Self {
        Self {
            null: kinds.has(Kinds::NULL),
            boolean: Bools {
                allow_true: kinds.has(Kinds::BOOLEAN),
                allow_false: kinds.has(Kinds::BOOLEAN),
            },
            number: kinds.has(Kinds::NUMBER).then_some(Numbers {
                integer: false,
                lits: None,
            }),
            string: kinds.has(Kinds::STRING).then_some(Strings {
                format: None,
                lits: None,
            }),
            array: kinds.has(Kinds::ARRAY).then_some(Array {
                prefix: Vec::new(),
                rest: any,
                min: 0,
            }),
            object: kinds.has(Kinds::OBJECT).then_some(Object {
                props: Vec::new(),
                patterns: Vec::new(),
                rest: any,
                extra: any,
            }),
        }
    }

    /// The kinds this allows any value of.
    #[must_use]
    pub fn kinds(&self) -> Kinds {
        let mut k = Kinds::NONE;
        if self.null {
            k = k.union(Kinds::NULL);
        }
        if self.boolean.allow_true || self.boolean.allow_false {
            k = k.union(Kinds::BOOLEAN);
        }
        if self
            .number
            .as_ref()
            .is_some_and(|n| n.lits.as_ref().is_none_or(|l| !l.is_empty()))
        {
            k = k.union(Kinds::NUMBER);
        }
        if self
            .string
            .as_ref()
            .is_some_and(|s| s.lits.as_ref().is_none_or(|l| !l.is_empty()))
        {
            k = k.union(Kinds::STRING);
        }
        if self.array.is_some() {
            k = k.union(Kinds::ARRAY);
        }
        if self.object.is_some() {
            k = k.union(Kinds::OBJECT);
        }
        k
    }

    /// Keeps only the kinds in `kinds`.
    pub fn keep(&mut self, kinds: Kinds) {
        self.null &= kinds.has(Kinds::NULL);
        if !kinds.has(Kinds::BOOLEAN) {
            self.boolean = Bools::default();
        }
        if !kinds.has(Kinds::NUMBER) {
            self.number = None;
        }
        if !kinds.has(Kinds::STRING) {
            self.string = None;
        }
        if !kinds.has(Kinds::ARRAY) {
            self.array = None;
        }
        if !kinds.has(Kinds::OBJECT) {
            self.object = None;
        }
    }
}
