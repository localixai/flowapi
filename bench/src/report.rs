//! `results/speed.md`, written from what `speed` measured (the same JSON it writes to
//! `results/speed.json`), never by hand.

use crate::machine;
use serde_json::Value;
use std::fmt::Write;

/// A time in milliseconds, as a reader takes it in.
pub fn time(ms: f64) -> String {
    match ms {
        ms if ms.is_nan() => "-".to_owned(),
        ms if ms < 10.0 => format!("{ms:.1} ms"),
        ms if ms < 1000.0 => format!("{ms:.0} ms"),
        ms => format!("{:.2} s", ms / 1e3),
    }
}

/// A size in bytes.
pub fn bytes(n: f64) -> String {
    match n {
        n if n < 1e6 => format!("{:.0} KB", n / 1e3),
        n => format!("{:.1} MB", n / 1e6),
    }
}

/// Resident memory, given in KB.
pub fn memory(kb: f64) -> String {
    format!("{:.0} MB", kb / 1024.0)
}

/// The row of `tool` on `file`.
pub fn row<'a>(all: &'a Value, file: &str, tool: &str) -> Option<&'a Value> {
    all["rows"]
        .as_array()?
        .iter()
        .find(|row| row["file"] == file && row["tool"] == tool)
}

/// The files measured, in order, each once.
pub fn files(all: &Value) -> Vec<String> {
    let mut files: Vec<String> = vec![];
    for row in all["rows"].as_array().into_iter().flatten() {
        let name = row["file"].as_str().unwrap_or("").to_owned();
        if !files.contains(&name) {
            files.push(name);
        }
    }
    files
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

fn checks(row: &Value) -> &'static str {
    match row["typeChecks"].as_bool() {
        Some(true) => "yes",
        Some(false) => "no",
        None => "not checked",
    }
}

fn header(all: &Value, out: &mut String) {
    let versions = &all["versions"];
    let rounds = all["rounds"].as_u64().unwrap_or(0);
    let commit = versions["flowapiCommit"].as_str().unwrap_or("?");
    let dirty = if versions["flowapiUncommitted"] == true {
        ", with changes not committed"
    } else {
        ""
    };
    let _ = writeln!(out, "# flowapi against the other generators\n");
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
        "Against openapi-typescript {} and @hey-api/openapi-ts {} (its `@hey-api/typescript` plugin \
         alone) with TypeScript {}, on Node.js {}.",
        versions["openapi-typescript"].as_str().unwrap_or("?"),
        versions["@hey-api/openapi-ts"].as_str().unwrap_or("?"),
        versions["typescriptOfTheGenerators"]
            .as_str()
            .unwrap_or("?"),
        versions["node"].as_str().unwrap_or("?"),
    );
    let rounds_said = if rounds == 1 {
        "one round (`--quick`): a first look, not the published figures".to_owned()
    } else {
        format!("the median of {rounds} rounds")
    };
    let _ = writeln!(
        out,
        "Each tool is run as a fresh process, the whole process timed; the time is {rounds_said}. \
         Memory is the process's peak resident set. Outputs are type-checked by {} as `{}`.\n",
        versions["tsc"]
            .as_str()
            .unwrap_or("tsc ?")
            .replace("Version", "TypeScript"),
        all["typeCheck"]["command"].as_str().unwrap_or("?"),
    );
}

fn summary(all: &Value, tools: &[&str], out: &mut String) {
    let _ = writeln!(out, "## Time\n");
    let _ = write!(out, "| Description | Size |");
    for tool in tools {
        let _ = write!(out, " `{tool}` |");
    }
    let _ = writeln!(out, " flowapi faster than the fastest other by |");
    let _ = writeln!(
        out,
        "| --- | ---: |{} ---: |",
        " ---: |".repeat(tools.len())
    );
    for file in files(all) {
        let size = row(all, &file, tools[0]).map_or(f64::NAN, |row| number(&row["bytes"]));
        let _ = write!(out, "| {file} | {} |", bytes(size));
        for tool in tools {
            let cell = match row(all, &file, tool) {
                Some(row) if row["ok"] == true => time(number(&row["medianMs"])),
                _ => "failed".to_owned(),
            };
            let _ = write!(out, " {cell} |");
        }
        let lead = lead(all, &file, tools, "medianMs");
        let _ = writeln!(
            out,
            " {} |",
            lead.map_or_else(|| "-".to_owned(), |x| format!("{x:.1}x"))
        );
    }
}

/// How many times `key` of the best other tool is flowapi's, on `file`; `None` when flowapi or
/// every other tool failed there.
pub fn lead(all: &Value, file: &str, tools: &[&str], key: &str) -> Option<f64> {
    let ours = row(all, file, "flowapi").filter(|row| row["ok"] == true)?;
    let best = (tools.iter().filter(|tool| **tool != "flowapi"))
        .filter_map(|tool| row(all, file, tool).filter(|row| row["ok"] == true))
        .map(|row| number(&row[key]))
        .min_by(f64::total_cmp)?;
    Some(best / number(&ours[key]))
}

fn details(all: &Value, tools: &[&str], out: &mut String) {
    let _ = writeln!(out, "\n## Each description\n");
    let _ = writeln!(
        out,
        "| Description | Tool | Time | Peak memory | Output | Type-checks |"
    );
    let _ = writeln!(out, "| --- | --- | ---: | ---: | ---: | --- |");
    let mut failures = vec![];
    for file in files(all) {
        for tool in tools {
            let Some(row) = row(all, &file, tool) else {
                continue;
            };
            if row["ok"] != true {
                let _ = writeln!(out, "| {file} | `{tool}` | failed | | | |");
                failures.push(format!(
                    "- {file}, `{tool}`: {}",
                    row["error"].as_str().unwrap_or("?")
                ));
                continue;
            }
            let mut said = checks(row).to_owned();
            if row["typeChecks"] == false {
                let errors = row["typeErrors"].as_u64().unwrap_or(0);
                let _ = write!(
                    said,
                    " ({errors} error{})",
                    if errors == 1 { "" } else { "s" }
                );
                failures.push(format!(
                    "- {file}, `{tool}`, type errors: {}",
                    row["typeErrorSample"]
                        .as_str()
                        .unwrap_or("")
                        .lines()
                        .next()
                        .unwrap_or("")
                ));
            }
            let _ = writeln!(
                out,
                "| {file} | `{tool}` | {} | {} | {} | {said} |",
                time(number(&row["medianMs"])),
                memory(number(&row["peakRssKb"])),
                bytes(number(&row["outputBytes"])),
            );
        }
    }
    if !failures.is_empty() {
        let _ = writeln!(out, "\n## What failed\n\n{}", failures.join("\n"));
    }
}

/// `results/speed.md`.
pub fn speed(all: &Value) -> String {
    let tools: Vec<&str> = (all["tools"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    let mut out = String::new();
    header(all, &mut out);
    summary(all, &tools, &mut out);
    details(all, &tools, &mut out);
    out
}
