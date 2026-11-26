//! What is on disk at each path `generate` asks about, for `flowapi_sdk::plan`, which holds the
//! rules (what is written, kept, deleted or a conflict): a file's SHA-256 (ours, `sha256_hex`) and
//! whether it is executable, a path through a symbolic link refused. The npm package reads the
//! disk as this does (`npm/src/generate-disk.ts`).

use flowapi_sdk::{Disk, sha256_hex};
use std::path::{Path, PathBuf};

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

/// What is at `path` inside `dir`; the error refuses everything.
pub(crate) fn disk(dir: &Path, path: &str) -> Result<Disk, String> {
    let at = inside(dir, path)?;
    match std::fs::symlink_metadata(&at) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Disk::Absent),
        Err(e) => Err(format!("flowapi: cannot read {}: {e}", at.display())),
        Ok(m) if !m.is_file() => Ok(Disk::Other),
        Ok(_) => std::fs::read(&at)
            .map(|bytes| Disk::File {
                sha256: sha256_hex(&bytes),
                executable: executable(&at),
            })
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
