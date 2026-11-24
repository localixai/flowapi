//! `scope.json`: which requirements flowapi answers for, by rules on ids.
//!
//! A rule's `match` is an id or an id prefix that ends at a `.`; `*` in it stands for a version
//! (digits and dots), so `oas-*.server-object` is every OpenAPI version's Server Object. The
//! longest `match` that fits an id classifies it. A rule that fits no id is stale, and an id no
//! rule fits is unclassified: both are errors.

use crate::requirements::Requirement;
use serde_json::Value;

/// The class a rule gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// flowapi answers for it: a test marks it, or it waits for one.
    InScope,
    /// Not a types generator's concern, with a reason.
    OutOfScope,
}

/// One rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// The id or prefix it fits.
    pub pattern: String,
    /// The class it gives.
    pub class: Class,
    /// Why (required for out of scope).
    pub reason: String,
}

/// The rules of `scope.json`.
#[derive(Debug, Clone)]
pub struct Scope {
    /// `draft, owner to confirm` until the owner has read it.
    pub status: String,
    /// The rules in file order.
    pub rules: Vec<Rule>,
}

/// Reads `scope.json`.
pub fn parse(source: &str) -> Result<Scope, String> {
    let value: Value = serde_json::from_str(source).map_err(|e| format!("scope.json: {e}"))?;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let list = value
        .get("rules")
        .and_then(Value::as_array)
        .ok_or("scope.json: no `rules`")?;
    let mut rules = Vec::new();
    for entry in list {
        let pattern = entry
            .get("match")
            .and_then(Value::as_str)
            .ok_or("scope.json: a rule has no `match`")?;
        let class = match entry.get("class").and_then(Value::as_str) {
            Some("in-scope") => Class::InScope,
            Some("out-of-scope") => Class::OutOfScope,
            other => return Err(format!("scope.json: `{pattern}` has class {other:?}")),
        };
        let reason = entry
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        if class == Class::OutOfScope && reason.trim().is_empty() {
            return Err(format!(
                "scope.json: `{pattern}` is out of scope without a reason"
            ));
        }
        if rules.iter().any(|r: &Rule| r.pattern == pattern) {
            return Err(format!("scope.json: `{pattern}` is listed twice"));
        }
        rules.push(Rule {
            pattern: pattern.to_owned(),
            class,
            reason,
        });
    }
    Ok(Scope { status, rules })
}

/// Whether `pattern` fits `id`: the id, or the id's prefix up to a `.`.
pub fn fits(pattern: &str, id: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let id: Vec<char> = id.chars().collect();
    prefix_ends(&pattern, &id, 0, 0)
}

fn prefix_ends(pattern: &[char], id: &[char], p: usize, i: usize) -> bool {
    match pattern.get(p) {
        None => matches!(id.get(i), None | Some('.')),
        Some('*') => {
            let mut end = i;
            while id.get(end).is_some_and(|c| c.is_ascii_digit() || *c == '.') {
                end += 1;
                if prefix_ends(pattern, id, p + 1, end) {
                    return true;
                }
            }
            false
        }
        Some(ch) => id.get(i) == Some(ch) && prefix_ends(pattern, id, p + 1, i + 1),
    }
}

/// The rule that classifies each requirement (`None`: unclassified), and the stale rules.
pub fn classify<'a>(
    scope: &'a Scope,
    requirements: &[Requirement],
) -> (Vec<Option<&'a Rule>>, Vec<&'a Rule>) {
    let mut used = vec![false; scope.rules.len()];
    let chosen = requirements
        .iter()
        .map(|requirement| {
            let mut best: Option<(usize, &Rule)> = None;
            for (index, rule) in scope.rules.iter().enumerate() {
                if fits(&rule.pattern, &requirement.id) {
                    used.get_mut(index).into_iter().for_each(|u| *u = true);
                    if best.is_none_or(|(_, b)| rule.pattern.len() > b.pattern.len()) {
                        best = Some((index, rule));
                    }
                }
            }
            best.map(|(_, rule)| rule)
        })
        .collect();
    let stale = scope
        .rules
        .iter()
        .zip(&used)
        .filter(|(_, used)| !**used)
        .map(|(rule, _)| rule)
        .collect();
    (chosen, stale)
}
