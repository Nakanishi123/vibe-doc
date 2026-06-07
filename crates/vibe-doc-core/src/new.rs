use crate::{
    document_relative_path, next_document_id, parse_numbered_document, scan_repository,
    validate_documents, DocumentLocation, IdAllocationError, ParseError, RepositoryScanError,
    SchemaLoadError,
};
use crate::{
    AdrStatus, DocumentId, DocumentKind, DocumentMetadata, Priority, RepositoryDocument, SourceId,
    TaskStatus, TaskType,
};
use serde::Serialize;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Options for creating a new document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NewOptions {
    pub dry_run: bool,
    pub force: bool,
}

/// Options for creating a new task document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewTaskOptions {
    pub task_type: Option<TaskType>,
    pub priority: Option<Priority>,
    pub specs: Vec<DocumentId>,
    pub designs: Vec<DocumentId>,
    pub adrs: Vec<DocumentId>,
    pub depends_on: Vec<DocumentId>,
}

/// Options for creating a new ADR document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewAdrOptions {
    pub status: Option<AdrStatus>,
    pub tags: Vec<String>,
    pub related_designs: Vec<DocumentId>,
}

/// A planned or applied new document creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPlan {
    pub changes: Vec<NewChange>,
}

/// One planned or applied new document filesystem change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChange {
    pub path: PathBuf,
    pub action: NewChangeAction,
}

/// The action planned or performed for a new document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewChangeAction {
    Create,
    Overwrite,
    Keep,
}

impl NewChangeAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Overwrite => "overwrite",
            Self::Keep => "keep",
        }
    }
}

/// Error produced while planning or creating a new document.
#[derive(Debug)]
pub enum NewError {
    Conflict { path: PathBuf },
    CreateDir { path: PathBuf, source: io::Error },
    WriteFile { path: PathBuf, source: io::Error },
    Allocation(IdAllocationError),
    Schema(SchemaLoadError),
    FrontmatterSerialize(serde_yaml::Error),
    Parse(ParseError),
    Validation(Vec<String>),
}

impl fmt::Display for NewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict { path } => write!(
                formatter,
                "new document would overwrite existing file: {}",
                path.display()
            ),
            Self::CreateDir { path, source } => write!(
                formatter,
                "failed to create directory {}: {source}",
                path.display()
            ),
            Self::WriteFile { path, source } => write!(
                formatter,
                "failed to write file {}: {source}",
                path.display()
            ),
            Self::Allocation(error) => error.fmt(formatter),
            Self::Schema(error) => error.fmt(formatter),
            Self::FrontmatterSerialize(error) => {
                write!(
                    formatter,
                    "failed to serialize generated frontmatter: {error}"
                )
            }
            Self::Parse(error) => write!(formatter, "generated document is invalid: {error}"),
            Self::Validation(issues) => {
                writeln!(formatter, "generated document failed validation:")?;
                for issue in issues {
                    writeln!(formatter, "- {}", issue)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for NewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Conflict { .. } => None,
            Self::CreateDir { source, .. } => Some(source),
            Self::WriteFile { source, .. } => Some(source),
            Self::Allocation(error) => Some(error),
            Self::Schema(error) => Some(error),
            Self::FrontmatterSerialize(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::Validation(_) => None,
        }
    }
}

impl From<IdAllocationError> for NewError {
    fn from(value: IdAllocationError) -> Self {
        Self::Allocation(value)
    }
}

impl From<RepositoryScanError> for NewError {
    fn from(value: RepositoryScanError) -> Self {
        Self::Allocation(IdAllocationError::RepositoryScan(value))
    }
}

impl From<SchemaLoadError> for NewError {
    fn from(value: SchemaLoadError) -> Self {
        Self::Schema(value)
    }
}

impl From<ParseError> for NewError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

pub fn new_spec(
    root: impl AsRef<Path>,
    title: &str,
    options: NewOptions,
) -> Result<NewPlan, NewError> {
    create_document(
        root,
        title,
        DocumentKind::Spec,
        DocumentLocation::Spec,
        None,
        options,
    )
}

pub fn new_design(
    root: impl AsRef<Path>,
    title: &str,
    options: NewOptions,
) -> Result<NewPlan, NewError> {
    create_document(
        root,
        title,
        DocumentKind::Design,
        DocumentLocation::Design,
        None,
        options,
    )
}

pub fn new_adr(
    root: impl AsRef<Path>,
    title: &str,
    adr_options: NewAdrOptions,
    options: NewOptions,
) -> Result<NewPlan, NewError> {
    create_document(
        root,
        title,
        DocumentKind::Adr,
        DocumentLocation::Adr,
        Some(DocumentOptions::Adr(adr_options)),
        options,
    )
}

pub fn new_task(
    root: impl AsRef<Path>,
    title: &str,
    task_options: NewTaskOptions,
    options: NewOptions,
) -> Result<NewPlan, NewError> {
    create_document(
        root,
        title,
        DocumentKind::Task,
        DocumentLocation::ActiveTask,
        Some(DocumentOptions::Task(task_options)),
        options,
    )
}

enum DocumentOptions {
    Adr(NewAdrOptions),
    Task(NewTaskOptions),
}

fn create_document(
    root: impl AsRef<Path>,
    title: &str,
    kind: DocumentKind,
    location: DocumentLocation,
    doc_options: Option<DocumentOptions>,
    options: NewOptions,
) -> Result<NewPlan, NewError> {
    let root = root.as_ref();
    let mut documents = scan_repository(root)?;
    let id = next_document_id(&documents)?;

    let relative_path = document_relative_path(location, id, title);
    let full_path = root.join(&relative_path);

    let action = if full_path.exists() {
        if options.force {
            NewChangeAction::Overwrite
        } else if options.dry_run {
            NewChangeAction::Keep
        } else {
            return Err(NewError::Conflict {
                path: relative_path,
            });
        }
    } else {
        NewChangeAction::Create
    };

    if action == NewChangeAction::Keep && options.dry_run {
        return Ok(NewPlan {
            changes: vec![NewChange {
                path: relative_path,
                action,
            }],
        });
    }

    let markdown = generate_markdown(id, title, kind, doc_options)?;

    let parsed_doc = parse_numbered_document(SourceId::from(relative_path.as_path()), &markdown)?;

    let expected_kind = crate::expected_kind_for_relative_path(&relative_path)
        .expect("generated path is supported");

    let new_repo_doc = RepositoryDocument {
        path: full_path.clone(),
        expected_kind,
        document: parsed_doc,
    };

    documents.push(new_repo_doc);

    let schemas = crate::load_schema_set(root)?;
    let report = validate_documents(root, &schemas, &documents);

    let relevant_issues: Vec<_> = report
        .issues
        .into_iter()
        .filter(|issue| {
            if let Some(issue_path) = &issue.path {
                issue_path == &relative_path
            } else {
                false
            }
        })
        .collect();

    if !relevant_issues.is_empty() {
        let messages = relevant_issues.into_iter().map(|i| i.message).collect();
        return Err(NewError::Validation(messages));
    }

    if options.dry_run {
        return Ok(NewPlan {
            changes: planned_changes(kind, &relative_path, action, &documents),
        });
    }

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|source| NewError::CreateDir {
            path: relative_path.clone(),
            source,
        })?;
    }

    fs::write(&full_path, markdown).map_err(|source| NewError::WriteFile {
        path: relative_path.clone(),
        source,
    })?;

    if kind == DocumentKind::Task {
        update_task_index(root, &documents)?;
    }

    Ok(NewPlan {
        changes: planned_changes(kind, &relative_path, action, &documents),
    })
}

fn generate_markdown(
    id: DocumentId,
    title: &str,
    kind: DocumentKind,
    doc_options: Option<DocumentOptions>,
) -> Result<String, NewError> {
    let (frontmatter, body) = match kind {
        DocumentKind::Spec => {
            let frontmatter = SpecFrontmatter {
                id: id.get(),
                title,
                kind: "spec",
            };
            let body = format!("# {title}\n\n## Goal\n\nDescribe the purpose of the spec.\n");
            (serialize_frontmatter(&frontmatter)?, body)
        }
        DocumentKind::Design => {
            let frontmatter = DesignFrontmatter {
                id: id.get(),
                title,
                kind: "design",
                specs: Vec::new(),
            };
            let body = format!("# {title}\n\n## Overview\n\nDescribe the design approach.\n");
            (serialize_frontmatter(&frontmatter)?, body)
        }
        DocumentKind::Adr => {
            let adr_options = match doc_options {
                Some(DocumentOptions::Adr(opts)) => opts,
                _ => NewAdrOptions::default(),
            };
            let frontmatter = AdrFrontmatter {
                id: id.get(),
                title,
                kind: "adr",
                status: adr_status_str(adr_options.status.unwrap_or(AdrStatus::Proposed)),
                tags: adr_options.tags,
                related_designs: document_ids_as_u64s(&adr_options.related_designs),
            };
            let body = format!(
                "# {title}\n\n## Context\n\nDescribe the context.\n\n## Decision\n\nDescribe the decision.\n"
            );
            (serialize_frontmatter(&frontmatter)?, body)
        }
        DocumentKind::Task => {
            let task_options = match doc_options {
                Some(DocumentOptions::Task(opts)) => opts,
                _ => NewTaskOptions::default(),
            };
            let frontmatter = TaskFrontmatter {
                id: id.get(),
                title,
                kind: "task",
                task_type: task_type_str(task_options.task_type.unwrap_or(TaskType::Feature)),
                status: "planned",
                priority: task_options.priority.map(priority_str),
                specs: document_ids_as_u64s(&task_options.specs),
                designs: document_ids_as_u64s(&task_options.designs),
                adrs: document_ids_as_u64s(&task_options.adrs),
                depends_on: document_ids_as_u64s(&task_options.depends_on),
            };
            let body = "\
## Goal

Describe the purpose of the task.

## Scope

- Scope item

## Out of Scope

- Out-of-scope item

## Checklist

- [ ] Work item

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
"
            .to_owned();
            (serialize_frontmatter(&frontmatter)?, body)
        }
        DocumentKind::TaskIndex => unreachable!(),
    };

    Ok(format!("---\n{frontmatter}---\n\n{body}"))
}

fn planned_changes(
    kind: DocumentKind,
    relative_path: &Path,
    action: NewChangeAction,
    documents: &[RepositoryDocument],
) -> Vec<NewChange> {
    let mut changes = vec![NewChange {
        path: relative_path.to_path_buf(),
        action,
    }];

    if kind == DocumentKind::Task && has_task_index(documents) {
        changes.push(NewChange {
            path: PathBuf::from("docs/tasks/index.md"),
            action: NewChangeAction::Overwrite,
        });
    }

    changes
}

fn update_task_index(root: &Path, documents: &[RepositoryDocument]) -> Result<(), NewError> {
    let index_path = PathBuf::from("docs/tasks/index.md");
    if !has_task_index(documents) {
        return Ok(());
    }

    let markdown = generate_task_index_markdown(documents);
    fs::write(root.join(&index_path), markdown).map_err(|source| NewError::WriteFile {
        path: index_path,
        source,
    })
}

fn has_task_index(documents: &[RepositoryDocument]) -> bool {
    documents
        .iter()
        .any(|document| document.document.metadata.common().kind == DocumentKind::TaskIndex)
}

fn generate_task_index_markdown(documents: &[RepositoryDocument]) -> String {
    let index = documents
        .iter()
        .find_map(|document| match &document.document.metadata {
            DocumentMetadata::TaskIndex(index) => Some(index),
            _ => None,
        })
        .expect("task index exists before generating markdown");

    let mut planned = Vec::<(DocumentId, String)>::new();
    let mut doing = Vec::<(DocumentId, String)>::new();
    let mut blocked = Vec::<(DocumentId, String)>::new();
    let mut done = Vec::<(DocumentId, String)>::new();

    for document in documents {
        let DocumentMetadata::Task(task) = &document.document.metadata else {
            continue;
        };
        let item = (
            task.common.id,
            format!("- {} {}", task.common.id.get(), task.common.title),
        );

        match task.status {
            TaskStatus::Planned => planned.push(item),
            TaskStatus::Doing => doing.push(item),
            TaskStatus::Blocked => blocked.push(item),
            TaskStatus::Done | TaskStatus::Dropped => done.push(item),
        }
    }

    let mut markdown = String::new();
    let frontmatter = TaskIndexFrontmatter {
        id: index.common.id.get(),
        title: &index.common.title,
        kind: "task-index",
    };
    markdown.push_str("---\n");
    markdown.push_str(
        &serialize_frontmatter(&frontmatter)
            .expect("task index frontmatter serialization cannot fail"),
    );
    markdown.push_str("---\n\n");
    markdown.push_str("This index is generated by `vdoc rebuild index`.\n\n");
    push_task_index_section(&mut markdown, "Doing", doing);
    push_task_index_section(&mut markdown, "Planned", planned);
    push_task_index_section(&mut markdown, "Blocked", blocked);
    push_task_index_section(&mut markdown, "Done", done);
    markdown
}

fn push_task_index_section(
    markdown: &mut String,
    heading: &str,
    mut items: Vec<(DocumentId, String)>,
) {
    markdown.push_str(&format!("## {heading}\n\n"));
    if items.is_empty() {
        markdown.push_str("No tasks.\n\n");
    } else {
        items.sort_by_key(|(id, _)| *id);
        for (_, item) in items {
            markdown.push_str(&item);
            markdown.push('\n');
        }
        markdown.push('\n');
    }
}

fn serialize_frontmatter<T: Serialize>(frontmatter: &T) -> Result<String, NewError> {
    serde_yaml::to_string(frontmatter).map_err(NewError::FrontmatterSerialize)
}

fn document_ids_as_u64s(ids: &[DocumentId]) -> Vec<u64> {
    ids.iter().map(|id| id.get()).collect()
}

#[derive(Serialize)]
struct SpecFrontmatter<'a> {
    id: u64,
    title: &'a str,
    kind: &'static str,
}

#[derive(Serialize)]
struct DesignFrontmatter<'a> {
    id: u64,
    title: &'a str,
    kind: &'static str,
    specs: Vec<u64>,
}

#[derive(Serialize)]
struct AdrFrontmatter<'a> {
    id: u64,
    title: &'a str,
    kind: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_designs: Vec<u64>,
}

#[derive(Serialize)]
struct TaskFrontmatter<'a> {
    id: u64,
    title: &'a str,
    kind: &'static str,
    #[serde(rename = "type")]
    task_type: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    specs: Vec<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    designs: Vec<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    adrs: Vec<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends_on: Vec<u64>,
}

#[derive(Serialize)]
struct TaskIndexFrontmatter<'a> {
    id: u64,
    title: &'a str,
    kind: &'static str,
}

fn task_type_str(t: TaskType) -> &'static str {
    match t {
        TaskType::Feature => "feature",
        TaskType::Bug => "bug",
        TaskType::Refactor => "refactor",
        TaskType::Chore => "chore",
        TaskType::Docs => "docs",
        TaskType::Test => "test",
        TaskType::Spike => "spike",
    }
}

fn priority_str(p: Priority) -> &'static str {
    match p {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Critical => "critical",
    }
}

fn adr_status_str(s: AdrStatus) -> &'static str {
    match s {
        AdrStatus::Proposed => "proposed",
        AdrStatus::Accepted => "accepted",
        AdrStatus::Rejected => "rejected",
        AdrStatus::Deprecated => "deprecated",
        AdrStatus::Superseded => "superseded",
    }
}
