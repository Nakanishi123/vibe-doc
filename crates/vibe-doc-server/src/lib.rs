//! API server and SPA host crate for vibe-doc.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use vibe_doc_core::{
    complete_task, rebuild_task_index, scan_repository, start_task, task_context,
    validate_repository, AdrMetadata, AdrStatus, CompleteTaskOptions, DesignMetadata, DocumentId,
    DocumentMetadata, Priority, RepositoryDocument, RepositoryScanError, SpecMetadata,
    TaskContextError, TaskContextItem, TaskContextItemKind, TaskIndexRebuildError,
    TaskIndexRebuildOptions, TaskLifecycleError, TaskLifecycleOptions, TaskLifecyclePlan,
    TaskMetadata, TaskStatus, TaskType, ValidationIssue, ValidationRunError,
};

/// Stable crate identifier used by workspace smoke tests.
pub const CRATE_NAME: &str = "vibe-doc-server";

/// Shared server state.
#[derive(Debug, Clone)]
pub struct ServerState {
    repository_root: Arc<FsPath>,
}

impl ServerState {
    /// Create server state rooted at a vibe-doc repository.
    pub fn new(repository_root: impl Into<PathBuf>) -> Self {
        Self {
            repository_root: Arc::from(repository_root.into().into_boxed_path()),
        }
    }

    fn root(&self) -> &FsPath {
        &self.repository_root
    }
}

/// Build the read-only API router for a vibe-doc repository.
pub fn api_router(repository_root: impl Into<PathBuf>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/documents", get(list_documents))
        .route("/api/documents/{id}", get(document_detail))
        .route("/api/specs", get(list_specs))
        .route("/api/designs", get(list_designs))
        .route("/api/adr", get(list_adrs))
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks/{id}", get(task_detail))
        .route("/api/validation", get(validation_report))
        .route("/api/context/task/{id}", get(task_context_detail))
        .route("/api/tasks/{id}/start", post(start_task_endpoint))
        .route("/api/tasks/{id}/complete", post(complete_task_endpoint))
        .route(
            "/api/tasks/index/rebuild",
            post(rebuild_task_index_endpoint),
        )
        .with_state(ServerState::new(repository_root))
}

async fn health(State(state): State<ServerState>) -> Result<Json<HealthResponse>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    Ok(Json(HealthResponse {
        status: "ok",
        repository_root: display_path(state.root()),
        document_count: documents.len(),
    }))
}

async fn list_documents(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DocumentSummary>>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    Ok(Json(
        documents
            .iter()
            .map(|document| document_summary(state.root(), document))
            .collect(),
    ))
}

async fn document_detail(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentDetail>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Document)?;
    let documents = scan_documents(state.root())?;
    let document = find_document(&documents, id, MissingCode::Document)?;
    Ok(Json(to_document_detail(
        state.root(),
        &documents,
        document,
    )?))
}

async fn list_specs(
    State(state): State<ServerState>,
) -> Result<Json<Vec<SpecSummary>>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    let summaries = documents
        .iter()
        .filter_map(|document| match &document.document.metadata {
            DocumentMetadata::Spec(metadata) => {
                Some(spec_summary(state.root(), &documents, document, metadata))
            }
            _ => None,
        })
        .collect();
    Ok(Json(summaries))
}

async fn list_designs(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DesignSummary>>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    let summaries = documents
        .iter()
        .filter_map(|document| match &document.document.metadata {
            DocumentMetadata::Design(metadata) => {
                Some(design_summary(state.root(), &documents, document, metadata))
            }
            _ => None,
        })
        .collect();
    Ok(Json(summaries))
}

async fn list_adrs(State(state): State<ServerState>) -> Result<Json<Vec<AdrSummary>>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    let summaries = documents
        .iter()
        .filter_map(|document| match &document.document.metadata {
            DocumentMetadata::Adr(metadata) => Some(adr_summary(state.root(), document, metadata)),
            _ => None,
        })
        .collect();
    Ok(Json(summaries))
}

async fn list_tasks(
    State(state): State<ServerState>,
) -> Result<Json<TaskGroupsResponse>, ApiFailure> {
    let documents = scan_documents(state.root())?;
    let mut response = TaskGroupsResponse::default();

    for document in &documents {
        let DocumentMetadata::Task(metadata) = &document.document.metadata else {
            continue;
        };
        let summary = task_summary(state.root(), document, metadata);
        match metadata.status {
            TaskStatus::Done | TaskStatus::Dropped => response.done.push(summary),
            TaskStatus::Blocked => response.blocked.push(summary),
            _ => response.active.push(summary),
        }
    }

    Ok(Json(response))
}

async fn task_detail(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentDetail>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Task)?;
    let documents = scan_documents(state.root())?;
    let document = find_document(&documents, id, MissingCode::Task)?;
    if !matches!(document.document.metadata, DocumentMetadata::Task(_)) {
        return Err(ApiFailure::not_found(MissingCode::Task, id));
    }
    Ok(Json(to_document_detail(
        state.root(),
        &documents,
        document,
    )?))
}

async fn validation_report(
    State(state): State<ServerState>,
) -> Result<Json<ValidationResponse>, ApiFailure> {
    let report = validate_repository(state.root()).map_err(ApiFailure::ValidationRun)?;
    let error_count = report.issues.len();
    Ok(Json(ValidationResponse {
        status: if report.is_valid() { "ok" } else { "error" },
        error_count,
        warning_count: 0,
        incomplete: report.incomplete,
        issues: report
            .issues
            .iter()
            .map(|issue| validation_issue_response(state.root(), issue))
            .collect(),
    }))
}

async fn task_context_detail(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<TaskContextResponse>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Task)?;
    let context = task_context(state.root(), id).map_err(ApiFailure::TaskContext)?;
    let documents = scan_documents(state.root())?;
    let task = find_document(&documents, id, MissingCode::Task)?;
    let DocumentMetadata::Task(metadata) = &task.document.metadata else {
        return Err(ApiFailure::not_found(MissingCode::Task, id));
    };

    Ok(Json(TaskContextResponse {
        task: task_summary(state.root(), task, metadata),
        files: context
            .items
            .iter()
            .map(|item| task_context_file(state.root(), item))
            .collect(),
    }))
}

async fn start_task_endpoint(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    body: Option<Json<TaskMutationRequest>>,
) -> Result<Json<TaskLifecycleResponse>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Task)?;
    let request = body.map(|Json(request)| request).unwrap_or_default();
    let options = TaskLifecycleOptions {
        dry_run: request.dry_run.unwrap_or(false),
        date: lifecycle_date(request.date)?,
    };
    let dry_run = options.dry_run;
    let plan = start_task(state.root(), id, options).map_err(ApiFailure::TaskLifecycle)?;
    Ok(Json(task_lifecycle_response("start task", dry_run, &plan)))
}

async fn complete_task_endpoint(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    body: Option<Json<TaskMutationRequest>>,
) -> Result<Json<TaskLifecycleResponse>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Task)?;
    let request = body.map(|Json(request)| request).unwrap_or_default();
    let options = CompleteTaskOptions {
        lifecycle: TaskLifecycleOptions {
            dry_run: request.dry_run.unwrap_or(false),
            date: lifecycle_date(request.date)?,
        },
        result: request.result,
    };
    let dry_run = options.lifecycle.dry_run;
    let plan = complete_task(state.root(), id, options).map_err(ApiFailure::TaskLifecycle)?;
    Ok(Json(task_lifecycle_response(
        "complete task",
        dry_run,
        &plan,
    )))
}

async fn rebuild_task_index_endpoint(
    State(state): State<ServerState>,
    body: Option<Json<RebuildIndexRequest>>,
) -> Result<Json<RebuildIndexResponse>, ApiFailure> {
    let request = body.map(|Json(request)| request).unwrap_or_default();
    let dry_run = request.dry_run.unwrap_or(false);
    let plan = rebuild_task_index(state.root(), TaskIndexRebuildOptions { dry_run })
        .map_err(ApiFailure::RebuildIndex)?;
    Ok(Json(RebuildIndexResponse {
        command: "rebuild index",
        dry_run,
        path: display_path(&plan.path),
        action: plan.action.as_str().to_owned(),
        content: plan.content,
    }))
}

fn scan_documents(root: &FsPath) -> Result<Vec<RepositoryDocument>, ApiFailure> {
    let mut documents = scan_repository(root).map_err(ApiFailure::Scan)?;
    documents.sort_by_key(|document| document.document.metadata.common().id);
    Ok(documents)
}

fn parse_document_id(raw: &str, missing_code: MissingCode) -> Result<DocumentId, ApiFailure> {
    let value = raw.parse::<u64>().map_err(|_| ApiFailure::InvalidId {
        raw: raw.to_owned(),
        missing_code,
    })?;
    DocumentId::new(value).ok_or_else(|| ApiFailure::InvalidId {
        raw: raw.to_owned(),
        missing_code,
    })
}

fn find_document(
    documents: &[RepositoryDocument],
    id: DocumentId,
    missing_code: MissingCode,
) -> Result<&RepositoryDocument, ApiFailure> {
    documents
        .iter()
        .find(|document| document.document.metadata.common().id == id)
        .ok_or_else(|| ApiFailure::not_found(missing_code, id))
}

fn to_document_detail(
    root: &FsPath,
    documents: &[RepositoryDocument],
    document: &RepositoryDocument,
) -> Result<DocumentDetail, ApiFailure> {
    let mut summary = document_summary(root, document);
    let related_ids = related_ids_for(&document.document.metadata, documents);
    let related_documents = related_ids
        .iter()
        .filter_map(|related| {
            documents
                .iter()
                .find(|candidate| candidate.document.metadata.common().id == related.id)
                .map(|candidate| RelatedDocument {
                    id: related.id.get(),
                    title: candidate.document.metadata.common().title.clone(),
                    kind: document_kind_as_str(&candidate.document.metadata).to_owned(),
                    relation: related.relation.clone(),
                })
        })
        .collect();
    let frontmatter = frontmatter_value(&document.document.frontmatter.raw)?;

    Ok(DocumentDetail {
        id: summary.id,
        title: std::mem::take(&mut summary.title),
        kind: std::mem::take(&mut summary.kind),
        path: std::mem::take(&mut summary.path),
        tags: summary.tags,
        frontmatter,
        markdown: document.document.body.clone(),
        related_ids: related_ids
            .into_iter()
            .map(|related| RelatedId {
                id: related.id.get(),
                relation: related.relation,
            })
            .collect(),
        related_documents,
        validation: Vec::new(),
    })
}

fn frontmatter_value(raw: &str) -> Result<Value, ApiFailure> {
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(raw).map_err(|source| ApiFailure::Frontmatter {
            message: source.to_string(),
        })?;
    serde_json::to_value(yaml).map_err(|source| ApiFailure::Frontmatter {
        message: source.to_string(),
    })
}

fn document_summary(root: &FsPath, document: &RepositoryDocument) -> DocumentSummary {
    let common = document.document.metadata.common();
    DocumentSummary {
        id: common.id.get(),
        title: common.title.clone(),
        kind: document_kind_as_str(&document.document.metadata).to_owned(),
        path: display_path(&relative_path(root, &document.path)),
        tags: common.tags.clone(),
    }
}

fn spec_summary(
    root: &FsPath,
    documents: &[RepositoryDocument],
    document: &RepositoryDocument,
    metadata: &SpecMetadata,
) -> SpecSummary {
    let id = metadata.common.id;
    SpecSummary {
        summary: document_summary(root, document),
        related_designs: documents
            .iter()
            .filter_map(|document| match &document.document.metadata {
                DocumentMetadata::Design(metadata) if metadata.specs.contains(&id) => {
                    Some(metadata.common.id.get())
                }
                _ => None,
            })
            .collect(),
        related_tasks: related_tasks(documents, |task| task.specs.contains(&id)),
    }
}

fn design_summary(
    root: &FsPath,
    documents: &[RepositoryDocument],
    document: &RepositoryDocument,
    metadata: &DesignMetadata,
) -> DesignSummary {
    let id = metadata.common.id;
    DesignSummary {
        summary: document_summary(root, document),
        specs: document_ids(&metadata.specs),
        adrs: document_ids(&metadata.adrs),
        related_tasks: related_tasks(documents, |task| task.designs.contains(&id)),
    }
}

fn adr_summary(root: &FsPath, document: &RepositoryDocument, metadata: &AdrMetadata) -> AdrSummary {
    AdrSummary {
        summary: document_summary(root, document),
        status: adr_status_as_str(metadata.status).to_owned(),
        date: metadata.date.clone(),
        related_designs: document_ids(&metadata.related_designs),
        supersedes: document_ids(&metadata.supersedes),
        superseded_by: metadata.superseded_by.map(DocumentId::get),
    }
}

fn task_summary(
    root: &FsPath,
    document: &RepositoryDocument,
    metadata: &TaskMetadata,
) -> TaskSummary {
    TaskSummary {
        summary: document_summary(root, document),
        status: task_status_as_str(metadata.status).to_owned(),
        task_type: task_type_as_str(metadata.task_type).to_owned(),
        priority: metadata
            .priority
            .map(priority_as_str)
            .unwrap_or("medium")
            .to_owned(),
        specs: document_ids(&metadata.specs),
        designs: document_ids(&metadata.designs),
        adrs: document_ids(&metadata.adrs),
        depends_on: document_ids(&metadata.depends_on),
    }
}

fn related_tasks(
    documents: &[RepositoryDocument],
    predicate: impl Fn(&TaskMetadata) -> bool,
) -> Vec<u64> {
    documents
        .iter()
        .filter_map(|document| match &document.document.metadata {
            DocumentMetadata::Task(metadata) if predicate(metadata) => {
                Some(metadata.common.id.get())
            }
            _ => None,
        })
        .collect()
}

fn related_ids_for(
    metadata: &DocumentMetadata,
    documents: &[RepositoryDocument],
) -> Vec<RelatedDocumentId> {
    let mut related = BTreeMap::new();

    match metadata {
        DocumentMetadata::Spec(metadata) => {
            let id = metadata.common.id;
            push_related_tasks(&mut related, documents, |task| task.specs.contains(&id));
            for document in documents {
                if let DocumentMetadata::Design(design) = &document.document.metadata {
                    if design.specs.contains(&id) {
                        related.insert(design.common.id, "design".to_owned());
                    }
                }
            }
        }
        DocumentMetadata::Design(metadata) => {
            insert_ids(&mut related, &metadata.specs, "spec");
            insert_ids(&mut related, &metadata.adrs, "adr");
            let id = metadata.common.id;
            push_related_tasks(&mut related, documents, |task| task.designs.contains(&id));
        }
        DocumentMetadata::Adr(metadata) => {
            insert_ids(&mut related, &metadata.related_designs, "design");
            insert_ids(&mut related, &metadata.supersedes, "adr");
            if let Some(id) = metadata.superseded_by {
                related.insert(id, "adr".to_owned());
            }
        }
        DocumentMetadata::Task(metadata) => {
            insert_ids(&mut related, &metadata.specs, "spec");
            insert_ids(&mut related, &metadata.designs, "design");
            insert_ids(&mut related, &metadata.adrs, "adr");
            insert_ids(&mut related, &metadata.depends_on, "dependency");
        }
        DocumentMetadata::TaskIndex(_) => {}
    }

    related
        .into_iter()
        .map(|(id, relation)| RelatedDocumentId { id, relation })
        .collect()
}

fn push_related_tasks(
    related: &mut BTreeMap<DocumentId, String>,
    documents: &[RepositoryDocument],
    predicate: impl Fn(&TaskMetadata) -> bool,
) {
    for document in documents {
        if let DocumentMetadata::Task(metadata) = &document.document.metadata {
            if predicate(metadata) {
                related.insert(metadata.common.id, "task".to_owned());
            }
        }
    }
}

fn insert_ids(related: &mut BTreeMap<DocumentId, String>, ids: &[DocumentId], relation: &str) {
    for id in ids {
        related.insert(*id, relation.to_owned());
    }
}

fn document_ids(ids: &[DocumentId]) -> Vec<u64> {
    ids.iter().map(|id| id.get()).collect()
}

fn document_kind_as_str(metadata: &DocumentMetadata) -> &'static str {
    match metadata {
        DocumentMetadata::Spec(_) => "spec",
        DocumentMetadata::Design(_) => "design",
        DocumentMetadata::Adr(_) => "adr",
        DocumentMetadata::Task(_) => "task",
        DocumentMetadata::TaskIndex(_) => "task-index",
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

fn relative_path(root: &FsPath, path: &FsPath) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn display_path(path: &FsPath) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn lifecycle_date(date: Option<String>) -> Result<String, ApiFailure> {
    let Some(date) = date else {
        return Ok(Local::now().date_naive().format("%Y-%m-%d").to_string());
    };
    parse_lifecycle_date(&date)?;
    Ok(date)
}

fn parse_lifecycle_date(date: &str) -> Result<NaiveDate, ApiFailure> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        ApiFailure::InvalidRequest(format!("invalid date `{date}`; expected YYYY-MM-DD"))
    })?;
    if parsed.format("%Y-%m-%d").to_string() != date {
        return Err(ApiFailure::InvalidRequest(format!(
            "invalid date `{date}`; expected YYYY-MM-DD"
        )));
    }
    Ok(parsed)
}

fn validation_issue_response(root: &FsPath, issue: &ValidationIssue) -> ValidationIssueResponse {
    ValidationIssueResponse {
        severity: "error",
        code: issue.code.as_str().to_owned(),
        message: issue.message.clone(),
        path: issue
            .path
            .as_ref()
            .map(|path| display_path(&relative_path(root, path))),
        document_id: None,
        suggested_fix: None,
    }
}

fn task_context_file(root: &FsPath, item: &TaskContextItem) -> TaskContextFile {
    TaskContextFile {
        path: display_path(&relative_path(root, &item.path)),
        role: task_context_role(item.kind),
        content: item.content.clone(),
    }
}

fn task_context_role(kind: TaskContextItemKind) -> &'static str {
    match kind {
        TaskContextItemKind::Task => "task",
        TaskContextItemKind::Spec => "spec",
        TaskContextItemKind::Design => "design",
        TaskContextItemKind::Adr => "adr",
    }
}

fn task_lifecycle_response(
    command: &'static str,
    dry_run: bool,
    plan: &TaskLifecyclePlan,
) -> TaskLifecycleResponse {
    TaskLifecycleResponse {
        command,
        dry_run,
        task_id: plan.task_id.get(),
        changes: plan
            .changes
            .iter()
            .map(|change| TaskLifecycleChangeResponse {
                path: display_path(&change.path),
                action: change.action.as_str().to_owned(),
            })
            .collect(),
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    repository_root: String,
    document_count: usize,
}

#[derive(Debug, Serialize)]
struct DocumentSummary {
    id: u64,
    title: String,
    kind: String,
    path: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DocumentDetail {
    id: u64,
    title: String,
    kind: String,
    path: String,
    tags: Vec<String>,
    frontmatter: Value,
    markdown: String,
    related_ids: Vec<RelatedId>,
    related_documents: Vec<RelatedDocument>,
    validation: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct RelatedId {
    id: u64,
    relation: String,
}

#[derive(Debug, Serialize)]
struct RelatedDocument {
    id: u64,
    title: String,
    kind: String,
    relation: String,
}

#[derive(Debug)]
struct RelatedDocumentId {
    id: DocumentId,
    relation: String,
}

#[derive(Debug, Serialize)]
struct SpecSummary {
    #[serde(flatten)]
    summary: DocumentSummary,
    related_designs: Vec<u64>,
    related_tasks: Vec<u64>,
}

#[derive(Debug, Serialize)]
struct DesignSummary {
    #[serde(flatten)]
    summary: DocumentSummary,
    specs: Vec<u64>,
    adrs: Vec<u64>,
    related_tasks: Vec<u64>,
}

#[derive(Debug, Serialize)]
struct AdrSummary {
    #[serde(flatten)]
    summary: DocumentSummary,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    related_designs: Vec<u64>,
    supersedes: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    superseded_by: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
struct TaskGroupsResponse {
    active: Vec<TaskSummary>,
    done: Vec<TaskSummary>,
    blocked: Vec<TaskSummary>,
}

#[derive(Debug, Serialize)]
struct TaskSummary {
    #[serde(flatten)]
    summary: DocumentSummary,
    status: String,
    #[serde(rename = "type")]
    task_type: String,
    priority: String,
    specs: Vec<u64>,
    designs: Vec<u64>,
    adrs: Vec<u64>,
    depends_on: Vec<u64>,
}

#[derive(Debug, Serialize)]
struct ValidationResponse {
    status: &'static str,
    error_count: usize,
    warning_count: usize,
    incomplete: bool,
    issues: Vec<ValidationIssueResponse>,
}

#[derive(Debug, Serialize)]
struct ValidationIssueResponse {
    severity: &'static str,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_fix: Option<String>,
}

#[derive(Debug, Serialize)]
struct TaskContextResponse {
    task: TaskSummary,
    files: Vec<TaskContextFile>,
}

#[derive(Debug, Serialize)]
struct TaskContextFile {
    path: String,
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize, Default)]
struct TaskMutationRequest {
    dry_run: Option<bool>,
    date: Option<String>,
    result: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RebuildIndexRequest {
    dry_run: Option<bool>,
}

#[derive(Debug, Serialize)]
struct TaskLifecycleResponse {
    command: &'static str,
    dry_run: bool,
    task_id: u64,
    changes: Vec<TaskLifecycleChangeResponse>,
}

#[derive(Debug, Serialize)]
struct TaskLifecycleChangeResponse {
    path: String,
    action: String,
}

#[derive(Debug, Serialize)]
struct RebuildIndexResponse {
    command: &'static str,
    dry_run: bool,
    path: String,
    action: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ApiError {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_id: Option<u64>,
}

#[derive(Debug, Error)]
enum ApiFailure {
    #[error(transparent)]
    Scan(#[from] RepositoryScanError),
    #[error(transparent)]
    ValidationRun(#[from] ValidationRunError),
    #[error(transparent)]
    TaskContext(#[from] TaskContextError),
    #[error(transparent)]
    TaskLifecycle(#[from] TaskLifecycleError),
    #[error(transparent)]
    RebuildIndex(#[from] TaskIndexRebuildError),
    #[error("invalid document ID `{raw}`")]
    InvalidId {
        raw: String,
        missing_code: MissingCode,
    },
    #[error("document {id:?} was not found")]
    NotFound {
        id: DocumentId,
        missing_code: MissingCode,
    },
    #[error("failed to parse document frontmatter for API response: {message}")]
    Frontmatter { message: String },
    #[error("{0}")]
    InvalidRequest(String),
}

impl ApiFailure {
    fn not_found(missing_code: MissingCode, id: DocumentId) -> Self {
        Self::NotFound { id, missing_code }
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Scan(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError {
                    code: "API_REPOSITORY_SCAN_FAILED",
                    message: error.to_string(),
                    path: None,
                    document_id: None,
                },
            ),
            Self::ValidationRun(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError {
                    code: "VALIDATION_RUN_FAILED",
                    message: error.to_string(),
                    path: None,
                    document_id: None,
                },
            ),
            Self::TaskContext(TaskContextError::TaskNotFound { id }) => (
                StatusCode::NOT_FOUND,
                ApiError {
                    code: "TASK_NOT_FOUND",
                    message: format!("task {} was not found", id.get()),
                    path: None,
                    document_id: Some(id.get()),
                },
            ),
            Self::TaskContext(TaskContextError::MissingRelatedDocument { id, .. }) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiError {
                    code: "TASK_CONTEXT_MISSING_RELATED_DOCUMENT",
                    message: self.to_string(),
                    path: None,
                    document_id: Some(id.get()),
                },
            ),
            Self::TaskContext(TaskContextError::RepositoryScan(error)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError {
                    code: "TASK_CONTEXT_SCAN_FAILED",
                    message: error.to_string(),
                    path: None,
                    document_id: None,
                },
            ),
            Self::TaskLifecycle(error) => task_lifecycle_api_error(error),
            Self::RebuildIndex(error) => rebuild_index_api_error(error),
            Self::InvalidId { raw, missing_code } => (
                StatusCode::BAD_REQUEST,
                ApiError {
                    code: missing_code.invalid_id_code(),
                    message: format!("invalid document ID `{raw}`"),
                    path: None,
                    document_id: None,
                },
            ),
            Self::NotFound { id, missing_code } => (
                StatusCode::NOT_FOUND,
                ApiError {
                    code: missing_code.not_found_code(),
                    message: format!("document {} was not found", id.get()),
                    path: None,
                    document_id: Some(id.get()),
                },
            ),
            Self::Frontmatter { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError {
                    code: "API_FRONTMATTER_SERIALIZE_FAILED",
                    message,
                    path: None,
                    document_id: None,
                },
            ),
            Self::InvalidRequest(message) => (
                StatusCode::BAD_REQUEST,
                ApiError {
                    code: "INVALID_REQUEST",
                    message,
                    path: None,
                    document_id: None,
                },
            ),
        };

        (status, Json(json!({ "error": error }))).into_response()
    }
}

fn task_lifecycle_api_error(error: TaskLifecycleError) -> (StatusCode, ApiError) {
    match error {
        TaskLifecycleError::TaskNotFound { id } => (
            StatusCode::NOT_FOUND,
            ApiError {
                code: "TASK_LIFECYCLE_TASK_NOT_FOUND",
                message: format!("task {} was not found", id.get()),
                path: None,
                document_id: Some(id.get()),
            },
        ),
        TaskLifecycleError::InvalidTaskStatus {
            id,
            status,
            expected,
        } => (
            StatusCode::CONFLICT,
            ApiError {
                code: "TASK_LIFECYCLE_INVALID_STATUS",
                message: format!(
                    "task {} has status {}; expected {expected}",
                    id.get(),
                    task_status_as_str(status)
                ),
                path: None,
                document_id: Some(id.get()),
            },
        ),
        TaskLifecycleError::InvalidTaskLocation { ref path }
        | TaskLifecycleError::DestinationExists { ref path }
        | TaskLifecycleError::ReadFile { ref path, .. }
        | TaskLifecycleError::CreateDir { ref path, .. }
        | TaskLifecycleError::WriteFile { ref path, .. }
        | TaskLifecycleError::DeleteFile { ref path, .. } => (
            StatusCode::CONFLICT,
            ApiError {
                code: "TASK_LIFECYCLE_IO_FAILED",
                message: error.to_string(),
                path: Some(display_path(&path)),
                document_id: None,
            },
        ),
        TaskLifecycleError::RepositoryScan(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "TASK_LIFECYCLE_SCAN_FAILED",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        TaskLifecycleError::FrontmatterParse(_)
        | TaskLifecycleError::FrontmatterNotMapping
        | TaskLifecycleError::FrontmatterSerialize(_)
        | TaskLifecycleError::Parse(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiError {
                code: "TASK_LIFECYCLE_PARSE_FAILED",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        TaskLifecycleError::Validation(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiError {
                code: "TASK_LIFECYCLE_VALIDATION_FAILED",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        TaskLifecycleError::ValidationRun(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "TASK_LIFECYCLE_VALIDATION_RUN_FAILED",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
    }
}

fn rebuild_index_api_error(error: TaskIndexRebuildError) -> (StatusCode, ApiError) {
    match error {
        TaskIndexRebuildError::RepositoryScan(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "REBUILD_INDEX_SCAN_FAILED",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        TaskIndexRebuildError::MissingTaskIndex => (
            StatusCode::NOT_FOUND,
            ApiError {
                code: "REBUILD_INDEX_MISSING_TASK_INDEX",
                message: error.to_string(),
                path: Some("docs/tasks/index.md".to_owned()),
                document_id: None,
            },
        ),
        TaskIndexRebuildError::ReadFile { ref path, .. }
        | TaskIndexRebuildError::WriteFile { ref path, .. } => (
            StatusCode::CONFLICT,
            ApiError {
                code: "REBUILD_INDEX_IO_FAILED",
                message: error.to_string(),
                path: Some(display_path(&path)),
                document_id: None,
            },
        ),
    }
}

#[derive(Debug, Clone, Copy)]
enum MissingCode {
    Document,
    Task,
}

impl MissingCode {
    fn invalid_id_code(self) -> &'static str {
        match self {
            Self::Document => "INVALID_DOCUMENT_ID",
            Self::Task => "INVALID_TASK_ID",
        }
    }

    fn not_found_code(self) -> &'static str {
        match self {
            Self::Document => "DOCUMENT_NOT_FOUND",
            Self::Task => "TASK_NOT_FOUND",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_reports_repository_state() {
        let repo = TestRepo::new("health");
        repo.seed();
        let response = request_json(api_router(repo.path()), "/api/health").await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["status"], "ok");
        assert_eq!(response.body["document_count"], 6);
    }

    #[tokio::test]
    async fn document_list_and_detail_endpoints_return_content_and_relationships() {
        let repo = TestRepo::new("documents");
        repo.seed();
        let list = request_json(api_router(repo.path()), "/api/documents").await;

        assert_eq!(list.status, StatusCode::OK);
        assert_eq!(list.body.as_array().unwrap().len(), 6);
        assert_eq!(list.body[0]["id"], 9);
        assert_eq!(list.body[0]["path"], "docs/specs/9-model.md");

        let detail = request_json(api_router(repo.path()), "/api/documents/10").await;

        assert_eq!(detail.status, StatusCode::OK);
        assert_eq!(detail.body["id"], 10);
        assert_eq!(detail.body["kind"], "design");
        assert_eq!(detail.body["frontmatter"]["specs"][0], 9);
        assert!(detail.body["markdown"]
            .as_str()
            .unwrap()
            .contains("# Design"));
        assert_eq!(detail.body["related_ids"][0]["id"], 9);
        assert_eq!(detail.body["related_ids"][0]["relation"], "spec");
    }

    #[tokio::test]
    async fn kind_specific_endpoints_return_expected_shapes() {
        let repo = TestRepo::new("kinds");
        repo.seed();

        let specs = request_json(api_router(repo.path()), "/api/specs").await;
        assert_eq!(specs.status, StatusCode::OK);
        assert_eq!(specs.body[0]["related_designs"][0], 10);
        assert_eq!(specs.body[0]["related_tasks"][0], 28);

        let designs = request_json(api_router(repo.path()), "/api/designs").await;
        assert_eq!(designs.status, StatusCode::OK);
        assert_eq!(designs.body[0]["specs"][0], 9);
        assert_eq!(designs.body[0]["related_tasks"][0], 28);

        let adrs = request_json(api_router(repo.path()), "/api/adr").await;
        assert_eq!(adrs.status, StatusCode::OK);
        assert_eq!(adrs.body[0]["status"], "accepted");

        let tasks = request_json(api_router(repo.path()), "/api/tasks").await;
        assert_eq!(tasks.status, StatusCode::OK);
        assert_eq!(tasks.body["active"][0]["id"], 28);
        assert_eq!(tasks.body["active"][0]["type"], "feature");
        assert_eq!(tasks.body["done"][1]["id"], 29);
        assert_eq!(tasks.body["done"][1]["status"], "dropped");

        let task = request_json(api_router(repo.path()), "/api/tasks/28").await;
        assert_eq!(task.status, StatusCode::OK);
        assert_eq!(task.body["id"], 28);
        assert_eq!(task.body["kind"], "task");
    }

    #[tokio::test]
    async fn missing_and_invalid_ids_return_stable_api_errors() {
        let repo = TestRepo::new("errors");
        repo.seed();

        let missing = request_json(api_router(repo.path()), "/api/documents/999").await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND);
        assert_eq!(missing.body["error"]["code"], "DOCUMENT_NOT_FOUND");
        assert_eq!(missing.body["error"]["document_id"], 999);

        let invalid = request_json(api_router(repo.path()), "/api/tasks/not-a-number").await;
        assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid.body["error"]["code"], "INVALID_TASK_ID");

        let traversal = request(api_router(repo.path()), "/api/tasks/%2E%2E/%2E%2E/9").await;
        assert_eq!(traversal.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn validation_endpoint_returns_stable_errors() {
        let repo = TestRepo::new("validation");
        repo.seed();

        let response = request_json(api_router(repo.path()), "/api/validation").await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["status"], "error");
        assert_eq!(response.body["issues"][0]["severity"], "error");
        assert_eq!(response.body["issues"][0]["code"], "SCHEMA_NOT_FOUND");
        assert!(response.body["error_count"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn task_context_endpoint_returns_task_context_files() {
        let repo = TestRepo::new("context");
        repo.seed();

        let response = request_json(api_router(repo.path()), "/api/context/task/28").await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["task"]["id"], 28);
        assert_eq!(response.body["files"][0]["role"], "task");
        assert_eq!(response.body["files"][1]["role"], "spec");
        assert_eq!(response.body["files"][2]["role"], "design");
        assert_eq!(response.body["files"][3]["role"], "adr");
    }

    #[tokio::test]
    async fn task_mutation_endpoints_use_core_lifecycle_behavior() {
        let repo = TestRepo::new("mutations");
        repo.seed_mutable();

        let start = post_json(
            api_router(repo.path()),
            "/api/tasks/2/start",
            json!({ "date": "2026-06-08" }),
        )
        .await;

        assert_eq!(start.status, StatusCode::OK);
        assert_eq!(start.body["command"], "start task");
        assert_eq!(start.body["task_id"], 2);
        assert_eq!(start.body["changes"][0]["action"], "overwrite");
        let active = fs::read_to_string(repo.root.join("docs/tasks/active/2-api.md")).unwrap();
        assert!(active.contains("status: doing"));
        assert!(active.contains("started_at: 2026-06-08"));

        let complete = post_json(
            api_router(repo.path()),
            "/api/tasks/2/complete",
            json!({
                "date": "2026-06-08",
                "result": "Implemented server mutation APIs."
            }),
        )
        .await;

        assert_eq!(complete.status, StatusCode::OK);
        assert_eq!(complete.body["command"], "complete task");
        assert!(!repo.root.join("docs/tasks/active/2-api.md").exists());
        let done = fs::read_to_string(repo.root.join("docs/tasks/done/2-api.md")).unwrap();
        assert!(done.contains("status: done"));
        assert!(done.contains("completed_at: 2026-06-08"));
        assert!(done.contains("Implemented server mutation APIs."));
    }

    #[tokio::test]
    async fn mutation_endpoints_reject_invalid_ids_and_invalid_statuses() {
        let repo = TestRepo::new("mutation-errors");
        repo.seed_mutable();

        let invalid_id = post_json(
            api_router(repo.path()),
            "/api/tasks/not-a-number/start",
            json!({}),
        )
        .await;
        assert_eq!(invalid_id.status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid_id.body["error"]["code"], "INVALID_TASK_ID");

        let invalid_status =
            post_json(api_router(repo.path()), "/api/tasks/2/complete", json!({})).await;
        assert_eq!(invalid_status.status, StatusCode::CONFLICT);
        assert_eq!(
            invalid_status.body["error"]["code"],
            "TASK_LIFECYCLE_INVALID_STATUS"
        );

        let traversal = request(api_router(repo.path()), "/api/tasks/%2E%2E/%2E%2E/start").await;
        assert_eq!(traversal.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rebuild_index_endpoint_is_limited_to_task_index_generation() {
        let repo = TestRepo::new("rebuild-index");
        repo.seed_mutable();

        let response = post_json(
            api_router(repo.path()),
            "/api/tasks/index/rebuild",
            json!({}),
        )
        .await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["command"], "rebuild index");
        assert_eq!(response.body["path"], "docs/tasks/index.md");
        let index = fs::read_to_string(repo.root.join("docs/tasks/index.md")).unwrap();
        assert!(index.contains("- 2 API"));
    }

    async fn request_json(router: Router, uri: &str) -> TestResponse {
        let response = request(router, uri).await;
        let body = serde_json::from_slice(&response.body).unwrap();
        TestResponse {
            status: response.status,
            body,
        }
    }

    async fn post_json(router: Router, uri: &str, body: Value) -> TestResponse {
        let response = request_with_body(router, Method::POST, uri, body.to_string()).await;
        let body = serde_json::from_slice(&response.body).unwrap();
        TestResponse {
            status: response.status,
            body,
        }
    }

    async fn request(router: Router, uri: &str) -> RawTestResponse {
        request_with_body(router, Method::GET, uri, String::new()).await
    }

    async fn request_with_body(
        router: Router,
        method: Method,
        uri: &str,
        body: String,
    ) -> RawTestResponse {
        let response = router
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        RawTestResponse {
            status,
            body: body.to_vec(),
        }
    }

    struct TestResponse {
        status: StatusCode,
        body: Value,
    }

    struct RawTestResponse {
        status: StatusCode,
        body: Vec<u8>,
    }

    struct TestRepo {
        root: PathBuf,
    }

    impl TestRepo {
        fn new(name: &str) -> Self {
            let mut root = std::env::temp_dir();
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            root.push(format!("vibe-doc-server-{name}-{unique}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn path(&self) -> PathBuf {
            self.root.clone()
        }

        fn seed(&self) {
            self.write(
                "docs/specs/9-model.md",
                "\
---
id: 9
title: Model
kind: spec
tags:
  - core
---

# Model
",
            );
            self.write(
                "docs/designs/10-design.md",
                "\
---
id: 10
title: Design
kind: design
specs:
  - 9
adrs:
  - 11
---

# Design
",
            );
            self.write(
                "docs/adr/11-decision.md",
                "\
---
id: 11
title: Decision
kind: adr
status: accepted
related_designs:
  - 10
---

# Decision
",
            );
            self.write(
                "docs/tasks/active/28-api.md",
                "\
---
id: 28
title: API
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
designs:
  - 10
adrs:
  - 11
depends_on: []
---

# API
",
            );
            self.write(
                "docs/tasks/done/27-done.md",
                "\
---
id: 27
title: Done Task
kind: task
type: chore
status: done
priority: low
specs: []
designs: []
adrs: []
depends_on: []
---

# Done
",
            );
            self.write(
                "docs/tasks/done/29-dropped.md",
                "\
---
id: 29
title: Dropped Task
kind: task
type: chore
status: dropped
priority: low
specs: []
designs: []
adrs: []
depends_on: []
---

# Dropped
",
            );
        }

        fn seed_mutable(&self) {
            vibe_doc_core::init_repository(
                &self.root,
                vibe_doc_core::InitOptions {
                    dry_run: false,
                    force: false,
                },
            )
            .unwrap();
            self.write(
                "docs/tasks/active/2-api.md",
                "\
---
id: 2
title: API
kind: task
type: feature
status: planned
priority: high
specs: []
designs: []
adrs: []
depends_on: []
---

## Goal

Expose APIs.

## Result

Not implemented.
",
            );
        }

        fn write(&self, relative_path: &str, content: &str) {
            let path = self.root.join(relative_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
