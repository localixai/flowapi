//! Section ids: the readable path a heading contributes to its requirements' ids.
//!
//! The nearest base at or above a heading (an OpenAPI Object, a JSON Schema keyword section, a
//! section with an explicit anchor) starts the path; without one, the nearest heading whose slug
//! is unique in the text does. The path is that start and the heading itself
//! (`parameter-object.style-examples`); when two headings would get the same short path, both
//! take every heading between (`parameter-object.fixed-fields.common-fixed-fields`); a path still
//! taken twice gets `-2`, `-3`, ... in document order.

use crate::doc::Heading;
use std::collections::BTreeMap;

/// The section id of every heading, in the same order.
pub fn section_ids(headings: &[Heading]) -> Vec<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for heading in headings {
        *counts.entry(heading.component.as_str()).or_default() += 1;
    }
    let unique = |index: usize| {
        headings
            .get(index)
            .is_some_and(|h| counts.get(h.component.as_str()) == Some(&1))
    };
    let chains: Vec<Vec<usize>> = (0..headings.len())
        .map(|index| chain(headings, index))
        .collect();
    let starts: Vec<usize> = chains
        .iter()
        .map(|chain| {
            let base = chain
                .iter()
                .position(|i| headings.get(*i).is_some_and(|h| h.base));
            let fallback = chain.iter().position(|i| unique(*i));
            base.or(fallback).unwrap_or(chain.len().saturating_sub(1))
        })
        .collect();
    let component = |index: &usize| {
        headings
            .get(*index)
            .map(|h| h.component.clone())
            .unwrap_or_default()
    };
    let short: Vec<String> = chains
        .iter()
        .zip(&starts)
        .map(|(chain, start)| match (chain.first(), chain.get(*start)) {
            (Some(own), Some(base)) if own == base => component(own),
            (Some(own), Some(base)) => format!("{}.{}", component(base), component(own)),
            _ => String::new(),
        })
        .collect();
    let mut taken: BTreeMap<&str, usize> = BTreeMap::new();
    for id in &short {
        *taken.entry(id.as_str()).or_default() += 1;
    }
    let full: Vec<String> = chains
        .iter()
        .zip(&starts)
        .zip(&short)
        .map(|((chain, start), id)| match taken.get(id.as_str()) {
            Some(1) => id.clone(),
            _ => {
                let path = chain.get(..=*start).unwrap_or_default();
                path.iter()
                    .rev()
                    .map(component)
                    .collect::<Vec<_>>()
                    .join(".")
            }
        })
        .collect();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    full.into_iter()
        .map(|id| {
            let count = seen.entry(id.clone()).or_default();
            *count += 1;
            if *count == 1 {
                id
            } else {
                format!("{id}-{count}")
            }
        })
        .collect()
}

/// The heading and its ancestors, innermost first.
fn chain(headings: &[Heading], index: usize) -> Vec<usize> {
    let mut found = vec![index];
    let mut at = headings.get(index).and_then(|h| h.parent);
    while let Some(parent) = at {
        found.push(parent);
        at = headings.get(parent).and_then(|h| h.parent);
    }
    found
}

/// Joins id parts with `.`, leaving out empty ones.
pub fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(".")
}
