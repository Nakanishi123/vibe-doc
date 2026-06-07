//! Command-line entry point for `vdoc`.

use serde_json::json;
use std::env;
use std::path::Path;
use std::process::ExitCode;
use vibe_doc_core::{
    init_repository, new_adr, new_design, new_spec, new_task, AdrStatus, DocumentId, InitError,
    InitOptions, InitPlan, NewAdrOptions, NewError, NewPlan, NewTaskOptions, Priority, TaskType,
};

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
        Command::NewSpec(title, options) => run_new_spec(title, options),
        Command::NewDesign(title, options) => run_new_design(title, options),
        Command::NewAdr(title, adr_opts, options) => run_new_adr(title, adr_opts, options),
        Command::NewTask(title, task_opts, options) => run_new_task(title, task_opts, options),
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

fn run_new_spec(title: String, options: NewCommandOptions) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let opts = vibe_doc_core::NewOptions {
        dry_run: options.dry_run,
        force: options.force,
    };
    match new_spec(&root, &title, opts) {
        Ok(plan) => {
            if options.json {
                print_new_json("new spec", &plan, options.dry_run, options.force);
            } else {
                print_new_text(&plan, options.dry_run);
            }
            Ok(())
        }
        Err(error) => {
            if options.json {
                print_new_error_json(&error);
            }
            Err(CliError::New(error))
        }
    }
}

fn run_new_design(title: String, options: NewCommandOptions) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let opts = vibe_doc_core::NewOptions {
        dry_run: options.dry_run,
        force: options.force,
    };
    match new_design(&root, &title, opts) {
        Ok(plan) => {
            if options.json {
                print_new_json("new design", &plan, options.dry_run, options.force);
            } else {
                print_new_text(&plan, options.dry_run);
            }
            Ok(())
        }
        Err(error) => {
            if options.json {
                print_new_error_json(&error);
            }
            Err(CliError::New(error))
        }
    }
}

fn run_new_adr(
    title: String,
    adr_opts: NewAdrOptions,
    options: NewCommandOptions,
) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let opts = vibe_doc_core::NewOptions {
        dry_run: options.dry_run,
        force: options.force,
    };
    match new_adr(&root, &title, adr_opts, opts) {
        Ok(plan) => {
            if options.json {
                print_new_json("new adr", &plan, options.dry_run, options.force);
            } else {
                print_new_text(&plan, options.dry_run);
            }
            Ok(())
        }
        Err(error) => {
            if options.json {
                print_new_error_json(&error);
            }
            Err(CliError::New(error))
        }
    }
}

fn run_new_task(
    title: String,
    task_opts: NewTaskOptions,
    options: NewCommandOptions,
) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let opts = vibe_doc_core::NewOptions {
        dry_run: options.dry_run,
        force: options.force,
    };
    match new_task(&root, &title, task_opts, opts) {
        Ok(plan) => {
            if options.json {
                print_new_json("new task", &plan, options.dry_run, options.force);
            } else {
                print_new_text(&plan, options.dry_run);
            }
            Ok(())
        }
        Err(error) => {
            if options.json {
                print_new_error_json(&error);
            }
            Err(CliError::New(error))
        }
    }
}

fn print_new_text(plan: &NewPlan, dry_run: bool) {
    if dry_run {
        println!("vdoc new dry-run:");
    } else {
        println!("vdoc new complete:");
    }
    for change in &plan.changes {
        println!(
            "- {} {}",
            change.action.as_str(),
            display_path(&change.path)
        );
    }
}

fn print_new_json(cmd: &str, plan: &NewPlan, dry_run: bool, force: bool) {
    let changes: Vec<_> = plan
        .changes
        .iter()
        .map(|change| {
            json!({
                "path": display_path(&change.path),
                "action": change.action.as_str(),
            })
        })
        .collect();

    println!(
        "{}",
        json!({
            "command": cmd,
            "dry_run": dry_run,
            "force": force,
            "changes": changes,
        })
    );
}

fn print_new_error_json(error: &NewError) {
    let payload = match error {
        NewError::Conflict { path } => json!({
            "error": {
                "code": "NEW_CONFLICT",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        NewError::CreateDir { path, .. } => json!({
            "error": {
                "code": "NEW_CREATE_DIR_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        NewError::WriteFile { path, .. } => json!({
            "error": {
                "code": "NEW_WRITE_FILE_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        NewError::Allocation(_) => json!({
            "error": {
                "code": "NEW_ALLOCATION_FAILED",
                "message": error.to_string(),
            }
        }),
        NewError::Schema(_) => json!({
            "error": {
                "code": "NEW_SCHEMA_LOAD_FAILED",
                "message": error.to_string(),
            }
        }),
        NewError::FrontmatterSerialize(_) => json!({
            "error": {
                "code": "NEW_FRONTMATTER_SERIALIZE_FAILED",
                "message": error.to_string(),
            }
        }),
        NewError::Parse(_) => json!({
            "error": {
                "code": "NEW_PARSE_FAILED",
                "message": error.to_string(),
            }
        }),
        NewError::Validation(issues) => json!({
            "error": {
                "code": "NEW_VALIDATION_FAILED",
                "message": error.to_string(),
                "issues": issues,
            }
        }),
    };
    eprintln!("{payload}");
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
        "new" => parse_new_args(&args[1..]),
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

fn parse_new_args(args: &[String]) -> Result<Command, CliError> {
    if args.is_empty() {
        return Err(CliError::Usage(
            "missing document kind for `vdoc new`".to_string(),
        ));
    }
    match args[0].as_str() {
        "spec" => parse_new_spec_args(&args[1..]),
        "design" => parse_new_design_args(&args[1..]),
        "adr" => parse_new_adr_args(&args[1..]),
        "task" => parse_new_task_args(&args[1..]),
        "-h" | "--help" => {
            print_new_help();
            std::process::exit(0);
        }
        unknown => Err(CliError::Usage(format!(
            "unknown document kind `{unknown}` for `vdoc new`"
        ))),
    }
}

fn parse_document_id(s: &str) -> Result<DocumentId, CliError> {
    let value: u64 = s
        .parse()
        .map_err(|_| CliError::Usage(format!("invalid document ID: {s}")))?;
    DocumentId::new(value)
        .ok_or_else(|| CliError::Usage(format!("document ID must be positive: {s}")))
}

fn parse_new_spec_args(args: &[String]) -> Result<Command, CliError> {
    let (title, options) = parse_new_common_args("spec", args)?;
    Ok(Command::NewSpec(title, options))
}

fn parse_new_design_args(args: &[String]) -> Result<Command, CliError> {
    let (title, options) = parse_new_common_args("design", args)?;
    Ok(Command::NewDesign(title, options))
}

fn parse_new_adr_args(args: &[String]) -> Result<Command, CliError> {
    let mut adr_opts = NewAdrOptions::default();
    let mut title = String::new();
    let mut options = NewCommandOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => options.dry_run = true,
            "--json" => options.json = true,
            "--force" => options.force = true,
            "--status" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --status".to_string()));
                }
                adr_opts.status = Some(match args[i].as_str() {
                    "proposed" => AdrStatus::Proposed,
                    "accepted" => AdrStatus::Accepted,
                    "rejected" => AdrStatus::Rejected,
                    "deprecated" => AdrStatus::Deprecated,
                    "superseded" => AdrStatus::Superseded,
                    unknown => {
                        return Err(CliError::Usage(format!("invalid ADR status `{unknown}`")))
                    }
                });
            }
            "--tag" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --tag".to_string()));
                }
                adr_opts.tags.push(args[i].clone());
            }
            "--related-design" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage(
                        "missing value for --related-design".to_string(),
                    ));
                }
                adr_opts.related_designs.push(parse_document_id(&args[i])?);
            }
            arg if !arg.starts_with('-') => {
                if !title.is_empty() {
                    return Err(CliError::Usage(format!("unexpected argument `{arg}`")));
                }
                title = arg.to_string();
            }
            unknown => return Err(CliError::Usage(format!("unknown option `{unknown}`"))),
        }
        i += 1;
    }
    if title.is_empty() {
        return Err(CliError::Usage(
            "missing title for `vdoc new adr`".to_string(),
        ));
    }
    Ok(Command::NewAdr(title, adr_opts, options))
}

fn parse_new_task_args(args: &[String]) -> Result<Command, CliError> {
    let mut task_opts = NewTaskOptions::default();
    let mut title = String::new();
    let mut options = NewCommandOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => options.dry_run = true,
            "--json" => options.json = true,
            "--force" => options.force = true,
            "--type" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --type".to_string()));
                }
                task_opts.task_type = Some(match args[i].as_str() {
                    "feature" => TaskType::Feature,
                    "bug" => TaskType::Bug,
                    "refactor" => TaskType::Refactor,
                    "chore" => TaskType::Chore,
                    "docs" => TaskType::Docs,
                    "test" => TaskType::Test,
                    "spike" => TaskType::Spike,
                    unknown => {
                        return Err(CliError::Usage(format!("invalid task type `{unknown}`")))
                    }
                });
            }
            "--priority" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --priority".to_string()));
                }
                task_opts.priority = Some(match args[i].as_str() {
                    "low" => Priority::Low,
                    "medium" => Priority::Medium,
                    "high" => Priority::High,
                    "critical" => Priority::Critical,
                    unknown => {
                        return Err(CliError::Usage(format!("invalid priority `{unknown}`")))
                    }
                });
            }
            "--spec" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --spec".to_string()));
                }
                task_opts.specs.push(parse_document_id(&args[i])?);
            }
            "--design" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --design".to_string()));
                }
                task_opts.designs.push(parse_document_id(&args[i])?);
            }
            "--adr" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage("missing value for --adr".to_string()));
                }
                task_opts.adrs.push(parse_document_id(&args[i])?);
            }
            "--depends-on" => {
                i += 1;
                if i >= args.len() {
                    return Err(CliError::Usage(
                        "missing value for --depends-on".to_string(),
                    ));
                }
                task_opts.depends_on.push(parse_document_id(&args[i])?);
            }
            arg if !arg.starts_with('-') => {
                if !title.is_empty() {
                    return Err(CliError::Usage(format!("unexpected argument `{arg}`")));
                }
                title = arg.to_string();
            }
            unknown => return Err(CliError::Usage(format!("unknown option `{unknown}`"))),
        }
        i += 1;
    }
    if title.is_empty() {
        return Err(CliError::Usage(
            "missing title for `vdoc new task`".to_string(),
        ));
    }
    Ok(Command::NewTask(title, task_opts, options))
}

fn parse_new_common_args(
    kind: &str,
    args: &[String],
) -> Result<(String, NewCommandOptions), CliError> {
    let mut title = String::new();
    let mut options = NewCommandOptions::default();
    for arg in args {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--json" => options.json = true,
            "--force" => options.force = true,
            arg if !arg.starts_with('-') => {
                if !title.is_empty() {
                    return Err(CliError::Usage(format!("unexpected argument `{arg}`")));
                }
                title = arg.to_string();
            }
            unknown => return Err(CliError::Usage(format!("unknown option `{unknown}`"))),
        }
    }
    if title.is_empty() {
        return Err(CliError::Usage(format!(
            "missing title for `vdoc new {kind}`"
        )));
    }
    Ok((title, options))
}

fn print_help() {
    println!(
        "\
vdoc

Usage:
  vdoc

Usage:
  vdoc init [--dry-run] [--json] [--force]
  vdoc new <kind> <title> [options...]
"
    );
}

fn print_new_help() {
    println!(
        "\
vdoc new

Usage:
  vdoc new spec <title> [--dry-run] [--json] [--force]
  vdoc new design <title> [--dry-run] [--json] [--force]
  vdoc new adr <title> [--dry-run] [--json] [--force] [--status <status>] [--tag <tag>...] [--related-design <id>...]
  vdoc new task <title> [--dry-run] [--json] [--force] [--type <type>] [--priority <priority>] [--spec <id>...] [--design <id>...] [--adr <id>...] [--depends-on <id>...]
"
    );
}
fn print_init_help() {
    println!(
        "\
vdoc init

Usage:
  vdoc

Usage:
  vdoc init [--dry-run] [--json] [--force]
  vdoc new <kind> <title> [options...]
"
    );
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug)]
enum Command {
    Init(InitCommandOptions),
    NewSpec(String, NewCommandOptions),
    NewDesign(String, NewCommandOptions),
    NewAdr(String, NewAdrOptions, NewCommandOptions),
    NewTask(String, NewTaskOptions, NewCommandOptions),
    Help,
}

#[derive(Debug, Default)]
struct NewCommandOptions {
    dry_run: bool,
    json: bool,
    force: bool,
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
    New(NewError),
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDir(error) => {
                write!(formatter, "failed to get current directory: {error}")
            }
            Self::Init(error) => error.fmt(formatter),
            Self::New(error) => error.fmt(formatter),
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
