//! `results/sdk-repo.md`, written from what `sdk-repo` measured (the same JSON it writes to
//! `results/sdk-repo.json`), never by hand.

use crate::machine;
use crate::report::{bytes, memory, time};
use serde_json::Value;
use std::fmt::Write;

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

fn header(all: &Value, out: &mut String) {
    let versions = &all["versions"];
    let commit = versions["flowapiCommit"].as_str().unwrap_or("?");
    let dirty = if versions["flowapiUncommitted"] == true {
        ", with changes not committed"
    } else {
        ""
    };
    let rounds = all["rounds"].as_u64().unwrap_or(0);
    let _ = writeln!(
        out,
        "# An SDK repository from a description: generate, install, judge\n"
    );
    let _ = writeln!(
        out,
        "{}, {} ({commit}{dirty}), {}, on {}.",
        all["date"].as_str().unwrap_or("?"),
        versions["flowapi"].as_str().unwrap_or("flowapi ?"),
        versions["rustc"].as_str().unwrap_or("rustc ?"),
        machine::line(&all["machine"]),
    );
    let _ = writeln!(
        out,
        "Node.js {}, npm {}. Written by `cargo run --release --bin sdk-repo` in `bench/`; do not edit.",
        versions["node"].as_str().unwrap_or("?"),
        versions["npm"].as_str().unwrap_or("?"),
    );
    let rounds_said = if rounds == 1 {
        "one round (`--quick`): a first look, not the published figures".to_owned()
    } else {
        format!("the median of {rounds} rounds")
    };
    let _ = writeln!(
        out,
        "`generate` is run as a fresh process into an empty folder, the whole process timed; the time \
         is {rounds_said}, the memory the process's peak resident set. `npm install` runs once and is \
         the network's time, not flowapi's: no budget holds it. The judge is the agent plugin's \
         (`plugins/flowapi/bin/judge.mjs --steps 0-6 --keep-going`), each step timed by it; step 5 \
         fetches publint and attw with `npx`.\n",
    );
}

fn summary(all: &Value, out: &mut String) {
    let _ = writeln!(
        out,
        "| Repository | Config | Description | `generate` | Peak memory | Files | Bytes | `npm install` | Judge |"
    );
    let _ = writeln!(
        out,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for row in all["rows"].as_array().into_iter().flatten() {
        let name = row["name"].as_str().unwrap_or("?");
        let config = match (row["source"].as_str(), row["config"]["warnings"].as_u64()) {
            (Some("drafted"), _) => "drafted".to_owned(),
            (_, Some(0)) => "the example's".to_owned(),
            (_, Some(n)) => format!("the example's, {n} warnings"),
            _ => "the example's".to_owned(),
        };
        let size = bytes(number(&row["descriptionBytes"]));
        let g = &row["generate"];
        if g["ok"] != true {
            let _ = writeln!(out, "| {name} | {config} | {size} | failed | | | | | |");
            continue;
        }
        let install = match row["install"]["ok"].as_bool() {
            Some(true) => time(number(&row["install"]["ms"])),
            _ => "failed".to_owned(),
        };
        let _ = writeln!(
            out,
            "| {name} | {config} | {size} | {} | {} | {} | {} | {install} | {} |",
            time(number(&g["medianMs"])),
            memory(number(&g["peakRssKb"])),
            g["files"],
            bytes(number(&g["bytes"])),
            row["judge"]["verdict"].as_str().unwrap_or("-"),
        );
    }
}

fn steps(all: &Value, out: &mut String) {
    let _ = writeln!(out, "\n## The judge's steps\n");
    let _ = writeln!(out, "| Repository | Step | Status | Time | Note |");
    let _ = writeln!(out, "| --- | --- | --- | ---: | --- |");
    let mut failures = vec![];
    for row in all["rows"].as_array().into_iter().flatten() {
        let name = row["name"].as_str().unwrap_or("?");
        for s in row["judge"]["steps"].as_array().into_iter().flatten() {
            let status = s["status"].as_str().unwrap_or("?");
            let _ = writeln!(
                out,
                "| {name} | {} {} | {status} | {} | {} |",
                s["step"],
                s["name"].as_str().unwrap_or("?"),
                time(number(&s["ms"])),
                s["reason"].as_str().unwrap_or(""),
            );
            if status == "fail" {
                let said = s["output"].as_str().unwrap_or("").replace('\n', " / ");
                failures.push(format!("- {name}, step {}: {said}", s["step"]));
            }
        }
        for key in ["error", "generate", "install"] {
            let error = if key == "error" {
                &row[key]
            } else {
                &row[key]["error"]
            };
            if let Some(error) = error.as_str() {
                failures.push(format!("- {name}, {key}: {}", error.replace('\n', " / ")));
            }
        }
    }
    if !failures.is_empty() {
        let _ = writeln!(out, "\n## What failed\n\n{}", failures.join("\n"));
    }
}

/// `results/sdk-repo.md`.
pub fn sdk_repo(all: &Value) -> String {
    let mut out = String::new();
    header(all, &mut out);
    summary(all, &mut out);
    steps(all, &mut out);
    out
}
