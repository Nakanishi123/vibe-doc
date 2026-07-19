//! Markdown文書とYAML Front Matterの解析。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde_yaml::Value;
use walkdir::WalkDir;

use crate::document::{Document, Metadata};

/// 文書ツリーの読み込み中に見つかった、処理を継続できる問題。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

/// 1つのMarkdownファイルを解析した結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDocument {
    pub document: Document,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// 文書ルートから収集した文書と診断。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentTree {
    pub documents: BTreeMap<PathBuf, Document>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// [`DocumentStore::refresh`] の呼び出しで影響を受けたパス。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentChanges {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

impl DocumentChanges {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
    }
}

/// 追加・変更されたファイルだけを再解析する文書ツリーのスナップショット。
#[derive(Debug, Clone)]
pub struct DocumentStore {
    root: PathBuf,
    documents: BTreeMap<PathBuf, Document>,
    diagnostics: BTreeMap<PathBuf, Vec<ParseDiagnostic>>,
    scan_diagnostics: Vec<ParseDiagnostic>,
    fingerprints: BTreeMap<PathBuf, u64>,
}

impl DocumentStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            documents: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            scan_diagnostics: Vec::new(),
            fingerprints: BTreeMap::new(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn documents(&self) -> &BTreeMap<PathBuf, Document> {
        &self.documents
    }

    pub fn diagnostics(&self) -> Vec<ParseDiagnostic> {
        self.scan_diagnostics
            .iter()
            .chain(self.diagnostics.values().flatten())
            .cloned()
            .collect()
    }

    /// 文書ルートを再走査し、追加・更新・削除されたファイルだけを反映する。
    ///
    /// 呼び出しごとに全てのMarkdownファイルの内容を指紋化するが、前回から内容が
    /// 変わっていないファイルは再解析しない。新規または変更済みのファイルについては
    /// `Document` と診断を置き換え、消えたファイルについては両方を取り除く。
    /// そのため、ファイル監視機構からこのメソッドを呼ぶだけで、利用側は常に最新の
    /// `documents` と `diagnostics` を参照できる。
    pub fn refresh(&mut self) -> DocumentChanges {
        let mut changes = DocumentChanges::default();
        let mut current_paths = BTreeSet::new();
        let (paths, scan_diagnostics) = markdown_paths(&self.root);
        self.scan_diagnostics = scan_diagnostics;

        for path in paths {
            current_paths.insert(path.clone());
            match fs::read_to_string(&path) {
                Ok(source) => {
                    let fingerprint = fingerprint(&source);
                    if self.fingerprints.get(&path) == Some(&fingerprint) {
                        continue;
                    }

                    let is_new = !self.fingerprints.contains_key(&path);
                    let parsed = parse_markdown(&path, &source);
                    self.documents.insert(path.clone(), parsed.document);
                    self.diagnostics.insert(path.clone(), parsed.diagnostics);
                    self.fingerprints.insert(path.clone(), fingerprint);
                    if is_new {
                        changes.added.push(path);
                    } else {
                        changes.modified.push(path);
                    }
                }
                Err(error) => {
                    let diagnostic = ParseDiagnostic {
                        path: path.clone(),
                        message: format!("failed to read document: {error}"),
                    };
                    let is_new = !self.fingerprints.contains_key(&path);
                    self.documents.remove(&path);
                    self.diagnostics.insert(path.clone(), vec![diagnostic]);
                    self.fingerprints.insert(path.clone(), 0);
                    if is_new {
                        changes.added.push(path);
                    } else {
                        changes.modified.push(path);
                    }
                }
            }
        }

        let removed: Vec<_> = self
            .fingerprints
            .keys()
            .filter(|path| !current_paths.contains(*path))
            .cloned()
            .collect();
        for path in removed {
            self.fingerprints.remove(&path);
            self.documents.remove(&path);
            self.diagnostics.remove(&path);
            changes.removed.push(path);
        }
        changes
    }
}

/// `root` 以下にある全ての `*.md` ファイルを解析する。ルートが存在しない場合は空のツリーを返す。
pub fn parse_document_tree(root: impl AsRef<Path>) -> DocumentTree {
    let mut tree = DocumentTree::default();
    let (paths, scan_diagnostics) = markdown_paths(root.as_ref());
    tree.diagnostics = scan_diagnostics;
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(source) => {
                let parsed = parse_markdown(&path, &source);
                tree.diagnostics.extend(parsed.diagnostics);
                tree.documents.insert(path, parsed.document);
            }
            Err(error) => tree.diagnostics.push(ParseDiagnostic {
                path,
                message: format!("failed to read document: {error}"),
            }),
        }
    }
    tree
}

/// Markdownソースを共通の文書モデルへ解析する。
pub fn parse_markdown(path: impl Into<PathBuf>, source: &str) -> ParsedDocument {
    let path = path.into();
    let (front_matter, body, mut diagnostics) = split_front_matter(&path, source);
    let metadata = front_matter
        .and_then(|yaml| parse_metadata(&path, yaml, &mut diagnostics))
        .unwrap_or_default();

    ParsedDocument {
        document: Document {
            path,
            metadata,
            title: extract_title(body),
            body: body.to_owned(),
        },
        diagnostics,
    }
}

/// `root` 以下のMarkdownファイルと、走査そのものに失敗した箇所を収集する。
///
/// `WalkDir` のエラーを無視すると、権限不足などによって欠落した文書を利用側が
/// 正常な空の結果と区別できなくなる。走査エラーも文書の構文エラーと同じ診断形式で
/// 返し、ほかの到達可能なファイルの解析は継続する。文書ルート直下の `README.md` は
/// 運用ガイドであり、管理対象の文書ではないため除外する。
fn markdown_paths(root: &Path) -> (Vec<PathBuf>, Vec<ParseDiagnostic>) {
    if let Err(error) = fs::metadata(root) {
        return if error.kind() == std::io::ErrorKind::NotFound {
            (Vec::new(), Vec::new())
        } else {
            (
                Vec::new(),
                vec![ParseDiagnostic {
                    path: root.to_path_buf(),
                    message: format!("failed to access document tree: {error}"),
                }],
            )
        };
    }

    let mut paths = Vec::new();
    let mut diagnostics = Vec::new();
    let guide_path = root.join("README.md");
    for entry in WalkDir::new(root).follow_links(false) {
        match entry {
            Ok(entry)
                if entry.file_type().is_file()
                    && entry.path() != guide_path
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("md")) =>
            {
                paths.push(entry.into_path())
            }
            // ディレクトリやMarkdown以外のファイルは管理対象外である。
            Ok(_) => continue,
            Err(error) => diagnostics.push(ParseDiagnostic {
                path: error.path().unwrap_or(root).to_path_buf(),
                message: format!("failed to scan document tree: {error}"),
            }),
        }
    }
    (paths, diagnostics)
}

fn split_front_matter<'a>(
    path: &Path,
    source: &'a str,
) -> (Option<&'a str>, &'a str, Vec<ParseDiagnostic>) {
    let Some(after_opening) = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
    else {
        return (None, source, Vec::new());
    };
    let opening_len = source.len() - after_opening.len();
    let mut offset = opening_len;
    for line in after_opening.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" || trimmed == "..." {
            let yaml = &source[opening_len..offset];
            return (Some(yaml), &source[offset + line.len()..], Vec::new());
        }
        offset += line.len();
    }
    (
        None,
        source,
        vec![ParseDiagnostic {
            path: path.to_path_buf(),
            message: "YAML Front Matter starts with `---` but has no closing delimiter".to_owned(),
        }],
    )
}

/// YAMLのマッピングから既知のメタデータを取り出し、未知の項目も失わずに保存する。
///
/// Front Matter全体が壊れている場合は `None` を返す。一方、特定フィールドだけが
/// 不正な場合は、ほかの正しいフィールドを利用できるようにし、そのフィールドだけを
/// 空値として診断を追加する。これによりlintやUIが不完全な文書も表示できる。
fn parse_metadata(
    path: &Path,
    yaml: &str,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Option<Metadata> {
    let value: Value = match serde_yaml::from_str(yaml) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(ParseDiagnostic {
                path: path.to_path_buf(),
                message: format!("invalid YAML Front Matter: {error}"),
            });
            return None;
        }
    };
    let Some(mapping) = value.as_mapping() else {
        diagnostics.push(ParseDiagnostic {
            path: path.to_path_buf(),
            message: "YAML Front Matter must be a mapping".to_owned(),
        });
        return None;
    };

    let mut metadata = Metadata::default();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            diagnostics.push(ParseDiagnostic {
                path: path.to_path_buf(),
                message: "YAML Front Matter keys must be strings".to_owned(),
            });
            continue;
        };
        match key {
            "vibedoc" => metadata.schema_version = yaml_u32(path, key, value, diagnostics),
            "id" => metadata.id = yaml_string(path, key, value, diagnostics),
            "kind" => metadata.kind = yaml_string(path, key, value, diagnostics),
            "status" => metadata.status = yaml_string(path, key, value, diagnostics),
            "tags" => metadata.tags = yaml_strings(path, key, value, diagnostics),
            "related" => metadata.related = yaml_strings(path, key, value, diagnostics),
            "depends_on" => metadata.depends_on = yaml_strings(path, key, value, diagnostics),
            _ => {
                metadata.extra.insert(key.to_owned(), yaml_value(value));
            }
        }
    }
    Some(metadata)
}

fn yaml_string(
    path: &Path,
    key: &str,
    value: &Value,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Option<String> {
    match value.as_str() {
        Some(value) => Some(value.to_owned()),
        None => {
            diagnostics.push(invalid_field(path, key, "a string"));
            None
        }
    }
}

fn yaml_u32(
    path: &Path,
    key: &str,
    value: &Value,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Option<u32> {
    match value.as_u64().and_then(|value| u32::try_from(value).ok()) {
        Some(value) => Some(value),
        None => {
            diagnostics.push(invalid_field(path, key, "a non-negative 32-bit integer"));
            None
        }
    }
}

fn yaml_strings(
    path: &Path,
    key: &str,
    value: &Value,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Vec<String> {
    let Some(values) = value.as_sequence() else {
        diagnostics.push(invalid_field(path, key, "a list of strings"));
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|value| match value.as_str() {
            Some(value) => Some(value.to_owned()),
            None => {
                diagnostics.push(invalid_field(path, key, "a list containing only strings"));
                None
            }
        })
        .collect()
}

fn invalid_field(path: &Path, key: &str, expected: &str) -> ParseDiagnostic {
    ParseDiagnostic {
        path: path.to_path_buf(),
        message: format!("Front Matter field `{key}` must be {expected}"),
    }
}

fn yaml_value(value: &Value) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim()
        .to_owned()
}

fn extract_title(body: &str) -> Option<String> {
    let mut fenced = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if !fenced && trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().trim_end_matches('#').trim().to_owned());
        }
    }
    None
}

fn fingerprint(source: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_and_unknown_front_matter_fields() {
        let parsed = parse_markdown(
            "docs/tasks/todo/0010-document-parser.md",
            "---\nvibedoc: 1\nid: TASK-0010\nkind: task\nstatus: todo\ntags: [rust, parser]\nrelated:\n  - ARCH-0002\ndepends_on: []\npriority: high\n---\n# Parser\n\nBody\n",
        );

        assert!(parsed.diagnostics.is_empty());
        assert_eq!(parsed.document.metadata.schema_version, Some(1));
        assert_eq!(parsed.document.metadata.tags, ["rust", "parser"]);
        assert_eq!(parsed.document.metadata.extra["priority"], "high");
        assert_eq!(parsed.document.title.as_deref(), Some("Parser"));
        assert_eq!(parsed.document.body, "# Parser\n\nBody\n");
    }

    #[test]
    fn reports_invalid_yaml_without_discarding_markdown() {
        let parsed = parse_markdown("broken.md", "---\nid: [oops\n---\n# Still readable\n");

        assert_eq!(parsed.document.title.as_deref(), Some("Still readable"));
        assert!(parsed.document.metadata.id.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
    }

    #[test]
    fn ignores_headings_inside_fenced_code_blocks() {
        let parsed = parse_markdown(
            "example.md",
            "```md\n# Not a title\n```\n\n# Actual title #\n",
        );
        assert_eq!(parsed.document.title.as_deref(), Some("Actual title"));
    }

    #[test]
    fn refresh_reparses_added_changed_and_removed_files() {
        let root =
            std::env::temp_dir().join(format!("vibe-doc-parser-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("task.md");
        fs::write(&path, "# First\n").unwrap();

        let mut store = DocumentStore::new(&root);
        let first = store.refresh();
        assert_eq!(first.added, vec![path.clone()]);
        assert_eq!(store.documents()[&path].title.as_deref(), Some("First"));

        fs::write(&path, "# Changed\n").unwrap();
        let second = store.refresh();
        assert_eq!(second.modified, vec![path.clone()]);
        assert_eq!(store.documents()[&path].title.as_deref(), Some("Changed"));

        fs::remove_file(&path).unwrap();
        let third = store.refresh();
        assert_eq!(third.removed, vec![path]);
        assert!(store.documents().is_empty());

        fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn excludes_the_document_root_readme_from_managed_documents() {
        let root =
            std::env::temp_dir().join(format!("vibe-doc-parser-guide-test-{}", std::process::id()));
        fs::create_dir_all(root.join("tasks/todo")).unwrap();
        fs::write(root.join("README.md"), "# Documentation guide\n").unwrap();
        let task = root.join("tasks/todo/010-parser.md");
        fs::write(&task, "# Parser\n").unwrap();

        let tree = parse_document_tree(&root);
        assert_eq!(tree.documents.len(), 1);
        assert!(tree.documents.contains_key(&task));

        fs::remove_dir_all(&root).unwrap();
    }
}
