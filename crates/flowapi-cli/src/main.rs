//! The `flowapi` command, the shell around the core: it reads the description and the files its
//! references name, hands their text to the core, prints what was found and writes the output.
//! All the I/O of flowapi is here (G5).

mod args;
mod load;
mod source;

use flowapi::{Documents, Severity};
use std::process::ExitCode;

#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the command's output and its errors"
)]
fn main() -> ExitCode {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(args::Command::Help) => {
            println!("{}", args::HELP);
            return ExitCode::SUCCESS;
        }
        Ok(args::Command::Version) => {
            println!("flowapi {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(args::Command::Generate(a)) => a,
        Err(message) => {
            eprintln!("flowapi: {message}\n\n{}", args::HELP);
            return ExitCode::from(2);
        }
    };
    let texts = match load::all(&args.input) {
        Ok(t) => t,
        Err(message) => {
            eprintln!("flowapi: {message}");
            return ExitCode::FAILURE;
        }
    };
    let mut docs = Documents::new();
    for source in &texts {
        if let Err(d) = docs.add(source.uri.clone(), &source.json) {
            eprintln!("{d}\n  at {}", source.describe(d.at.offset));
            return ExitCode::FAILURE;
        }
    }
    let path = args.output.clone().unwrap_or_else(|| String::from("-"));
    let target: Box<dyn flowapi::Target> = if args.rust {
        Box::new(flowapi_rust::Rust::new(path))
    } else {
        Box::new(flowapi_ts::TypeScript::new(path))
    };
    let (output, diags) = flowapi::generate(&docs, target.as_ref());
    for d in diags.all() {
        if d.severity == Severity::Error || !args.quiet {
            let at = texts.get(d.at.doc.index()).map_or_else(
                || docs.describe(d.at),
                |source| source.describe(d.at.offset),
            );
            eprintln!("{d}\n  at {at}");
        }
    }
    let Some(output) = output else {
        return ExitCode::FAILURE;
    };
    for file in &output.files {
        let written = if file.path == "-" {
            print!("{}", file.text);
            Ok(())
        } else {
            std::fs::write(&file.path, &file.text)
                .map_err(|e| format!("cannot write {}: {e}", file.path))
        };
        if let Err(message) = written {
            eprintln!("flowapi: {message}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
