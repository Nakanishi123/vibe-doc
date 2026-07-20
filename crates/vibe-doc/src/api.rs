//! `/api` 用のAxumルートと、Web UIへ公開する読み取り専用DTOを定義する。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use vibe_doc_core::document::Document;
use vibe_doc_core::index::DocumentIndex;
use vibe_doc_core::lint::{LintDiagnostic, lint};
use vibe_doc_core::next_index::{IndexedKind, next_index};
use vibe_doc_core::parser::{DocumentTree, parse_document_tree};

/// 起動時と再読込時に構築し、全APIハンドラーで共有する文書スナップショット。
#[derive(Clone)]
pub(crate) struct ApiState {
    document_root: PathBuf,
    snapshot: Arc<RwLock<ApiSnapshot>>,
}

struct ApiSnapshot {
    tree: DocumentTree,
    index: DocumentIndex,
    diagnostics: Vec<LintDiagnostic>,
}

impl ApiState {
    pub(crate) fn new(document_root: impl Into<PathBuf>) -> Self {
        let document_root = document_root.into();
        let tree = parse_document_tree(&document_root);
        let index = DocumentIndex::from_document_map(&tree.documents);
        let diagnostics = lint(&tree);
        Self {
            document_root,
            snapshot: Arc::new(RwLock::new(ApiSnapshot {
                tree,
                index,
                diagnostics,
            })),
        }
    }

    fn reload(&self) -> ReloadResponse {
        let tree = parse_document_tree(&self.document_root);
        let document_count = tree.documents.len();
        let index = DocumentIndex::from_document_map(&tree.documents);
        let diagnostics = lint(&tree);
        let diagnostic_count = diagnostics.len();
        let mut snapshot = self
            .snapshot
            .write()
            .expect("API snapshot lock should not be poisoned");
        *snapshot = ApiSnapshot {
            tree,
            index,
            diagnostics,
        };
        ReloadResponse {
            document_count,
            diagnostic_count,
        }
    }
}

pub(crate) fn router(state: ApiState) -> Router {
    Router::new()
        .route("/documents", get(list_documents))
        .route("/documents/{id}", get(get_document))
        .route("/tags", get(list_tags))
        .route("/tags/{tag}", get(get_tag))
        .route("/links", get(list_links))
        .route("/lint", get(get_lint))
        .route("/next-index/{kind}", get(get_next_index))
        .route("/reload", post(reload))
        .with_state(state)
}

#[derive(Debug, Deserialize, Default)]
struct DocumentQuery {
    q: Option<String>,
    kind: Option<String>,
    status: Option<String>,
    tag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentSummary {
    id: String,
    title: String,
    path: String,
    kind: Option<String>,
    status: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentDetail {
    #[serde(flatten)]
    summary: DocumentSummary,
    schema_version: Option<u32>,
    body: String,
    extra: BTreeMap<String, String>,
    related: Vec<DocumentSummary>,
    dependencies: Vec<DocumentSummary>,
    dependents: Vec<DocumentSummary>,
    backlinks: Vec<DocumentSummary>,
    body_links: Vec<DocumentSummary>,
}

#[derive(Debug, Serialize)]
struct TagSummary {
    name: String,
    count: usize,
}

#[derive(Debug, Serialize)]
struct TagDetail {
    name: String,
    documents: Vec<DocumentSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LinkSummary {
    source: DocumentSummary,
    target: DocumentSummary,
    relation: LinkRelation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum LinkRelation {
    Related,
    DependsOn,
    MarkdownLink,
}

#[derive(Debug, Serialize)]
struct LintResponse {
    errors: usize,
    warnings: usize,
    diagnostics: Vec<LintItem>,
}

#[derive(Debug, Serialize)]
struct LintItem {
    level: String,
    path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct NextIndexResponse {
    kind: String,
    index: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReloadResponse {
    document_count: usize,
    diagnostic_count: usize,
}

async fn list_documents(
    State(state): State<ApiState>,
    Query(query): Query<DocumentQuery>,
) -> Json<Vec<DocumentSummary>> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let documents: Vec<&Document> = query
        .q
        .as_deref()
        .filter(|query| !query.trim().is_empty())
        .map_or_else(
            || snapshot.index.documents().collect(),
            |query| snapshot.index.search(query.trim()),
        );
    Json(
        documents
            .into_iter()
            .filter(|document| {
                optional_eq(document.metadata.kind.as_deref(), query.kind.as_deref())
            })
            .filter(|document| {
                optional_eq(document.metadata.status.as_deref(), query.status.as_deref())
            })
            .filter(|document| {
                query
                    .tag
                    .as_ref()
                    .is_none_or(|tag| document.metadata.tags.iter().any(|value| value == tag))
            })
            .map(document_summary)
            .collect(),
    )
}

async fn get_document(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentDetail>, StatusCode> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let document = snapshot.index.document(&id).ok_or(StatusCode::NOT_FOUND)?;
    let dependencies = document
        .metadata
        .depends_on
        .iter()
        .filter_map(|id| snapshot.index.document(id))
        .map(document_summary)
        .collect();
    let body_links = snapshot
        .index
        .links()
        .iter()
        .filter(|link| link.source == document.path)
        .filter_map(|link| document_by_path(&snapshot.index, &link.target))
        .map(document_summary)
        .collect();

    Ok(Json(DocumentDetail {
        summary: document_summary(document),
        schema_version: document.metadata.schema_version,
        body: document.body.clone(),
        extra: document.metadata.extra.clone(),
        related: snapshot
            .index
            .related_documents(&id)
            .into_iter()
            .map(document_summary)
            .collect(),
        dependencies,
        dependents: snapshot
            .index
            .dependent_documents(&id)
            .into_iter()
            .map(document_summary)
            .collect(),
        backlinks: snapshot
            .index
            .backlink_documents(&id)
            .into_iter()
            .map(document_summary)
            .collect(),
        body_links,
    }))
}

async fn list_tags(State(state): State<ApiState>) -> Json<Vec<TagSummary>> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    Json(
        snapshot
            .index
            .tags()
            .map(|tag| TagSummary {
                name: tag.to_owned(),
                count: snapshot.index.documents_with_tag(tag).len(),
            })
            .collect(),
    )
}

async fn get_tag(
    State(state): State<ApiState>,
    Path(tag): Path<String>,
) -> Result<Json<TagDetail>, StatusCode> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let documents: Vec<_> = snapshot
        .index
        .documents_with_tag(&tag)
        .into_iter()
        .map(document_summary)
        .collect();
    if documents.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(TagDetail {
        name: tag,
        documents,
    }))
}

/// 索引にある三種類の関係を、重複しない有向エッジの一覧へ変換する。
///
/// `related` は対称関係として索引に保存されるためIDの辞書順で片側だけを採用する。
/// `depends_on` とMarkdownリンクは入力に方向があるため、その向きを保つ。UIがリンク元・
/// リンク先を同じ形式で描画できるよう、いずれも文書概要を含むDTOとして返す。
async fn list_links(State(state): State<ApiState>) -> Json<Vec<LinkSummary>> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let mut links = Vec::new();
    for source in snapshot.index.documents() {
        let source_id = source.metadata.id.as_deref().unwrap_or_default();
        for target in snapshot.index.related_documents(source_id) {
            let target_id = target.metadata.id.as_deref().unwrap_or_default();
            if source_id < target_id {
                links.push(link_summary(source, target, LinkRelation::Related));
            }
        }
        for dependency in &source.metadata.depends_on {
            if let Some(target) = snapshot.index.document(dependency) {
                links.push(link_summary(source, target, LinkRelation::DependsOn));
            }
        }
    }
    for link in snapshot.index.links() {
        if let (Some(source), Some(target)) = (
            document_by_path(&snapshot.index, &link.source),
            document_by_path(&snapshot.index, &link.target),
        ) {
            links.push(link_summary(source, target, LinkRelation::MarkdownLink));
        }
    }
    Json(links)
}

async fn get_lint(State(state): State<ApiState>) -> Json<LintResponse> {
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let errors = snapshot
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.level.as_str() == "error")
        .count();
    let warnings = snapshot.diagnostics.len() - errors;
    Json(LintResponse {
        errors,
        warnings,
        diagnostics: snapshot
            .diagnostics
            .iter()
            .map(|diagnostic| LintItem {
                level: diagnostic.level.as_str().to_owned(),
                path: path_string(&diagnostic.path),
                message: diagnostic.message.clone(),
            })
            .collect(),
    })
}

async fn get_next_index(
    State(state): State<ApiState>,
    Path(kind): Path<String>,
) -> Result<Json<NextIndexResponse>, StatusCode> {
    let indexed_kind = match kind.as_str() {
        "decision" => IndexedKind::Decision,
        "task" => IndexedKind::Task,
        "research" => IndexedKind::Research,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let snapshot = state
        .snapshot
        .read()
        .expect("API snapshot lock should not be poisoned");
    let index = next_index(indexed_kind, snapshot.tree.documents.values().cloned());
    Ok(Json(NextIndexResponse {
        kind,
        index: format!("{index:04}"),
    }))
}

async fn reload(State(state): State<ApiState>) -> Json<ReloadResponse> {
    Json(state.reload())
}

fn optional_eq(value: Option<&str>, filter: Option<&str>) -> bool {
    filter.is_none_or(|filter| value == Some(filter))
}

fn document_summary(document: &Document) -> DocumentSummary {
    DocumentSummary {
        id: document.metadata.id.clone().unwrap_or_default(),
        title: document
            .title
            .clone()
            .unwrap_or_else(|| "Untitled document".to_owned()),
        path: path_string(&document.path),
        kind: document.metadata.kind.clone(),
        status: document.metadata.status.clone(),
        tags: document.metadata.tags.clone(),
    }
}

fn path_string(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn document_by_path<'a>(index: &'a DocumentIndex, path: &std::path::Path) -> Option<&'a Document> {
    index.documents().find(|document| document.path == path)
}

fn link_summary(source: &Document, target: &Document, relation: LinkRelation) -> LinkSummary {
    LinkSummary {
        source: document_summary(source),
        target: document_summary(target),
        relation,
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiState, DocumentQuery, list_documents, reload};
    use axum::extract::{Query, State};
    use std::fs;

    #[tokio::test]
    async fn document_list_searches_and_filters_the_in_memory_index() {
        let root = std::env::temp_dir().join(format!("vibe-doc-api-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("task.md"),
            "---\nid: TASK-0010\nkind: task\nstatus: todo\ntags: [rust]\n---\n# Search target\nNeedle body\n",
        )
        .unwrap();
        let response = list_documents(
            State(ApiState::new(&root)),
            Query(DocumentQuery {
                q: Some("needle".into()),
                kind: Some("task".into()),
                ..Default::default()
            }),
        )
        .await;
        assert_eq!(response.0.len(), 1);
        assert_eq!(response.0[0].id, "TASK-0010");
        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn reload_rebuilds_the_in_memory_index_from_disk() {
        let root =
            std::env::temp_dir().join(format!("vibe-doc-api-reload-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("first.md"),
            "---\nid: ARCH-0001\nkind: architecture\n---\n# First\n",
        )
        .unwrap();
        let state = ApiState::new(&root);

        fs::write(
            root.join("second.md"),
            "---\nid: ARCH-0002\nkind: architecture\n---\n# Second\n",
        )
        .unwrap();
        let response = reload(State(state.clone())).await;
        let documents = list_documents(State(state), Query(DocumentQuery::default())).await;

        assert_eq!(response.0.document_count, 2);
        assert_eq!(documents.0.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
