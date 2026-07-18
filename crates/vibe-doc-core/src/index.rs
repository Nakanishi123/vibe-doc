//! ID、タグ、全文検索、逆リンクのインメモリ索引を定義する。

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::document::Document;
use crate::links::{ManagedLink, managed_links};

/// 文書を横断して検索・集計するための、永続化しない索引。
///
/// IDのない文書は管理対象文書として参照できないため除く。IDが重複する場合はパス順で
/// 最初の文書を採用する。重複の診断はlintの責務であり、この型は各IDを一意に扱う。
#[derive(Debug, Clone, Default)]
pub struct DocumentIndex {
    documents: BTreeMap<String, Document>,
    tags: BTreeMap<String, BTreeSet<String>>,
    related: BTreeMap<String, BTreeSet<String>>,
    dependents: BTreeMap<String, BTreeSet<String>>,
    backlinks: BTreeMap<String, BTreeSet<String>>,
    links: Vec<ManagedLink>,
}

impl DocumentIndex {
    /// 文書集合からすべての一覧・逆引きを再構築する。
    ///
    /// まずIDを持つ文書だけを確定させ、その集合を基準にタグ、`related`、
    /// `depends_on` を解決する。存在しないIDへの参照は索引へ入れない。これは
    /// 不完全な編集中でも利用側へ壊れた表示項目を渡さず、問題の報告をlintに
    /// 一元化するためである。最後に文書パスを基準に本文リンクを解決し、同じ
    /// リンクや関係が複数回書かれていても `BTreeSet` により一件だけ保持する。
    pub fn new(documents: impl IntoIterator<Item = Document>) -> Self {
        let documents_by_path: BTreeMap<PathBuf, Document> = documents
            .into_iter()
            .map(|document| (document.path.clone(), document))
            .collect();
        let paths: Vec<_> = documents_by_path.keys().cloned().collect();
        let mut index = Self::default();
        for document in documents_by_path.values() {
            if let Some(id) = &document.metadata.id {
                index
                    .documents
                    .entry(id.clone())
                    .or_insert_with(|| document.clone());
            }
        }
        for (id, document) in &index.documents {
            for tag in &document.metadata.tags {
                index
                    .tags
                    .entry(tag.clone())
                    .or_default()
                    .insert(id.clone());
            }
            for target in &document.metadata.related {
                if index.documents.contains_key(target) {
                    index
                        .related
                        .entry(id.clone())
                        .or_default()
                        .insert(target.clone());
                    index
                        .related
                        .entry(target.clone())
                        .or_default()
                        .insert(id.clone());
                }
            }
            for dependency in &document.metadata.depends_on {
                if index.documents.contains_key(dependency) {
                    index
                        .dependents
                        .entry(dependency.clone())
                        .or_default()
                        .insert(id.clone());
                }
            }
        }
        let path_to_id: BTreeMap<_, _> = index
            .documents
            .values()
            .filter_map(|document| {
                document
                    .metadata
                    .id
                    .clone()
                    .map(|id| (document.path.clone(), id))
            })
            .collect();
        let mut resolved_links = BTreeSet::new();
        for document in index.documents.values() {
            for link in managed_links(&document.path, &document.body, paths.iter()) {
                if let (Some(source), Some(target)) =
                    (path_to_id.get(&link.source), path_to_id.get(&link.target))
                {
                    index
                        .backlinks
                        .entry(target.clone())
                        .or_default()
                        .insert(source.clone());
                    resolved_links.insert(link);
                }
            }
        }
        index.links = resolved_links.into_iter().collect();
        index
    }
    pub fn from_document_map(documents: &BTreeMap<PathBuf, Document>) -> Self {
        Self::new(documents.values().cloned())
    }
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }
    pub fn tags(&self) -> impl Iterator<Item = &str> {
        self.tags.keys().map(String::as_str)
    }
    pub fn documents_with_tag(&self, tag: &str) -> Vec<&Document> {
        self.document_ids(&self.tags, tag)
    }
    /// ID、タイトル、タグ、本文を対象に大文字・小文字を区別しない部分一致検索を行う。
    pub fn search(&self, query: &str) -> Vec<&Document> {
        let query = query.to_lowercase();
        self.documents()
            .filter(|document| {
                document
                    .metadata
                    .id
                    .as_deref()
                    .is_some_and(|id| id.to_lowercase().contains(&query))
                    || document
                        .title
                        .as_deref()
                        .is_some_and(|title| title.to_lowercase().contains(&query))
                    || document
                        .metadata
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query))
                    || document.body.to_lowercase().contains(&query)
            })
            .collect()
    }
    pub fn related_documents(&self, id: &str) -> Vec<&Document> {
        self.document_ids(&self.related, id)
    }
    pub fn dependent_documents(&self, id: &str) -> Vec<&Document> {
        self.document_ids(&self.dependents, id)
    }
    pub fn backlink_documents(&self, id: &str) -> Vec<&Document> {
        self.document_ids(&self.backlinks, id)
    }
    pub fn links(&self) -> &[ManagedLink] {
        &self.links
    }
    fn document_ids<'a>(
        &'a self,
        relationships: &BTreeMap<String, BTreeSet<String>>,
        id: &str,
    ) -> Vec<&'a Document> {
        relationships
            .get(id)
            .into_iter()
            .flatten()
            .filter_map(|related_id| self.document(related_id))
            .collect()
    }
}

impl FromIterator<Document> for DocumentIndex {
    fn from_iter<T: IntoIterator<Item = Document>>(iter: T) -> Self {
        Self::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentIndex;
    use crate::document::{Document, Metadata};
    use std::path::PathBuf;
    fn document(id: &str, path: &str, title: &str, body: &str) -> Document {
        Document {
            path: PathBuf::from(path),
            metadata: Metadata {
                id: Some(id.to_owned()),
                ..Default::default()
            },
            title: Some(title.to_owned()),
            body: body.to_owned(),
        }
    }
    #[test]
    fn builds_tag_search_and_relationship_indexes_without_duplicates() {
        let mut first = document(
            "TASK-0010",
            "docs/tasks/first.md",
            "First title",
            "A [model](../architecture/model.md).",
        );
        first.metadata.tags = vec!["rust".into(), "rust".into()];
        first.metadata.related = vec!["ARCH-0001".into(), "ARCH-0001".into()];
        let mut second = document(
            "ARCH-0001",
            "docs/architecture/model.md",
            "Model",
            "Architecture body",
        );
        second.metadata.tags = vec!["design".into(), "rust".into()];
        let mut third = document("TASK-0020", "docs/tasks/third.md", "Dependent", "Needle");
        third.metadata.depends_on = vec!["TASK-0010".into(), "TASK-0010".into()];
        let index = DocumentIndex::new([first, second, third]);
        assert_eq!(index.tags().collect::<Vec<_>>(), ["design", "rust"]);
        assert_eq!(index.documents_with_tag("rust").len(), 2);
        assert_eq!(
            index.search("nEeDlE")[0].metadata.id.as_deref(),
            Some("TASK-0020")
        );
        assert_eq!(
            index.related_documents("ARCH-0001")[0]
                .metadata
                .id
                .as_deref(),
            Some("TASK-0010")
        );
        assert_eq!(
            index.dependent_documents("TASK-0010")[0]
                .metadata
                .id
                .as_deref(),
            Some("TASK-0020")
        );
        assert_eq!(
            index.backlink_documents("ARCH-0001")[0]
                .metadata
                .id
                .as_deref(),
            Some("TASK-0010")
        );
    }
}
