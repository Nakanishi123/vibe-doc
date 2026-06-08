use crate::{
    scan_repository, AdrStatus, DocumentId, DocumentMetadata, RepositoryDocument,
    RepositoryScanError, TaskStatus,
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskContext {
    pub task_id: DocumentId,
    pub items: Vec<TaskContextItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskContextItem {
    pub document_id: Option<DocumentId>,
    pub kind: TaskContextItemKind,
    pub title: Option<String>,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskContextItemKind {
    Task,
    Spec,
    Design,
    Adr,
}

impl TaskContextItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Spec => "spec",
            Self::Design => "design",
            Self::Adr => "adr",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGuardReport {
    pub task_id: DocumentId,
    pub ready: bool,
    pub issues: Vec<TaskGuardIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGuardIssue {
    pub code: TaskGuardCode,
    pub message: String,
    pub document_id: Option<DocumentId>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskGuardCode {
    TaskNotFound,
    TaskNotActive,
    InvalidTaskStatus,
    MissingDependency,
    IncompleteDependency,
    MissingRelatedDocument,
    InvalidRelatedAdrStatus,
}

impl TaskGuardCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TaskNotFound => "TASK_NOT_FOUND",
            Self::TaskNotActive => "TASK_NOT_ACTIVE",
            Self::InvalidTaskStatus => "INVALID_TASK_STATUS",
            Self::MissingDependency => "MISSING_DEPENDENCY",
            Self::IncompleteDependency => "INCOMPLETE_DEPENDENCY",
            Self::MissingRelatedDocument => "MISSING_RELATED_DOCUMENT",
            Self::InvalidRelatedAdrStatus => "INVALID_RELATED_ADR_STATUS",
        }
    }
}

#[derive(Debug, Error)]
pub enum TaskContextError {
    #[error("task {} was not found", id.get())]
    TaskNotFound { id: DocumentId },
    #[error("related {} {} was not found", kind.as_str(), id.get())]
    MissingRelatedDocument {
        id: DocumentId,
        kind: TaskContextItemKind,
    },
    #[error(transparent)]
    RepositoryScan(#[from] RepositoryScanError),
    #[error("failed to read {}: {source}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub fn task_context(root: &Path, task_id: DocumentId) -> Result<TaskContext, TaskContextError> {
    let documents = scan_repository(root).map_err(TaskContextError::RepositoryScan)?;
    let task =
        find_task(&documents, task_id).ok_or(TaskContextError::TaskNotFound { id: task_id })?;
    let DocumentMetadata::Task(task_metadata) = &task.document.metadata else {
        unreachable!("find_task only returns tasks");
    };

    let mut items = Vec::new();
    items.push(context_item(task, TaskContextItemKind::Task)?);
    append_related(
        &mut items,
        &documents,
        &task_metadata.specs,
        TaskContextItemKind::Spec,
    )?;
    append_related(
        &mut items,
        &documents,
        &task_metadata.designs,
        TaskContextItemKind::Design,
    )?;
    append_related(
        &mut items,
        &documents,
        &task_metadata.adrs,
        TaskContextItemKind::Adr,
    )?;

    Ok(TaskContext { task_id, items })
}

pub fn guard_task(root: &Path, task_id: DocumentId) -> Result<TaskGuardReport, TaskContextError> {
    let documents = scan_repository(root).map_err(TaskContextError::RepositoryScan)?;
    let mut issues = Vec::new();
    let Some(task) = find_task(&documents, task_id) else {
        issues.push(TaskGuardIssue {
            code: TaskGuardCode::TaskNotFound,
            message: format!("task {} was not found", task_id.get()),
            document_id: Some(task_id),
            path: None,
        });
        return Ok(TaskGuardReport {
            task_id,
            ready: false,
            issues,
        });
    };

    let DocumentMetadata::Task(task_metadata) = &task.document.metadata else {
        unreachable!("find_task only returns tasks");
    };

    if !is_active_task_path(root, &task.path) {
        issues.push(TaskGuardIssue {
            code: TaskGuardCode::TaskNotActive,
            message: format!("task {} is not in docs/tasks/active", task_id.get()),
            document_id: Some(task_id),
            path: Some(task.path.clone()),
        });
    }

    if !matches!(
        task_metadata.status,
        TaskStatus::Planned | TaskStatus::Doing
    ) {
        issues.push(TaskGuardIssue {
            code: TaskGuardCode::InvalidTaskStatus,
            message: format!(
                "task {} has status `{}`; expected planned or doing",
                task_id.get(),
                task_status_str(task_metadata.status)
            ),
            document_id: Some(task_id),
            path: Some(task.path.clone()),
        });
    }

    for dependency_id in sorted_unique_ids(&task_metadata.depends_on) {
        match find_task(&documents, dependency_id) {
            Some(dependency) => {
                let DocumentMetadata::Task(metadata) = &dependency.document.metadata else {
                    unreachable!("find_task only returns tasks");
                };
                if metadata.status != TaskStatus::Done {
                    issues.push(TaskGuardIssue {
                        code: TaskGuardCode::IncompleteDependency,
                        message: format!(
                            "dependency task {} has status `{}`; expected done",
                            dependency_id.get(),
                            task_status_str(metadata.status)
                        ),
                        document_id: Some(dependency_id),
                        path: Some(dependency.path.clone()),
                    });
                }
            }
            None => issues.push(TaskGuardIssue {
                code: TaskGuardCode::MissingDependency,
                message: format!("dependency task {} was not found", dependency_id.get()),
                document_id: Some(dependency_id),
                path: None,
            }),
        }
    }

    check_related_documents(
        &mut issues,
        &documents,
        &task_metadata.specs,
        "spec",
        matches_spec,
    );
    check_related_documents(
        &mut issues,
        &documents,
        &task_metadata.designs,
        "design",
        matches_design,
    );
    check_related_documents(
        &mut issues,
        &documents,
        &task_metadata.adrs,
        "adr",
        matches_adr,
    );

    for adr_id in sorted_unique_ids(&task_metadata.adrs) {
        if let Some(adr) = find_document(&documents, adr_id, matches_adr) {
            let DocumentMetadata::Adr(metadata) = &adr.document.metadata else {
                unreachable!("matches_adr only returns ADRs");
            };
            if matches!(metadata.status, AdrStatus::Rejected | AdrStatus::Superseded) {
                issues.push(TaskGuardIssue {
                    code: TaskGuardCode::InvalidRelatedAdrStatus,
                    message: format!(
                        "related ADR {} has status `{}`",
                        adr_id.get(),
                        adr_status_str(metadata.status)
                    ),
                    document_id: Some(adr_id),
                    path: Some(adr.path.clone()),
                });
            }
        }
    }

    Ok(TaskGuardReport {
        task_id,
        ready: issues.is_empty(),
        issues,
    })
}

fn append_related(
    items: &mut Vec<TaskContextItem>,
    documents: &[RepositoryDocument],
    ids: &[DocumentId],
    kind: TaskContextItemKind,
) -> Result<(), TaskContextError> {
    for id in sorted_unique_ids(ids) {
        let document = find_document(documents, id, |metadata| match kind {
            TaskContextItemKind::Spec => matches_spec(metadata),
            TaskContextItemKind::Design => matches_design(metadata),
            TaskContextItemKind::Adr => matches_adr(metadata),
            TaskContextItemKind::Task => false,
        });
        if let Some(document) = document {
            items.push(context_item(document, kind)?);
        } else {
            return Err(TaskContextError::MissingRelatedDocument { id, kind });
        }
    }
    Ok(())
}

fn context_item(
    document: &RepositoryDocument,
    kind: TaskContextItemKind,
) -> Result<TaskContextItem, TaskContextError> {
    let common = document.document.metadata.common();
    Ok(TaskContextItem {
        document_id: Some(common.id),
        kind,
        title: Some(common.title.clone()),
        path: document.path.clone(),
        content: read_file(&document.path)?,
    })
}

fn read_file(path: &Path) -> Result<String, TaskContextError> {
    fs::read_to_string(path).map_err(|source| TaskContextError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn check_related_documents(
    issues: &mut Vec<TaskGuardIssue>,
    documents: &[RepositoryDocument],
    ids: &[DocumentId],
    kind: &'static str,
    matches: fn(&DocumentMetadata) -> bool,
) {
    for id in sorted_unique_ids(ids) {
        if find_document(documents, id, matches).is_none() {
            issues.push(TaskGuardIssue {
                code: TaskGuardCode::MissingRelatedDocument,
                message: format!("related {kind} {} was not found", id.get()),
                document_id: Some(id),
                path: None,
            });
        }
    }
}

fn find_task(documents: &[RepositoryDocument], id: DocumentId) -> Option<&RepositoryDocument> {
    find_document(documents, id, matches_task)
}

fn find_document<F>(
    documents: &[RepositoryDocument],
    id: DocumentId,
    matches: F,
) -> Option<&RepositoryDocument>
where
    F: Fn(&DocumentMetadata) -> bool,
{
    documents.iter().find(|document| {
        document.document.metadata.common().id == id && matches(&document.document.metadata)
    })
}

fn sorted_unique_ids(ids: &[DocumentId]) -> Vec<DocumentId> {
    let mut seen = HashSet::new();
    let mut values: Vec<_> = ids
        .iter()
        .copied()
        .filter(|id| seen.insert(id.get()))
        .collect();
    values.sort();
    values
}

fn is_active_task_path(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.starts_with(Path::new("docs/tasks/active"))
}

fn matches_spec(metadata: &DocumentMetadata) -> bool {
    matches!(metadata, DocumentMetadata::Spec(_))
}

fn matches_design(metadata: &DocumentMetadata) -> bool {
    matches!(metadata, DocumentMetadata::Design(_))
}

fn matches_adr(metadata: &DocumentMetadata) -> bool {
    matches!(metadata, DocumentMetadata::Adr(_))
}

fn matches_task(metadata: &DocumentMetadata) -> bool {
    matches!(metadata, DocumentMetadata::Task(_))
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

fn adr_status_str(status: AdrStatus) -> &'static str {
    match status {
        AdrStatus::Proposed => "proposed",
        AdrStatus::Accepted => "accepted",
        AdrStatus::Rejected => "rejected",
        AdrStatus::Deprecated => "deprecated",
        AdrStatus::Superseded => "superseded",
    }
}
