//! What the benchmark binaries share: where the bench's files are, the corpus it reads
//! (`corpus/corpus.tsv`), one process run and measured (`process`), the machine it ran on
//! (`machine`), and an SDK repository built and judged (`sdk_repo`, its table `sdk_report`).

pub mod machine;
pub mod process;
pub mod report;
pub mod sdk_repo;
pub mod sdk_report;

use std::path::PathBuf;

/// The bench's own directory, whatever the directory it is run from.
pub fn bench() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A file of `results/`.
pub fn results(name: &str) -> PathBuf {
    bench().join("results").join(name)
}

/// One description of the corpus: its name, the JSON `corpus/fetch.sh` writes for it, its size.
pub struct Description {
    pub name: String,
    pub path: PathBuf,
    pub bytes: u64,
}

/// The corpus, in the order `corpus/corpus.tsv` lists it (smallest first).
pub fn corpus() -> Result<Vec<Description>, String> {
    let list = bench().join("corpus/corpus.tsv");
    let text =
        std::fs::read_to_string(&list).map_err(|err| format!("{}: {err}", list.display()))?;
    let mut rows = text.lines().filter(|line| !line.starts_with('#'));
    rows.next(); // the header
    rows.filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split('\t').collect();
            let (Some(name), Some(bytes)) = (cells.first(), cells.get(4)) else {
                return Err(format!("corpus.tsv: a row of too few columns: {line}"));
            };
            let bytes = bytes
                .parse()
                .map_err(|err| format!("corpus.tsv: {bytes}: {err}"))?;
            let path = bench().join("data").join(format!("{name}.json"));
            Ok(Description {
                name: (*name).to_owned(),
                path,
                bytes,
            })
        })
        .collect()
}

/// The last `lines` lines of `text`, at most `chars` characters of them: what an error says.
pub fn tail(text: &str, lines: usize, chars: usize) -> String {
    let all: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let kept = all[all.len().saturating_sub(lines)..].join("\n");
    let count = kept.chars().count();
    kept.chars().skip(count.saturating_sub(chars)).collect()
}
