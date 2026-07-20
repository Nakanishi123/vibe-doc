//! 壊れた文書と参照切れを検出する診断を定義する。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::document::Document;
use crate::links::markdown_link_destinations;
use crate::parser::{DocumentTree, ParseDiagnostic};

/// lint診断の重要度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

impl DiagnosticLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// lintが報告する一件の問題。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintDiagnostic {
    pub level: DiagnosticLevel,
    pub path: PathBuf,
    pub message: String,
}

/// パース済みの文書ツリーを軽量な規約に照らして検査する。
///
/// パーサーが報告した問題もerrorとして引き継ぎ、構文エラーがある文書を見落とさない。
/// 続いてIDの一意性、既知のkind/status、メタデータ参照、ローカルMarkdownリンク、
/// Taskの配置を検査する。必須項目や本文の見出しは意図的に検査しない。
pub fn lint(tree: &DocumentTree) -> Vec<LintDiagnostic> {
    let mut diagnostics = parse_diagnostics(&tree.diagnostics);
    let ids = ids_by_document(&tree.documents);

    for (id, paths) in &ids {
        if paths.len() > 1 {
            for path in paths {
                diagnostics.push(error(path, format!("duplicate document id `{id}`")));
            }
        }
    }

    let known_ids: BTreeSet<_> = ids.keys().map(String::as_str).collect();
    for document in tree.documents.values() {
        lint_kind_and_status(document, &mut diagnostics);
        lint_references(document, &known_ids, &mut diagnostics);
        lint_links(document, &mut diagnostics);
        lint_task_location(document, &mut diagnostics);
    }

    diagnostics.sort_by(|left, right| {
        (left.path.as_path(), left.level, left.message.as_str()).cmp(&(
            right.path.as_path(),
            right.level,
            right.message.as_str(),
        ))
    });
    diagnostics
}

fn parse_diagnostics(diagnostics: &[ParseDiagnostic]) -> Vec<LintDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| error(&diagnostic.path, diagnostic.message.clone()))
        .collect()
}

fn ids_by_document(documents: &BTreeMap<PathBuf, Document>) -> BTreeMap<String, Vec<PathBuf>> {
    let mut ids = BTreeMap::<String, Vec<PathBuf>>::new();
    for document in documents.values() {
        if let Some(id) = &document.metadata.id {
            ids.entry(id.clone())
                .or_default()
                .push(document.path.clone());
        }
    }
    ids
}

fn lint_kind_and_status(document: &Document, diagnostics: &mut Vec<LintDiagnostic>) {
    if let Some(kind) = &document.metadata.kind
        && !matches!(
            kind.as_str(),
            "architecture" | "decision" | "task" | "research"
        )
    {
        diagnostics.push(error(
            &document.path,
            format!("unknown document kind `{kind}`"),
        ));
    }

    if let Some(status) = &document.metadata.status
        && !matches!(
            status.as_str(),
            "todo"
                | "in-progress"
                | "done"
                | "wont-do"
                | "accepted"
                | "proposed"
                | "rejected"
                | "superseded"
                | "deprecated"
        )
    {
        diagnostics.push(error(
            &document.path,
            format!("unknown document status `{status}`"),
        ));
    }
}

fn lint_references(
    document: &Document,
    known_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    for related in &document.metadata.related {
        if !known_ids.contains(related.as_str()) {
            diagnostics.push(error(
                &document.path,
                format!("related document `{related}` was not found"),
            ));
        }
    }
    for dependency in &document.metadata.depends_on {
        if !known_ids.contains(dependency.as_str()) {
            diagnostics.push(error(
                &document.path,
                format!("dependency document `{dependency}` was not found"),
            ));
        }
    }
}

fn lint_links(document: &Document, diagnostics: &mut Vec<LintDiagnostic>) {
    for destination in markdown_link_destinations(&document.body) {
        let path_part = destination.split(['#', '?']).next().unwrap_or_default();
        if path_part.is_empty() || is_external_destination(path_part) || !path_part.ends_with(".md")
        {
            continue;
        }
        let target = resolve_link(&document.path, path_part);
        if !target.exists() {
            diagnostics.push(error(
                &document.path,
                format!("linked document `{path_part}` was not found"),
            ));
        }
    }
}

fn lint_task_location(document: &Document, diagnostics: &mut Vec<LintDiagnostic>) {
    if document.metadata.kind.as_deref() != Some("task") {
        return;
    }
    let Some(status) = document.metadata.status.as_deref() else {
        return;
    };
    let Some(directory) = task_status_directory(&document.path) else {
        return;
    };
    if directory != status {
        diagnostics.push(LintDiagnostic {
            level: DiagnosticLevel::Warning,
            path: document.path.clone(),
            message: format!("task status `{status}` does not match `{directory}` directory"),
        });
    }
}

fn task_status_directory(path: &Path) -> Option<&str> {
    let mut components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str());
    while components.next()? != "tasks" {}
    components.next()
}

fn resolve_link(source: &Path, destination: &str) -> PathBuf {
    let target = if Path::new(destination).is_absolute() {
        PathBuf::from(destination)
    } else {
        source
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(destination)
    };
    normalize_path(&target)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component);
            }
        }
    }
    normalized
}

fn is_external_destination(destination: &str) -> bool {
    destination.starts_with("//")
        || destination.contains("://")
        || destination.starts_with("mailto:")
        || destination.starts_with("tel:")
}

fn error(path: &Path, message: impl Into<String>) -> LintDiagnostic {
    LintDiagnostic {
        level: DiagnosticLevel::Error,
        path: path.to_path_buf(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticLevel, lint};
    use crate::parser::parse_document_tree;
    use std::fs;

    #[test]
    fn reports_errors_and_task_status_mismatch_warning() {
        let root = std::env::temp_dir().join(format!("vibe-doc-lint-test-{}", std::process::id()));
        fs::create_dir_all(root.join("tasks/todo")).unwrap();
        fs::write(root.join("README.md"), "# Guide\n").unwrap();
        fs::write(
            root.join("tasks/todo/0010-example.md"),
            "---\nid: TASK-0010\nkind: task\nstatus: done\nrelated: [TASK-9999]\n---\n[guide](../../README.md) [missing](missing.md)\n",
        )
        .unwrap();

        let diagnostics = lint(&parse_document_tree(&root));
        assert_eq!(
            diagnostics
                .iter()
                .filter(|item| item.level == DiagnosticLevel::Error)
                .count(),
            2
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|item| item.level == DiagnosticLevel::Warning)
                .count(),
            1
        );

        fs::remove_dir_all(root).unwrap();
    }
}
