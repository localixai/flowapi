//! Oracle O5. From fixed seeds: a random schema (every keyword a type can show, references,
//! unions and intersections), then random values; flowschema says which values are valid. Each
//! schema's type is printed by flowapi and each value written as a literal of it; one run of the
//! TypeScript compiler says which literals fit. A value flowschema accepts and the compiler
//! refuses is unsound. Schemas whose keywords flowschema does not check are left out.

mod random;

use std::fmt::Write as _;
use std::path::Path;

const SCHEMAS: usize = 600;
const VALUES: usize = 12;

fn main() -> std::process::ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = root.join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("out/");
    // Another seed for a new sample: `cargo run --release -- <seed>`; the results file is the
    // default seed's.
    let seed = std::env::args().nth(1).and_then(|s| s.parse::<u64>().ok());
    let mut r = random::Rng(seed.unwrap_or(0x5eed_f10a_0001));
    let mut files = Vec::new();
    let mut cases: Vec<(String, usize, bool, String, String)> = Vec::new();
    let mut skipped = 0;
    for n in 0..SCHEMAS {
        let schema = random::schema(&mut r, 4);
        let checked = flowschema::Schema::new(&schema);
        if !checked.unchecked().is_empty() {
            skipped += 1;
            continue;
        }
        let text = schema.to_string();
        let mut docs = flowapi::Documents::new();
        docs.add("root.json", &text).expect("JSON");
        let (output, _) = flowapi::generate(&docs, &flowapi_ts::TypeScript::new("x.ts"));
        let Some(output) = output else { continue };
        let mut ts = output.files.into_iter().map(|f| f.text).collect::<String>();
        let first_case_line = ts.lines().count() + 1;
        for v in 0..VALUES {
            let value = random::value(&mut r, 4, &schema);
            let valid = checked.matches(&value);
            let _ = writeln!(ts, "const c{v}: Root = {value};");
            cases.push((
                format!("s{n}.ts"),
                first_case_line + v,
                valid,
                text.clone(),
                value.to_string(),
            ));
        }
        let file = out.join(format!("s{n}.ts"));
        std::fs::write(&file, ts).expect("written");
        files.push(file);
    }
    let tsc = root.join("../ts/node_modules/.bin/tsc");
    let run = std::process::Command::new(tsc)
        .args([
            "--ignoreConfig",
            "--strict",
            "--exactOptionalPropertyTypes",
            "--noEmit",
            "--target",
            "es2022",
            "--skipLibCheck",
            "--pretty",
            "false",
        ])
        .args(&files)
        .output()
        .expect("tsc runs");
    let diagnostics = String::from_utf8_lossy(&run.stdout).into_owned();
    let refused = |file: &str, line: usize| {
        diagnostics
            .lines()
            .any(|l| l.contains(&format!("{file}({line},")))
    };
    let type_errors = diagnostics
        .lines()
        .filter(|l| l.contains("error TS"))
        .filter(|l| {
            cases
                .iter()
                .all(|(f, line, ..)| !l.contains(&format!("{f}({line},")))
        })
        .count();
    let (mut valid, mut accepted, mut invalid, mut rejected) = (0, 0, 0, 0);
    let mut unsound = String::new();
    for (file, line, ok, schema, value) in &cases {
        let no = refused(file, *line);
        if *ok {
            valid += 1;
            if no {
                let _ = writeln!(unsound, "- `{schema}` refused `{value}`");
            } else {
                accepted += 1;
            }
        } else {
            invalid += 1;
            rejected += usize::from(no);
        }
    }
    let report = format!(
        "# O5: random schemas, flowschema against the TypeScript compiler\n\nWritten by `cd conformance/props && cargo run --release`; do not edit. {SCHEMAS} schemas from a fixed seed ({skipped} left out: a keyword flowschema does not check), {VALUES} values each.\n\n| Valid values | Accepted | Invalid values | Rejected | Diagnostics outside the cases |\n| ---: | ---: | ---: | ---: | ---: |\n| {valid} | {accepted} | {invalid} | {rejected} | {type_errors} |\n\n## Unsound\n\n{}\n",
        if unsound.is_empty() {
            "None.".to_owned()
        } else {
            unsound.clone()
        }
    );
    let path = match seed {
        None => root.join("../results/props.md"),
        Some(n) => root.join(format!("out/props-{n}.md")),
    };
    std::fs::write(path, report).expect("results");
    println!(
        "valid {valid}, accepted {accepted}; invalid {invalid}, rejected {rejected}; other diagnostics {type_errors}"
    );
    if accepted == valid && type_errors == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
