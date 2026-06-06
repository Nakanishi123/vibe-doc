use crate::{parse_numbered_document, DocumentKind, NumberedDocument, ParseError, SourceId};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A numbered document discovered in the supported repository layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryDocument {
    pub path: PathBuf,
    pub expected_kind: DocumentKind,
    pub document: NumberedDocument,
}

/// Error produced while scanning a vibe-doc repository.
#[derive(Debug)]
pub enum RepositoryScanError {
    ReadDir { path: PathBuf, source: io::Error },
    ReadFile { path: PathBuf, source: io::Error },
    Parse(ParseError),
}

impl fmt::Display for RepositoryScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDir { path, source } => {
                write!(
                    formatter,
                    "failed to read directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read file {}: {source}",
                    path.display()
                )
            }
            Self::Parse(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RepositoryScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDir { source, .. } => Some(source),
            Self::ReadFile { source, .. } => Some(source),
            Self::Parse(error) => Some(error),
        }
    }
}

impl From<ParseError> for RepositoryScanError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

/// Scan supported docs locations and return numbered documents sorted by path.
pub fn scan_repository(
    root: impl AsRef<Path>,
) -> Result<Vec<RepositoryDocument>, RepositoryScanError> {
    let root = root.as_ref();
    let mut paths = Vec::new();

    collect_markdown_files(root, Path::new("docs/specs"), &mut paths)?;
    collect_markdown_files(root, Path::new("docs/designs"), &mut paths)?;
    collect_markdown_files(root, Path::new("docs/adr"), &mut paths)?;
    push_file_if_exists(root, Path::new("docs/tasks/index.md"), &mut paths);
    collect_markdown_files(root, Path::new("docs/tasks/active"), &mut paths)?;
    collect_markdown_files(root, Path::new("docs/tasks/done"), &mut paths)?;

    paths.sort_by_key(|left| relative_sort_key(root, left));

    paths
        .into_iter()
        .map(|path| {
            let expected_kind =
                expected_kind_for_path(root, &path).expect("scanner only collects supported paths");
            let markdown =
                fs::read_to_string(&path).map_err(|source| RepositoryScanError::ReadFile {
                    path: path.clone(),
                    source,
                })?;
            let document = parse_numbered_document(SourceId::from(path.as_path()), &markdown)?;

            Ok(RepositoryDocument {
                path,
                expected_kind,
                document,
            })
        })
        .collect()
}

/// Infer the expected document kind from a repository-relative path.
pub fn expected_kind_for_relative_path(path: impl AsRef<Path>) -> Option<DocumentKind> {
    let path = path.as_ref();

    if path == Path::new("docs/tasks/index.md") {
        return Some(DocumentKind::TaskIndex);
    }

    let components = path_components(path);
    match components.as_slice() {
        ["docs", "specs", file] if is_numbered_markdown_candidate(file) => Some(DocumentKind::Spec),
        ["docs", "designs", file] if is_numbered_markdown_candidate(file) => {
            Some(DocumentKind::Design)
        }
        ["docs", "adr", file] if is_numbered_markdown_candidate(file) => Some(DocumentKind::Adr),
        ["docs", "tasks", "active", file] if is_numbered_markdown_candidate(file) => {
            Some(DocumentKind::Task)
        }
        ["docs", "tasks", "done", file] if is_numbered_markdown_candidate(file) => {
            Some(DocumentKind::Task)
        }
        _ => None,
    }
}

/// Infer the expected document kind for a path under a repository root.
pub fn expected_kind_for_path(
    root: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Option<DocumentKind> {
    path.as_ref()
        .strip_prefix(root.as_ref())
        .ok()
        .and_then(expected_kind_for_relative_path)
}

fn collect_markdown_files(
    root: &Path,
    relative_dir: &Path,
    paths: &mut Vec<PathBuf>,
) -> Result<(), RepositoryScanError> {
    let dir = root.join(relative_dir);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(RepositoryScanError::ReadDir { path: dir, source }),
    };

    for entry in entries {
        let entry = entry.map_err(|source| RepositoryScanError::ReadDir {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix(root).unwrap_or(path.as_path());
        if expected_kind_for_relative_path(relative_path).is_some() {
            paths.push(path);
        }
    }

    Ok(())
}

fn push_file_if_exists(root: &Path, relative_file: &Path, paths: &mut Vec<PathBuf>) {
    let path = root.join(relative_file);
    if path.is_file() {
        paths.push(path);
    }
}

fn relative_sort_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn path_components(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect()
}

fn is_numbered_markdown_candidate(file_name: &str) -> bool {
    file_name != "README.md" && file_name.ends_with(".md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentMetadata, ParseErrorKind};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolves_expected_kind_from_supported_paths() {
        assert_eq!(
            expected_kind_for_relative_path("docs/specs/9-model.md"),
            Some(DocumentKind::Spec)
        );
        assert_eq!(
            expected_kind_for_relative_path("docs/designs/10-cli.md"),
            Some(DocumentKind::Design)
        );
        assert_eq!(
            expected_kind_for_relative_path("docs/adr/11-rust.md"),
            Some(DocumentKind::Adr)
        );
        assert_eq!(
            expected_kind_for_relative_path("docs/tasks/index.md"),
            Some(DocumentKind::TaskIndex)
        );
        assert_eq!(
            expected_kind_for_relative_path("docs/tasks/active/12-work.md"),
            Some(DocumentKind::Task)
        );
        assert_eq!(
            expected_kind_for_relative_path("docs/tasks/done/13-done.md"),
            Some(DocumentKind::Task)
        );
        assert_eq!(expected_kind_for_relative_path("AGENTS.md"), None);
        assert_eq!(
            expected_kind_for_relative_path("docs/specs/README.md"),
            None
        );
    }

    #[test]
    fn scans_supported_layout_and_sorts_by_relative_path() {
        let repo = TestRepo::new("supported-layout");
        repo.write("AGENTS.md", "# Agents\n");
        repo.write("docs/README.md", "# Docs\n");
        repo.write("docs/specs/README.md", "# Specs\n");
        repo.write(
            "docs/designs/10-design.md",
            "\
---
id: 10
title: Design
kind: design
specs:
  - 9
---

# Design
",
        );
        repo.write(
            "docs/adr/11-decision.md",
            "\
---
id: 11
title: Decision
kind: adr
status: accepted
---

# Decision
",
        );
        repo.write(
            "docs/tasks/active/17-task.md",
            task_markdown(17, "Task", "planned"),
        );
        repo.write(
            "docs/specs/9-model.md",
            "\
---
id: 9
title: Model
kind: spec
---

# Model
",
        );
        repo.write(
            "docs/tasks/index.md",
            "\
---
id: 7
title: Task Index
kind: task-index
---

# Task Index
",
        );
        repo.write(
            "docs/tasks/done/16-parser.md",
            task_markdown(16, "Parser", "done"),
        );

        let documents = scan_repository(repo.path()).unwrap();
        let relative_paths: Vec<_> = documents
            .iter()
            .map(|document| relative_sort_key(repo.path(), &document.path))
            .collect();

        assert_eq!(
            relative_paths,
            [
                "docs/adr/11-decision.md",
                "docs/designs/10-design.md",
                "docs/specs/9-model.md",
                "docs/tasks/active/17-task.md",
                "docs/tasks/done/16-parser.md",
                "docs/tasks/index.md",
            ]
        );
        assert_eq!(documents[0].expected_kind, DocumentKind::Adr);
        assert_eq!(documents[1].expected_kind, DocumentKind::Design);
        assert_eq!(documents[2].expected_kind, DocumentKind::Spec);
        assert_eq!(documents[3].expected_kind, DocumentKind::Task);
        assert_eq!(documents[4].expected_kind, DocumentKind::Task);
        assert_eq!(documents[5].expected_kind, DocumentKind::TaskIndex);
    }

    #[test]
    fn scan_represents_kind_mismatch_for_later_validation() {
        let repo = TestRepo::new("kind-mismatch");
        repo.write(
            "docs/tasks/active/9-wrong-kind.md",
            "\
---
id: 9
title: Wrong Kind
kind: spec
---

# Wrong Kind
",
        );

        let documents = scan_repository(repo.path()).unwrap();

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].expected_kind, DocumentKind::Task);
        assert_eq!(
            documents[0].document.metadata.common().kind,
            DocumentKind::Spec
        );
    }

    #[test]
    fn scan_reports_parse_errors_for_numbered_locations() {
        let repo = TestRepo::new("parse-errors");
        repo.write("docs/specs/9-missing-frontmatter.md", "# Missing\n");

        let error = scan_repository(repo.path()).unwrap_err();

        let RepositoryScanError::Parse(error) = error else {
            panic!("expected parse error");
        };
        assert_eq!(error.kind, ParseErrorKind::MissingFrontmatter);
    }

    #[test]
    fn scan_keeps_parsed_metadata_with_discovered_paths() {
        let repo = TestRepo::new("metadata");
        repo.write(
            "docs/specs/9-model.md",
            "\
---
id: 9
title: Model
kind: spec
---

# Model
",
        );

        let documents = scan_repository(repo.path()).unwrap();

        assert_eq!(documents.len(), 1);
        assert!(documents[0].path.ends_with("docs/specs/9-model.md"));
        let DocumentMetadata::Spec(metadata) = &documents[0].document.metadata else {
            panic!("expected spec metadata");
        };
        assert_eq!(metadata.common.title, "Model");
    }

    struct TestRepo {
        root: PathBuf,
    }

    impl TestRepo {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("vibe-doc-{name}-{unique}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn write(&self, relative_path: &str, content: impl AsRef<str>) {
            let path = self.root.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content.as_ref()).unwrap();
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn task_markdown(id: u64, title: &str, status: &str) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: task
type: feature
status: {status}
---

# {title}
"
        )
    }
}
