use crate::args::{
    Cli, Command, InitCommandOptions, ListCommand, NewCommand, NewCommandOptions, NewKindCommand,
    ShowCommand, ShowMode, ValidationCommand,
};
use crate::error::CliError;
use crate::format::{
    display_path, document_summary_json, metadata_kind, print_init_error_json, print_init_json,
    print_init_text, print_new_error_json, print_new_json, print_new_text, print_validation_json,
    print_validation_text, relative_path, show_json,
};
use clap::CommandFactory;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use vibe_doc_core::{
    check_repository, init_repository, new_adr, new_design, new_spec, new_task, scan_repository,
    validate_repository, DocumentId, DocumentMetadata, InitOptions, NewAdrOptions, NewTaskOptions,
    RepositoryDocument, ValidationReport,
};

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Some(Command::Init(options)) => run_init(options),
        Some(Command::New(command)) => run_new(command),
        Some(Command::List(command)) => run_list(command),
        Some(Command::Show(command)) => run_show(command),
        Some(Command::Validate(command)) => {
            run_validation("validate", command, |root| validate_repository(root))
        }
        Some(Command::Check(command)) => {
            run_validation("check", command, |root| check_repository(root))
        }
        None => {
            Cli::command().print_help().map_err(CliError::WriteHelp)?;
            println!();
            Ok(())
        }
    }
}

fn run_validation<F>(
    command_name: &'static str,
    command: ValidationCommand,
    run: F,
) -> Result<(), CliError>
where
    F: FnOnce(&Path) -> Result<ValidationReport, vibe_doc_core::ValidationRunError>,
{
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let mut report = run(&root).map_err(|error| CliError::ValidationRun {
        command: command_name,
        json: command.json,
        error,
    })?;
    filter_validation_report(&root, &command.paths, &mut report);

    if command.json {
        print_validation_json(command_name, &report);
    } else {
        print_validation_text(command_name, &report);
    }

    if report.is_valid() {
        Ok(())
    } else {
        Err(CliError::ReportedIssues)
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
                print_init_error_json(&error);
            }
            Err(CliError::Init(error))
        }
    }
}

fn run_new(command: NewCommand) -> Result<(), CliError> {
    match command.kind {
        NewKindCommand::Spec { title } => run_new_spec(title, command.options),
        NewKindCommand::Design { title } => run_new_design(title, command.options),
        NewKindCommand::Adr {
            title,
            status,
            tag,
            related_design,
        } => {
            let adr_opts = NewAdrOptions {
                status: status.map(|status| status.into_core()),
                tags: tag,
                related_designs: related_design
                    .into_iter()
                    .map(document_id_from_u64)
                    .collect::<Result<_, _>>()?,
            };
            run_new_adr(title, adr_opts, command.options)
        }
        NewKindCommand::Task {
            title,
            task_type,
            priority,
            spec,
            design,
            adr,
            depends_on,
        } => {
            let task_opts = NewTaskOptions {
                task_type: task_type.map(|task_type| task_type.into_core()),
                priority: priority.map(|priority| priority.into_core()),
                specs: spec
                    .into_iter()
                    .map(document_id_from_u64)
                    .collect::<Result<_, _>>()?,
                designs: design
                    .into_iter()
                    .map(document_id_from_u64)
                    .collect::<Result<_, _>>()?,
                adrs: adr
                    .into_iter()
                    .map(document_id_from_u64)
                    .collect::<Result<_, _>>()?,
                depends_on: depends_on
                    .into_iter()
                    .map(document_id_from_u64)
                    .collect::<Result<_, _>>()?,
            };
            run_new_task(title, task_opts, command.options)
        }
    }
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

fn run_list(command: ListCommand) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let mut documents = scan_repository(&root).map_err(CliError::Scan)?;
    documents.retain(|document| list_filter_matches(&command, document));
    documents.sort_by_key(|document| document.document.metadata.common().id);

    if command.json {
        let documents: Vec<_> = documents
            .iter()
            .map(|document| document_summary_json(&root, document))
            .collect();
        println!(
            "{}",
            json!({
                "command": format!("list {}", command.kind.as_str()),
                "documents": documents,
            })
        );
    } else {
        for document in documents {
            let common = document.document.metadata.common();
            println!(
                "{}\t{}\t{}\t{}",
                common.id.get(),
                metadata_kind(&document.document.metadata),
                display_path(&relative_path(&root, &document.path)),
                common.title
            );
        }
    }

    Ok(())
}

fn run_show(command: ShowCommand) -> Result<(), CliError> {
    let root = env::current_dir().map_err(CliError::CurrentDir)?;
    let (kind, raw_id) = command.target()?;
    let id = document_id_from_u64(raw_id)?;
    let documents = scan_repository(&root).map_err(CliError::Scan)?;
    let document = documents
        .iter()
        .find(|document| {
            document.document.metadata.common().id == id
                && kind
                    .map(|kind| kind.matches_document(&document.document.metadata))
                    .unwrap_or(true)
        })
        .ok_or(CliError::DocumentNotFound {
            id,
            kind,
            json: command.json,
        })?;

    if command.json {
        println!("{}", show_json(&root, document, command.mode())?);
        return Ok(());
    }

    match command.mode() {
        ShowMode::Full => {
            print!(
                "{}",
                fs::read_to_string(&document.path).map_err(|source| CliError::ReadFile {
                    path: document.path.clone(),
                    source,
                })?
            );
        }
        ShowMode::PathOnly => println!("{}", display_path(&relative_path(&root, &document.path))),
        ShowMode::FrontmatterOnly => print!("{}", document.document.frontmatter.raw),
    }

    Ok(())
}

fn list_filter_matches(command: &ListCommand, document: &RepositoryDocument) -> bool {
    if !command.kind.matches_document(&document.document.metadata) {
        return false;
    }

    let common = document.document.metadata.common();
    if let Some(tag) = &command.tag {
        if !common.tags.iter().any(|value| value == tag) {
            return false;
        }
    }

    match &document.document.metadata {
        DocumentMetadata::Adr(metadata) => command
            .status
            .map(|status| status.matches_adr(metadata.status))
            .unwrap_or(true),
        DocumentMetadata::Task(metadata) => {
            command
                .status
                .map(|status| status.matches_task(metadata.status))
                .unwrap_or(true)
                && command
                    .task_type
                    .map(|task_type| task_type.into_core() == metadata.task_type)
                    .unwrap_or(true)
                && command
                    .priority
                    .map(|priority| Some(priority.into_core()) == metadata.priority)
                    .unwrap_or(true)
        }
        _ => command.status.is_none() && command.task_type.is_none() && command.priority.is_none(),
    }
}

fn filter_validation_report(root: &Path, paths: &[PathBuf], report: &mut ValidationReport) {
    if paths.is_empty() {
        return;
    }

    let original_issues = report.issues.clone();
    let filters: Vec<_> = paths
        .iter()
        .map(|path| normalize_filter_path(root, path))
        .collect();

    report.issues.retain(|issue| {
        issue.path.as_ref().is_some_and(|path| {
            filters
                .iter()
                .any(|filter| path == filter || path.starts_with(filter))
        })
    });

    if report.incomplete && report.issues.is_empty() {
        report.issues = original_issues;
    }
}

fn normalize_filter_path(root: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    absolute
        .strip_prefix(root)
        .unwrap_or(path)
        .components()
        .collect()
}

fn document_id_from_u64(value: u64) -> Result<DocumentId, CliError> {
    DocumentId::new(value)
        .ok_or_else(|| CliError::Usage(format!("document ID must be positive: {value}")))
}
