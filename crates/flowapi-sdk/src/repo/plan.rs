//! What `generate` changes on disk, decided before anything is written, by the previous lock
//! (pitfalls 10 to 12 of the study): a generated file is written where it is absent, left where it
//! already holds the new bytes, and overwritten only while it holds what the lock says was
//! written; one changed since is a conflict, unless it is under `src/lib/` or `examples/`, where it
//! is the user's and is kept. A file the lock does not list is never touched: one in the way is a
//! conflict. A file the lock lists and the output no longer has is deleted while unchanged, kept
//! with a warning when changed. Every path is relative, inside the folder, or nothing is planned.
//!
//! No I/O (G5): the host says what is at each path ([`Disk`], the SHA-256 of a file's bytes
//! rather than the bytes), the plan says what to change and what to print, and the host makes the
//! changes (`flowapi-cli`'s `generate/apply.rs`, the npm package's `src/generate-disk.ts`). One
//! copy of the rules for both hosts, so the command and `npx` cannot decide differently.

use super::lock::is_relative;
use super::{LOCK_PATH, Lock, Owner, RepoFile, is_own};
use crate::sha256::sha256_hex;
use std::fmt::Write as _;

/// What the host found at a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disk {
    /// Nothing.
    Absent,
    /// A file (never a symbolic link: the host refuses those): the SHA-256 (hex) of its bytes,
    /// and whether it is run as a program.
    File {
        /// The SHA-256 (hex) of its bytes.
        sha256: String,
        /// Whether it is run as a program.
        executable: bool,
    },
    /// A directory, or anything else that is not a file.
    Other,
}

/// A file of the output as the plan needs it: no text, only where it goes and whose it is.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    /// From the repository's root, with `/`.
    pub path: &'a str,
    /// Who it belongs to.
    pub owner: Owner,
    /// Whether it is run as a program.
    pub executable: bool,
}

impl RepoFile {
    /// The file as the plan needs it.
    #[must_use]
    pub fn entry(&self) -> Entry<'_> {
        Entry {
            path: &self.path,
            owner: self.owner,
            executable: self.executable,
        }
    }
}

/// A change to make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// A file written (anew or over one): the output's text at `path`, or the plan's lock at
    /// [`LOCK_PATH`]; with whether it is run as a program.
    Write {
        /// From the repository's root.
        path: String,
        /// Whether it is run as a program.
        executable: bool,
    },
    /// A file made executable, its bytes as they are.
    Mode {
        /// From the repository's root.
        path: String,
    },
    /// A file no longer generated, deleted.
    Delete {
        /// From the repository's root.
        path: String,
    },
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write { path, .. } => write!(f, "write {path}"),
            Self::Mode { path } => write!(f, "make {path} executable"),
            Self::Delete { path } => write!(f, "delete {path}"),
        }
    }
}

/// Every change, in the order they are made (the lock last), and what stops or warns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// What to change, in order.
    pub changes: Vec<Change>,
    /// What stops it: nothing is written while there is one.
    pub conflicts: Vec<String>,
    /// What is kept that the output would change.
    pub warnings: Vec<String>,
    /// Files that hold their new bytes already.
    pub unchanged: usize,
    /// The lock written: the output's, less the user's files that were changed.
    pub lock: String,
}

/// What the host does after printing a [`Report`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// Make the changes, then print [`Plan::summary`]; exit 0, or 1 when a write fails.
    Apply,
    /// Exit 0: `--check` found nothing to change.
    Done,
    /// Exit 1: a conflict, or `--check` found a change.
    Failed,
}

/// What the command prints on standard error before it writes, and what it does next.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Lines, each ending in a newline.
    pub text: String,
    /// What the host does then.
    pub next: Next,
}

impl Plan {
    /// How many files are written.
    #[must_use]
    pub fn written(&self) -> usize {
        let n = |c: &&Change| matches!(c, Change::Write { .. });
        self.changes.iter().filter(n).count()
    }

    /// How many files are deleted.
    #[must_use]
    pub fn deleted(&self) -> usize {
        let n = |c: &&Change| matches!(c, Change::Delete { .. });
        self.changes.iter().filter(n).count()
    }

    /// The warnings, then the conflicts (which stop it) or, with `check`, the changes it would
    /// make; `out` is the folder as the command was given it.
    #[must_use]
    pub fn report(&self, out: &str, check: bool) -> Report {
        let mut text = String::new();
        for warning in &self.warnings {
            let _ = writeln!(text, "warning: {warning}");
        }
        let plural = |n: usize| if n == 1 { "" } else { "s" };
        if !self.conflicts.is_empty() {
            for c in &self.conflicts {
                let _ = writeln!(text, "conflict: {c}");
            }
            let n = self.conflicts.len();
            let _ = writeln!(
                text,
                "flowapi: {n} conflict{}: nothing was written. A generated file is changed in flowapi-sdk.yml, or in src/lib/ and examples/, never by hand: restore it (or delete it) and generate again",
                plural(n)
            );
            return Report {
                text,
                next: Next::Failed,
            };
        }
        if !check {
            return Report {
                text,
                next: Next::Apply,
            };
        }
        if self.changes.is_empty() {
            let _ = writeln!(text, "flowapi: {out} is as generated");
            return Report {
                text,
                next: Next::Done,
            };
        }
        for change in &self.changes {
            let _ = writeln!(text, "{change}");
        }
        let n = self.changes.len();
        let _ = writeln!(
            text,
            "flowapi: {out} differs from what generate writes in {n} file{}",
            plural(n)
        );
        Report {
            text,
            next: Next::Failed,
        }
    }

    /// The line printed once the changes are made, with no newline.
    #[must_use]
    pub fn summary(&self, out: &str) -> String {
        format!(
            "flowapi: {out}: {} written, {} deleted, {} unchanged",
            self.written(),
            self.deleted(),
            self.unchanged
        )
    }
}

/// Every path [`plan`] asks the host about, in the order it asks, so a host that cannot answer
/// a question when it is asked (the WebAssembly module's) reads them all first. A path that is
/// not inside the folder is left out: the plan refuses it before asking.
#[must_use]
pub fn paths_to_read(files: &[Entry<'_>], lock: &Lock, previous: Option<&Lock>) -> Vec<String> {
    let orphans = previous
        .into_iter()
        .flat_map(|p| p.files.keys())
        .filter(|path| !lock.files.contains_key(*path))
        .map(String::as_str);
    files
        .iter()
        .map(|f| f.path)
        .chain(orphans)
        .chain([LOCK_PATH])
        .filter(|path| is_relative(path))
        .map(str::to_owned)
        .collect()
}

/// The plan for the output's `files` and `lock`, printed after `previous`, against what `disk`
/// finds at each path inside `out` (the folder as the command was given it, named in a refusal).
/// The error refuses everything: a path outside `out`, or one `disk` cannot read or refuses (a
/// symbolic link on the way).
///
/// # Errors
///
/// The message to print, as `disk` gives it or for a path that leaves `out`.
pub fn plan(
    out: &str,
    files: &[Entry<'_>],
    lock: &Lock,
    previous: Option<&Lock>,
    disk: &mut dyn FnMut(&str) -> Result<Disk, String>,
) -> Result<Plan, String> {
    let mut ask = |path: &str| {
        if is_relative(path) {
            disk(path)
        } else {
            Err(format!(
                "flowapi: `{path}` is not a path inside {out}: refused, nothing was written"
            ))
        }
    };
    let mut p = Plan::default();
    let mut kept = lock.clone();
    let empty = Lock::default();
    let before = previous.unwrap_or(&empty);
    for f in files {
        let write = Change::Write {
            path: f.path.to_owned(),
            executable: f.executable,
        };
        let new = lock.files.get(f.path);
        match (ask(f.path)?, f.owner) {
            (Disk::Absent, _) => p.changes.push(write),
            (Disk::Other, _) => p.conflicts.push(format!("{} is not a file", f.path)),
            (Disk::File { .. }, Owner::Seed) => {}
            (Disk::File { sha256, executable }, Owner::Generated) if new == Some(&sha256) => {
                p.unchanged += 1;
                if f.executable && !executable {
                    p.changes.push(Change::Mode {
                        path: f.path.to_owned(),
                    });
                }
            }
            (Disk::File { sha256, .. }, Owner::Generated) => match before.files.get(f.path) {
                Some(h) if *h == sha256 => p.changes.push(write),
                Some(_) if is_own(f.path) => {
                    p.warnings
                        .push(format!("{} was changed: it is yours, and kept", f.path));
                    kept.files.remove(f.path);
                }
                Some(_) => p.conflicts.push(format!(
                    "{} was changed since it was generated (its bytes are not those the lock records)",
                    f.path
                )),
                None if is_own(f.path) => {
                    kept.files.remove(f.path);
                }
                None => p.conflicts.push(format!(
                    "{} is in the way: it was not written by generate (the lock does not list it)",
                    f.path
                )),
            },
        }
    }
    for (path, hash) in &before.files {
        if lock.files.contains_key(path) {
            continue;
        }
        match ask(path)? {
            Disk::Absent => {}
            Disk::File { sha256, .. } if sha256 == *hash => {
                p.changes.push(Change::Delete { path: path.clone() });
            }
            Disk::File { .. } | Disk::Other => p.warnings.push(format!(
                "{path} is no longer generated, but was changed: kept, and no longer in the lock"
            )),
        }
    }
    p.lock = kept.json();
    match ask(LOCK_PATH)? {
        Disk::File { sha256, .. } if sha256 == sha256_hex(p.lock.as_bytes()) => {}
        Disk::Other => p.conflicts.push(format!("{LOCK_PATH} is not a file")),
        Disk::Absent | Disk::File { .. } => p.changes.push(Change::Write {
            path: LOCK_PATH.to_owned(),
            executable: false,
        }),
    }
    Ok(p)
}
