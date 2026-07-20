//! Decision・Task・Researchの連番を計算する処理を定義する。

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::document::Document;

/// 採番対象となる文書種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexedKind {
    Decision,
    Task,
    Research,
}

impl IndexedKind {
    fn id_prefix(self) -> &'static str {
        match self {
            Self::Decision => "DEC-",
            Self::Task => "TASK-",
            Self::Research => "RES-",
        }
    }

    fn directory_name(self) -> &'static str {
        match self {
            Self::Decision => "decisions",
            Self::Task => "tasks",
            Self::Research => "research",
        }
    }

    fn increment(self) -> u32 {
        match self {
            Self::Decision => 1,
            Self::Task => 10,
            Self::Research => 1,
        }
    }
}

/// 指定できない採番対象が渡された際のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidIndexedKind;

impl fmt::Display for InvalidIndexedKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("kind must be `decision`, `task`, or `research`")
    }
}

impl std::error::Error for InvalidIndexedKind {}

impl FromStr for IndexedKind {
    type Err = InvalidIndexedKind;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "decision" => Ok(Self::Decision),
            "task" => Ok(Self::Task),
            "research" => Ok(Self::Research),
            _ => Err(InvalidIndexedKind),
        }
    }
}

/// 文書のIDまたは対象ディレクトリ内のファイル名から次の採番候補を求める。
///
/// IDが未記入の編集中の文書も、`decisions/` または `tasks/` 以下にあり先頭が数値の
/// ファイル名なら採番済み候補として扱う。これにより、Front Matterの記入途中でも次の
/// 番号が重複しにくくなる。候補が一つもなければ各種別の開始番号を返す。
pub fn next_index(kind: IndexedKind, documents: impl IntoIterator<Item = Document>) -> u32 {
    let maximum = documents
        .into_iter()
        .filter_map(|document| document_number(kind, &document))
        .max()
        .unwrap_or(0);
    maximum + kind.increment()
}

fn document_number(kind: IndexedKind, document: &Document) -> Option<u32> {
    let id_number = document
        .metadata
        .id
        .as_deref()
        .and_then(|id| id.strip_prefix(kind.id_prefix()))
        .and_then(parse_number);
    let filename_number = is_under_kind_directory(kind, &document.path)
        .then(|| document.path.file_stem())
        .flatten()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.split_once('-').map_or(stem, |(number, _)| number))
        .and_then(parse_number);
    id_number.into_iter().chain(filename_number).max()
}

fn is_under_kind_directory(kind: IndexedKind, path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == kind.directory_name())
}

fn parse_number(value: &str) -> Option<u32> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::{IndexedKind, next_index};
    use crate::document::{Document, Metadata};
    use std::path::PathBuf;

    fn document(path: &str, id: Option<&str>) -> Document {
        Document {
            path: PathBuf::from(path),
            metadata: Metadata {
                id: id.map(str::to_owned),
                ..Default::default()
            },
            title: None,
            body: String::new(),
        }
    }

    #[test]
    fn calculates_kind_specific_candidate_from_ids_and_filenames() {
        let documents = [
            document("docs/decisions/product/0007-product.md", Some("DEC-0002")),
            document("docs/tasks/todo/0140-task.md", Some("TASK-0120")),
            document("docs/architecture/9999-ignored.md", Some("ARCH-9999")),
            document("docs/research/0003-research.md", Some("RES-0002")),
        ];

        assert_eq!(next_index(IndexedKind::Decision, documents.clone()), 8);
        assert_eq!(next_index(IndexedKind::Task, documents.clone()), 150);
        assert_eq!(next_index(IndexedKind::Research, documents), 4);
    }
}
