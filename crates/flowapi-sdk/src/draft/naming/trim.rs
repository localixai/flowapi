//! A method's name without the words its resource already says: `client.actions` needs no
//! `actions` in `getActionsCacheRetentionLimit`, nor `client.repos` a `repos` in `reposListForOrg`.
//! A word is the resource's when the resource has it, or (for a word that is no connector such as
//! `for` or `the`) when the two are one noun but for a plural (`customer` in `customers`, `policy`
//! in `policies`). The resource's words are taken out by runs of them:
//!
//! - a run at the front or in the middle goes (`actions get actions cache` is `get cache`);
//! - a run at the end goes only when it is the verb's object (`list flow` in `flows` is `list`)
//!   or a connector puts it there (`list followers for user` in `users` is `list followers`); a
//!   run a word of its own qualifies stays (`list followers for authenticated user` keeps `user`,
//!   whose words `authenticated user` say who, not what);
//! - a run of connectors alone stays (`for` in `workers_for_platforms` is never a resource's),
//!   and so does a run beside `and` or `or`, half of a pair (`list captions or subtitles`).
//!
//! What is left may be nothing: the caller then keeps a longer name.

#[cfg(test)]
#[path = "trim_tests.rs"]
mod tests;

/// Words that join a name's parts and name nothing: a resource's run after one of them is where
/// the name says whose it is, not what it does.
const CONNECTORS: &[&str] = &[
    "a", "an", "the", "for", "of", "in", "to", "by", "from", "with", "on", "at", "into",
];

fn connector(w: &str) -> bool {
    CONNECTORS.contains(&w)
}

/// A noun without its plural: `policies` is `policy`, `pets` is `pet`, `address` stays.
fn singular(w: &str) -> String {
    if let Some(stem) = w.strip_suffix("ies")
        && !stem.is_empty()
    {
        return format!("{stem}y");
    }
    match w.strip_suffix('s') {
        Some(stem) if !stem.is_empty() && !stem.ends_with('s') => stem.to_owned(),
        Some(_) | None => w.to_owned(),
    }
}

/// Whether `w` is a word of the resource `group`.
fn of_group(w: &str, group: &[String]) -> bool {
    if group.iter().any(|g| g == w) {
        return true;
    }
    !connector(w) && group.iter().any(|g| singular(g) == singular(w))
}

/// `name` without the words of `group` it repeats, by the rules above.
pub(crate) fn trim(name: &[String], group: &[String]) -> Vec<String> {
    let mine: Vec<bool> = name.iter().map(|w| of_group(w, group)).collect();
    let mut drop = vec![false; name.len()];
    // The first word that is not the resource's: the verb a run at the end may follow.
    let verb = mine.iter().position(|m| !m);
    let mut i = 0;
    while i < name.len() {
        if !mine.get(i).copied().unwrap_or(false) {
            i += 1;
            continue;
        }
        let start = i;
        while mine.get(i).copied().unwrap_or(false) {
            i += 1;
        }
        let run = name.get(start..i).unwrap_or(&[]);
        let paired = [start.checked_sub(1), Some(i)]
            .into_iter()
            .flatten()
            .filter_map(|j| name.get(j))
            .any(|w| w == "and" || w == "or");
        if paired || run.iter().all(|w| connector(w)) {
            continue;
        }
        let before = start.checked_sub(1);
        let at_end = i == name.len();
        let joined = before
            .and_then(|b| name.get(b))
            .is_some_and(|w| connector(w));
        if !at_end || before.is_none() || before == verb {
            drop_range(&mut drop, start, i);
        } else if joined {
            drop_range(&mut drop, start - 1, i);
        }
    }
    name.iter()
        .zip(drop)
        .filter(|(_, d)| !d)
        .map(|(w, _)| w.clone())
        .collect()
}

fn drop_range(drop: &mut [bool], from: usize, to: usize) {
    for d in drop.iter_mut().take(to).skip(from) {
        *d = true;
    }
}
