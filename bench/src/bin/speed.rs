//! The flowapi command against the other TypeScript generators, on every description of the
//! corpus: each run as a fresh process, `--rounds` times (5 by default, `--quick` once), its median
//! wall time and peak resident memory kept, its output's size, and whether that output type-checks
//! with TypeScript's own compiler. Writes `results/speed.json` and `results/speed.md`.
//!
//!     cargo run --release --bin speed -- [--quick | --rounds N] [--timeout SECONDS] [--only NAME]
//!
//! `--only NAME` measures one description of the corpus and prints its table, writing nothing to
//! `results/`: a look at one file, never a result.
//!
//! The runs are sequential, one process at a time: nothing else should run meanwhile. Linux (and
//! macOS) only: memory is read from `wait4`.

use flowapi_bench::process::{self, Run};
use flowapi_bench::{bench, corpus, machine, report, results, tail};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

/// The generators compared, in the order of the tables.
const TOOLS: [&str; 3] = ["flowapi", "openapi-typescript", "hey-api"];

/// The flags every output is type-checked with: as strict as a project that uses it can be.
const TSC_FLAGS: [&str; 7] = [
    "--ignoreConfig",
    "--strict",
    "--exactOptionalPropertyTypes",
    "--noEmit",
    "--target",
    "es2022",
    "--skipLibCheck",
];

struct Options {
    rounds: usize,
    limit: Duration,
    only: Option<String>,
}

fn options() -> Result<Options, String> {
    let mut options = Options {
        rounds: 5,
        limit: Duration::from_secs(300),
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
                    "unknown argument {other}; see the comment atop speed.rs"
                ));
            }
        }
    }
    Ok(options)
}

fn flowapi_binary() -> PathBuf {
    bench().join("../target/release/flowapi")
}

fn tsc() -> PathBuf {
    bench().join("../conformance/ts/node_modules/.bin/tsc")
}

fn command(tool: &str, input: &Path, out: &Path) -> Command {
    if tool == "flowapi" {
        let mut command = Command::new(flowapi_binary());
        command.arg(input).args(["-q", "-o"]).arg(out);
        return command;
    }
    let mut command = Command::new("node");
    command
        .arg(bench().join("competitors/run.mjs"))
        .arg(tool)
        .arg(input)
        .arg(out);
    command
}

/// What a failed run said: the lines that name an error, stack frames left out, else its last
/// lines. Paths under the bench are given from it (`bench/data/...`), never as this machine's.
fn said(log: &str) -> String {
    let here = bench().display().to_string();
    let log = log
        .replace(&format!("file://{here}"), "bench")
        .replace(&here, "bench");
    let log = log.as_str();
    let named: Vec<&str> = (log.lines().map(str::trim))
        .filter(|line| !line.starts_with("at ") && !line.starts_with("file://"))
        .filter(|line| line.contains("rror") || line.starts_with('✘'))
        .take(4)
        .collect();
    if named.is_empty() {
        tail(log, 12, 1500)
    } else {
        tail(&named.join("\n"), 4, 1500)
    }
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    match values.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => values[mid],
        _ => f64::midpoint(values[mid - 1], values[mid]),
    }
}

/// Whether `out` type-checks: `(checks, errors counted, the first of them)`.
fn type_check(out: &Path, log: &Path, limit: Duration) -> (bool, usize, String) {
    let mut command = Command::new(tsc());
    command.args(TSC_FLAGS).arg(out);
    match process::measure(&mut command, log, limit) {
        Err(err) => (false, 0, format!("tsc did not run: {err}")),
        Ok(run) => {
            let text = std::fs::read_to_string(log).unwrap_or_default();
            let errors = text
                .lines()
                .filter(|line| line.contains("error TS"))
                .count();
            let first: Vec<&str> = text.lines().take(8).collect();
            let sample = match (run.succeeded(), first.is_empty()) {
                (true, _) => String::new(),
                (false, true) => run.failure(limit),
                (false, false) => first.join("\n"),
            };
            (run.succeeded(), errors, sample)
        }
    }
}

/// Every round of one tool on one description, then its output type-checked.
fn measure(tool: &str, input: &Path, options: &Options, checker: bool) -> Value {
    let dir = bench().join("data/out").join(tool);
    let _ = std::fs::create_dir_all(&dir);
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("out");
    let (out, log) = (
        dir.join(format!("{stem}.ts")),
        dir.join(format!("{stem}.log")),
    );
    let mut times = vec![];
    let mut peak = 0;
    for _ in 0..options.rounds {
        let _ = std::fs::remove_file(&out);
        let run: Run = match process::measure(&mut command(tool, input, &out), &log, options.limit)
        {
            Ok(run) => run,
            Err(err) => return json!({ "ok": false, "error": format!("did not start: {err}") }),
        };
        if !run.succeeded() || !out.exists() {
            let said = said(&std::fs::read_to_string(&log).unwrap_or_default());
            let what = if run.succeeded() {
                "wrote no output".to_owned()
            } else {
                run.failure(options.limit)
            };
            let ms = run.wall.as_secs_f64() * 1e3;
            let error = if said.is_empty() {
                what
            } else {
                format!("{what}: {said}")
            };
            return json!({ "ok": false, "error": error, "failedAfterMs": ms,
                           "peakRssKb": run.peak_rss_kb });
        }
        times.push(run.wall.as_secs_f64() * 1e3);
        peak = peak.max(run.peak_rss_kb);
    }
    let output_bytes = std::fs::metadata(&out).map_or(0, |meta| meta.len());
    let mut row = json!({
        "ok": true, "error": null, "roundsMs": times.clone(), "medianMs": median(&mut times),
        "peakRssKb": peak, "outputBytes": output_bytes, "typeChecks": null,
    });
    if checker {
        let (checks, errors, sample) =
            type_check(&out, &dir.join(format!("{stem}.tsc.log")), options.limit);
        row["typeChecks"] = json!(checks);
        row["typeErrors"] = json!(errors);
        row["typeErrorSample"] = json!(sample);
    }
    row
}

fn versions() -> Value {
    let root = bench().join("..");
    let bin = flowapi_binary();
    let flowapi = process::output(bin.to_str().unwrap_or(""), &["--version"], &root);
    let commit = process::output("git", &["rev-parse", "--short", "HEAD"], &root);
    let dirty =
        process::output("git", &["status", "--porcelain"], &root).is_some_and(|s| !s.is_empty());
    let competitors = process::output("node", &["competitors/run.mjs", "versions"], &bench())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or(Value::Null);
    let tsc = process::output(tsc().to_str().unwrap_or(""), &["--version"], &root);
    json!({
        "flowapi": flowapi,
        "flowapiCommit": commit.unwrap_or_else(|| "no commit yet".to_owned()),
        "flowapiUncommitted": dirty,
        "rustc": process::output("rustc", &["-V"], &root),
        "node": process::output("node", &["--version"], &root),
        "openapi-typescript": competitors["openapi-typescript"],
        "@hey-api/openapi-ts": competitors["@hey-api/openapi-ts"],
        "typescriptOfTheGenerators": competitors["typescript"],
        "tsc": tsc,
    })
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(err) => return fail(&err),
    };
    if !flowapi_binary().exists() {
        return fail("no ../target/release/flowapi: run `cargo build --release` in flowapi first");
    }
    let corpus = match corpus() {
        Ok(corpus) => corpus,
        Err(err) => return fail(&err),
    };
    let only = options.only.as_deref();
    let corpus: Vec<_> = (corpus.into_iter())
        .filter(|file| only.is_none_or(|name| file.name == name))
        .collect();
    if corpus.is_empty() {
        return fail("no such description in corpus/corpus.tsv");
    }
    if let Some(missing) = corpus.iter().find(|file| !file.path.exists()) {
        return fail(&format!(
            "no {}: run corpus/fetch.sh",
            missing.path.display()
        ));
    }
    let checker = tsc().exists();
    if !checker {
        eprintln!(
            "speed: no {}; outputs are not type-checked",
            tsc().display()
        );
    }
    let mut rows = vec![];
    for file in &corpus {
        for tool in TOOLS {
            eprintln!("speed: {} with {tool}", file.name);
            let mut row = measure(tool, &file.path, &options, checker);
            row["file"] = json!(file.name);
            row["bytes"] = json!(file.bytes);
            row["tool"] = json!(tool);
            rows.push(row);
        }
    }
    let date = process::output("date", &["-u", "+%Y-%m-%d"], &bench());
    let all = json!({
        "date": date,
        "rounds": options.rounds,
        "timeoutS": options.limit.as_secs(),
        "machine": machine::describe(),
        "versions": versions(),
        "typeCheck": { "available": checker, "command": format!("tsc {}", TSC_FLAGS.join(" ")) },
        "tools": TOOLS,
        "rows": rows,
    });
    if only.is_some() {
        print!("{}", report::speed(&all));
        return ExitCode::SUCCESS;
    }
    let text = serde_json::to_string_pretty(&all).unwrap_or_default();
    let written = std::fs::create_dir_all(results(""))
        .and_then(|()| std::fs::write(results("speed.json"), format!("{text}\n")))
        .and_then(|()| std::fs::write(results("speed.md"), report::speed(&all)));
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => fail(&format!("results/: {err}")),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("speed: {message}");
    ExitCode::FAILURE
}
