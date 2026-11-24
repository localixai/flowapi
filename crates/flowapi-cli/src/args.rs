//! The command line, read by hand: no dependency for a few flags (G11).

pub(crate) const HELP: &str = "\
flowapi: types from an OpenAPI description or a JSON Schema

Usage: flowapi <description> [-o <types.ts>] [--quiet]

  -o, --output <file>   where to write the types (standard output by default)
  -q, --quiet           print errors only, not warnings
  -h, --help            this help
  -V, --version         the version

The description is OpenAPI 3.0, 3.1 or 3.2, or a JSON Schema (2020-12 when it names no
$schema), in JSON or YAML (.yaml, .yml, or any text not opening with { or [). Files its
$refs name by a relative path are read too.";

pub(crate) struct Generate {
    pub(crate) input: String,
    pub(crate) output: Option<String>,
    pub(crate) quiet: bool,
}

pub(crate) enum Command {
    Help,
    Version,
    Generate(Generate),
}

pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut input = None;
    let mut output = None;
    let mut quiet = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "-q" | "--quiet" => quiet = true,
            "-o" | "--output" => output = Some(args.next().ok_or("`-o` needs a file")?),
            flag if flag.starts_with('-') && flag != "-" => {
                return Err(format!("unknown flag `{flag}`"));
            }
            path => {
                if input.replace(path.to_owned()).is_some() {
                    return Err(String::from("one description at a time"));
                }
            }
        }
    }
    let input = input.ok_or("a description to read")?;
    Ok(Command::Generate(Generate {
        input,
        output,
        quiet,
    }))
}
