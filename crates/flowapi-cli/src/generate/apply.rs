//! A plan made true, all or nothing as far as a file system allows: every new file is first
//! written beside its target under a temporary name (`.<name>.flowapi-new`), and any failure
//! there removes them and the folders made for them, leaving the repository as it was. Only then
//! are they renamed over their targets, which does not fail on a folder just written to, and the
//! orphans deleted (with the folders they leave empty); the lock is renamed last, so a run cut
//! short leaves files the next run finds already holding their new bytes.

use super::plan::{Change, Plan, inside};
use std::path::{Path, PathBuf};

/// A file written under its temporary name.
struct Staged {
    temp: PathBuf,
    target: PathBuf,
}

pub(crate) fn apply(dir: &Path, plan: &Plan) -> Result<(), String> {
    let mut made: Vec<PathBuf> = Vec::new();
    let mut staged: Vec<Staged> = Vec::new();
    if let Err(message) = stage(dir, plan, &mut made, &mut staged) {
        for s in &staged {
            drop(std::fs::remove_file(&s.temp));
        }
        for folder in made.iter().rev() {
            drop(std::fs::remove_dir(folder));
        }
        return Err(format!("{message}; nothing was written"));
    }
    for s in &staged {
        std::fs::rename(&s.temp, &s.target)
            .map_err(|e| format!("flowapi: cannot write {}: {e}", s.target.display()))?;
    }
    for change in &plan.changes {
        match change {
            Change::Delete { path } => {
                let at = inside(dir, path)?;
                std::fs::remove_file(&at)
                    .map_err(|e| format!("flowapi: cannot delete {}: {e}", at.display()))?;
                prune(dir, &at);
            }
            Change::Mode { path } => set_executable(&inside(dir, path)?)?,
            Change::Write { .. } => {}
        }
    }
    Ok(())
}

/// Every write staged: its folders made (each recorded), its bytes under the temporary name.
fn stage(
    dir: &Path,
    plan: &Plan,
    made: &mut Vec<PathBuf>,
    staged: &mut Vec<Staged>,
) -> Result<(), String> {
    for change in &plan.changes {
        let Change::Write {
            path,
            bytes,
            executable,
        } = change
        else {
            continue;
        };
        let target = inside(dir, path)?;
        let folder = target.parent().unwrap_or(dir).to_path_buf();
        make_folders(&folder, made)?;
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let temp = folder.join(format!(".{name}.flowapi-new"));
        std::fs::write(&temp, bytes)
            .map_err(|e| format!("flowapi: cannot write {}: {e}", temp.display()))?;
        staged.push(Staged {
            temp: temp.clone(),
            target,
        });
        if *executable {
            set_executable(&temp)?;
        }
    }
    // The lock is the last renamed: it is the last write planned.
    Ok(())
}

/// A folder and those above it that are missing, made; each one made is recorded.
fn make_folders(folder: &Path, made: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut missing = Vec::new();
    let mut at = Some(folder);
    while let Some(f) = at.filter(|f| !f.as_os_str().is_empty() && !f.exists()) {
        missing.push(f.to_path_buf());
        at = f.parent();
    }
    for f in missing.into_iter().rev() {
        std::fs::create_dir(&f)
            .map_err(|e| format!("flowapi: cannot make {}: {e}", f.display()))?;
        made.push(f);
    }
    Ok(())
}

/// The folders a deleted file leaves empty, removed up to `dir`.
fn prune(dir: &Path, file: &Path) {
    let mut at = file.parent();
    while let Some(folder) = at.filter(|f| *f != dir && f.starts_with(dir)) {
        if std::fs::remove_dir(folder).is_err() {
            break;
        }
        at = folder.parent();
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = std::fs::metadata(path)
        .map_err(|e| format!("flowapi: cannot read {}: {e}", path.display()))?
        .permissions()
        .mode();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode | 0o755))
        .map_err(|e| format!("flowapi: cannot make {} executable: {e}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_: &Path) -> Result<(), String> {
    Ok(())
}
