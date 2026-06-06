//! Command-line entry point for `vdoc`.

use serde_json::json;
use std::env;
use std::path::Path;
use std::process::ExitCode;
use vibe_doc_core::{init_repository, InitError, InitOptions, InitPlan};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    let command = parse_command(args)?;

    match command {
        Command::Init(options) => run_init(options),
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn run_init(options: InitCommandOptions) -> Result<(), CliError> {
    match init_repository(
        env::current_dir().map_err(CliError::CurrentDir)?,
        InitOptions {
            dry_run: options.dry_run,
            force: options.force,
        },
    ) {
        Ok(plan) => {
            if options.json {
                print_init_json(&plan, options.dry_run, options.force);
            } else {
                print_init_text(&plan, options.dry_run);
            }
            Ok(())
        }
        Err(error) => {
            if options.json {
                print_error_json(&error);
            }
            Err(CliError::Init(error))
        }
    }
}

fn print_init_text(plan: &InitPlan, dry_run: bool) {
    if dry_run {
        println!("vdoc init dry-run:");
    } else {
        println!("vdoc init complete:");
    }

    for change in &plan.changes {
        println!(
            "- {} {} {}",
            change.action.as_str(),
            change.kind.as_str(),
            display_path(&change.path)
        );
    }
}

fn print_init_json(plan: &InitPlan, dry_run: bool, force: bool) {
    let writes: Vec<_> = plan
        .changes
        .iter()
        .map(|change| {
            json!({
                "path": display_path(&change.path),
                "kind": change.kind.as_str(),
                "action": change.action.as_str(),
            })
        })
        .collect();

    println!(
        "{}",
        json!({
            "command": "init",
            "dry_run": dry_run,
            "force": force,
            "changes": writes,
        })
    );
}

fn print_error_json(error: &InitError) {
    let payload = match error {
        InitError::Conflicts { paths } => json!({
            "error": {
                "code": "INIT_CONFLICT",
                "message": error.to_string(),
                "paths": paths.iter().map(|path| display_path(path)).collect::<Vec<_>>(),
            }
        }),
        InitError::CreateDir { path, .. } => json!({
            "error": {
                "code": "INIT_CREATE_DIR_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        InitError::WriteFile { path, .. } => json!({
            "error": {
                "code": "INIT_WRITE_FILE_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
    };

    eprintln!("{payload}");
}

fn parse_command(args: Vec<String>) -> Result<Command, CliError> {
    if args.is_empty() {
        return Ok(Command::Help);
    }

    match args[0].as_str() {
        "init" => parse_init_args(&args[1..]).map(Command::Init),
        "-h" | "--help" | "help" => Ok(Command::Help),
        unknown => Err(CliError::Usage(format!("unknown command `{unknown}`"))),
    }
}

fn parse_init_args(args: &[String]) -> Result<InitCommandOptions, CliError> {
    let mut options = InitCommandOptions::default();

    for arg in args {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--json" => options.json = true,
            "--force" => options.force = true,
            "-h" | "--help" => {
                print_init_help();
                std::process::exit(0);
            }
            unknown => {
                return Err(CliError::Usage(format!(
                    "unknown option for `vdoc init`: `{unknown}`"
                )));
            }
        }
    }

    Ok(options)
}

fn print_help() {
    println!(
        "\
vdoc

Usage:
  vdoc init [--dry-run] [--json] [--force]
"
    );
}

fn print_init_help() {
    println!(
        "\
vdoc init

Usage:
  vdoc init [--dry-run] [--json] [--force]
"
    );
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug)]
enum Command {
    Init(InitCommandOptions),
    Help,
}

#[derive(Debug, Default)]
struct InitCommandOptions {
    dry_run: bool,
    json: bool,
    force: bool,
}

#[derive(Debug)]
enum CliError {
    CurrentDir(std::io::Error),
    Init(InitError),
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDir(error) => {
                write!(formatter, "failed to get current directory: {error}")
            }
            Self::Init(error) => error.fmt(formatter),
            Self::Usage(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depends_on_core_crate() {
        assert_eq!(vibe_doc_core::CRATE_NAME, "vibe-doc-core");
    }

    #[test]
    fn parses_init_options() {
        let options = parse_init_args(&[
            "--dry-run".to_string(),
            "--json".to_string(),
            "--force".to_string(),
        ])
        .unwrap();

        assert!(options.dry_run);
        assert!(options.json);
        assert!(options.force);
    }
}
