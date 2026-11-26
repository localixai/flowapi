//! `flowapi generate <flowapi-sdk.yml> --out <dir> [--check]`: an SDK's repository written. The
//! config is read (YAML or JSON), its description found by its `spec`, relative to the config,
//! and the previous lock at `<dir>/.flowapi/lock.json`; `flowapi-sdk` prints every file and the
//! new lock without I/O; `generate/plan.rs` holds them to the files on disk (a conflict stops it)
//! and `generate/apply.rs` writes all or nothing. `--check` plans and writes nothing: it exits 1
//! when a file would change, for CI.

mod apply;
mod plan;

use crate::load;
use crate::sdk::fail;
use crate::source::Source;
use flowapi::{Documents, Severity};
use flowapi_sdk::{Input, Lock};
use std::path::Path;
use std::process::ExitCode;

/// Runs the command; what it prints and how it ends: 0 when written or up to date, 1 on any
/// refusal (an issue that is an error, a conflict, a path outside `out`, a file that cannot be
/// read or written) or, with `--check`, a difference.
#[expect(clippy::print_stderr, reason = "the command's report")]
pub(crate) fn run(config: &str, out: &str, check: bool) -> ExitCode {
    let (repo, previous) = match generate(config, out) {
        Ok(done) => done,
        Err(message) => return fail(&message),
    };
    let dir = Path::new(out);
    let planned = match plan::plan(dir, &repo, previous.as_ref()) {
        Ok(p) => p,
        Err(message) => return fail(&message),
    };
    for warning in &planned.warnings {
        eprintln!("warning: {warning}");
    }
    if !planned.conflicts.is_empty() {
        for c in &planned.conflicts {
            eprintln!("conflict: {c}");
        }
        return fail(&format!(
            "flowapi: {} conflict{}: nothing was written. A generated file is changed in flowapi-sdk.yml, or in src/lib/ and examples/, never by hand: restore it (or delete it) and generate again",
            planned.conflicts.len(),
            if planned.conflicts.len() == 1 {
                ""
            } else {
                "s"
            }
        ));
    }
    if check {
        if planned.changes.is_empty() {
            eprintln!("flowapi: {out} is as generated");
            return ExitCode::SUCCESS;
        }
        for change in &planned.changes {
            eprintln!("{change}");
        }
        return fail(&format!(
            "flowapi: {out} differs from what generate writes in {} file{}",
            planned.changes.len(),
            if planned.changes.len() == 1 { "" } else { "s" }
        ));
    }
    match apply::apply(dir, &planned) {
        Ok(()) => {
            eprintln!(
                "flowapi: {out}: {} written, {} deleted, {} unchanged",
                planned.written(),
                planned.deleted(),
                planned.unchanged
            );
            ExitCode::SUCCESS
        }
        Err(message) => fail(&message),
    }
}

/// The repository printed, with the previous lock it was printed after; an error is the message.
#[expect(
    clippy::print_stderr,
    reason = "the description's and the config's findings"
)]
fn generate(config: &str, out: &str) -> Result<(flowapi_sdk::Repo, Option<Lock>), String> {
    let source = Source::read(Path::new(config))?;
    let config_bytes =
        std::fs::read(config).map_err(|e| format!("flowapi: cannot read {config}: {e}"))?;
    let spec = match flowapi_sdk::spec_of(source.json()) {
        Some(s) if s.contains("://") => {
            return Err(format!(
                "flowapi: the config's spec is a URL ({s}), which generate does not fetch: download it and name the file"
            ));
        }
        Some(s) => Path::new(config)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(s)
            .to_string_lossy()
            .into_owned(),
        None => {
            let issues = flowapi_sdk::check(&Documents::new(), source.json());
            let place = |offset: u32| source.place(offset);
            eprint!("{}", flowapi_sdk::issues_text(&issues, config, &place));
            return Err(format!("flowapi: {config} names no description (`spec`)"));
        }
    };
    let texts = load::all(&spec)?;
    let spec_bytes =
        std::fs::read(&spec).map_err(|e| format!("flowapi: cannot read {spec}: {e}"))?;
    let mut docs = Documents::new();
    for s in &texts {
        if let Err(d) = docs.add(s.uri.clone(), s.json()) {
            return Err(format!("{d}\n  at {}", s.describe(d.at.offset)));
        }
    }
    let target = flowapi_ts::TypeScript::new("src/api.ts").with_sdk();
    let (program, diags) = flowapi::program(&docs, &target);
    let describe = |at: flowapi::Loc| {
        texts
            .get(at.doc.index())
            .map_or_else(|| docs.describe(at), |s| s.describe(at.offset))
    };
    for d in diags.all().iter().filter(|d| d.severity == Severity::Error) {
        eprintln!("{d}\n  at {}", describe(d.at));
    }
    let program = program.ok_or_else(|| format!("flowapi: {spec} cannot be read into types"))?;
    let lock_path = Path::new(out).join(flowapi_sdk::LOCK_PATH);
    let previous = match std::fs::read_to_string(&lock_path) {
        Ok(text) => Some(Lock::read(&text).map_err(|e| {
            format!(
                "flowapi: {} cannot be read: {e}. generate reads it to know which files it wrote; restore it from version control",
                lock_path.display()
            )
        })?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("flowapi: cannot read {}: {e}", lock_path.display())),
    };
    let input = Input {
        config: source.json(),
        config_bytes: &config_bytes,
        spec_bytes: &spec_bytes,
    };
    let repo = flowapi_sdk::repo_files(&docs, &program, &input, previous.as_ref());
    let place = |offset: u32| source.place(offset);
    eprint!("{}", flowapi_sdk::issues_text(&repo.issues, config, &place));
    for d in repo.diagnostics.all() {
        eprintln!("{d}\n  at {}", describe(d.at));
    }
    if repo.issues.iter().any(|i| i.severity == Severity::Error) {
        return Err(format!(
            "flowapi: {config} has errors (above): nothing was written"
        ));
    }
    Ok((repo, previous))
}
