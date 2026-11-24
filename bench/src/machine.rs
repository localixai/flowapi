//! The machine a run was taken on, as the results describe it: the processor, its cores, the
//! memory and the operating system. Read from `/proc` and `/etc/os-release` on Linux, from
//! `sysctl` and `sw_vers` on macOS.

use crate::process::output;
use serde_json::{Value, json};
use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The value of the first `key: value` line of `/proc/cpuinfo` or `/proc/meminfo`.
fn proc_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find(|line| line.split(':').next().is_some_and(|k| k.trim() == key))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().to_owned())
}

fn cpu() -> String {
    proc_field(&read("/proc/cpuinfo"), "model name")
        .or_else(|| {
            output(
                "sysctl",
                &["-n", "machdep.cpu.brand_string"],
                Path::new("/"),
            )
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn memory_gb() -> f64 {
    let kb = proc_field(&read("/proc/meminfo"), "MemTotal")
        .and_then(|value| value.split_whitespace().next()?.parse::<f64>().ok());
    let bytes = kb.map(|kb| kb * 1024.0).or_else(|| {
        output("sysctl", &["-n", "hw.memsize"], Path::new("/"))?
            .parse::<f64>()
            .ok()
    });
    bytes.map_or(0.0, |bytes| (bytes / 1e9 * 10.0).round() / 10.0)
}

fn os() -> String {
    let release = read("/etc/os-release");
    let pretty = release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|name| name.trim_matches('"').to_owned());
    let name = pretty
        .or_else(|| {
            let version = output("sw_vers", &["-productVersion"], Path::new("/"))?;
            Some(format!("macOS {version}"))
        })
        .unwrap_or_else(|| std::env::consts::OS.to_owned());
    let kernel = output("uname", &["-sr"], Path::new("/")).unwrap_or_default();
    format!("{name}, {kernel}")
}

/// The machine, as JSON.
pub fn describe() -> Value {
    let cores = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    json!({ "cpu": cpu(), "cores": cores, "memoryGb": memory_gb(), "os": os() })
}

/// The machine in one line of prose.
pub fn line(machine: &Value) -> String {
    format!(
        "{}, {} cores, {} GB, {}",
        machine["cpu"].as_str().unwrap_or("?"),
        machine["cores"],
        machine["memoryGb"],
        machine["os"].as_str().unwrap_or("?"),
    )
}
