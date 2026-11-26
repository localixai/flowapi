//! The command line, read by hand: no dependency for a few flags (G11).

pub(crate) const HELP: &str = "\
flowapi: types from an OpenAPI description or a JSON Schema

Usage: flowapi <description> [-o <types.ts>] [--target ts|rust] [--sdk] [--quiet]
       flowapi summary <description> [--tag <tag>] [--path <prefix>] [--max-bytes <n>]
       flowapi config draft <description> [-o <flowapi-sdk.yml>]
       flowapi config schema
       flowapi config check <flowapi-sdk.yml> [--spec <description>] [--json]

  -o, --output <file>   where to write the types (standard output by default)
  -t, --target <lang>   the language of the types: ts (the default) or rust (serde)
      --sdk             also a function per operation that writes its request and one that
                        reads its response, with no HTTP library and no I/O (Rust: serde and
                        serde_json only, as the types)
  -q, --quiet           print errors only, not warnings
  -h, --help            this help
  -V, --version         the version

The description is Swagger 2.0, OpenAPI 3.0, 3.1 or 3.2, or a JSON Schema (2020-12 when it names no
$schema), in JSON or YAML (.yaml, .yml, or any text not opening with { or [). Files its
$refs name by a relative path are read too.

For an SDK's config, flowapi-sdk.yml, which an agent writes:
  summary         the description in compact JSON: operations with their parameters, media
                  types and the pagination they look like they have; over --max-bytes (60000)
                  the tags alone, to be read on by --tag or --path
  config draft    a first config, every operation in a resource, as YAML
  config schema   the config's format, as JSON Schema
  config check    every issue of a config against its description (its `spec`, relative to the
                  config, or --spec): text, or JSON with --json; exits 1 on an error

A description named `summary` or `config` is read by its path: `flowapi ./summary`.";

pub(crate) struct Generate {
    pub(crate) input: String,
    pub(crate) output: Option<String>,
    pub(crate) quiet: bool,
    /// The Rust target rather than TypeScript.
    pub(crate) rust: bool,
    /// The SDK after the types.
    pub(crate) sdk: bool,
}

/// A command about an SDK's config (`sdk.rs`).
pub(crate) enum Sdk {
    Summary {
        input: String,
        filter: flowapi_sdk::Filter,
    },
    Draft {
        input: String,
        output: Option<String>,
    },
    Schema,
    Check {
        config: String,
        spec: Option<String>,
        json: bool,
    },
}

pub(crate) enum Command {
    Help,
    Version,
    Generate(Generate),
    Sdk(Sdk),
}

pub(crate) fn parse(args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut args = args.peekable();
    match args.peek().map(String::as_str) {
        Some("summary") => {
            args.next();
            return summary(args);
        }
        Some("config") => {
            args.next();
            return config(args);
        }
        Some(_) | None => {}
    }
    let mut input = None;
    let mut output = None;
    let mut quiet = false;
    let mut rust = false;
    let mut sdk = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "-q" | "--quiet" => quiet = true,
            "--sdk" => sdk = true,
            "-t" | "--target" => match args.next().as_deref() {
                Some("ts" | "typescript") => rust = false,
                Some("rust") => rust = true,
                _ => return Err(String::from("`--target` is `ts` or `rust`")),
            },
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
        rust,
        sdk,
    }))
}

/// `summary <description> [--tag T] [--path P] [--max-bytes N]`.
fn summary(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut input = None;
    let mut filter = flowapi_sdk::Filter::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--tag" => filter.tag = Some(args.next().ok_or("`--tag` needs a tag")?),
            "--path" => filter.path = Some(args.next().ok_or("`--path` needs a path's start")?),
            "--max-bytes" => {
                let n = args.next().ok_or("`--max-bytes` needs a number")?;
                filter.max_bytes = n.parse().map_err(|e| format!("`--max-bytes {n}`: {e}"))?;
            }
            flag if flag.starts_with('-') => return Err(format!("unknown flag `{flag}`")),
            path => {
                if input.replace(path.to_owned()).is_some() {
                    return Err(String::from("one description at a time"));
                }
            }
        }
    }
    let input = input.ok_or("a description to summarize")?;
    Ok(Command::Sdk(Sdk::Summary { input, filter }))
}

/// `config draft|schema|check …`.
fn config(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let what = args
        .next()
        .ok_or("`config` is followed by `draft`, `schema` or `check`")?;
    let mut path = None;
    let mut output = None;
    let mut spec = None;
    let mut json = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-o" | "--output" if what == "draft" => {
                output = Some(args.next().ok_or("`-o` needs a file")?)
            }
            "--spec" if what == "check" => {
                spec = Some(args.next().ok_or("`--spec` needs a description")?)
            }
            "--json" if what == "check" => json = true,
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag `{flag}` for `config {what}`"));
            }
            p => {
                if path.replace(p.to_owned()).is_some() {
                    return Err(String::from("one file at a time"));
                }
            }
        }
    }
    let sdk = match (what.as_str(), path) {
        ("schema", None) => Sdk::Schema,
        ("draft", Some(input)) => Sdk::Draft { input, output },
        ("check", Some(config)) => Sdk::Check { config, spec, json },
        ("schema", Some(_)) => return Err(String::from("`config schema` reads no file")),
        ("draft", None) => return Err(String::from("`config draft` needs a description")),
        ("check", None) => return Err(String::from("`config check` needs a config")),
        (other, _) => {
            return Err(format!(
                "`config {other}`: the commands are `draft`, `schema` and `check`"
            ));
        }
    };
    Ok(Command::Sdk(sdk))
}
