//! What `sdk-repo` measures on one SDK repository: the repositories (each example of `../examples/`
//! with a config, and the corpus descriptions of `DRAFTED` with a config drafted on the spot), and
//! the steps a user takes with one: `config check`, `generate` as a fresh process `rounds` times
//! into an empty folder (its wall time, its peak memory, the files and bytes it wrote), `npm
//! install` once (timed and kept apart: it is the network's time, not flowapi's), and the agent
//! plugin's judge on steps 0-6 with each step's time as the judge reports it.

use crate::process::{self, Run};
use crate::{bench, corpus};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Corpus descriptions measured with a config `config draft` writes, as a user's first one: the
/// largest of the corpus, with no example of its own (GitHub's and Stripe's are examples).
pub const DRAFTED: [&str; 1] = ["cloudflare"];

/// One repository to measure: its name, where its config comes from, and its description.
pub struct Case {
    pub name: String,
    /// `example` (its config is `../examples/<name>/flowapi-sdk.yml`) or `drafted`.
    pub source: &'static str,
    /// The example's folder; `None` for a drafted one.
    pub example: Option<PathBuf>,
    /// The description, and the file name the config gives it.
    pub spec: PathBuf,
    pub spec_name: String,
}

/// The release build the bench runs.
pub fn flowapi() -> PathBuf {
    bench().join("../target/release/flowapi")
}

fn examples() -> PathBuf {
    bench().join("../examples")
}

/// The `spec:` of an example's config, as the examples write it (one line, a file beside it).
fn spec_of(config: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(config).map_err(|e| format!("{}: {e}", config.display()))?;
    text.lines()
        .find_map(|l| l.strip_prefix("spec:"))
        .map(|s| s.trim().trim_matches('"').to_owned())
        .ok_or_else(|| format!("{}: no `spec:` line", config.display()))
}

/// Every repository, the examples by name, then the drafted ones.
pub fn cases() -> Result<Vec<Case>, String> {
    let mut out = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(examples())
        .map_err(|e| format!("../examples: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("flowapi-sdk.yml").is_file())
        .collect();
    dirs.sort();
    for dir in dirs {
        let spec_name = spec_of(&dir.join("flowapi-sdk.yml"))?;
        let name = dir
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        out.push(Case {
            name,
            source: "example",
            spec: dir.join(&spec_name),
            example: Some(dir),
            spec_name,
        });
    }
    for file in corpus()?
        .into_iter()
        .filter(|d| DRAFTED.contains(&d.name.as_str()))
    {
        let spec_name = "openapi.json".to_owned();
        out.push(Case {
            name: file.name,
            source: "drafted",
            example: None,
            spec: file.path,
            spec_name,
        });
    }
    Ok(out)
}

/// The files under `dir` and their bytes, `node_modules` left out.
fn size(dir: &Path) -> (u64, u64) {
    let (mut files, mut bytes) = (0, 0);
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if entry.file_name() != "node_modules" {
                    stack.push(path);
                }
            } else {
                files += 1;
                bytes += entry.metadata().map_or(0, |m| m.len());
            }
        }
    }
    (files, bytes)
}

fn median(values: &[f64]) -> f64 {
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    match values.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => values[mid],
        _ => f64::midpoint(values[mid - 1], values[mid]),
    }
}

fn ms(run: &Run) -> f64 {
    run.wall.as_secs_f64() * 1e3
}

/// One command in `dir`, measured, its output into `log`; `Err` is what went wrong.
fn step(
    dir: &Path,
    program: &Path,
    args: &[&str],
    log: &Path,
    limit: Duration,
) -> Result<Run, String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(dir);
    let run =
        process::measure(&mut command, log, limit).map_err(|e| format!("did not start: {e}"))?;
    if run.succeeded() {
        return Ok(run);
    }
    let said = std::fs::read_to_string(log).unwrap_or_default();
    Err(format!(
        "{}: {}",
        run.failure(limit),
        crate::tail(&said, 6, 800)
    ))
}

/// The folder `case` is set up in, `repo/` its SDK's root holding the config and the description.
fn prepare(case: &Case, work: &Path, limit: Duration) -> Result<Value, String> {
    let _ = std::fs::remove_dir_all(work);
    let repo = work.join("repo");
    std::fs::create_dir_all(&repo).map_err(|e| format!("{}: {e}", repo.display()))?;
    if !case.spec.is_file() {
        return Err(format!(
            "no {}: run examples/fetch.sh (or corpus/fetch.sh)",
            case.spec.display()
        ));
    }
    std::fs::copy(&case.spec, repo.join(&case.spec_name))
        .map_err(|e| format!("the description: {e}"))?;
    let mut said = json!({});
    if let Some(example) = &case.example {
        std::fs::copy(
            example.join("flowapi-sdk.yml"),
            repo.join("flowapi-sdk.yml"),
        )
        .map_err(|e| format!("the config: {e}"))?;
    } else {
        let args = [
            "config",
            "draft",
            case.spec_name.as_str(),
            "-o",
            "flowapi-sdk.yml",
        ];
        let run = step(&repo, &flowapi(), &args, &work.join("draft.log"), limit)?;
        said["draftMs"] = json!(ms(&run));
    }
    let out = Command::new(flowapi())
        .args(["config", "check", "flowapi-sdk.yml", "--json"])
        .current_dir(&repo)
        .output()
        .map_err(|e| format!("config check did not start: {e}"))?;
    let check: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let codes: Vec<&Value> = check["issues"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|i| &i["code"])
        .collect();
    said["config"] =
        json!({ "errors": check["errors"], "warnings": check["warnings"], "codes": codes });
    Ok(said)
}

/// `generate` `rounds` times into an empty folder, then once into `repo/` for the judge.
fn generate(work: &Path, rounds: usize, limit: Duration) -> Value {
    let repo = work.join("repo");
    let (mut times, mut peak) = (vec![], 0);
    let fresh = work.join("fresh");
    for _ in 0..rounds {
        let _ = std::fs::remove_dir_all(&fresh);
        let args = ["generate", "flowapi-sdk.yml", "--out", "../fresh"];
        match step(&repo, &flowapi(), &args, &work.join("generate.log"), limit) {
            Ok(run) => {
                times.push(ms(&run));
                peak = peak.max(run.peak_rss_kb);
            }
            Err(error) => return json!({ "ok": false, "error": error }),
        }
    }
    let (files, bytes) = size(&fresh);
    let into = step(
        &repo,
        &flowapi(),
        &["generate", "flowapi-sdk.yml", "--out", "."],
        &work.join("generate.log"),
        limit,
    );
    if let Err(error) = into {
        return json!({ "ok": false, "error": format!("into its folder: {error}") });
    }
    json!({ "ok": true, "error": null, "roundsMs": times, "medianMs": median(&times),
            "peakRssKb": peak, "files": files, "bytes": bytes })
}

/// `npm install` once in `repo/`: its time is the network's and the registry's.
fn install(work: &Path, limit: Duration) -> Value {
    let npm = PathBuf::from("npm");
    match step(
        &work.join("repo"),
        &npm,
        &["install", "--no-audit", "--no-fund"],
        &work.join("install.log"),
        limit,
    ) {
        Ok(run) => json!({ "ok": true, "ms": ms(&run), "error": null }),
        Err(error) => json!({ "ok": false, "ms": null, "error": error }),
    }
}

/// The judge on steps 0-6, every step run (`--keep-going`), with the flowapi measured here.
fn judge(work: &Path, limit: Duration) -> Value {
    let judge = bench().join("../plugins/flowapi/bin/judge.mjs");
    let started = Instant::now();
    let out = Command::new("node")
        .arg(judge)
        .arg(work.join("repo"))
        .args(["--steps", "0-6", "--json", "--keep-going", "--timeout"])
        .arg(limit.as_secs().to_string())
        .env("FLOWAPI_PLUGIN_FLOWAPI", flowapi())
        .output();
    let wall = started.elapsed().as_secs_f64() * 1e3;
    let report: Value = match out {
        Ok(out) => serde_json::from_slice(&out.stdout).unwrap_or(Value::Null),
        Err(e) => return json!({ "verdict": "did not start", "error": e.to_string(), "steps": [] }),
    };
    let steps: Vec<Value> = (report["steps"].as_array().into_iter().flatten())
        .map(|s| {
            json!({ "step": s["step"], "name": s["name"], "status": s["status"],
                         "ms": s["duration_ms"], "reason": s["reason"],
                         "output": s["output"].as_str().map(|o| crate::tail(o, 6, 600)) })
        })
        .collect();
    json!({ "verdict": report["verdict"].as_str().unwrap_or("no report"), "ms": wall, "steps": steps })
}

/// Every measure of one repository, its work under `work`.
pub fn measure(case: &Case, work: &Path, rounds: usize, limit: Duration) -> Value {
    let bytes = std::fs::metadata(&case.spec).map_or(0, |m| m.len());
    let mut row = json!({ "name": case.name, "source": case.source, "descriptionBytes": bytes });
    let prepared = match prepare(case, work, limit) {
        Ok(prepared) => prepared,
        Err(error) => {
            row["error"] = json!(error);
            return row;
        }
    };
    for (key, value) in prepared.as_object().into_iter().flatten() {
        row[key] = value.clone();
    }
    row["generate"] = generate(work, rounds, limit);
    if row["generate"]["ok"] != true {
        return row;
    }
    row["install"] = install(work, limit);
    row["judge"] = judge(work, limit);
    row
}
