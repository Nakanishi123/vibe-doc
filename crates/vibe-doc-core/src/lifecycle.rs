use crate::task_index::generate_task_index_markdown;
use crate::{
    expected_kind_for_relative_path, parse_numbered_document, scan_repository, validate_repository,
    DocumentId, DocumentMetadata, RepositoryDocument, SourceId, TaskStatus, ValidationIssue,
};
use serde_yaml::{Mapping, Value};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Options for task lifecycle mutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLifecycleOptions {
    pub dry_run: bool,
    pub date: String,
}

/// Options for completing a task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteTaskOptions {
    pub lifecycle: TaskLifecycleOptions,
    pub result: Option<String>,
}

/// A planned or applied task lifecycle mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLifecyclePlan {
    pub task_id: DocumentId,
    pub changes: Vec<TaskLifecycleChange>,
}

/// One planned or applied task lifecycle filesystem change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLifecycleChange {
    pub path: PathBuf,
    pub action: TaskLifecycleAction,
}

/// The action planned or performed for a task lifecycle change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycleAction {
    Overwrite,
    Create,
    Delete,
    Keep,
}

impl TaskLifecycleAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::Create => "create",
            Self::Delete => "delete",
            Self::Keep => "keep",
        }
    }
}

/// Error produced while planning or applying a task lifecycle mutation.
#[derive(Debug)]
pub enum TaskLifecycleError {
    RepositoryScan(crate::RepositoryScanError),
    TaskNotFound {
        id: DocumentId,
    },
    InvalidTaskStatus {
        id: DocumentId,
        status: TaskStatus,
        expected: &'static str,
    },
    InvalidTaskLocation {
        path: PathBuf,
    },
    DestinationExists {
        path: PathBuf,
    },
    ReadFile {
        path: PathBuf,
        source: io::Error,
    },
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    WriteFile {
        path: PathBuf,
        source: io::Error,
    },
    DeleteFile {
        path: PathBuf,
        source: io::Error,
    },
    FrontmatterParse(serde_yaml::Error),
    FrontmatterNotMapping,
    FrontmatterSerialize(serde_yaml::Error),
    Parse(crate::ParseError),
    Validation(Vec<ValidationIssue>),
    ValidationRun(crate::ValidationRunError),
}

impl fmt::Display for TaskLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepositoryScan(error) => error.fmt(formatter),
            Self::TaskNotFound { id } => write!(formatter, "task {} was not found", id.get()),
            Self::InvalidTaskStatus {
                id,
                status,
                expected,
            } => write!(
                formatter,
                "task {} has status {}; expected {expected}",
                id.get(),
                task_status_str(*status)
            ),
            Self::InvalidTaskLocation { path } => {
                write!(
                    formatter,
                    "task is not in a valid lifecycle path: {}",
                    path.display()
                )
            }
            Self::DestinationExists { path } => {
                write!(
                    formatter,
                    "destination task file already exists: {}",
                    path.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
            Self::CreateDir { path, source } => {
                write!(
                    formatter,
                    "failed to create directory {}: {source}",
                    path.display()
                )
            }
            Self::WriteFile { path, source } => {
                write!(formatter, "failed to write {}: {source}", path.display())
            }
            Self::DeleteFile { path, source } => {
                write!(formatter, "failed to delete {}: {source}", path.display())
            }
            Self::FrontmatterParse(error) => {
                write!(formatter, "failed to parse task frontmatter: {error}")
            }
            Self::FrontmatterNotMapping => {
                formatter.write_str("task frontmatter must be a mapping")
            }
            Self::FrontmatterSerialize(error) => {
                write!(formatter, "failed to serialize task frontmatter: {error}")
            }
            Self::Parse(error) => write!(formatter, "updated task document is invalid: {error}"),
            Self::Validation(issues) => {
                writeln!(formatter, "task lifecycle mutation failed validation:")?;
                for issue in issues {
                    writeln!(formatter, "- {}", issue.message)?;
                }
                Ok(())
            }
            Self::ValidationRun(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for TaskLifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RepositoryScan(error) => Some(error),
            Self::ReadFile { source, .. }
            | Self::CreateDir { source, .. }
            | Self::WriteFile { source, .. }
            | Self::DeleteFile { source, .. } => Some(source),
            Self::FrontmatterParse(error) | Self::FrontmatterSerialize(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::ValidationRun(error) => Some(error),
            Self::TaskNotFound { .. }
            | Self::InvalidTaskStatus { .. }
            | Self::InvalidTaskLocation { .. }
            | Self::DestinationExists { .. }
            | Self::FrontmatterNotMapping
            | Self::Validation(_) => None,
        }
    }
}

impl From<crate::RepositoryScanError> for TaskLifecycleError {
    fn from(error: crate::RepositoryScanError) -> Self {
        Self::RepositoryScan(error)
    }
}

impl From<crate::ParseError> for TaskLifecycleError {
    fn from(error: crate::ParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<crate::ValidationRunError> for TaskLifecycleError {
    fn from(error: crate::ValidationRunError) -> Self {
        Self::ValidationRun(error)
    }
}

/// Start a planned or blocked task by setting status to `doing`.
pub fn start_task(
    root: impl AsRef<Path>,
    id: DocumentId,
    options: TaskLifecycleOptions,
) -> Result<TaskLifecyclePlan, TaskLifecycleError> {
    let root = root.as_ref();
    let documents = scan_repository(root)?;
    let task = find_task(&documents, id)?;
    let task_path = relative_path(root, &task.path);
    let task_full_path = task.path.clone();
    let DocumentMetadata::Task(metadata) = &task.document.metadata else {
        unreachable!("find_task only returns task documents");
    };

    if !matches!(metadata.status, TaskStatus::Planned | TaskStatus::Blocked) {
        return Err(TaskLifecycleError::InvalidTaskStatus {
            id,
            status: metadata.status,
            expected: "planned or blocked",
        });
    }
    if !task_path.starts_with("docs/tasks/active") {
        return Err(TaskLifecycleError::InvalidTaskLocation { path: task_path });
    }

    let raw =
        fs::read_to_string(&task_full_path).map_err(|source| TaskLifecycleError::ReadFile {
            path: task_path.clone(),
            source,
        })?;
    let updated = update_task_markdown(&raw, "doing", Some(("started_at", &options.date)), None)?;
    let mut updated_documents = replace_document(root, documents, &task_path, updated.clone())?;
    let index_change = rebuild_index_change(root, &mut updated_documents, options.dry_run)?;

    if !options.dry_run {
        fs::write(&task_full_path, updated).map_err(|source| TaskLifecycleError::WriteFile {
            path: task_path.clone(),
            source,
        })?;
        write_task_index(root, &updated_documents)?;
        ensure_valid(root)?;
    }

    Ok(TaskLifecyclePlan {
        task_id: id,
        changes: with_index_change(
            vec![TaskLifecycleChange {
                path: task_path,
                action: TaskLifecycleAction::Overwrite,
            }],
            index_change,
        ),
    })
}

/// Complete a doing task by moving it to `docs/tasks/done/`.
pub fn complete_task(
    root: impl AsRef<Path>,
    id: DocumentId,
    options: CompleteTaskOptions,
) -> Result<TaskLifecyclePlan, TaskLifecycleError> {
    let root = root.as_ref();
    let documents = scan_repository(root)?;
    let task = find_task(&documents, id)?;
    let source_path = relative_path(root, &task.path);
    let source_full_path = task.path.clone();
    let DocumentMetadata::Task(metadata) = &task.document.metadata else {
        unreachable!("find_task only returns task documents");
    };

    if metadata.status != TaskStatus::Doing {
        return Err(TaskLifecycleError::InvalidTaskStatus {
            id,
            status: metadata.status,
            expected: "doing",
        });
    }
    if !source_path.starts_with("docs/tasks/active") {
        return Err(TaskLifecycleError::InvalidTaskLocation { path: source_path });
    }

    let file_name =
        source_path
            .file_name()
            .ok_or_else(|| TaskLifecycleError::InvalidTaskLocation {
                path: source_path.clone(),
            })?;
    let destination_path = PathBuf::from("docs/tasks/done").join(file_name);
    let destination_full_path = root.join(&destination_path);
    if destination_full_path.exists() {
        return Err(TaskLifecycleError::DestinationExists {
            path: destination_path,
        });
    }

    let raw =
        fs::read_to_string(&source_full_path).map_err(|source| TaskLifecycleError::ReadFile {
            path: source_path.clone(),
            source,
        })?;
    let updated = update_task_markdown(
        &raw,
        "done",
        Some(("completed_at", &options.lifecycle.date)),
        options.result.as_deref(),
    )?;
    let mut updated_documents = move_document(
        root,
        documents,
        &source_path,
        &destination_path,
        updated.clone(),
    )?;
    let index_change =
        rebuild_index_change(root, &mut updated_documents, options.lifecycle.dry_run)?;

    if !options.lifecycle.dry_run {
        if let Some(parent) = destination_full_path.parent() {
            fs::create_dir_all(parent).map_err(|source| TaskLifecycleError::CreateDir {
                path: PathBuf::from("docs/tasks/done"),
                source,
            })?;
        }
        fs::write(&destination_full_path, updated).map_err(|source| {
            TaskLifecycleError::WriteFile {
                path: destination_path.clone(),
                source,
            }
        })?;
        fs::remove_file(&source_full_path).map_err(|source| TaskLifecycleError::DeleteFile {
            path: source_path.clone(),
            source,
        })?;
        write_task_index(root, &updated_documents)?;
        ensure_valid(root)?;
    }

    Ok(TaskLifecyclePlan {
        task_id: id,
        changes: with_index_change(
            vec![
                TaskLifecycleChange {
                    path: destination_path,
                    action: TaskLifecycleAction::Create,
                },
                TaskLifecycleChange {
                    path: source_path,
                    action: TaskLifecycleAction::Delete,
                },
            ],
            index_change,
        ),
    })
}

fn find_task(
    documents: &[RepositoryDocument],
    id: DocumentId,
) -> Result<&RepositoryDocument, TaskLifecycleError> {
    documents
        .iter()
        .find(|document| {
            document.document.metadata.common().id == id
                && matches!(document.document.metadata, DocumentMetadata::Task(_))
        })
        .ok_or(TaskLifecycleError::TaskNotFound { id })
}

fn update_task_markdown(
    markdown: &str,
    status: &'static str,
    date_field: Option<(&'static str, &str)>,
    result: Option<&str>,
) -> Result<String, TaskLifecycleError> {
    let document = parse_numbered_document("task.md", markdown)?;
    let mut frontmatter: Value = serde_yaml::from_str(&document.frontmatter.raw)
        .map_err(TaskLifecycleError::FrontmatterParse)?;
    let mapping = frontmatter
        .as_mapping_mut()
        .ok_or(TaskLifecycleError::FrontmatterNotMapping)?;
    set_mapping_string(mapping, "status", status);
    if let Some((field, value)) = date_field {
        set_mapping_string(mapping, field, value);
    }

    let frontmatter =
        serde_yaml::to_string(&frontmatter).map_err(TaskLifecycleError::FrontmatterSerialize)?;
    let body = result
        .map(|result| replace_result_section(&document.body, result))
        .unwrap_or(document.body);
    Ok(format!("---\n{frontmatter}---\n{body}"))
}

fn set_mapping_string(mapping: &mut Mapping, key: &'static str, value: &str) {
    mapping.insert(
        Value::String(key.to_string()),
        Value::String(value.to_string()),
    );
}

fn replace_result_section(body: &str, result: &str) -> String {
    let Some(start) = body.find("## Result") else {
        let mut updated = body.to_owned();
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str("\n## Result\n\n");
        updated.push_str(result.trim());
        updated.push('\n');
        return updated;
    };

    let after_heading = start + "## Result".len();
    let content_start = if body[after_heading..].starts_with("\r\n") {
        after_heading + 2
    } else if body[after_heading..].starts_with('\n') {
        after_heading + 1
    } else {
        after_heading
    };
    let next_section = body[content_start..]
        .find("\n## ")
        .map(|offset| content_start + offset)
        .unwrap_or(body.len());

    let mut updated = String::new();
    updated.push_str(&body[..content_start]);
    updated.push('\n');
    updated.push_str(result.trim());
    updated.push('\n');
    updated.push_str(&body[next_section..]);
    updated
}

fn replace_document(
    root: &Path,
    mut documents: Vec<RepositoryDocument>,
    path: &Path,
    markdown: String,
) -> Result<Vec<RepositoryDocument>, TaskLifecycleError> {
    let full_path = root.join(path);
    let index = documents
        .iter()
        .position(|document| relative_path(root, &document.path) == path)
        .expect("document selected from scanned documents");
    let expected_kind =
        expected_kind_for_relative_path(path).expect("existing task path is supported");
    let document = parse_numbered_document(SourceId::from(path), &markdown)?;
    documents[index] = RepositoryDocument {
        path: full_path,
        expected_kind,
        document,
    };
    Ok(documents)
}

fn move_document(
    root: &Path,
    mut documents: Vec<RepositoryDocument>,
    source_path: &Path,
    destination_path: &Path,
    markdown: String,
) -> Result<Vec<RepositoryDocument>, TaskLifecycleError> {
    let index = documents
        .iter()
        .position(|document| relative_path(root, &document.path) == source_path)
        .expect("document selected from scanned documents");
    let expected_kind =
        expected_kind_for_relative_path(destination_path).expect("done task path is supported");
    let document = parse_numbered_document(SourceId::from(destination_path), &markdown)?;
    documents[index] = RepositoryDocument {
        path: root.join(destination_path),
        expected_kind,
        document,
    };
    Ok(documents)
}

fn rebuild_index_change(
    root: &Path,
    documents: &mut [RepositoryDocument],
    dry_run: bool,
) -> Result<Option<TaskLifecycleChange>, TaskLifecycleError> {
    let Some(content) = generate_task_index_markdown(documents) else {
        return Ok(None);
    };
    let path = PathBuf::from("docs/tasks/index.md");
    let current =
        fs::read_to_string(root.join(&path)).map_err(|source| TaskLifecycleError::ReadFile {
            path: path.clone(),
            source,
        })?;
    let action = if current == content {
        TaskLifecycleAction::Keep
    } else {
        TaskLifecycleAction::Overwrite
    };
    if dry_run || action == TaskLifecycleAction::Keep {
        return Ok(Some(TaskLifecycleChange { path, action }));
    }
    Ok(Some(TaskLifecycleChange { path, action }))
}

fn write_task_index(
    root: &Path,
    documents: &[RepositoryDocument],
) -> Result<(), TaskLifecycleError> {
    let Some(content) = generate_task_index_markdown(documents) else {
        return Ok(());
    };
    let path = PathBuf::from("docs/tasks/index.md");
    fs::write(root.join(&path), content)
        .map_err(|source| TaskLifecycleError::WriteFile { path, source })
}

fn ensure_valid(root: &Path) -> Result<(), TaskLifecycleError> {
    let report = validate_repository(root)?;
    if report.is_valid() {
        Ok(())
    } else {
        Err(TaskLifecycleError::Validation(report.issues))
    }
}

fn with_index_change(
    mut changes: Vec<TaskLifecycleChange>,
    index_change: Option<TaskLifecycleChange>,
) -> Vec<TaskLifecycleChange> {
    if let Some(change) = index_change {
        changes.push(change);
    }
    changes
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn task_status_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Doing => "doing",
        TaskStatus::Blocked => "blocked",
        TaskStatus::Done => "done",
        TaskStatus::Dropped => "dropped",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_result_section() {
        let body = "\n## Goal\n\nDo work.\n\n## Result\n\nNot implemented.\n";

        let updated = replace_result_section(body, "Implemented work.");

        assert_eq!(
            updated,
            "\n## Goal\n\nDo work.\n\n## Result\n\nImplemented work.\n"
        );
    }
}
