//! One run of the TypeScript compiler over every file written, its diagnostics read back by
//! file and line.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// The compiler's diagnostics: by file, by line, each message with its continuation lines.
#[derive(Default)]
pub(crate) struct Diagnostics {
    pub(crate) by_file: BTreeMap<String, BTreeMap<usize, Vec<String>>>,
    /// Lines that name no file of ours: the compiler's own complaints.
    pub(crate) other: Vec<String>,
}

/// Checks `files` (names inside `out`) under the oracle's options, from inside `out`.
pub(crate) fn check(tsc: &Path, out: &Path, files: &[String]) -> Diagnostics {
    let output = Command::new(tsc)
        .current_dir(out)
        .args([
            "--ignoreConfig",
            "--strict",
            "--exactOptionalPropertyTypes",
            "--noEmit",
        ])
        .args(["--target", "es2022", "--skipLibCheck", "--pretty", "false"])
        .args(files)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "cannot run {}: {e} (npm ci in conformance/ts)",
                tsc.display()
            )
        });
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    parse(&text)
}

/// `file(line,column): error TSnnnn: message`, then lines indented under it.
fn parse(text: &str) -> Diagnostics {
    let mut d = Diagnostics::default();
    let mut last: Option<(String, usize)> = None;
    for line in text.lines() {
        if line.starts_with(' ') {
            match &last {
                Some((file, n)) => {
                    let messages = d.by_file.get_mut(file).and_then(|m| m.get_mut(n));
                    if let Some(message) = messages.and_then(|m| m.last_mut()) {
                        message.push('\n');
                        message.push_str(line.trim_end());
                    }
                }
                None => d.other.push(line.to_owned()),
            }
            continue;
        }
        match located(line) {
            Some((file, n, message)) => {
                d.by_file
                    .entry(file.clone())
                    .or_default()
                    .entry(n)
                    .or_default()
                    .push(message);
                last = Some((file, n));
            }
            None => {
                if !line.trim().is_empty() {
                    d.other.push(line.to_owned());
                }
                last = None;
            }
        }
    }
    d
}

fn located(line: &str) -> Option<(String, usize, String)> {
    let (head, message) = line.split_once("): ")?;
    let (file, position) = head.rsplit_once('(')?;
    let (n, _) = position.split_once(',')?;
    Some((file.to_owned(), n.parse().ok()?, message.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn reads_files_lines_and_continuations() {
        let d = parse(
            "a-0.ts(4,7): error TS2322: Type 'x' is not assignable.\n  Because.\nb-1.ts(2,1): error TS1: y\n",
        );
        assert_eq!(
            d.by_file["a-0.ts"][&4],
            vec!["error TS2322: Type 'x' is not assignable.\n  Because."]
        );
        assert_eq!(d.by_file["b-1.ts"][&2], vec!["error TS1: y"]);
        assert!(d.other.is_empty());
    }
}
