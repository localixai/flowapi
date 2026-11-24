//! What a type is in Rust's terms, before it is printed: the JSON kinds and shapes it allows as
//! the variants of a choice ([`Part`]), and an object, or an intersection of objects merged, as
//! the fields of a struct ([`Shape`]). Every reading keeps the type's values or widens them to
//! `serde_json::Value`, never narrows them (G3).

use flowapi::Program;
use flowapi::graph::{Array, Branches, Kind, Kinds, Lit, Object};
use flowapi::id::TypeId;

/// The type of a field, a map's value or an array's item: a type, or every value, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Ref(TypeId),
    Any,
    Never,
}

impl Slot {
    pub(crate) fn of(p: &Program<'_>, id: TypeId) -> Self {
        if p.is_any(id) {
            Self::Any
        } else if p.is_never(id) {
            Self::Never
        } else {
            Self::Ref(p.resolve(id))
        }
    }

    /// Both slots' values: a type the other allows every value of, or `Any` for two types no
    /// Rust type states the meet of (sound, and recorded by the caller).
    fn meet(self, other: Self, widened: &mut bool) -> Self {
        match (self, other) {
            (Self::Any, s) | (s, Self::Any) => s,
            (Self::Never, _) | (_, Self::Never) => Self::Never,
            (Self::Ref(a), Self::Ref(b)) if a == b => self,
            (Self::Ref(_), Self::Ref(_)) => {
                *widened = true;
                Self::Any
            }
        }
    }
}

/// A field of a struct.
#[derive(Clone, Debug)]
pub(crate) struct Field<'p> {
    /// The key, decoded.
    pub(crate) name: &'p str,
    pub(crate) ty: Slot,
    pub(crate) required: bool,
    /// The type whose title and description document the field.
    pub(crate) doc: Option<TypeId>,
}

/// A struct: named fields, and the type of every other property's value.
#[derive(Clone, Debug)]
pub(crate) struct Shape<'p> {
    pub(crate) fields: Vec<Field<'p>>,
    pub(crate) extra: Slot,
    /// Members of an intersection disagreed on a field, which is `serde_json::Value` there.
    pub(crate) widened: bool,
}

impl<'p> Shape<'p> {
    pub(crate) fn of(p: &Program<'_>, o: &'p Object<'_>) -> Self {
        Self {
            fields: o
                .props
                .iter()
                .map(|q| Field {
                    name: q.name.as_ref(),
                    ty: Slot::of(p, q.ty),
                    required: q.required,
                    doc: Some(q.ty),
                })
                .collect(),
            extra: Slot::of(p, o.extra),
            widened: false,
        }
    }
}

/// One variant of a choice.
#[derive(Clone, Debug)]
pub(crate) enum Part<'p, 't> {
    Null,
    Bool,
    Int,
    Num,
    Str,
    /// Only these strings.
    Lits(Vec<&'p Lit<'t>>),
    Array(&'p Array),
    /// An object, from the type `src`.
    Object(Shape<'p>, TypeId),
    /// Another type, printed by reference.
    Ref(TypeId),
}

/// What a type is: every value, or a choice among parts (none: no value).
pub(crate) enum Choice<'p, 't> {
    Any,
    Parts(Vec<Part<'p, 't>>),
}

/// A type's own reading, its name aside.
pub(crate) fn expand<'p, 't>(p: &'p Program<'t>, id: TypeId) -> Choice<'p, 't> {
    let mut parts = Vec::new();
    match &p.get(id).kind {
        Kind::Any => return Choice::Any,
        Kind::Never => {}
        Kind::Alias(t) => match Slot::of(p, *t) {
            Slot::Any => return Choice::Any,
            Slot::Never => {}
            Slot::Ref(r) => parts.push(Part::Ref(r)),
        },
        Kind::Value(b) => branches(p, b, id, &mut parts),
        Kind::Union(ms) => {
            for &m in ms {
                match Slot::of(p, m) {
                    Slot::Any => return Choice::Any,
                    Slot::Never => {}
                    Slot::Ref(r) => match &p.get(r).kind {
                        Kind::Value(b) if p.name_of(r).is_none() => branches(p, b, r, &mut parts),
                        Kind::Any
                        | Kind::Never
                        | Kind::Value(_)
                        | Kind::Union(_)
                        | Kind::Intersection(_)
                        | Kind::Alias(_) => parts.push(Part::Ref(r)),
                    },
                }
            }
        }
        Kind::Intersection(ms) => {
            if ms.iter().any(|&m| p.is_never(m)) {
                return Choice::Parts(parts);
            }
            match merge(p, ms) {
                Some(shape) => parts.push(Part::Object(shape, id)),
                None => return Choice::Any,
            }
        }
    }
    Choice::Parts(normalize(p, parts))
}

fn branches<'p, 't>(
    p: &'p Program<'t>,
    b: &'p Branches<'t>,
    src: TypeId,
    out: &mut Vec<Part<'p, 't>>,
) {
    let kinds = b.kinds();
    if let Some(o) = &b.object {
        out.push(Part::Object(Shape::of(p, o), src));
    }
    if let Some(a) = &b.array {
        out.push(Part::Array(a));
    }
    if let Some(s) = b.string.as_ref().filter(|_| kinds.has(Kinds::STRING)) {
        out.push(match &s.lits {
            Some(lits) => Part::Lits(lits.iter().collect()),
            None => Part::Str,
        });
    }
    if let Some(n) = b.number.as_ref().filter(|_| kinds.has(Kinds::NUMBER)) {
        // Literal numbers are widened to their kind: Rust has no literal number type.
        out.push(if n.integer { Part::Int } else { Part::Num });
    }
    if kinds.has(Kinds::BOOLEAN) {
        out.push(Part::Bool);
    }
    if b.null {
        out.push(Part::Null);
    }
}

/// One part per primitive kind (`Str` takes in literals, `Num` takes in `Int`), each type
/// once, then ordered so that an untagged enum tries the narrowest variant first.
fn normalize<'p, 't>(p: &Program<'_>, parts: Vec<Part<'p, 't>>) -> Vec<Part<'p, 't>> {
    let has = |f: fn(&Part<'_, '_>) -> bool| parts.iter().any(f);
    let any_str = has(|x| matches!(x, Part::Str));
    let any_num = has(|x| matches!(x, Part::Num));
    let mut out: Vec<Part<'p, 't>> = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            Part::Lits(_) if any_str => {}
            Part::Int if any_num => {}
            Part::Lits(lits) => {
                if let Some(Part::Lits(kept)) = out.iter_mut().find(|x| matches!(x, Part::Lits(_)))
                {
                    for l in lits {
                        if !kept.iter().any(|k| k.norm == l.norm) {
                            kept.push(l);
                        }
                    }
                } else {
                    out.push(Part::Lits(lits));
                }
            }
            Part::Null | Part::Bool | Part::Int | Part::Num | Part::Str => {
                if !out
                    .iter()
                    .any(|x| std::mem::discriminant(x) == std::mem::discriminant(&part))
                {
                    out.push(part);
                }
            }
            Part::Ref(r) => {
                if !out.iter().any(|x| matches!(x, Part::Ref(k) if *k == r)) {
                    out.push(Part::Ref(r));
                }
            }
            Part::Array(_) | Part::Object(..) => out.push(part),
        }
    }
    out.sort_by_key(|x| rank(p, x));
    out
}

/// How wide a part is: primitives and literals first, then arrays (a struct reads an array by
/// position, so an array must meet its own variant first), then objects with more required
/// fields, then plain strings and numbers, which a named type of one of them narrows.
fn rank(p: &Program<'_>, part: &Part<'_, '_>) -> (u8, usize) {
    let fields = |s: &Shape<'_>| usize::MAX - s.fields.iter().filter(|f| f.required).count();
    match part {
        Part::Null | Part::Bool | Part::Int | Part::Lits(_) => (0, 0),
        Part::Array(_) => (1, 0),
        Part::Object(s, _) => (2, fields(s)),
        Part::Ref(r) => match &p.get(*r).kind {
            Kind::Value(b) => match &b.object {
                Some(o) => (2, fields(&Shape::of(p, o))),
                None => (1, 0),
            },
            Kind::Any => (9, 0),
            Kind::Never | Kind::Union(_) | Kind::Intersection(_) | Kind::Alias(_) => (3, 0),
        },
        Part::Num | Part::Str => (4, 0),
    }
}

/// An intersection of object types merged into one struct: a field for every property any
/// member names, its type the meet of what each member says of it (its own property, or its
/// other properties' type); `None` when a member is not an object type.
pub(crate) fn merge<'p>(p: &'p Program<'_>, members: &[TypeId]) -> Option<Shape<'p>> {
    let mut objects: Vec<&'p Object<'_>> = Vec::with_capacity(members.len());
    for &m in members {
        let r = p.resolve(m);
        match &p.get(r).kind {
            Kind::Value(b) if b.kinds() == Kinds::OBJECT => objects.push(b.object.as_ref()?),
            Kind::Any
            | Kind::Never
            | Kind::Value(_)
            | Kind::Union(_)
            | Kind::Intersection(_)
            | Kind::Alias(_) => return None,
        }
    }
    let mut widened = false;
    let mut fields: Vec<Field<'p>> = Vec::new();
    for o in &objects {
        for q in &o.props {
            if fields.iter().any(|f| f.name == q.name.as_ref()) {
                continue;
            }
            let mut ty = Slot::Any;
            let mut required = false;
            for other in &objects {
                let own = other.props.iter().find(|x| x.name == q.name);
                required |= own.is_some_and(|x| x.required);
                let said = Slot::of(p, own.map_or(other.extra, |x| x.ty));
                ty = ty.meet(said, &mut widened);
            }
            fields.push(Field {
                name: q.name.as_ref(),
                ty,
                required,
                doc: Some(q.ty),
            });
        }
    }
    let mut extra = Slot::Any;
    for o in &objects {
        extra = extra.meet(Slot::of(p, o.extra), &mut widened);
    }
    Some(Shape {
        fields,
        extra,
        widened,
    })
}

/// Whether the Rust type of a slot reads `null`: what `deserialize_some` is for.
pub(crate) fn nullable(p: &Program<'_>, slot: Slot) -> bool {
    let Slot::Ref(start) = slot else {
        return matches!(slot, Slot::Any);
    };
    let mut seen: Vec<bool> = Vec::new();
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        let id = p.resolve(id);
        if seen.len() <= id.index() {
            seen.resize(id.index() + 1, false);
        }
        if seen.get(id.index()).copied().unwrap_or(true) {
            continue;
        }
        if let Some(s) = seen.get_mut(id.index()) {
            *s = true;
        }
        match &p.get(id).kind {
            Kind::Any => return true,
            Kind::Never => {}
            Kind::Value(b) if b.null => return true,
            Kind::Value(_) => {}
            Kind::Alias(t) => stack.push(*t),
            Kind::Union(ms) => stack.extend(ms.iter().copied()),
            Kind::Intersection(ms) => {
                if !ms.iter().any(|&m| p.is_never(m)) && merge(p, ms).is_none() {
                    return true;
                }
            }
        }
    }
    false
}
