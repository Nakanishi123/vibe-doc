use crate::args::ShowMode;
use crate::error::CliError;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use vibe_doc_core::{
    AdrStatus, DesignStatus, DocumentMetadata, InitError, InitPlan, NewError, NewPlan, Priority,
    RepositoryDocument, SpecStatus, TaskStatus, TaskType,
};

pub(crate) fn print_init_text(plan: &InitPlan, dry_run: bool) {
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

pub(crate) fn print_init_json(plan: &InitPlan, dry_run: bool, force: bool) {
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

pub(crate) fn print_new_text(plan: &NewPlan, dry_run: bool) {
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

pub(crate) fn print_new_json(cmd: &str, plan: &NewPlan, dry_run: bool, force: bool) {
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

pub(crate) fn print_new_error_json(error: &NewError) {
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

pub(crate) fn print_init_error_json(error: &InitError) {
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

pub(crate) fn document_summary_json(root: &Path, document: &RepositoryDocument) -> Value {
    let common = document.document.metadata.common();
    let mut value = json!({
        "id": common.id.get(),
        "title": common.title,
        "kind": metadata_kind(&document.document.metadata),
        "path": display_path(&relative_path(root, &document.path)),
        "tags": common.tags,
    });

    if let Value::Object(ref mut object) = value {
        match &document.document.metadata {
            DocumentMetadata::Spec(metadata) => {
                if let Some(status) = metadata.status {
                    object.insert("status".to_string(), json!(spec_status_as_str(status)));
                }
            }
            DocumentMetadata::Design(metadata) => {
                if let Some(status) = metadata.status {
                    object.insert("status".to_string(), json!(design_status_as_str(status)));
                }
            }
            DocumentMetadata::Adr(metadata) => {
                object.insert(
                    "status".to_string(),
                    json!(adr_status_as_str(metadata.status)),
                );
            }
            DocumentMetadata::Task(metadata) => {
                object.insert(
                    "status".to_string(),
                    json!(task_status_as_str(metadata.status)),
                );
                object.insert(
                    "type".to_string(),
                    json!(task_type_as_str(metadata.task_type)),
                );
                if let Some(priority) = metadata.priority {
                    object.insert("priority".to_string(), json!(priority_as_str(priority)));
                }
            }
            DocumentMetadata::TaskIndex(_) => {}
        }
    }

    value
}

pub(crate) fn show_json(
    root: &Path,
    document: &RepositoryDocument,
    mode: ShowMode,
) -> Result<Value, CliError> {
    let mut value = document_summary_json(root, document);

    if let Value::Object(ref mut object) = value {
        object.insert("mode".to_string(), json!(mode.as_str()));
        match mode {
            ShowMode::Full => {
                let content =
                    fs::read_to_string(&document.path).map_err(|source| CliError::ReadFile {
                        path: document.path.clone(),
                        source,
                    })?;
                object.insert("content".to_string(), json!(content));
            }
            ShowMode::PathOnly => {}
            ShowMode::FrontmatterOnly => {
                object.insert(
                    "frontmatter".to_string(),
                    json!(document.document.frontmatter.raw),
                );
            }
        }
    }

    Ok(json!({
        "command": "show",
        "document": value,
    }))
}

pub(crate) fn metadata_kind(metadata: &DocumentMetadata) -> &'static str {
    match metadata {
        DocumentMetadata::Spec(_) => "spec",
        DocumentMetadata::Design(_) => "design",
        DocumentMetadata::Adr(_) => "adr",
        DocumentMetadata::Task(_) => "task",
        DocumentMetadata::TaskIndex(_) => "task-index",
    }
}

fn spec_status_as_str(status: SpecStatus) -> &'static str {
    match status {
        SpecStatus::Deprecated => "deprecated",
    }
}

fn design_status_as_str(status: DesignStatus) -> &'static str {
    match status {
        DesignStatus::Deprecated => "deprecated",
    }
}

fn adr_status_as_str(status: AdrStatus) -> &'static str {
    match status {
        AdrStatus::Proposed => "proposed",
        AdrStatus::Accepted => "accepted",
        AdrStatus::Rejected => "rejected",
        AdrStatus::Deprecated => "deprecated",
        AdrStatus::Superseded => "superseded",
    }
}

fn task_status_as_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Doing => "doing",
        TaskStatus::Blocked => "blocked",
        TaskStatus::Done => "done",
        TaskStatus::Dropped => "dropped",
    }
}

fn task_type_as_str(task_type: TaskType) -> &'static str {
    match task_type {
        TaskType::Feature => "feature",
        TaskType::Bug => "bug",
        TaskType::Refactor => "refactor",
        TaskType::Chore => "chore",
        TaskType::Docs => "docs",
        TaskType::Test => "test",
        TaskType::Spike => "spike",
    }
}

fn priority_as_str(priority: Priority) -> &'static str {
    match priority {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Critical => "critical",
    }
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
