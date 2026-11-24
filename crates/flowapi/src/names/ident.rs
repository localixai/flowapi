//! Names made identifiers, in phases: each name in Pascal case; where two meet, each written as
//! it is with only what is not allowed replaced; where they still meet, the later ones numbered
//! in the output's order, which a warning names.

use crate::program::IdentifierRules;
use std::collections::HashMap;

/// An identifier chosen.
pub(super) struct Chosen {
    pub(super) name: String,
    pub(super) numbered: bool,
}

pub(super) fn choose(bases: &[String], rules: IdentifierRules) -> Vec<Chosen> {
    let first: Vec<String> = bases.iter().map(|b| pascal(b)).collect();
    let clash = clashes(&first, rules);
    let second: Vec<String> = first
        .iter()
        .zip(bases)
        .zip(&clash)
        .map(|((p, b), &c)| if c { kept(b) } else { p.clone() })
        .collect();
    let mut taken: HashMap<String, ()> = HashMap::with_capacity(second.len());
    let mut out = Vec::with_capacity(second.len());
    for name in second {
        let mut candidate = name.clone();
        let mut n = 1;
        while taken.contains_key(&candidate) || rules.reserved.contains(&candidate.as_str()) {
            n += 1;
            candidate = format!("{name}{n}");
        }
        taken.insert(candidate.clone(), ());
        out.push(Chosen {
            numbered: candidate != name,
            name: candidate,
        });
    }
    out
}

fn clashes(names: &[String], rules: IdentifierRules) -> Vec<bool> {
    let mut count: HashMap<&str, usize> = HashMap::with_capacity(names.len());
    for n in names {
        *count.entry(n.as_str()).or_insert(0) += 1;
    }
    names
        .iter()
        .map(|n| {
            count.get(n.as_str()).copied().unwrap_or(0) > 1 || rules.reserved.contains(&n.as_str())
        })
        .collect()
}

/// `pet owner`, `pet-owner`, `pet_owner` and `petOwner` are `PetOwner`; a name that starts with
/// a digit is given a `T` before it.
fn pascal(base: &str) -> String {
    let mut out = String::with_capacity(base.len());
    for word in base
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            out.push(c.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    start(out)
}

/// The name as written, with each character an identifier cannot hold made `_`.
fn kept(base: &str) -> String {
    start(
        base.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect(),
    )
}

fn start(name: String) -> String {
    match name.chars().next() {
        None => String::from("Type"),
        Some(c) if c.is_ascii_digit() => format!("T{name}"),
        Some(_) => name,
    }
}
