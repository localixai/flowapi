//! One command run as a fresh process and measured as `speed` measures a generator (`process`):
//! its wall time from before it was started to when it was reaped, and its peak resident memory.
//! For the benchmarks written in Node (`competitors/sdk-compare.mjs`), which cannot read a
//! child's `wait4` themselves. Prints one line of JSON: `{"ok", "ms", "peakRssKb", "failure"}`.
//!
//!     target/release/measure <log> <timeout-seconds> -- <program> [args...]
//!     target/release/measure --machine     # the machine, as the results name it
//!
//! The command's output goes to `<log>`.

use flowapi_bench::{machine, process};
use serde_json::json;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--machine") {
        let described = machine::describe();
        let line = machine::line(&described);
        println!("{}", json!({ "machine": described, "line": line }));
        return ExitCode::SUCCESS;
    }
    let (Some(log), Some(limit), Some("--"), Some(program)) = (
        args.first(),
        args.get(1).and_then(|s| s.parse::<u64>().ok()),
        args.get(2).map(String::as_str),
        args.get(3),
    ) else {
        eprintln!("usage: measure <log> <timeout-seconds> -- <program> [args...] | --machine");
        return ExitCode::from(2);
    };
    let limit = Duration::from_secs(limit);
    let mut command = Command::new(program);
    command.args(&args[4..]);
    match process::measure(&mut command, Path::new(log), limit) {
        Ok(run) => {
            let ms = run.wall.as_secs_f64() * 1e3;
            let failure = run.failure(limit);
            let line = json!({ "ok": run.succeeded(), "ms": ms, "peakRssKb": run.peak_rss_kb, "failure": failure });
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            println!(
                "{}",
                json!({ "ok": false, "failure": format!("did not start: {err}") })
            );
            ExitCode::SUCCESS
        }
    }
}
