use crate::args::ShowMode;
use crate::error::CliError;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use vibe_doc_core::{
    AdrStatus, DesignStatus, DocumentMetadata, InitError, InitPlan, NewError, NewPlan, Priority,
    RepositoryDocument, SpecStatus, TaskContext, TaskContextItem, TaskGuardIssue, TaskGuardReport,
    TaskIndexRebuildError, TaskIndexRebuildPlan, TaskLifecycleError, TaskLifecyclePlan, TaskStatus,
    TaskType, ValidationIssue, ValidationReport,
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

pub(crate) fn print_rebuild_index_text(plan: &TaskIndexRebuildPlan, dry_run: bool) {
    if dry_run {
        println!("vdoc rebuild index dry-run:");
        print!("{}", plan.content);
    } else {
        println!("vdoc rebuild index complete:");
        println!("- {} {}", plan.action.as_str(), display_path(&plan.path));
    }
}

pub(crate) fn print_rebuild_index_json(plan: &TaskIndexRebuildPlan, dry_run: bool) {
    println!(
        "{}",
        json!({
            "command": "rebuild index",
            "dry_run": dry_run,
            "path": display_path(&plan.path),
            "action": plan.action.as_str(),
            "content": plan.content,
        })
    );
}

pub(crate) fn print_rebuild_index_error_json(error: &TaskIndexRebuildError) {
    let payload = match error {
        TaskIndexRebuildError::RepositoryScan(_) => json!({
            "error": {
                "code": "REBUILD_INDEX_SCAN_FAILED",
                "message": error.to_string(),
            }
        }),
        TaskIndexRebuildError::MissingTaskIndex => json!({
            "error": {
                "code": "REBUILD_INDEX_MISSING_TASK_INDEX",
                "message": error.to_string(),
            }
        }),
        TaskIndexRebuildError::ReadFile { path, .. } => json!({
            "error": {
                "code": "REBUILD_INDEX_READ_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        TaskIndexRebuildError::WriteFile { path, .. } => json!({
            "error": {
                "code": "REBUILD_INDEX_WRITE_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
    };
    eprintln!("{payload}");
}

pub(crate) fn print_task_lifecycle_text(command: &str, plan: &TaskLifecyclePlan, dry_run: bool) {
    if dry_run {
        println!("vdoc {command} dry-run:");
    } else {
        println!("vdoc {command} complete:");
    }
    for change in &plan.changes {
        println!(
            "- {} {}",
            change.action.as_str(),
            display_path(&change.path)
        );
    }
}

pub(crate) fn print_task_lifecycle_json(command: &str, plan: &TaskLifecyclePlan, dry_run: bool) {
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
            "command": command,
            "dry_run": dry_run,
            "task_id": plan.task_id.get(),
            "changes": changes,
        })
    );
}

pub(crate) fn print_task_context_text(root: &Path, context: &TaskContext) {
    println!("vdoc context task {}:", context.task_id.get());
    for item in &context.items {
        let title = item.title.as_deref().unwrap_or("");
        println!(
            "- {} {}{}",
            item.kind.as_str(),
            display_path(&relative_path(root, &item.path)),
            if title.is_empty() {
                String::new()
            } else {
                format!(" {title}")
            }
        );
    }
    for item in &context.items {
        println!(
            "\n--- {} {} ---\n{}",
            item.kind.as_str(),
            display_path(&relative_path(root, &item.path)),
            item.content
        );
    }
}

pub(crate) fn print_task_context_json(root: &Path, context: &TaskContext) {
    let items: Vec<_> = context
        .items
        .iter()
        .map(|item| task_context_item_json(root, item))
        .collect();
    println!(
        "{}",
        json!({
            "command": "context task",
            "task_id": context.task_id.get(),
            "items": items,
        })
    );
}

pub(crate) fn print_task_guard_text(root: &Path, report: &TaskGuardReport) {
    if report.ready {
        println!("vdoc guard task {}: ready", report.task_id.get());
        return;
    }

    println!(
        "vdoc guard task {}: {} issue{}",
        report.task_id.get(),
        report.issues.len(),
        if report.issues.len() == 1 { "" } else { "s" }
    );
    for issue in &report.issues {
        match &issue.path {
            Some(path) => println!(
                "- [{}] {}: {}",
                issue.code.as_str(),
                display_path(&relative_path(root, path)),
                issue.message
            ),
            None => println!("- [{}] {}", issue.code.as_str(), issue.message),
        }
    }
}

pub(crate) fn print_task_guard_json(root: &Path, report: &TaskGuardReport) {
    let issues: Vec<_> = report
        .issues
        .iter()
        .map(|issue| task_guard_issue_json(root, issue))
        .collect();
    println!(
        "{}",
        json!({
            "command": "guard task",
            "task_id": report.task_id.get(),
            "ready": report.ready,
            "issue_count": report.issues.len(),
            "issues": issues,
        })
    );
}

fn task_context_item_json(root: &Path, item: &TaskContextItem) -> Value {
    let mut value = json!({
        "kind": item.kind.as_str(),
        "path": display_path(&relative_path(root, &item.path)),
        "content": item.content,
    });
    if let Value::Object(ref mut object) = value {
        if let Some(id) = item.document_id {
            object.insert("id".to_string(), json!(id.get()));
        }
        if let Some(title) = &item.title {
            object.insert("title".to_string(), json!(title));
        }
    }
    value
}

fn task_guard_issue_json(root: &Path, issue: &TaskGuardIssue) -> Value {
    let mut value = json!({
        "code": issue.code.as_str(),
        "message": issue.message,
    });
    if let Value::Object(ref mut object) = value {
        if let Some(id) = issue.document_id {
            object.insert("id".to_string(), json!(id.get()));
        }
        if let Some(path) = &issue.path {
            object.insert(
                "path".to_string(),
                json!(display_path(&relative_path(root, path))),
            );
        }
    }
    value
}

pub(crate) fn print_task_lifecycle_error_json(error: &TaskLifecycleError) {
    let payload = match error {
        TaskLifecycleError::TaskNotFound { id } => json!({
            "error": {
                "code": "TASK_LIFECYCLE_TASK_NOT_FOUND",
                "message": error.to_string(),
                "task_id": id.get(),
            }
        }),
        TaskLifecycleError::InvalidTaskStatus {
            id,
            status,
            expected,
        } => json!({
            "error": {
                "code": "TASK_LIFECYCLE_INVALID_STATUS",
                "message": error.to_string(),
                "task_id": id.get(),
                "status": task_status_as_str(*status),
                "expected": expected,
            }
        }),
        TaskLifecycleError::InvalidTaskLocation { path }
        | TaskLifecycleError::DestinationExists { path }
        | TaskLifecycleError::ReadFile { path, .. }
        | TaskLifecycleError::CreateDir { path, .. }
        | TaskLifecycleError::WriteFile { path, .. }
        | TaskLifecycleError::DeleteFile { path, .. } => json!({
            "error": {
                "code": "TASK_LIFECYCLE_IO_FAILED",
                "message": error.to_string(),
                "path": display_path(path),
            }
        }),
        TaskLifecycleError::RepositoryScan(_) => json!({
            "error": {
                "code": "TASK_LIFECYCLE_SCAN_FAILED",
                "message": error.to_string(),
            }
        }),
        TaskLifecycleError::FrontmatterParse(_)
        | TaskLifecycleError::FrontmatterNotMapping
        | TaskLifecycleError::FrontmatterSerialize(_)
        | TaskLifecycleError::Parse(_) => json!({
            "error": {
                "code": "TASK_LIFECYCLE_PARSE_FAILED",
                "message": error.to_string(),
            }
        }),
        TaskLifecycleError::Validation(issues) => json!({
            "error": {
                "code": "TASK_LIFECYCLE_VALIDATION_FAILED",
                "message": error.to_string(),
                "issues": issues.iter().map(validation_issue_json).collect::<Vec<_>>(),
            }
        }),
        TaskLifecycleError::ValidationRun(_) => json!({
            "error": {
                "code": "TASK_LIFECYCLE_VALIDATION_RUN_FAILED",
                "message": error.to_string(),
            }
        }),
    };
    eprintln!("{payload}");
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

pub(crate) fn print_validation_text(command: &str, report: &ValidationReport) {
    if report.is_valid() {
        println!("vdoc {command}: ok");
        return;
    }

    println!(
        "vdoc {command}: {} issue{}",
        report.issues.len(),
        if report.issues.len() == 1 { "" } else { "s" }
    );
    for issue in &report.issues {
        match &issue.path {
            Some(path) => println!(
                "- [{}] {}: {}",
                issue.code.as_str(),
                display_path(path),
                issue.message
            ),
            None => println!("- [{}] {}", issue.code.as_str(), issue.message),
        }
    }
}

pub(crate) fn print_validation_json(command: &str, report: &ValidationReport) {
    let issues: Vec<_> = report.issues.iter().map(validation_issue_json).collect();
    println!(
        "{}",
        json!({
            "command": command,
            "valid": report.is_valid(),
            "incomplete": report.incomplete,
            "issue_count": report.issues.len(),
            "issues": issues,
        })
    );
}

pub(crate) fn validation_issue_json(issue: &ValidationIssue) -> Value {
    let mut value = json!({
        "code": issue.code.as_str(),
        "message": issue.message,
    });

    if let (Value::Object(ref mut object), Some(path)) = (&mut value, &issue.path) {
        object.insert("path".to_string(), json!(display_path(path)));
    }

    value
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
