//! `sdk-repo`'s rules, read against `results/sdk-repo.json`: every config clean, the judge passing
//! every step on every repository, and `generate`'s time on the repositories a rule names; and the
//! one rule of the live agent runs (`results/agent.json`): the judge passes on each.

use flowapi_bench::report::time;
use serde_json::Value;

fn rows(found: &Value) -> Vec<&Value> {
    found["rows"].as_array().into_iter().flatten().collect()
}

fn name(row: &Value) -> &str {
    row["name"].as_str().unwrap_or("?")
}

/// Every config passes `config check` with no error and no warning.
pub fn configs_clean(found: &Value) -> (bool, String) {
    let all = rows(found);
    let wrong: Vec<String> = all
        .iter()
        .filter(|row| row["config"]["errors"] != 0 || row["config"]["warnings"] != 0)
        .map(|row| format!("{} {}", name(row), row["config"]["codes"]))
        .collect();
    let holds = wrong.is_empty() && !all.is_empty();
    (
        holds,
        format!(
            "{} of {} clean; not: {wrong:?}",
            all.len() - wrong.len(),
            all.len()
        ),
    )
}

/// The judge's verdict is `pass` on every repository, with each of steps 0-6 run and passed.
pub fn judge_passes(found: &Value) -> (bool, String) {
    let all = rows(found);
    let mut wrong = vec![];
    for row in &all {
        let steps: Vec<&Value> = row["judge"]["steps"]
            .as_array()
            .into_iter()
            .flatten()
            .collect();
        let failed: Vec<String> = steps
            .iter()
            .filter(|s| s["status"] != "pass")
            .map(|s| format!("{} {}", s["step"], s["status"].as_str().unwrap_or("?")))
            .collect();
        if row["judge"]["verdict"] != "pass" || steps.len() != 7 || !failed.is_empty() {
            let verdict = row["judge"]["verdict"].as_str().unwrap_or("not judged");
            wrong.push(format!("{} {verdict} {failed:?}", name(row)));
        }
    }
    let holds = wrong.is_empty() && !all.is_empty();
    (
        holds,
        format!(
            "{} of {} pass; not: {wrong:?}",
            all.len() - wrong.len(),
            all.len()
        ),
    )
}

/// `generate`'s median on each repository the rule's `maxMs` names, at most its number there.
pub fn generate_time(found: &Value, rule: &Value) -> (bool, String) {
    let mut holds = true;
    let mut said = vec![];
    for (repo, max) in rule["maxMs"].as_object().into_iter().flatten() {
        let max = max.as_f64().unwrap_or(f64::NAN);
        let row = rows(found)
            .into_iter()
            .find(|row| row["name"] == repo.as_str());
        let Some(ms) = row
            .filter(|r| r["generate"]["ok"] == true)
            .and_then(|r| r["generate"]["medianMs"].as_f64())
        else {
            holds = false;
            said.push(format!("{repo} not measured BROKEN"));
            continue;
        };
        let ok = ms <= max;
        holds &= ok;
        let mark = if ok { "" } else { " BROKEN" };
        said.push(format!("{repo} {} (<= {}){mark}", time(ms), time(max)));
    }
    (holds && !said.is_empty(), said.join(", "))
}

/// On every agent run kept, the judge (run by the runner, after the agent) passed every step.
pub fn agent_judge_passes(found: &Value) -> (bool, String) {
    let runs: Vec<&Value> = found["runs"].as_array().into_iter().flatten().collect();
    let wrong: Vec<String> = runs
        .iter()
        .filter(|run| {
            let steps = run["judge"]["steps"].as_array().map_or(0, Vec::len);
            run["judge"]["verdict"] != "pass" || steps != 7
        })
        .map(|run| run["scenario"].as_str().unwrap_or("?").to_owned())
        .collect();
    let holds = wrong.is_empty() && !runs.is_empty();
    (
        holds,
        format!(
            "{} of {} pass; not: {wrong:?}",
            runs.len() - wrong.len(),
            runs.len()
        ),
    )
}
