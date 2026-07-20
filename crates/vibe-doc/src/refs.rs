//! `vibe-doc refs` の入力解決と出力整形。
//!
//! 指定された文書を参照している文書を、`related`、`depends_on` の逆向き、
//! Markdownリンクのバックリンクの3種から逆引きして表示する。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use vibe_doc_core::document::Document;
use vibe_doc_core::index::DocumentIndex;

/// `refs` の対象として解決された文書。
struct RefsTarget {
    id: String,
    path: PathBuf,
}

/// JSON出力の1参照元。
#[derive(Serialize)]
struct RefEntry {
    id: String,
    title: Option<String>,
    path: PathBuf,
}

/// JSON出力のルート。空のグループも空配列として必ず含める。
#[derive(Serialize)]
struct RefsReport {
    id: String,
    path: PathBuf,
    refs: RefsGroups,
}

#[derive(Serialize)]
struct RefsGroups {
    related: Vec<RefEntry>,
    dependents: Vec<RefEntry>,
    backlinks: Vec<RefEntry>,
}

/// 引数をIDまたはファイルパスとして文書に解決する。
///
/// まずIDとして索引を引き、見つからなければ文書ツリーのパスとして解決する。
/// パスは走査結果のキー(`docs/`からの相対パス)と一致させるため、先頭の`./`を
/// 取り除いて比較する。パスで見つかった文書がIDを持たない場合は、参照元の索引が
/// IDを基準にしているため解決失敗として扱う。
fn resolve_target(
    target: &str,
    index: &DocumentIndex,
    documents: &BTreeMap<PathBuf, Document>,
) -> Result<RefsTarget, String> {
    if let Some(document) = index.document(target) {
        return Ok(RefsTarget {
            id: target.to_string(),
            path: document.path.clone(),
        });
    }
    let path = Path::new(target.strip_prefix("./").unwrap_or(target));
    match documents.get(path) {
        Some(document) => match &document.metadata.id {
            Some(id) => Ok(RefsTarget {
                id: id.clone(),
                path: document.path.clone(),
            }),
            None => Err(format!(
                "{target}: この文書はIDを持たないため参照元を索引できない"
            )),
        },
        None => Err(format!("{target}: IDにもファイルパスにも解決できない")),
    }
}

fn ref_entries(documents: Vec<&Document>) -> Vec<RefEntry> {
    documents
        .into_iter()
        .filter_map(|document| {
            document.metadata.id.clone().map(|id| RefEntry {
                id,
                title: document.title.clone(),
                path: document.path.clone(),
            })
        })
        .collect()
}

fn print_group(label: &str, entries: &[RefEntry]) {
    if entries.is_empty() {
        return;
    }
    println!();
    println!("{label}:");
    for entry in entries {
        match &entry.title {
            Some(title) => println!("  {}  {}  ({})", entry.id, title, entry.path.display()),
            None => println!("  {}  ({})", entry.id, entry.path.display()),
        }
    }
}

/// 対象文書の参照元を収集して表示する。
///
/// 出力はデフォルトで人間向けに関係種別ごとへグループ化し、空のグループは
/// 省略する。参照元がひとつもない場合は`no references`と表示する。`json`が
/// 真のときは空グループも空配列として含むJSONを1つ出力する。解決できない
/// 入力はエラーメッセージを返し、呼び出し側が終了コード1にする。
pub(crate) fn run_refs(
    target: &str,
    json: bool,
    index: &DocumentIndex,
    documents: &BTreeMap<PathBuf, Document>,
) -> Result<(), String> {
    let resolved = resolve_target(target, index, documents)?;
    let report = RefsReport {
        refs: RefsGroups {
            related: ref_entries(index.related_documents(&resolved.id)),
            dependents: ref_entries(index.dependent_documents(&resolved.id)),
            backlinks: ref_entries(index.backlink_documents(&resolved.id)),
        },
        id: resolved.id,
        path: resolved.path,
    };
    if json {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("JSONへの変換に失敗した: {error}"))?;
        println!("{rendered}");
        return Ok(());
    }
    let target_document = index.document(&report.id);
    let title = target_document.and_then(|document| document.title.as_deref());
    match title {
        Some(title) => println!("{}  {}  ({})", report.id, title, report.path.display()),
        None => println!("{}  ({})", report.id, report.path.display()),
    }
    if report.refs.related.is_empty()
        && report.refs.dependents.is_empty()
        && report.refs.backlinks.is_empty()
    {
        println!();
        println!("no references");
        return Ok(());
    }
    print_group("related", &report.refs.related);
    print_group("depended on by", &report.refs.dependents);
    print_group("linked from", &report.refs.backlinks);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_target, run_refs};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use vibe_doc_core::document::{Document, Metadata};
    use vibe_doc_core::index::DocumentIndex;

    fn document(path: &str, id: Option<&str>, related: &[&str]) -> Document {
        Document {
            path: PathBuf::from(path),
            metadata: Metadata {
                id: id.map(String::from),
                related: related.iter().map(|value| value.to_string()).collect(),
                ..Metadata::default()
            },
            title: Some(format!("title of {path}")),
            body: String::new(),
        }
    }

    fn fixture() -> (DocumentIndex, BTreeMap<PathBuf, Document>) {
        let documents: BTreeMap<PathBuf, Document> = [
            document("docs/tasks/todo/0030-cli.md", Some("TASK-0030"), &[]),
            document(
                "docs/architecture/004-cli.md",
                Some("ARCH-0004"),
                &["TASK-0030"],
            ),
            document("docs/README.md", None, &[]),
        ]
        .into_iter()
        .map(|doc| (doc.path.clone(), doc))
        .collect();
        (DocumentIndex::from_document_map(&documents), documents)
    }

    #[test]
    fn resolves_id_and_path_to_the_same_target() {
        let (index, documents) = fixture();
        let by_id = resolve_target("TASK-0030", &index, &documents).unwrap();
        let by_path = resolve_target("docs/tasks/todo/0030-cli.md", &index, &documents).unwrap();
        assert_eq!(by_id.id, "TASK-0030");
        assert_eq!(by_id.id, by_path.id);
        assert_eq!(by_id.path, by_path.path);
    }

    #[test]
    fn rejects_unknown_targets_and_documents_without_an_id() {
        let (index, documents) = fixture();
        assert!(resolve_target("TASK-9999", &index, &documents).is_err());
        assert!(resolve_target("docs/README.md", &index, &documents).is_err());
    }

    #[test]
    fn reports_related_references() {
        let (index, documents) = fixture();
        assert!(run_refs("TASK-0030", false, &index, &documents).is_ok());
        assert!(run_refs("TASK-0030", true, &index, &documents).is_ok());
    }
}
