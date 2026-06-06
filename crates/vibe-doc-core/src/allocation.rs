use crate::{scan_repository, DocumentId, RepositoryDocument, RepositoryScanError};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// Supported document locations for generated repository-relative paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentLocation {
    Spec,
    Design,
    Adr,
    ActiveTask,
    DoneTask,
    TaskIndex,
}

/// One duplicated global document ID and every path where it appears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateDocumentId {
    pub id: DocumentId,
    pub paths: Vec<PathBuf>,
}

/// Error produced while allocating the next global document ID.
#[derive(Debug)]
pub enum IdAllocationError {
    RepositoryScan(RepositoryScanError),
    DuplicateIds(Vec<DuplicateDocumentId>),
}

impl fmt::Display for IdAllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepositoryScan(error) => error.fmt(formatter),
            Self::DuplicateIds(duplicates) => {
                write!(formatter, "duplicate document IDs found")?;
                for duplicate in duplicates {
                    write!(formatter, ": {}", duplicate.id.get())?;
                    for path in &duplicate.paths {
                        write!(formatter, " {}", path.display())?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for IdAllocationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RepositoryScan(error) => Some(error),
            Self::DuplicateIds(_) => None,
        }
    }
}

impl From<RepositoryScanError> for IdAllocationError {
    fn from(value: RepositoryScanError) -> Self {
        Self::RepositoryScan(value)
    }
}

/// Scan a repository and return the next available positive global document ID.
pub fn allocate_next_document_id(
    root: impl AsRef<std::path::Path>,
) -> Result<DocumentId, IdAllocationError> {
    let documents = scan_repository(root)?;
    next_document_id(&documents)
}

/// Return all document IDs sorted numerically.
pub fn sorted_document_ids(documents: &[RepositoryDocument]) -> Vec<DocumentId> {
    let mut ids = documents
        .iter()
        .map(|document| document.document.metadata.common().id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

/// Return duplicate global document IDs sorted numerically by ID.
pub fn duplicate_document_ids(documents: &[RepositoryDocument]) -> Vec<DuplicateDocumentId> {
    let mut paths_by_id = BTreeMap::<DocumentId, Vec<PathBuf>>::new();

    for document in documents {
        paths_by_id
            .entry(document.document.metadata.common().id)
            .or_default()
            .push(document.path.clone());
    }

    paths_by_id
        .into_iter()
        .filter_map(|(id, paths)| (paths.len() > 1).then_some(DuplicateDocumentId { id, paths }))
        .collect()
}

/// Return the next positive global document ID for already scanned documents.
pub fn next_document_id(documents: &[RepositoryDocument]) -> Result<DocumentId, IdAllocationError> {
    let duplicates = duplicate_document_ids(documents);
    if !duplicates.is_empty() {
        return Err(IdAllocationError::DuplicateIds(duplicates));
    }

    let next = sorted_document_ids(documents)
        .last()
        .map(|id| id.get() + 1)
        .unwrap_or(1);

    Ok(DocumentId::new(next).expect("next ID is always positive"))
}

/// Convert a document title into a lowercase ASCII filename slug.
pub fn slugify_title(title: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('-');
            previous_was_separator = true;
        }
    }

    if previous_was_separator {
        slug.pop();
    }

    if slug.is_empty() {
        "untitled".to_owned()
    } else {
        slug
    }
}

/// Generate an unpadded numbered Markdown filename from an ID and title.
pub fn document_filename(id: DocumentId, title: &str) -> String {
    format!("{}-{}.md", id.get(), slugify_title(title))
}

/// Generate a repository-relative path following the managed docs layout.
pub fn document_relative_path(location: DocumentLocation, id: DocumentId, title: &str) -> PathBuf {
    match location {
        DocumentLocation::Spec => PathBuf::from("docs/specs").join(document_filename(id, title)),
        DocumentLocation::Design => {
            PathBuf::from("docs/designs").join(document_filename(id, title))
        }
        DocumentLocation::Adr => PathBuf::from("docs/adr").join(document_filename(id, title)),
        DocumentLocation::ActiveTask => {
            PathBuf::from("docs/tasks/active").join(document_filename(id, title))
        }
        DocumentLocation::DoneTask => {
            PathBuf::from("docs/tasks/done").join(document_filename(id, title))
        }
        DocumentLocation::TaskIndex => PathBuf::from("docs/tasks/index.md"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan_repository;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn next_id_is_one_for_empty_repositories() {
        let repo = TestRepo::new("empty");

        let id = allocate_next_document_id(repo.path()).unwrap();

        assert_eq!(id, DocumentId::new(1).unwrap());
    }

    #[test]
    fn next_id_uses_highest_existing_id_plus_one() {
        let repo = TestRepo::new("normal");
        repo.write("docs/specs/9-model.md", spec_markdown(9, "Model"));
        repo.write("docs/designs/10-design.md", design_markdown(10, "Design"));
        repo.write("docs/adr/11-decision.md", adr_markdown(11, "Decision"));
        repo.write(
            "docs/tasks/active/12-task.md",
            task_markdown(12, "Task", "planned"),
        );

        let id = allocate_next_document_id(repo.path()).unwrap();

        assert_eq!(id, DocumentId::new(13).unwrap());
    }

    #[test]
    fn next_id_uses_highest_id_even_when_gaps_exist() {
        let repo = TestRepo::new("gaps");
        repo.write("docs/specs/2-two.md", spec_markdown(2, "Two"));
        repo.write(
            "docs/tasks/active/100-one-hundred.md",
            task_markdown(100, "One Hundred", "planned"),
        );

        let id = allocate_next_document_id(repo.path()).unwrap();

        assert_eq!(id, DocumentId::new(101).unwrap());
    }

    #[test]
    fn duplicate_ids_are_reported_before_allocating() {
        let repo = TestRepo::new("duplicates");
        repo.write("docs/specs/9-model.md", spec_markdown(9, "Model"));
        repo.write(
            "docs/tasks/active/10-task.md",
            task_markdown(9, "Duplicate", "planned"),
        );

        let error = allocate_next_document_id(repo.path()).unwrap_err();

        let IdAllocationError::DuplicateIds(duplicates) = error else {
            panic!("expected duplicate IDs");
        };
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].id, DocumentId::new(9).unwrap());
        assert_eq!(duplicates[0].paths.len(), 2);
    }

    #[test]
    fn invalid_ids_are_reported_as_scan_errors() {
        let repo = TestRepo::new("invalid");
        repo.write(
            "docs/specs/0-invalid.md",
            "\
---
id: 0
title: Invalid
kind: spec
---

# Invalid
",
        );

        let error = allocate_next_document_id(repo.path()).unwrap_err();

        assert!(matches!(error, IdAllocationError::RepositoryScan(_)));
    }

    #[test]
    fn ids_are_sorted_numerically_not_lexicographically() {
        let repo = TestRepo::new("sort");
        repo.write("docs/specs/10-ten.md", spec_markdown(10, "Ten"));
        repo.write("docs/specs/2-two.md", spec_markdown(2, "Two"));
        repo.write("docs/specs/1-one.md", spec_markdown(1, "One"));
        let documents = scan_repository(repo.path()).unwrap();

        let ids = sorted_document_ids(&documents)
            .into_iter()
            .map(DocumentId::get)
            .collect::<Vec<_>>();

        assert_eq!(ids, [1, 2, 10]);
    }

    #[test]
    fn slug_generation_handles_common_title_input() {
        assert_eq!(slugify_title(" Implement vdoc new! "), "implement-vdoc-new");
        assert_eq!(slugify_title("API/CLI: JSON output"), "api-cli-json-output");
        assert_eq!(slugify_title("Already---Separated"), "already-separated");
        assert_eq!(slugify_title(""), "untitled");
        assert_eq!(slugify_title("!!!"), "untitled");
    }

    #[test]
    fn filenames_use_unpadded_ids_and_slugged_titles() {
        let id = DocumentId::new(7).unwrap();

        assert_eq!(
            document_filename(id, "Task Index Drift"),
            "7-task-index-drift.md"
        );
    }

    #[test]
    fn relative_paths_follow_document_directory_conventions() {
        let id = DocumentId::new(18).unwrap();

        assert_eq!(
            document_relative_path(DocumentLocation::Spec, id, "Document Model"),
            PathBuf::from("docs/specs/18-document-model.md")
        );
        assert_eq!(
            document_relative_path(DocumentLocation::Design, id, "CLI Design"),
            PathBuf::from("docs/designs/18-cli-design.md")
        );
        assert_eq!(
            document_relative_path(DocumentLocation::Adr, id, "Use Rust"),
            PathBuf::from("docs/adr/18-use-rust.md")
        );
        assert_eq!(
            document_relative_path(DocumentLocation::ActiveTask, id, "Implement Feature"),
            PathBuf::from("docs/tasks/active/18-implement-feature.md")
        );
        assert_eq!(
            document_relative_path(DocumentLocation::DoneTask, id, "Implement Feature"),
            PathBuf::from("docs/tasks/done/18-implement-feature.md")
        );
        assert_eq!(
            document_relative_path(DocumentLocation::TaskIndex, id, "Ignored"),
            PathBuf::from("docs/tasks/index.md")
        );
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
            let root = std::env::temp_dir().join(format!("vibe-doc-allocation-{name}-{unique}"));
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

    fn spec_markdown(id: u64, title: &str) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: spec
---

# {title}
"
        )
    }

    fn design_markdown(id: u64, title: &str) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: design
specs: []
---

# {title}
"
        )
    }

    fn adr_markdown(id: u64, title: &str) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: adr
status: proposed
---

# {title}
"
        )
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
