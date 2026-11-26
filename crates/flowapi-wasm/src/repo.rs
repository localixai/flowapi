//! `generate` for a host that has only the module (the npm package's `flowapi generate`), in two
//! commands of `flowapi_sdk`, around the host's reads of the disk: `repo` prints the repository
//! as the command's `generate.rs` does (`flowapi_sdk::repo_files`, every message it prints on the
//! way), and names the paths the plan asks about; the host reads them and sends what it found to
//! `plan` (`flowapi_sdk::plan`), which says what to write, keep and delete, and what to print.
//!
//! ```json
//! { "command": "repo", "config": { "uri": "flowapi-sdk.yml", "text": "…" },
//!   "documents": [{ "uri": "spec.json", "text": "…" }],
//!   "lock": { "path": "sdk/.flowapi/lock.json", "text": "…" } }
//! { "text": "…", "error": null, "files": [{ "path": "README.md", "text": "…",
//!   "executable": false, "owner": "generated" }], "lock": "…", "read": ["README.md", …],
//!   "diagnostics": [] }
//! { "command": "plan", "out": "sdk", "check": false, "files": [{ "path": …, "owner": …,
//!   "executable": … }], "lock": "…", "previous": "…",
//!   "disk": [{ "path": "README.md", "kind": "file", "sha256": "…", "executable": false }] }
//! { "text": "…", "next": "apply", "changes": [{ "change": "write", "path": "README.md",
//!   "executable": false }], "lock": "…", "summary": "…", "diagnostics": [] }
//! ```
//!
//! `text` is what the command prints on standard error, `error` its last line when it stops. A
//! lock's `error` is the host's message for one it could not read; a disk entry's `kind` is
//! `absent`, `file`, `other` or `error` (with its `message`).

use crate::answer::{self, Found};
use crate::request::{self, Doc};
use crate::{found, read_all, unread};
use flowapi::id::NodeId;
use flowapi::json::Tree;
use flowapi::{Documents, Severity};
use flowapi_sdk::{Change, Disk, Entry, Input, Lock, Next, Owner, Repo};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// `repo`: the config, the description's documents, and the previous lock as the host found it.
pub(crate) struct RepoRequest<'t> {
    config: Doc<'t>,
    documents: Vec<Doc<'t>>,
    lock_path: Cow<'t, str>,
    lock: Option<Result<Cow<'t, str>, Cow<'t, str>>>,
}

/// `plan`: the folder, the output's files and lock, the previous lock, what is on disk.
pub(crate) struct PlanRequest<'t> {
    out: Cow<'t, str>,
    check: bool,
    files: Vec<(Cow<'t, str>, Owner, bool)>,
    lock: Cow<'t, str>,
    previous: Option<Cow<'t, str>>,
    disk: BTreeMap<String, Result<Disk, String>>,
}

pub(crate) fn read_repo<'t>(
    tree: &Tree<'t>,
    root: NodeId,
    config: Doc<'t>,
    documents: Vec<Doc<'t>>,
) -> Result<RepoRequest<'t>, String> {
    let node = tree.get(root, "lock").ok_or("the request has no `lock`")?;
    let lock_path = request::string(tree, node, "path").ok_or("the lock has no `path`")?;
    let lock = match (
        request::string(tree, node, "text"),
        request::string(tree, node, "error"),
    ) {
        (_, Some(error)) => Some(Err(error)),
        (Some(text), None) => Some(Ok(text)),
        (None, None) => None,
    };
    Ok(RepoRequest {
        config,
        documents,
        lock_path,
        lock,
    })
}

pub(crate) fn read_plan<'t>(tree: &Tree<'t>, root: NodeId) -> Result<PlanRequest<'t>, String> {
    let get = |node: NodeId, key: &str| request::string(tree, node, key);
    let flag = |node: NodeId, key: &str| tree.get(node, key).and_then(|n| tree.as_bool(n));
    let list = |key: &str| tree.get(root, key).map_or(&[][..], |n| tree.items(n));
    let mut files = Vec::new();
    for &f in list("files") {
        let path = get(f, "path").ok_or("a file has no `path`")?;
        let owner = match get(f, "owner").as_deref() {
            Some("seed") => Owner::Seed,
            _ => Owner::Generated,
        };
        files.push((path, owner, flag(f, "executable") == Some(true)));
    }
    let mut disk = BTreeMap::new();
    for &d in list("disk") {
        let path = get(d, "path").ok_or("a disk entry has no `path`")?;
        let found = match get(d, "kind").as_deref() {
            Some("absent") => Ok(Disk::Absent),
            Some("other") => Ok(Disk::Other),
            Some("file") => Ok(Disk::File {
                sha256: get(d, "sha256")
                    .ok_or("a file has no `sha256`")?
                    .into_owned(),
                executable: flag(d, "executable") == Some(true),
            }),
            _ => Err(get(d, "message").unwrap_or_default().into_owned()),
        };
        disk.insert(path.into_owned(), found);
    }
    Ok(PlanRequest {
        out: get(root, "out").ok_or("the request has no `out`")?,
        check: flag(root, "check") == Some(true),
        files,
        lock: get(root, "lock").ok_or("the request has no `lock`")?,
        previous: get(root, "previous"),
        disk,
    })
}

/// The answer to `repo`.
pub(crate) fn repo(r: RepoRequest<'_>) -> String {
    let mut text = String::new();
    let done = generate(r, &mut text);
    let mut out = String::from("{\"text\":");
    answer::string(&mut out, &text);
    match done {
        Err(message) => {
            out.push_str(",\"error\":");
            answer::string(&mut out, &message);
            out.push_str(",\"files\":[],\"lock\":null,\"read\":[]");
        }
        Ok((repo, previous)) => {
            out.push_str(",\"error\":null,\"files\":[");
            for (i, f) in repo.files.iter().enumerate() {
                out.push_str(if i == 0 { "{\"path\":" } else { ",{\"path\":" });
                answer::string(&mut out, &f.path);
                out.push_str(",\"text\":");
                answer::string(&mut out, &f.text);
                let owner = match f.owner {
                    Owner::Generated => "generated",
                    Owner::Seed => "seed",
                };
                let _ = write!(
                    out,
                    ",\"executable\":{},\"owner\":\"{owner}\"}}",
                    f.executable
                );
            }
            out.push_str("],\"lock\":");
            answer::string(&mut out, &repo.lock.json());
            let files: Vec<Entry<'_>> = repo
                .files
                .iter()
                .map(flowapi_sdk::RepoFile::entry)
                .collect();
            let read = flowapi_sdk::paths_to_read(&files, &repo.lock, previous.as_ref());
            out.push_str(",\"read\":");
            out.push_str(&answer::strings(&read));
        }
    }
    answer::diagnostics(&mut out, &[]);
    out
}

/// `generate.rs`'s steps after its reads: the documents given to the core, the program, the
/// previous lock, the repository; every message it prints on the way written to `text`, the one
/// it stops with returned.
fn generate(r: RepoRequest<'_>, text: &mut String) -> Result<(Repo, Option<Lock>), String> {
    let line = |f: &Found<'_>| {
        let level = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let (code, message, uri) = (f.code, f.message, f.uri);
        format!(
            "{level}[{code}]: {message}\n  at {uri}:{}:{}",
            f.line, f.column
        )
    };
    let config = flowapi_yaml::Text::read(&r.config.uri, r.config.text.clone())
        .map_err(|e| line(&unread(&r.config.uri, &e)))?;
    let spec = r
        .documents
        .first()
        .map(|d| d.text.clone())
        .unwrap_or_default();
    let sources = read_all(r.documents).map_err(|(uri, e)| line(&unread(&uri, &e)))?;
    let mut docs = Documents::new();
    for source in &sources {
        docs.add(source.uri.as_ref(), source.text.json())
            .map_err(|d| line(&found(&sources, &d)))?;
    }
    let target = flowapi_ts::TypeScript::new("src/api.ts").with_sdk();
    let (program, diags) = flowapi::program(&docs, &target);
    // The core's warnings too, as the command prints them.
    for d in diags.all() {
        let _ = writeln!(text, "{}", line(&found(&sources, d)));
    }
    let uri = sources.first().map_or("", |s| s.uri.as_ref());
    let program = program.ok_or_else(|| format!("flowapi: {uri} cannot be read into types"))?;
    let previous = match r.lock {
        None => None,
        Some(Err(error)) => return Err(error.into_owned()),
        Some(Ok(lock)) => Some(Lock::read(&lock).map_err(|e| {
            format!(
                "flowapi: {} cannot be read: {e}. generate reads it to know which files it wrote; restore it from version control",
                r.lock_path
            )
        })?),
    };
    let input = Input {
        config: config.json(),
        config_bytes: r.config.text.as_bytes(),
        spec_bytes: spec.as_bytes(),
    };
    let repo = flowapi_sdk::repo_files(&docs, &program, &input, previous.as_ref());
    let place = |offset: u32| config.place(offset);
    text.push_str(&flowapi_sdk::issues_text(
        &repo.issues,
        &r.config.uri,
        &place,
    ));
    for d in repo.diagnostics.all() {
        let _ = writeln!(text, "{}", line(&found(&sources, d)));
    }
    if repo.issues.iter().any(|i| i.severity == Severity::Error) {
        return Err(format!(
            "flowapi: {} has errors (above): nothing was written",
            r.config.uri
        ));
    }
    Ok((repo, previous))
}

/// The answer to `plan`.
pub(crate) fn plan(r: &PlanRequest<'_>) -> String {
    let lock = Lock::read(&r.lock);
    let previous = r.previous.as_deref().map(Lock::read).transpose();
    let (lock, previous) = match (lock, previous) {
        (Ok(lock), Ok(previous)) => (lock, previous),
        (Err(e), _) | (_, Err(e)) => {
            let refusal = format!("the request's lock cannot be read: {e}");
            return answer::sdk(None, None, &[answer::refusal(&refusal)]);
        }
    };
    let files: Vec<Entry<'_>> = r
        .files
        .iter()
        .map(|(path, owner, executable)| Entry {
            path,
            owner: *owner,
            executable: *executable,
        })
        .collect();
    let mut ask = |path: &str| {
        r.disk
            .get(path)
            .cloned()
            .unwrap_or_else(|| Err(format!("flowapi: {path} was not read")))
    };
    let planned = flowapi_sdk::plan(&r.out, &files, &lock, previous.as_ref(), &mut ask);
    let (text, next, planned) = match planned {
        Ok(p) => {
            let report = p.report(&r.out, r.check);
            (report.text, report.next, p)
        }
        Err(message) => (
            format!("{message}\n"),
            Next::Failed,
            flowapi_sdk::Plan::default(),
        ),
    };
    let next = match next {
        Next::Apply => "apply",
        Next::Done => "done",
        Next::Failed => "failed",
    };
    let mut out = String::from("{\"text\":");
    answer::string(&mut out, &text);
    let _ = write!(out, ",\"next\":\"{next}\",\"changes\":[");
    for (i, c) in planned.changes.iter().enumerate() {
        let (change, path, executable) = match c {
            Change::Write { path, executable } => ("write", path, *executable),
            Change::Mode { path } => ("mode", path, true),
            Change::Delete { path } => ("delete", path, false),
        };
        let comma = if i == 0 { "" } else { "," };
        let _ = write!(out, "{comma}{{\"change\":\"{change}\",\"path\":");
        answer::string(&mut out, path);
        let _ = write!(out, ",\"executable\":{executable}}}");
    }
    out.push_str("],\"lock\":");
    answer::string(&mut out, &planned.lock);
    out.push_str(",\"summary\":");
    answer::string(&mut out, &planned.summary(&r.out));
    answer::diagnostics(&mut out, &[]);
    out
}
