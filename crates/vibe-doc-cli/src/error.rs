use crate::args::ShowKindArg;
use serde_json::{json, Value};
use std::path::PathBuf;
use vibe_doc_core::{
    DocumentId, InitError, NewError, RepositoryScanError, TaskIndexRebuildError,
    TaskLifecycleError, ValidationRunError,
};

#[derive(Debug)]
pub(crate) enum CliError {
    CurrentDir(std::io::Error),
    WriteHelp(std::io::Error),
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    Init(InitError),
    New(NewError),
    RebuildIndex(TaskIndexRebuildError),
    TaskLifecycle {
        json: bool,
        error: TaskLifecycleError,
    },
    Scan(RepositoryScanError),
    ValidationRun {
        command: &'static str,
        json: bool,
        error: ValidationRunError,
    },
    ReportedIssues,
    DocumentNotFound {
        id: DocumentId,
        kind: Option<ShowKindArg>,
        json: bool,
    },
    Usage(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDir(error) => {
                write!(formatter, "failed to get current directory: {error}")
            }
            Self::WriteHelp(error) => write!(formatter, "failed to write help: {error}"),
            Self::ReadFile { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::Init(error) => error.fmt(formatter),
            Self::New(error) => error.fmt(formatter),
            Self::RebuildIndex(error) => error.fmt(formatter),
            Self::TaskLifecycle { json, error } => {
                if *json {
                    crate::format::print_task_lifecycle_error_json(error);
                }
                error.fmt(formatter)
            }
            Self::Scan(error) => error.fmt(formatter),
            Self::ValidationRun {
                command,
                json,
                error,
            } => {
                if *json {
                    print_validation_run_error_json(command, error);
                }
                error.fmt(formatter)
            }
            Self::ReportedIssues => formatter.write_str("validation issues reported"),
            Self::DocumentNotFound { id, kind, json } => {
                if *json {
                    print_not_found_error_json(*id, *kind);
                }
                formatter.write_str(&missing_document_message(*id, *kind))
            }
            Self::Usage(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CliError {}

fn print_not_found_error_json(id: DocumentId, kind: Option<ShowKindArg>) {
    let mut error = json!({
        "code": "DOCUMENT_NOT_FOUND",
        "message": missing_document_message(id, kind),
        "id": id.get(),
    });
    if let (Value::Object(ref mut object), Some(kind)) = (&mut error, kind) {
        object.insert("kind".to_string(), json!(kind.as_str()));
    }
    eprintln!("{}", json!({ "error": error }));
}

fn missing_document_message(id: DocumentId, kind: Option<ShowKindArg>) -> String {
    if let Some(kind) = kind {
        format!(
            "document {} with kind `{}` was not found",
            id.get(),
            kind.as_str()
        )
    } else {
        format!("document {} was not found", id.get())
    }
}

fn print_validation_run_error_json(command: &str, error: &ValidationRunError) {
    eprintln!(
        "{}",
        json!({
            "error": {
                "code": "VALIDATION_RUN_FAILED",
                "command": command,
                "message": error.to_string(),
            }
        })
    );
}
