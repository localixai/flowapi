//! The commands about an SDK's config, `flowapi-sdk.yml`: `summary`, `config draft`, `config
//! schema` and `config check` (`generate` is `generate.rs`). Each reads its files here and hands
//! their text to `flowapi-sdk`, which does no I/O; an issue's offset into the config's JSON is
//! printed as the YAML's line.

use crate::args::Sdk;
use crate::load;
use crate::source::Source;
use flowapi::Documents;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Runs one command; what it prints and how it ends.
pub(crate) fn run(command: Sdk) -> ExitCode {
    let done = match command {
        Sdk::Summary { input, filter } => with_docs(&input, |docs| {
            flowapi_sdk::summary(docs, &filter).ok_or_else(|| not_a_description(&input))
        })
        .and_then(|r| r)
        .map(|text| (text, None)),
        Sdk::Draft { input, output } => with_docs(&input, |docs| {
            let spec = output
                .as_deref()
                .map_or_else(|| input.clone(), |o| relative(&input, o));
            flowapi_sdk::draft(docs, &spec).ok_or_else(|| not_a_description(&input))
        })
        .and_then(|r| r)
        .map(|text| (text, output)),
        Sdk::Schema => Ok((flowapi_sdk::config_schema(), None)),
        Sdk::Check { config, spec, json } => return check(&config, spec.as_deref(), json),
        Sdk::Repo { config, out, check } => return crate::generate::run(&config, &out, check),
    };
    match done.and_then(|(text, output)| write(&text, output.as_deref())) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => fail(&message),
    }
}

#[expect(clippy::print_stderr, reason = "the command's errors")]
pub(crate) fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

fn not_a_description(input: &str) -> String {
    format!("flowapi: {input} is not an OpenAPI or Swagger description")
}

/// The text written to a file, or printed.
#[expect(clippy::print_stdout, reason = "the command's output")]
fn write(text: &str, output: Option<&str>) -> Result<(), String> {
    match output {
        None | Some("-") => {
            print!("{text}");
            Ok(())
        }
        Some(path) => {
            std::fs::write(path, text).map_err(|e| format!("flowapi: cannot write {path}: {e}"))
        }
    }
}

/// The description's path as the config written to `output` names it: relative to the
/// config's folder, as `config check` reads it; absolute when they share only the root; as given
/// when either cannot be found.
fn relative(input: &str, output: &str) -> String {
    let folder = Path::new(output)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let (Ok(spec), Ok(folder)) = (std::fs::canonicalize(input), std::fs::canonicalize(folder))
    else {
        return input.to_owned();
    };
    let a: Vec<_> = spec.components().collect();
    let b: Vec<_> = folder.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    if common <= 1 {
        // Nothing shared but the root: the absolute path reads better than a climb to it.
        return spec.to_string_lossy().into_owned();
    }
    let mut out = PathBuf::new();
    for _ in common..b.len() {
        out.push("..");
    }
    for part in a.iter().skip(common) {
        out.push(part);
    }
    out.to_string_lossy().into_owned()
}

/// The documents of a description, the files its references reach read too, given to `f`.
pub(crate) fn with_docs<R>(input: &str, f: impl FnOnce(&Documents<'_>) -> R) -> Result<R, String> {
    let texts = load::all(input)?;
    let mut docs = Documents::new();
    for source in &texts {
        if let Err(d) = docs.add(source.uri.clone(), source.json()) {
            return Err(format!("{d}\n  at {}", source.describe(d.at.offset)));
        }
    }
    Ok(f(&docs))
}

/// `config check`: the config read (YAML or JSON), its description found (`--spec`, else its
/// `spec` relative to the config), every issue printed; 1 when one is an error.
#[expect(clippy::print_stdout, reason = "the command's output")]
fn check(config: &str, spec: Option<&str>, json: bool) -> ExitCode {
    let source = match Source::read(Path::new(config)) {
        Ok(s) => s,
        Err(message) => return fail(&message),
    };
    let spec = match spec
        .map(str::to_owned)
        .or_else(|| flowapi_sdk::spec_of(source.json()))
    {
        Some(s) if s.contains("://") => {
            return fail(&format!(
                "flowapi: the config's spec is a URL ({s}), which this command does not fetch: name a file with --spec"
            ));
        }
        Some(s) if spec.is_some() => s,
        Some(s) => Path::new(config)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(s)
            .to_string_lossy()
            .into_owned(),
        None => String::new(),
    };
    let issues = if spec.is_empty() {
        flowapi_sdk::check(&Documents::new(), source.json())
    } else {
        match with_docs(&spec, |docs| flowapi_sdk::check(docs, source.json())) {
            Ok(issues) => issues,
            Err(message) => return fail(&message),
        }
    };
    let place = |offset: u32| source.place(offset);
    if json {
        print!("{}", flowapi_sdk::issues_json(&issues, &place));
    } else if issues.is_empty() {
        println!("{config}: no issues");
    } else {
        print!("{}", flowapi_sdk::issues_text(&issues, config, &place));
    }
    if issues
        .iter()
        .any(|i| i.severity == flowapi::Severity::Error)
    {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
