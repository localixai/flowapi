//! The committed results held to `budgets.json`: each rule read against the results it names
//! (`results`, `speed.json` when it names none), one line each, and a failure exit when any is
//! broken or has no results. Nothing is timed here; `run.sh` takes the times, and this reads what
//! it wrote: `speed`'s rules here, `sdk-repo`'s and the agent runs' in `check/sdk.rs`. A rule
//! marked `optional` reads results run.sh does not write: when they are missing it says so and
//! does not fail.
//!
//!     cargo run --release --bin check

#[path = "check/sdk.rs"]
mod sdk;

use flowapi_bench::report::{files, lead, row};
use flowapi_bench::{bench, results};
use serde_json::Value;
use std::process::ExitCode;

fn read(path: &std::path::Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn tools(speed: &Value) -> Vec<&str> {
    (speed["tools"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect()
}

/// The least a ratio may be on `file`: the rule's `except` for it, else its `min`.
fn floor(rule: &Value, key: &str, file: &str) -> f64 {
    rule["except"][file]
        .as_f64()
        .or_else(|| rule[key].as_f64())
        .unwrap_or(f64::NAN)
}

/// flowapi's output type-checks on every description.
fn type_checks(speed: &Value) -> (bool, String) {
    if speed["typeCheck"]["available"] != true {
        return (
            false,
            "the outputs were not type-checked (no tsc in ../conformance/ts)".to_owned(),
        );
    }
    let wrong: Vec<String> = files(speed)
        .into_iter()
        .filter(|file| {
            row(speed, file, "flowapi")
                .is_none_or(|row| row["ok"] != true || row["typeChecks"] != true)
        })
        .collect();
    let all = files(speed).len();
    (
        wrong.is_empty(),
        format!("{} of {all} type-check; not: {wrong:?}", all - wrong.len()),
    )
}

/// A ratio of the best other tool's figure to flowapi's, per description, at least the rule's.
/// Where no other tool finished, the rule holds and the line says so.
fn ratio(speed: &Value, rule: &Value, key: &str, faster: bool) -> (bool, String) {
    let tools = tools(speed);
    let mut holds = true;
    let mut found = vec![];
    for file in files(speed) {
        if row(speed, &file, "flowapi").is_none_or(|row| row["ok"] != true) {
            holds = false;
            found.push(format!("{file} flowapi failed"));
            continue;
        }
        let Some(x) = lead(speed, &file, &tools, key) else {
            found.push(format!("{file} (no other tool finished)"));
            continue;
        };
        let (x, min) = if faster {
            (x, floor(rule, "minRatio", &file))
        } else {
            (1.0 / x, floor(rule, "maxRatio", &file))
        };
        let ok = if faster { x >= min } else { x <= min };
        holds &= ok;
        let mark = if ok { "" } else { " BROKEN" };
        found.push(format!(
            "{file} {x:.2}x ({} {min}){mark}",
            if faster { ">=" } else { "<=" }
        ));
    }
    (holds, found.join(", "))
}

/// The results file a rule reads.
fn source(rule: &Value) -> &str {
    rule["results"].as_str().unwrap_or("speed.json")
}

/// Whether `rule` holds, and what was found; `None` when its results are missing.
fn judge(rule: &Value) -> Option<(bool, String)> {
    let id = rule["id"].as_str().unwrap_or("");
    let found = read(&results(source(rule)))?;
    Some(match id {
        "flowapi-type-checks" => type_checks(&found),
        "faster" => ratio(&found, rule, "medianMs", true),
        "memory" => ratio(&found, rule, "peakRssKb", false),
        "sdk-configs-clean" => sdk::configs_clean(&found),
        "sdk-judge-passes" => sdk::judge_passes(&found),
        "sdk-generate-time" => sdk::generate_time(&found, rule),
        "agent-judge-passes" => sdk::agent_judge_passes(&found),
        _ => (false, format!("no check for the rule {id:?}")),
    })
}

fn main() -> ExitCode {
    let budgets = read(&bench().join("budgets.json")).unwrap_or(Value::Null);
    let rules = budgets["rules"].as_array().cloned().unwrap_or_default();
    let mut broken = rules.is_empty();
    for rule in &rules {
        let id = rule["id"].as_str().unwrap_or("?");
        match judge(rule) {
            Some((true, found)) => println!("ok      {id}: {found}"),
            Some((false, found)) => {
                broken = true;
                println!("BROKEN  {id}: {found}");
            }
            // Results that run.sh does not write (the paid agent runs): their absence is said.
            None if rule["optional"] == true => {
                println!("NONE    {id}: no results/{}; not a failure", source(rule));
            }
            None => {
                broken = true;
                let file = source(rule);
                let program = file.trim_end_matches(".json");
                println!("MISSING {id}: no results/{file}; run `{program}` (run.sh)");
            }
        }
    }
    if broken {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
