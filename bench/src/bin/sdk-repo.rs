//! A whole SDK repository built from a description, as a user builds one, on every example of
//! `../examples/` and the corpus descriptions of `sdk_repo::DRAFTED`: `config check`, `generate` as
//! a fresh process `--rounds` times (5 by default, `--quick` once), `npm install` once, and the
//! agent plugin's judge on steps 0-6. Writes `results/sdk-repo.json` and `results/sdk-repo.md`.
//!
//!     cargo run --release --bin sdk-repo -- [--quick | --rounds N] [--timeout SECONDS] [--only NAME]
//!
//! `--only NAME` measures one repository and prints its table, writing nothing to `results/`: a
//! look at one, never a result. Each repository is set up under `data/sdk-repo/<name>/` (ignored
//! by git). The runs are sequential; nothing else should run meanwhile. The judge's step 5 and
//! `npm install` reach the network.

use flowapi_bench::sdk_repo::{self, Case};
use flowapi_bench::{bench, machine, process, results, sdk_report};
use serde_json::{Value, json};
use std::process::ExitCode;
use std::time::Duration;

struct Options {
    rounds: usize,
    limit: Duration,
    only: Option<String>,
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        rounds: 5,
        limit: Duration::from_secs(600),
        only: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut number = |what: &str| -> Result<u64, String> {
            let value = args
                .next()
                .ok_or_else(|| format!("{what} needs a number"))?;
            value
                .parse()
                .map_err(|err| format!("{what} {value}: {err}"))
        };
        match arg.as_str() {
            "--quick" => options.rounds = 1,
            "--rounds" => options.rounds = usize::try_from(number("--rounds")?.max(1)).unwrap_or(1),
            "--timeout" => options.limit = Duration::from_secs(number("--timeout")?),
            "--only" => options.only = Some(args.next().ok_or("--only needs a name")?),
            other => {
                return Err(format!(
                    "unknown argument {other}; see the comment atop sdk-repo.rs"
                ));
            }
        }
    }
    Ok(options)
}

fn versions() -> Value {
    let root = bench().join("..");
    let bin = sdk_repo::flowapi();
    let commit = process::output("git", &["rev-parse", "--short", "HEAD"], &root);
    let dirty =
        process::output("git", &["status", "--porcelain"], &root).is_some_and(|s| !s.is_empty());
    json!({
        "flowapi": process::output(bin.to_str().unwrap_or(""), &["--version"], &root),
        "flowapiCommit": commit.unwrap_or_else(|| "no commit yet".to_owned()),
        "flowapiUncommitted": dirty,
        "rustc": process::output("rustc", &["-V"], &root),
        "node": process::output("node", &["--version"], &root),
        "npm": process::output("npm", &["--version"], &root),
    })
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(err) => return fail(&err),
    };
    if !sdk_repo::flowapi().exists() {
        return fail(
            "no ../target/release/flowapi: run `cargo build --release -p flowapi-cli` first",
        );
    }
    let cases = match sdk_repo::cases() {
        Ok(cases) => cases,
        Err(err) => return fail(&err),
    };
    let only = options.only.as_deref();
    let cases: Vec<Case> = cases
        .into_iter()
        .filter(|c| only.is_none_or(|name| c.name == name))
        .collect();
    if cases.is_empty() {
        return fail("no such repository: an example's folder name, or one of sdk_repo::DRAFTED");
    }
    let mut rows = vec![];
    for case in &cases {
        eprintln!("sdk-repo: {}", case.name);
        let work = bench().join("data/sdk-repo").join(&case.name);
        rows.push(sdk_repo::measure(
            case,
            &work,
            options.rounds,
            options.limit,
        ));
    }
    let all = json!({
        "date": process::output("date", &["-u", "+%Y-%m-%d"], &bench()),
        "rounds": options.rounds,
        "timeoutS": options.limit.as_secs(),
        "machine": machine::describe(),
        "versions": versions(),
        "rows": rows,
    });
    if only.is_some() {
        print!("{}", sdk_report::sdk_repo(&all));
        return ExitCode::SUCCESS;
    }
    let text = serde_json::to_string_pretty(&all).unwrap_or_default();
    let written = std::fs::create_dir_all(results(""))
        .and_then(|()| std::fs::write(results("sdk-repo.json"), format!("{text}\n")))
        .and_then(|()| std::fs::write(results("sdk-repo.md"), sdk_report::sdk_repo(&all)));
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(&format!("results/: {err}")),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("sdk-repo: {message}");
    ExitCode::FAILURE
}
