//! API server and SPA host crate for vibe-doc.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    convert::Infallible,
    io,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use tokio_stream::wrappers::ReceiverStream;
use vibe_doc_core::{
    accept_agent_run, approve_agent_run_prompt, complete_task, execute_agent_run,
    preflight_agent_run_execution, prepare_agent_run, read_agent_run_artifact,
    read_agent_run_metadata, rebuild_task_index, reject_agent_run, scan_repository, start_task,
    task_context, validate_repository, write_agent_run_review, AdrMetadata, AdrStatus,
    AgentCommand, AgentRun, AgentRunStorageError, AgentRunStreamEvent, CompleteTaskOptions,
    DesignMetadata, DocumentId, DocumentMetadata, PrepareAgentRunOptions, Priority,
    RepositoryDocument, RepositoryScanError, SpecMetadata, TaskContextError, TaskContextItem,
    TaskContextItemKind, TaskGuardIssue, TaskGuardReport, TaskIndexRebuildError,
    TaskIndexRebuildOptions, TaskLifecycleError, TaskLifecycleOptions, TaskLifecyclePlan,
    TaskMetadata, TaskStatus, TaskType, ValidationIssue, ValidationRunError,
};

/// Stable crate identifier used by workspace smoke tests.
pub const CRATE_NAME: &str = "vibe-doc-server";

struct EmbeddedAsset {
    path: &'static str,
    bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

/// Shared server state.
#[derive(Debug, Clone)]
pub struct ServerState {
    repository_root: Arc<FsPath>,
    agent_commands: Arc<BTreeMap<String, AgentCommand>>,
}

impl ServerState {
    /// Create server state rooted at a vibe-doc repository.
    pub fn new(repository_root: impl Into<PathBuf>) -> Self {
        Self {
            repository_root: Arc::from(repository_root.into().into_boxed_path()),
            agent_commands: Arc::new(default_agent_commands()),
        }
    }

    pub fn with_agent_commands(
        repository_root: impl Into<PathBuf>,
        agent_commands: impl IntoIterator<Item = AgentCommand>,
    ) -> Self {
        Self {
            repository_root: Arc::from(repository_root.into().into_boxed_path()),
            agent_commands: Arc::new(agent_command_map(agent_commands)),
        }
    }

    fn root(&self) -> &FsPath {
        &self.repository_root
    }
}

/// Build the read-only API router for a vibe-doc repository.
pub fn api_router(repository_root: impl Into<PathBuf>) -> Router {
    api_routes().with_state(ServerState::new(repository_root))
}

pub fn api_router_with_agent_commands(
    repository_root: impl Into<PathBuf>,
    agent_commands: impl IntoIterator<Item = AgentCommand>,
) -> Router {
    api_routes().with_state(ServerState::with_agent_commands(
        repository_root,
        agent_commands,
    ))
}

/// Build the full application router with API routes and embedded SPA serving.
pub fn app_router(repository_root: impl Into<PathBuf>) -> Router {
    api_routes()
        .fallback(spa_fallback)
        .with_state(ServerState::new(repository_root))
}

/// A bound HTTP server ready to report its final listen address and run.
pub struct BoundServer {
    listener: tokio::net::TcpListener,
    router: Router,
    local_addr: SocketAddr,
}

impl BoundServer {
    /// Return the socket address selected by the OS.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Run this server until it is stopped.
    pub async fn serve(self) -> std::io::Result<()> {
        axum::serve(self.listener, self.router).await
    }
}

/// Bind the API and embedded SPA router without starting request handling.
pub async fn bind(
    repository_root: impl Into<PathBuf>,
    addr: SocketAddr,
) -> std::io::Result<BoundServer> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    Ok(BoundServer {
        listener,
        router: app_router(repository_root),
        local_addr,
    })
}

/// Serve the API and embedded SPA on an already parsed socket address.
pub async fn serve(repository_root: impl Into<PathBuf>, addr: SocketAddr) -> std::io::Result<()> {
    bind(repository_root, addr).await?.serve().await
}

fn api_routes() -> Router<ServerState> {
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
        .route("/api/tasks/{id}/prepare-codex", post(prepare_codex_run))
        .route(
            "/api/runs/{run_id}/approve-prompt",
            post(approve_prompt_endpoint),
        )
        .route("/api/runs/{run_id}", get(agent_run_detail_endpoint))
        .route("/api/runs/{run_id}/start", post(start_agent_run_endpoint))
        .route(
            "/api/runs/{run_id}/review",
            post(write_agent_review_endpoint),
        )
        .route("/api/runs/{run_id}/accept", post(accept_agent_run_endpoint))
        .route("/api/runs/{run_id}/reject", post(reject_agent_run_endpoint))
        .route("/api/tasks/{id}/start", post(start_task_endpoint))
        .route("/api/tasks/{id}/complete", post(complete_task_endpoint))
        .route(
            "/api/tasks/index/rebuild",
            post(rebuild_task_index_endpoint),
        )
}

fn default_agent_commands() -> BTreeMap<String, AgentCommand> {
    agent_command_map([AgentCommand {
        name: "codex".to_owned(),
        agent_kind: "codex".to_owned(),
        program: "codex".to_owned(),
        args: vec![
            "exec".to_owned(),
            "--skip-git-repo-check".to_owned(),
            "-".to_owned(),
        ],
        prompt_stdin: true,
    }])
}

fn agent_command_map(
    commands: impl IntoIterator<Item = AgentCommand>,
) -> BTreeMap<String, AgentCommand> {
    commands
        .into_iter()
        .map(|command| (command.name.clone(), command))
        .collect()
}

fn ndjson_event(event: &AgentRunStreamEvent) -> Vec<u8> {
    match serde_json::to_vec(event) {
        Ok(mut content) => {
            content.push(b'\n');
            content
        }
        Err(error) => format!(
            "{{\"event\":\"error\",\"message\":{}}}\n",
            serde_json::to_string(&error.to_string())
                .unwrap_or_else(|_| "\"failed to serialize stream event\"".to_owned())
        )
        .into_bytes(),
    }
}

async fn spa_fallback(method: Method, uri: Uri) -> Response {
    let path = uri.path();

    if path == "/api" || path.starts_with("/api/") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "API_ROUTE_NOT_FOUND",
                    "message": format!("API route `{path}` was not found"),
                }
            })),
        )
            .into_response();
    }

    if method != Method::GET && method != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    if !is_safe_asset_path(path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "INVALID_SPA_ASSET_PATH",
                    "message": "SPA asset path must stay within embedded assets",
                }
            })),
        )
            .into_response();
    }

    if EMBEDDED_ASSETS.is_empty() {
        return missing_spa_response();
    }

    let requested = path.trim_start_matches('/');
    let asset_path = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };

    if let Some(asset) = embedded_asset(asset_path) {
        return asset_response(asset, method == Method::HEAD);
    }

    if path_has_extension(asset_path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    match embedded_asset("index.html") {
        Some(asset) => asset_response(asset, method == Method::HEAD),
        None => missing_spa_response(),
    }
}

fn embedded_asset(path: &str) -> Option<&'static EmbeddedAsset> {
    EMBEDDED_ASSETS.iter().find(|asset| asset.path == path)
}

fn asset_response(asset: &EmbeddedAsset, head_only: bool) -> Response {
    let body = if head_only {
        Body::empty()
    } else {
        Body::from(asset.bytes)
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type(asset.path))
        .body(body)
        .expect("static asset response should be valid")
}

fn missing_spa_response() -> Response {
    let message = "vdoc server is running, but embedded Web UI assets were not found. Run the Web UI build so apps/web/dist exists before compiling this binary, or use the API routes under /api.";
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message))
        .expect("missing SPA response should be valid")
}

fn is_safe_asset_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    !path.contains('\\')
        && !path.contains("..")
        && !lower.contains("%2e")
        && !lower.contains("%5c")
        && path.starts_with('/')
}

fn path_has_extension(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|name| name.contains('.'))
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
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

async fn prepare_codex_run(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<PrepareAgentRunResponse>, ApiFailure> {
    let id = parse_document_id(&id, MissingCode::Task)?;
    let prepared = prepare_agent_run(
        state.root(),
        PrepareAgentRunOptions {
            task_id: id,
            agent_kind: "codex".to_owned(),
        },
    )
    .map_err(ApiFailure::AgentRun)?;

    Ok(Json(PrepareAgentRunResponse {
        run: agent_run_response(state.root(), &prepared.run),
        guard: task_guard_response(state.root(), &prepared.guard),
        prompt: prepared.prompt,
    }))
}

async fn approve_prompt_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
) -> Result<Json<AgentRunResponse>, ApiFailure> {
    let run = approve_agent_run_prompt(state.root(), run_id).map_err(ApiFailure::AgentRun)?;
    Ok(Json(agent_run_response(state.root(), &run)))
}

async fn agent_run_detail_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
) -> Result<Json<AgentRunDetailResponse>, ApiFailure> {
    let run = read_agent_run_metadata(state.root(), run_id).map_err(ApiFailure::AgentRun)?;
    Ok(Json(agent_run_detail_response(state.root(), &run)?))
}

async fn write_agent_review_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
    body: Option<Json<AgentReviewRequest>>,
) -> Result<Json<AgentRunDetailResponse>, ApiFailure> {
    let run = read_agent_run_metadata(state.root(), &run_id).map_err(ApiFailure::AgentRun)?;
    let request = body.map(|Json(request)| request).unwrap_or_default();
    let content = request.content.unwrap_or_else(|| {
        generated_review_content(
            &run,
            &read_agent_run_artifact(&run.artifacts.diff).unwrap_or_default(),
        )
    });
    write_agent_run_review(state.root(), &run_id, content).map_err(ApiFailure::AgentRun)?;
    let run = read_agent_run_metadata(state.root(), run_id).map_err(ApiFailure::AgentRun)?;
    Ok(Json(agent_run_detail_response(state.root(), &run)?))
}

async fn accept_agent_run_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
) -> Result<Json<AgentRunDetailResponse>, ApiFailure> {
    let run = accept_agent_run(state.root(), run_id).map_err(ApiFailure::AgentRun)?;
    Ok(Json(agent_run_detail_response(state.root(), &run)?))
}

async fn reject_agent_run_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
) -> Result<Json<AgentRunDetailResponse>, ApiFailure> {
    let run = reject_agent_run(state.root(), run_id).map_err(ApiFailure::AgentRun)?;
    Ok(Json(agent_run_detail_response(state.root(), &run)?))
}

async fn start_agent_run_endpoint(
    State(state): State<ServerState>,
    Path(run_id): Path<String>,
    body: Option<Json<StartAgentRunRequest>>,
) -> Result<Response, ApiFailure> {
    let request = body.map(|Json(request)| request).unwrap_or_default();
    let command_name = request.command.unwrap_or_else(|| "codex".to_owned());
    let command = state
        .agent_commands
        .get(&command_name)
        .cloned()
        .ok_or_else(|| {
            ApiFailure::AgentRun(AgentRunStorageError::UnsupportedAgentCommand {
                run_id: run_id.clone(),
                command: command_name.clone(),
            })
        })?;
    preflight_agent_run_execution(state.root(), &run_id, &command).map_err(ApiFailure::AgentRun)?;

    let root = state.root().to_path_buf();
    let (sender, receiver) = tokio::sync::mpsc::channel::<Result<Vec<u8>, Infallible>>(16);
    tokio::task::spawn_blocking(move || {
        let stream_sender = sender.clone();
        let result = execute_agent_run(root, &run_id, &command, |event| {
            let _ = stream_sender.blocking_send(Ok(ndjson_event(event)));
        });
        match result {
            Ok(execution) => {
                let _ = sender.blocking_send(Ok(ndjson_event(&AgentRunStreamEvent::Completed {
                    status: execution.run.status,
                    exit_result: execution.run.exit_result,
                })));
            }
            Err(error) => {
                let _ = sender.blocking_send(Ok(ndjson_event(&AgentRunStreamEvent::Error {
                    message: error.to_string(),
                })));
            }
        }
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-ndjson")],
        Body::from_stream(ReceiverStream::new(receiver)),
    )
        .into_response())
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

fn agent_run_response(root: &FsPath, run: &AgentRun) -> AgentRunResponse {
    AgentRunResponse {
        task_id: run.task_id.get(),
        run_id: run.run_id.clone(),
        agent_kind: run.agent_kind.clone(),
        status: run.status.as_str().to_owned(),
        worktree_path: run
            .worktree_path
            .as_ref()
            .map(|path| display_path(&relative_path(root, path))),
        created_at: run.created_at.clone(),
        updated_at: run.updated_at.clone(),
        artifacts: AgentRunArtifactsResponse {
            directory: display_path(&relative_path(root, &run.artifacts.directory)),
            run_json: display_path(&relative_path(root, &run.artifacts.run_json)),
            prompt: display_path(&relative_path(root, &run.artifacts.prompt)),
            events: display_path(&relative_path(root, &run.artifacts.events)),
            terminal_log: display_path(&relative_path(root, &run.artifacts.terminal_log)),
            diff: display_path(&relative_path(root, &run.artifacts.diff)),
            review: display_path(&relative_path(root, &run.artifacts.review)),
        },
    }
}

fn agent_run_detail_response(
    root: &FsPath,
    run: &AgentRun,
) -> Result<AgentRunDetailResponse, ApiFailure> {
    Ok(AgentRunDetailResponse {
        run: agent_run_response(root, run),
        prompt: read_agent_run_artifact(&run.artifacts.prompt).map_err(ApiFailure::AgentRun)?,
        terminal_log: read_agent_run_artifact(&run.artifacts.terminal_log)
            .map_err(ApiFailure::AgentRun)?,
        diff: read_agent_run_artifact(&run.artifacts.diff).map_err(ApiFailure::AgentRun)?,
        review: read_agent_run_artifact(&run.artifacts.review).map_err(ApiFailure::AgentRun)?,
    })
}

fn generated_review_content(run: &AgentRun, diff: &str) -> String {
    let changed_files = diff
        .lines()
        .filter_map(|line| line.strip_prefix("diff --git a/"))
        .filter_map(|line| line.split(" b/").next())
        .collect::<Vec<_>>();
    let file_summary = if changed_files.is_empty() {
        "No changed files were captured in diff.patch.".to_owned()
    } else {
        format!("Changed files: {}.", changed_files.join(", "))
    };

    format!(
        "# Agent Run Review\n\nRun `{}` finished with status `{}`.\n\n{}\n\nReview decision still requires explicit user acceptance or rejection.\n",
        run.run_id, run.status, file_summary
    )
}

fn task_guard_response(root: &FsPath, report: &TaskGuardReport) -> TaskGuardResponse {
    TaskGuardResponse {
        task_id: report.task_id.get(),
        ready: report.ready,
        issues: report
            .issues
            .iter()
            .map(|issue| task_guard_issue_response(root, issue))
            .collect(),
    }
}

fn task_guard_issue_response(root: &FsPath, issue: &TaskGuardIssue) -> TaskGuardIssueResponse {
    TaskGuardIssueResponse {
        code: issue.code.as_str().to_owned(),
        message: issue.message.clone(),
        document_id: issue.document_id.map(DocumentId::get),
        path: issue
            .path
            .as_ref()
            .map(|path| display_path(&relative_path(root, path))),
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

#[derive(Debug, Serialize)]
struct PrepareAgentRunResponse {
    run: AgentRunResponse,
    guard: TaskGuardResponse,
    prompt: String,
}

#[derive(Debug, Serialize)]
struct AgentRunResponse {
    task_id: u64,
    run_id: String,
    agent_kind: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    worktree_path: Option<String>,
    created_at: String,
    updated_at: String,
    artifacts: AgentRunArtifactsResponse,
}

#[derive(Debug, Serialize)]
struct AgentRunDetailResponse {
    run: AgentRunResponse,
    prompt: String,
    terminal_log: String,
    diff: String,
    review: String,
}

#[derive(Debug, Serialize)]
struct AgentRunArtifactsResponse {
    directory: String,
    run_json: String,
    prompt: String,
    events: String,
    terminal_log: String,
    diff: String,
    review: String,
}

#[derive(Debug, Serialize)]
struct TaskGuardResponse {
    task_id: u64,
    ready: bool,
    issues: Vec<TaskGuardIssueResponse>,
}

#[derive(Debug, Serialize)]
struct TaskGuardIssueResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct StartAgentRunRequest {
    command: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct AgentReviewRequest {
    content: Option<String>,
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
    #[error(transparent)]
    AgentRun(#[from] AgentRunStorageError),
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
        if let Self::AgentRun(AgentRunStorageError::GuardFailed { report }) = self {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": {
                        "code": "AGENT_RUN_GUARD_FAILED",
                        "message": format!("task guard failed for task {}", report.task_id.get()),
                        "document_id": report.task_id.get(),
                    },
                    "guard": task_guard_response(FsPath::new(""), &report),
                })),
            )
                .into_response();
        }

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
            Self::AgentRun(error) => agent_run_api_error(error),
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
                path: Some(display_path(path)),
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
                path: Some(display_path(path)),
                document_id: None,
            },
        ),
    }
}

fn agent_run_api_error(error: AgentRunStorageError) -> (StatusCode, ApiError) {
    match error {
        AgentRunStorageError::RepositoryRootNotFound { .. } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "AGENT_RUN_REPOSITORY_ROOT_NOT_FOUND",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        AgentRunStorageError::UnsafeRunId { .. } => (
            StatusCode::BAD_REQUEST,
            ApiError {
                code: "INVALID_AGENT_RUN_ID",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        AgentRunStorageError::UnsupportedAgentCommand { .. } => (
            StatusCode::BAD_REQUEST,
            ApiError {
                code: "AGENT_RUN_UNSUPPORTED_COMMAND",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        AgentRunStorageError::ReadFile {
            ref path,
            ref source,
        } if source.kind() == io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            ApiError {
                code: "AGENT_RUN_NOT_FOUND",
                message: error.to_string(),
                path: Some(display_path(path)),
                document_id: None,
            },
        ),
        AgentRunStorageError::InvalidRunStatus { .. } => (
            StatusCode::CONFLICT,
            ApiError {
                code: "AGENT_RUN_INVALID_STATUS",
                message: error.to_string(),
                path: None,
                document_id: None,
            },
        ),
        AgentRunStorageError::TaskContext(error) => match error {
            TaskContextError::TaskNotFound { id } => (
                StatusCode::NOT_FOUND,
                ApiError {
                    code: "TASK_NOT_FOUND",
                    message: format!("task {} was not found", id.get()),
                    path: None,
                    document_id: Some(id.get()),
                },
            ),
            TaskContextError::MissingRelatedDocument { id, .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiError {
                    code: "TASK_CONTEXT_MISSING_RELATED_DOCUMENT",
                    message: error.to_string(),
                    path: None,
                    document_id: Some(id.get()),
                },
            ),
            TaskContextError::RepositoryScan(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError {
                    code: "TASK_CONTEXT_SCAN_FAILED",
                    message: error.to_string(),
                    path: None,
                    document_id: None,
                },
            ),
        },
        AgentRunStorageError::GuardFailed { .. } => unreachable!("handled before error mapping"),
        AgentRunStorageError::CreateDir { ref path, .. }
        | AgentRunStorageError::WriteFile { ref path, .. }
        | AgentRunStorageError::ReadFile { ref path, .. }
        | AgentRunStorageError::Deserialize { ref path, .. } => (
            StatusCode::CONFLICT,
            ApiError {
                code: "AGENT_RUN_ARTIFACT_IO_FAILED",
                message: error.to_string(),
                path: Some(display_path(path)),
                document_id: None,
            },
        ),
        AgentRunStorageError::Serialize(_)
        | AgentRunStorageError::ExhaustedRunIds { .. }
        | AgentRunStorageError::UnsafeWorktreePath { .. }
        | AgentRunStorageError::WorktreePathExists { .. }
        | AgentRunStorageError::GitWorktree { .. }
        | AgentRunStorageError::MissingWorktree { .. }
        | AgentRunStorageError::SpawnAgentCommand { .. }
        | AgentRunStorageError::ReadAgentOutput(_)
        | AgentRunStorageError::WriteAgentInput(_)
        | AgentRunStorageError::AgentInputWriterPanicked
        | AgentRunStorageError::AgentOutputReaderPanicked
        | AgentRunStorageError::CaptureDiff { .. } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError {
                code: "AGENT_RUN_FAILED",
                message: error.to_string(),
                path: None,
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
        http::{HeaderMap, Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use std::{
        fs,
        process::Command,
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

    #[tokio::test]
    async fn prepare_codex_endpoint_creates_prompt_and_run_artifacts() {
        let repo = TestRepo::new("prepare-codex");
        repo.seed();

        let response = post_json(
            api_router(repo.path()),
            "/api/tasks/28/prepare-codex",
            json!({}),
        )
        .await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["run"]["task_id"], 28);
        assert_eq!(response.body["run"]["agent_kind"], "codex");
        assert_eq!(response.body["run"]["status"], "prepared");
        assert_eq!(response.body["guard"]["ready"], true);
        assert!(response.body["prompt"]
            .as_str()
            .unwrap()
            .contains("### Task 28: API"));
        assert_eq!(
            response.body["run"]["artifacts"]["prompt"],
            format!(
                "{}/prompt.md",
                response.body["run"]["artifacts"]["directory"]
                    .as_str()
                    .unwrap()
            )
        );

        let prompt_path = repo.root.join(
            response.body["run"]["artifacts"]["prompt"]
                .as_str()
                .unwrap(),
        );
        let prompt = fs::read_to_string(prompt_path).unwrap();
        assert_eq!(prompt, response.body["prompt"].as_str().unwrap());
        let run_json_path = repo.root.join(
            response.body["run"]["artifacts"]["run_json"]
                .as_str()
                .unwrap(),
        );
        assert!(run_json_path.is_file());
    }

    #[tokio::test]
    async fn prepare_codex_endpoint_returns_guard_failure_without_artifacts() {
        let repo = TestRepo::new("prepare-codex-guard");
        repo.seed();

        let response = post_json(
            api_router(repo.path()),
            "/api/tasks/27/prepare-codex",
            json!({}),
        )
        .await;

        assert_eq!(response.status, StatusCode::CONFLICT);
        assert_eq!(response.body["error"]["code"], "AGENT_RUN_GUARD_FAILED");
        assert_eq!(response.body["guard"]["ready"], false);
        assert!(response.body["guard"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "TASK_NOT_ACTIVE"));
        assert!(!repo.root.join(".vdoc/runs").exists());
    }

    #[tokio::test]
    async fn approve_prompt_endpoint_updates_run_and_records_event() {
        let repo = TestRepo::new("approve-prompt");
        repo.seed();
        let prepared = post_json(
            api_router(repo.path()),
            "/api/tasks/28/prepare-codex",
            json!({}),
        )
        .await;
        let run_id = prepared.body["run"]["run_id"].as_str().unwrap();

        let approved = post_json(
            api_router(repo.path()),
            &format!("/api/runs/{run_id}/approve-prompt"),
            json!({}),
        )
        .await;

        assert_eq!(approved.status, StatusCode::OK);
        assert_eq!(approved.body["status"], "prompt-approved");
        let events_path = repo
            .root
            .join(approved.body["artifacts"]["events"].as_str().unwrap());
        let events = fs::read_to_string(events_path).unwrap();
        assert!(events.contains("\"event\":\"prompt-approved\""));

        let second = post_json(
            api_router(repo.path()),
            &format!("/api/runs/{run_id}/approve-prompt"),
            json!({}),
        )
        .await;
        assert_eq!(second.status, StatusCode::CONFLICT);
        assert_eq!(second.body["error"]["code"], "AGENT_RUN_INVALID_STATUS");
    }

    #[tokio::test]
    async fn start_agent_run_endpoint_streams_output_and_records_artifacts() {
        let repo = TestRepo::new("start-agent-run");
        repo.seed();
        repo.init_git();
        let router = api_router_with_agent_commands(
            repo.path(),
            [AgentCommand {
                name: "fixture".to_owned(),
                agent_kind: "codex".to_owned(),
                program: "sh".to_owned(),
                args: vec![
                    "-c".to_owned(),
                    "printf 'streamed output\\n'; printf 'changed\\n' > server-output.txt"
                        .to_owned(),
                ],
                prompt_stdin: false,
            }],
        );
        let prepared = post_json(router.clone(), "/api/tasks/28/prepare-codex", json!({})).await;
        let run_id = prepared.body["run"]["run_id"].as_str().unwrap();
        post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/approve-prompt"),
            json!({}),
        )
        .await;

        let response = post_raw(
            router,
            &format!("/api/runs/{run_id}/start"),
            json!({ "command": "fixture" }),
        )
        .await;

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("application/x-ndjson")
        );
        let body = String::from_utf8(response.body).unwrap();
        let events: Vec<Value> = body
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(events.iter().any(|event| event["event"] == "terminal"
            && event["data"]
                .as_str()
                .is_some_and(|data| data.contains("streamed output"))));
        assert!(events.iter().any(|event| event["event"] == "completed"
            && event["status"] == "succeeded"
            && event["exit_result"]["code"] == 0));

        let run_json: AgentRun = serde_json::from_str(
            &fs::read_to_string(repo.root.join(format!(".vdoc/runs/{run_id}/run.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(run_json.status, vibe_doc_core::AgentRunStatus::Succeeded);
        assert_eq!(run_json.exit_result.unwrap().code, Some(0));
        let terminal_log =
            fs::read_to_string(repo.root.join(format!(".vdoc/runs/{run_id}/terminal.log")))
                .unwrap();
        assert!(terminal_log.contains("streamed output"));
        let diff =
            fs::read_to_string(repo.root.join(format!(".vdoc/runs/{run_id}/diff.patch"))).unwrap();
        assert!(diff.contains("server-output.txt"));
        assert!(diff.contains("+changed"));
    }

    #[tokio::test]
    async fn start_agent_run_endpoint_streams_structured_errors() {
        let repo = TestRepo::new("start-agent-run-stream-error");
        repo.seed();
        repo.init_git();
        let router = api_router_with_agent_commands(
            repo.path(),
            [AgentCommand {
                name: "fixture".to_owned(),
                agent_kind: "codex".to_owned(),
                program: "missing-vdoc-test-command".to_owned(),
                args: vec![],
                prompt_stdin: false,
            }],
        );
        let prepared = post_json(router.clone(), "/api/tasks/28/prepare-codex", json!({})).await;
        let run_id = prepared.body["run"]["run_id"].as_str().unwrap();
        post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/approve-prompt"),
            json!({}),
        )
        .await;

        let response = post_raw(
            router,
            &format!("/api/runs/{run_id}/start"),
            json!({ "command": "fixture" }),
        )
        .await;

        assert_eq!(response.status, StatusCode::OK);
        let body = String::from_utf8(response.body).unwrap();
        let events: Vec<Value> = body
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(events.iter().any(|event| {
            event["event"] == "error"
                && event["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("failed to spawn agent command"))
        }));

        let run_json: AgentRun = serde_json::from_str(
            &fs::read_to_string(repo.root.join(format!(".vdoc/runs/{run_id}/run.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(run_json.status, vibe_doc_core::AgentRunStatus::Failed);
    }

    #[tokio::test]
    async fn start_agent_run_endpoint_rejects_unapproved_or_unknown_commands() {
        let repo = TestRepo::new("start-agent-run-errors");
        repo.seed();
        let router = api_router_with_agent_commands(
            repo.path(),
            [AgentCommand {
                name: "fixture".to_owned(),
                agent_kind: "codex".to_owned(),
                program: "sh".to_owned(),
                args: vec!["-c".to_owned(), "true".to_owned()],
                prompt_stdin: false,
            }],
        );
        let prepared = post_json(router.clone(), "/api/tasks/28/prepare-codex", json!({})).await;
        let run_id = prepared.body["run"]["run_id"].as_str().unwrap();

        let unapproved = post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/start"),
            json!({ "command": "fixture" }),
        )
        .await;
        assert_eq!(unapproved.status, StatusCode::CONFLICT);
        assert_eq!(unapproved.body["error"]["code"], "AGENT_RUN_INVALID_STATUS");

        post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/approve-prompt"),
            json!({}),
        )
        .await;
        let unsupported = post_json(
            router,
            &format!("/api/runs/{run_id}/start"),
            json!({ "command": "missing" }),
        )
        .await;
        assert_eq!(unsupported.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            unsupported.body["error"]["code"],
            "AGENT_RUN_UNSUPPORTED_COMMAND"
        );
    }

    #[tokio::test]
    async fn agent_run_detail_review_and_acceptance_endpoints_update_review_state_only() {
        let repo = TestRepo::new("agent-run-review");
        repo.seed();
        let router = api_router(repo.path());
        let prepared = post_json(router.clone(), "/api/tasks/28/prepare-codex", json!({})).await;
        let run_id = prepared.body["run"]["run_id"].as_str().unwrap();
        let run_json_path = repo.root.join(format!(".vdoc/runs/{run_id}/run.json"));
        let mut run: AgentRun =
            serde_json::from_str(&fs::read_to_string(&run_json_path).unwrap()).unwrap();
        run.status = vibe_doc_core::AgentRunStatus::Succeeded;
        vibe_doc_core::write_agent_run_metadata(&run).unwrap();
        fs::write(&run.artifacts.terminal_log, "terminal output\n").unwrap();
        fs::write(
            &run.artifacts.diff,
            "diff --git a/server-output.txt b/server-output.txt\n+changed\n",
        )
        .unwrap();

        let detail = request_json(router.clone(), &format!("/api/runs/{run_id}")).await;
        assert_eq!(detail.status, StatusCode::OK);
        assert_eq!(detail.body["run"]["status"], "succeeded");
        assert!(detail.body["prompt"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("### Task 28: API")));
        assert_eq!(detail.body["terminal_log"], "terminal output\n");
        assert!(detail.body["diff"]
            .as_str()
            .is_some_and(|diff| diff.contains("server-output.txt")));

        let reviewed = post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/review"),
            json!({}),
        )
        .await;
        assert_eq!(reviewed.status, StatusCode::OK);
        assert!(reviewed.body["review"]
            .as_str()
            .is_some_and(|review| review.contains("Changed files: server-output.txt.")));
        assert!(repo
            .root
            .join(format!(".vdoc/runs/{run_id}/review.md"))
            .is_file());

        let accepted = post_json(
            router.clone(),
            &format!("/api/runs/{run_id}/accept"),
            json!({}),
        )
        .await;
        assert_eq!(accepted.status, StatusCode::OK);
        assert_eq!(accepted.body["run"]["status"], "accepted");
        let task = fs::read_to_string(repo.root.join("docs/tasks/active/28-api.md")).unwrap();
        assert!(task.contains("status: planned"));

        let second_accept =
            post_json(router, &format!("/api/runs/{run_id}/accept"), json!({})).await;
        assert_eq!(second_accept.status, StatusCode::CONFLICT);
        assert_eq!(
            second_accept.body["error"]["code"],
            "AGENT_RUN_INVALID_STATUS"
        );
    }

    #[tokio::test]
    async fn app_router_serves_embedded_spa_assets_and_browser_routes() {
        let repo = TestRepo::new("spa");
        repo.seed();

        let index = request(app_router(repo.path()), "/").await;
        if EMBEDDED_ASSETS.is_empty() {
            assert_eq!(index.status, StatusCode::SERVICE_UNAVAILABLE);
            assert!(String::from_utf8_lossy(&index.body).contains("embedded Web UI assets"));
            return;
        }

        assert_eq!(index.status, StatusCode::OK);
        assert_eq!(
            index
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("text/html; charset=utf-8")
        );
        let body = String::from_utf8_lossy(&index.body);
        assert!(body.contains("<div id=\"root\"></div>"));

        let script_path = EMBEDDED_ASSETS
            .iter()
            .find(|asset| asset.path.ends_with(".js"))
            .map(|asset| format!("/{}", asset.path))
            .expect("test fixture should include a JavaScript asset");
        let script = request(app_router(repo.path()), &script_path).await;
        assert_eq!(script.status, StatusCode::OK);
        assert_eq!(
            script
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("text/javascript; charset=utf-8")
        );

        let browser_route = request(app_router(repo.path()), "/tasks/28").await;
        assert_eq!(browser_route.status, StatusCode::OK);
        assert!(String::from_utf8_lossy(&browser_route.body).contains("<div id=\"root\"></div>"));
    }

    #[tokio::test]
    async fn app_router_does_not_serve_spa_for_api_fallthrough_or_traversal() {
        let repo = TestRepo::new("spa-errors");
        repo.seed();

        let api_missing = request(app_router(repo.path()), "/api/not-found").await;
        assert_eq!(api_missing.status, StatusCode::NOT_FOUND);
        let api_error: Value = serde_json::from_slice(&api_missing.body).unwrap();
        assert_eq!(api_error["error"]["code"], "API_ROUTE_NOT_FOUND");

        let traversal = request(app_router(repo.path()), "/%2E%2E/index.html").await;
        assert_eq!(traversal.status, StatusCode::BAD_REQUEST);
        let traversal_error: Value = serde_json::from_slice(&traversal.body).unwrap();
        assert_eq!(traversal_error["error"]["code"], "INVALID_SPA_ASSET_PATH");
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

    async fn post_raw(router: Router, uri: &str, body: Value) -> RawTestResponse {
        request_with_body(router, Method::POST, uri, body.to_string()).await
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
        let headers = response.headers().clone();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        RawTestResponse {
            status,
            headers,
            body: body.to_vec(),
        }
    }

    struct TestResponse {
        status: StatusCode,
        body: Value,
    }

    struct RawTestResponse {
        status: StatusCode,
        headers: HeaderMap,
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

        fn init_git(&self) {
            self.run_git(["init"]);
            self.run_git(["config", "user.email", "vibe-doc@example.invalid"]);
            self.run_git(["config", "user.name", "vibe-doc tests"]);
            self.run_git(["config", "commit.gpgsign", "false"]);
            self.run_git(["add", "."]);
            self.run_git(["commit", "-m", "Initial commit"]);
        }

        fn run_git<const N: usize>(&self, args: [&str; N]) {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.root)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
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
