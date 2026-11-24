//! The emitter: every declaration of the program becomes an item, and every unnamed struct or
//! enum it holds becomes an item of its own, named after where it is (`Pet` `owner` is
//! `PetOwner`) and printed after its parent. A reference renders as text; a reference that
//! closes a cycle of by-value holds is boxed, as the [`Plan`] of a first pass says.

use crate::ident::{Taken, pascal};
use crate::shape::{Choice, Part, Shape, Slot, expand};
use flowapi::Program;
use flowapi::graph::{Array, Lit};
use flowapi::id::TypeId;
use std::collections::BTreeMap;

/// What the first pass found: the references to box, the aliases to make newtypes.
#[derive(Default)]
pub(crate) struct Plan {
    /// Edges `(from, to)` between items, sorted.
    pub(crate) boxed: Vec<(usize, usize)>,
    /// Items, sorted.
    pub(crate) newtype: Vec<usize>,
}

/// What an item is printed from.
pub(crate) enum Job<'p, 't> {
    Decl(TypeId),
    Shape(Shape<'p>, TypeId),
    Lits(Vec<&'p Lit<'t>>),
    Choice(Vec<Part<'p, 't>>),
}

/// An item: its name, what it is printed from, the items first named inside it.
pub(crate) struct Item<'p, 't> {
    pub(crate) name: String,
    pub(crate) job: Option<Job<'p, 't>>,
    pub(crate) children: Vec<usize>,
    pub(crate) text: String,
    /// Printed as `pub type`.
    pub(crate) alias: bool,
}

/// What the output needs besides serde's traits.
#[derive(Default)]
pub(crate) struct Uses {
    pub(crate) map: bool,
    pub(crate) helpers: Helpers,
}

/// The items the file defines only when a type uses them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Helpers {
    pub(crate) never: bool,
    pub(crate) some: bool,
    pub(crate) integer: bool,
}

/// Keys of the unnamed items, so that a type reached twice is one item.
const OBJECT: u8 = 1;
const LITS: u8 = 2;
const CHOICE: u8 = 3;
/// The literals of an untagged item's string variant, keyed by the item.
pub(crate) const ITEM_LITS: u8 = 4;

pub(crate) struct Emitter<'p, 't> {
    pub(crate) p: &'p Program<'t>,
    pub(crate) items: Vec<Item<'p, 't>>,
    decl_of: BTreeMap<usize, usize>,
    memo: BTreeMap<(usize, u8), usize>,
    names: Taken,
    /// `(from, to, by value)`.
    pub(crate) edges: Vec<(usize, usize, bool)>,
    pub(crate) plan: Plan,
    pub(crate) uses: Uses,
}

impl<'p, 't> Emitter<'p, 't> {
    pub(crate) fn new(p: &'p Program<'t>, plan: Plan, reserved: &[&str]) -> Self {
        let decls = p.decls();
        let mut names = Taken::of(reserved.iter().copied());
        let mut items = Vec::with_capacity(decls.len() * 2);
        let mut decl_of = BTreeMap::new();
        for d in decls {
            names.take(&d.name);
            decl_of.insert(d.ty.index(), items.len());
            items.push(Item {
                name: d.name.clone(),
                job: Some(Job::Decl(d.ty)),
                children: Vec::new(),
                text: String::new(),
                alias: false,
            });
        }
        Self {
            p,
            items,
            decl_of,
            memo: BTreeMap::new(),
            names,
            edges: Vec::new(),
            plan,
            uses: Uses::default(),
        }
    }

    /// Prints every item: each declaration, then the items named inside it, depth first.
    pub(crate) fn run(&mut self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.items.len());
        let decls = self.decl_of.len();
        for d in 0..decls {
            let mut stack = vec![d];
            while let Some(i) = stack.pop() {
                order.push(i);
                self.print(i);
                if let Some(item) = self.items.get(i) {
                    stack.extend(item.children.iter().rev().copied());
                }
            }
        }
        order
    }

    fn print(&mut self, i: usize) {
        let Some(job) = self.items.get_mut(i).and_then(|it| it.job.take()) else {
            return;
        };
        let mut out = String::new();
        match job {
            Job::Decl(id) => crate::item::decl(self, i, id, &mut out),
            Job::Shape(shape, src) => crate::item::shape(self, i, &shape, src, &mut out),
            Job::Lits(lits) => crate::item::lits(self, i, &lits, &mut out),
            Job::Choice(parts) => crate::item::untagged(self, i, &parts, &mut out),
        }
        if let Some(item) = self.items.get_mut(i) {
            item.text = out;
        }
    }

    pub(crate) fn name(&self, i: usize) -> &str {
        self.items.get(i).map_or("Never", |it| it.name.as_str())
    }

    /// A reference to an item, boxed where it closes a cycle of by-value holds.
    fn item_ref(&mut self, cur: usize, to: usize, direct: bool) -> String {
        self.edges.push((cur, to, direct));
        let name = self.name(to).to_owned();
        if direct && self.plan.boxed.binary_search(&(cur, to)).is_ok() {
            format!("Box<{name}>")
        } else {
            name
        }
    }

    /// An unnamed item, named after its parent and `ctx`; the same key gives the same item.
    fn alloc(&mut self, cur: usize, key: (usize, u8), ctx: &str, job: Job<'p, 't>) -> usize {
        if let Some(&i) = self.memo.get(&key) {
            return i;
        }
        let base = format!("{}{}", self.name(cur), pascal(ctx, "Type", ""));
        let name = self.names.take(&base);
        let i = self.items.len();
        self.items.push(Item {
            name,
            job: Some(job),
            children: Vec::new(),
            text: String::new(),
            alias: false,
        });
        if let Some(parent) = self.items.get_mut(cur) {
            parent.children.push(i);
        }
        self.memo.insert(key, i);
        i
    }

    pub(crate) fn never(&mut self) -> String {
        self.uses.helpers.never = true;
        String::from("Never")
    }

    /// A slot's Rust type; `direct` when it is held by value, not behind a `Vec` or a map.
    pub(crate) fn slot(&mut self, cur: usize, slot: Slot, ctx: &str, direct: bool) -> String {
        match slot {
            Slot::Any => String::from("serde_json::Value"),
            Slot::Never => self.never(),
            Slot::Ref(id) => self.reference(cur, id, ctx, direct),
        }
    }

    pub(crate) fn reference(&mut self, cur: usize, id: TypeId, ctx: &str, direct: bool) -> String {
        let p = self.p;
        let r = p.resolve(id);
        if let Some(&to) = self
            .decl_of
            .get(&r.index())
            .filter(|_| p.name_of(r).is_some())
        {
            return self.item_ref(cur, to, direct);
        }
        match expand(p, r) {
            Choice::Any => String::from("serde_json::Value"),
            Choice::Parts(parts) => self.choice(cur, r, parts, ctx, direct),
        }
    }

    /// A choice's Rust type: none is `Never`, one is that part, a part and `null` an `Option`,
    /// any other an untagged enum.
    pub(crate) fn choice(
        &mut self,
        cur: usize,
        key: TypeId,
        mut parts: Vec<Part<'p, 't>>,
        ctx: &str,
        direct: bool,
    ) -> String {
        if parts.len() == 2
            && let Some(at) = parts.iter().position(|x| matches!(x, Part::Null))
        {
            parts.remove(at);
            if let Some(other) = parts.pop() {
                let inner = self.part(cur, (key.index(), LITS), other, ctx, direct);
                return format!("Option<{inner}>");
            }
        }
        match parts.len() {
            0 => self.never(),
            1 => match parts.pop() {
                Some(only) => self.part(cur, (key.index(), LITS), only, ctx, direct),
                None => self.never(),
            },
            _ => {
                let to = self.alloc(cur, (key.index(), CHOICE), ctx, Job::Choice(parts));
                self.item_ref(cur, to, direct)
            }
        }
    }

    pub(crate) fn part(
        &mut self,
        cur: usize,
        lits_key: (usize, u8),
        part: Part<'p, 't>,
        ctx: &str,
        direct: bool,
    ) -> String {
        match part {
            Part::Null => String::from("()"),
            Part::Bool => String::from("bool"),
            Part::Int => {
                // JSON Schema's integer is any number with no fraction, `1.0` among them, which
                // serde's `i64` refuses: the printed `Integer` reads both.
                self.uses.helpers.integer = true;
                String::from("Integer")
            }
            Part::Num => String::from("f64"),
            Part::Str => String::from("String"),
            Part::Lits(lits) => {
                let to = self.alloc(cur, lits_key, ctx, Job::Lits(lits));
                self.item_ref(cur, to, direct)
            }
            Part::Array(a) => self.array(cur, a, ctx),
            Part::Object(shape, src) => {
                let to = self.alloc(cur, (src.index(), OBJECT), ctx, Job::Shape(shape, src));
                self.item_ref(cur, to, direct)
            }
            Part::Ref(r) => self.reference(cur, r, ctx, direct),
        }
    }

    /// An array: a `Vec` of its items, or a tuple when every position is fixed and nothing
    /// follows; items of several types are `serde_json::Value` (sound, wider).
    fn array(&mut self, cur: usize, a: &Array, ctx: &str) -> String {
        let p = self.p;
        let rest = Slot::of(p, a.rest);
        if a.prefix.is_empty() {
            let item = self.slot(cur, rest, &format!("{ctx} item"), false);
            return format!("Vec<{item}>");
        }
        if rest == Slot::Never && a.min >= a.prefix.len() {
            let mut elems = Vec::with_capacity(a.prefix.len());
            for &t in &a.prefix {
                elems.push(self.slot(cur, Slot::of(p, t), &format!("{ctx} item"), true));
            }
            return if elems.len() == 1 {
                format!("({},)", elems.join(""))
            } else {
                format!("({})", elems.join(", "))
            };
        }
        let mut slots: Vec<Slot> = Vec::new();
        for s in a.prefix.iter().map(|&t| Slot::of(p, t)).chain([rest]) {
            if s != Slot::Never && !slots.contains(&s) {
                slots.push(s);
            }
        }
        let item = match slots.as_slice() {
            [] => self.never(),
            [one] => self.slot(cur, *one, &format!("{ctx} item"), false),
            _ => String::from("serde_json::Value"),
        };
        format!("Vec<{item}>")
    }
}
