//! What `generate` would change on disk, decided before anything is written, by the previous
//! lock (pitfalls 10 to 12 of the study): a generated file is written where it is absent, left
//! where it already holds the new bytes, and overwritten only while it holds what the lock says
//! was written; one changed since is a conflict, unless it is under `src/lib/` or `examples/`,
//! where it is the user's and is kept. A file the lock does not list is never touched: one in
//! the way is a conflict. A file the lock lists and the output no longer has is deleted while
//! unchanged, kept with a warning when changed. Every path is relative, inside `<dir>`, and
//! reached through no symbolic link, or nothing is planned.

use flowapi_sdk::{LOCK_PATH, Lock, Owner, Repo, is_own, sha256_hex};
use std::path::{Path, PathBuf};

/// A change to make.
pub(crate) enum Change {
    /// A file written (anew or over one), with whether it is run as a program.
    Write {
        path: String,
        bytes: Vec<u8>,
        executable: bool,
    },
    /// A file made executable, its bytes as they are.
    Mode { path: String },
    /// A file no longer generated, deleted.
    Delete { path: String },
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
#[derive(Default)]
pub(crate) struct Plan {
    pub(crate) changes: Vec<Change>,
    pub(crate) conflicts: Vec<String>,
    pub(crate) warnings: Vec<String>,
    /// Files that hold their new bytes already.
    pub(crate) unchanged: usize,
}

impl Plan {
    pub(crate) fn written(&self) -> usize {
        let n = |c: &&Change| matches!(c, Change::Write { .. });
        self.changes.iter().filter(n).count()
    }

    pub(crate) fn deleted(&self) -> usize {
        let n = |c: &&Change| matches!(c, Change::Delete { .. });
        self.changes.iter().filter(n).count()
    }
}

/// What is on disk at a path.
enum Disk {
    Absent,
    File(Vec<u8>),
    /// A directory, or anything else that is not a file.
    Other,
}

/// The plan for `repo`, printed after `previous`, against `dir`. The error refuses everything:
/// a path that leaves `dir` or passes a symbolic link, or a file that cannot be read.
pub(crate) fn plan(dir: &Path, repo: &Repo, previous: Option<&Lock>) -> Result<Plan, String> {
    let mut p = Plan::default();
    let mut lock = repo.lock.clone();
    let empty = Lock::default();
    let before = previous.unwrap_or(&empty);
    for f in &repo.files {
        let disk = read(dir, &f.path)?;
        let new = f.text.as_bytes();
        let write = Change::Write {
            path: f.path.clone(),
            bytes: new.to_vec(),
            executable: f.executable,
        };
        match (disk, f.owner) {
            (Disk::Absent, _) => p.changes.push(write),
            (Disk::Other, _) => p.conflicts.push(format!("{} is not a file", f.path)),
            (Disk::File(_), Owner::Seed) => {}
            (Disk::File(bytes), Owner::Generated) if bytes == new => {
                p.unchanged += 1;
                if f.executable && !executable(&dir.join(&f.path)) {
                    p.changes.push(Change::Mode {
                        path: f.path.clone(),
                    });
                }
            }
            (Disk::File(bytes), Owner::Generated) => {
                let hash = sha256_hex(&bytes);
                match before.files.get(&f.path) {
                    Some(h) if *h == hash => p.changes.push(write),
                    Some(_) if is_own(&f.path) => {
                        p.warnings.push(format!("{} was changed: it is yours, and kept", f.path));
                        lock.files.remove(&f.path);
                    }
                    Some(_) => p.conflicts.push(format!(
                        "{} was changed since it was generated (its bytes are not those the lock records)",
                        f.path
                    )),
                    None if is_own(&f.path) => {
                        lock.files.remove(&f.path);
                    }
                    None => p.conflicts.push(format!(
                        "{} is in the way: it was not written by generate (the lock does not list it)",
                        f.path
                    )),
                }
            }
        }
    }
    for (path, hash) in &before.files {
        if repo.lock.files.contains_key(path) {
            continue;
        }
        match read(dir, path)? {
            Disk::Absent => {}
            Disk::File(bytes) if sha256_hex(&bytes) == *hash => {
                p.changes.push(Change::Delete { path: path.clone() })
            }
            Disk::File(_) | Disk::Other => p.warnings.push(format!(
                "{path} is no longer generated, but was changed: kept, and no longer in the lock"
            )),
        }
    }
    let text = lock.json();
    match read(dir, LOCK_PATH)? {
        Disk::File(bytes) if bytes == text.as_bytes() => {}
        Disk::Other => p.conflicts.push(format!("{LOCK_PATH} is not a file")),
        Disk::Absent | Disk::File(_) => p.changes.push(Change::Write {
            path: LOCK_PATH.to_owned(),
            bytes: text.into_bytes(),
            executable: false,
        }),
    }
    Ok(p)
}

/// A path checked to stay inside `dir`, joined to it.
pub(crate) fn inside(dir: &Path, path: &str) -> Result<PathBuf, String> {
    let fine = !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(['\\', ':', '\0'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");
    if !fine {
        return Err(format!(
            "flowapi: `{path}` is not a path inside {}: refused, nothing was written",
            dir.display()
        ));
    }
    let mut at = dir.to_path_buf();
    for part in path.split('/') {
        at.push(part);
        if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(format!(
                "flowapi: {} is a symbolic link: generate writes no file through one, nothing was written",
                at.display()
            ));
        }
    }
    Ok(at)
}

fn read(dir: &Path, path: &str) -> Result<Disk, String> {
    let at = inside(dir, path)?;
    match std::fs::symlink_metadata(&at) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Disk::Absent),
        Err(e) => Err(format!("flowapi: cannot read {}: {e}", at.display())),
        Ok(m) if !m.is_file() => Ok(Disk::Other),
        Ok(_) => std::fs::read(&at)
            .map(Disk::File)
            .map_err(|e| format!("flowapi: cannot read {}: {e}", at.display())),
    }
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(_: &Path) -> bool {
    true
}
