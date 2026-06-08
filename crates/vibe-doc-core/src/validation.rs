use crate::{
    duplicate_document_ids, scan_repository, AdrStatus, DocumentId, DocumentKind, DocumentMetadata,
    ParseErrorKind, RepositoryDocument, RepositoryScanError,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Stable validation issue code shared by CLI, server, and UI callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationCode {
    BadFrontmatter,
    MissingRequiredField,
    InvalidId,
    DuplicateId,
    InvalidKind,
    InvalidStatus,
    InvalidType,
    InvalidPriority,
    BrokenReference,
    MissingDependency,
    TaskDoneInActive,
    TaskActiveInDone,
    AdrSupersededWithoutReplacement,
    IndexOutOfSync,
    SchemaNotFound,
    ReadmeNotFound,
}

impl ValidationCode {
    /// Return the stable string representation used in external reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BadFrontmatter => "BAD_FRONTMATTER",
            Self::MissingRequiredField => "MISSING_REQUIRED_FIELD",
            Self::InvalidId => "INVALID_ID",
            Self::DuplicateId => "DUPLICATE_ID",
            Self::InvalidKind => "INVALID_KIND",
            Self::InvalidStatus => "INVALID_STATUS",
            Self::InvalidType => "INVALID_TYPE",
            Self::InvalidPriority => "INVALID_PRIORITY",
            Self::BrokenReference => "BROKEN_REFERENCE",
            Self::MissingDependency => "MISSING_DEPENDENCY",
            Self::TaskDoneInActive => "TASK_DONE_IN_ACTIVE",
            Self::TaskActiveInDone => "TASK_ACTIVE_IN_DONE",
            Self::AdrSupersededWithoutReplacement => "ADR_SUPERSEDED_WITHOUT_REPLACEMENT",
            Self::IndexOutOfSync => "INDEX_OUT_OF_SYNC",
            Self::SchemaNotFound => "SCHEMA_NOT_FOUND",
            Self::ReadmeNotFound => "README_NOT_FOUND",
        }
    }
}

impl fmt::Display for ValidationCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One validation issue tied to an optional repository path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub path: Option<PathBuf>,
    pub message: String,
}

impl ValidationIssue {
    fn new(code: ValidationCode, path: Option<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            code,
            path,
            message: message.into(),
        }
    }
}

/// Complete validation result for a repository or a scanner output slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub incomplete: bool,
}

impl ValidationReport {
    /// Return true when no validation issues were found.
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty() && !self.incomplete
    }
}

/// Loaded repository schema files.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaSet {
    pub document: Option<Value>,
    pub spec: Option<Value>,
    pub design: Option<Value>,
    pub adr: Option<Value>,
    pub task: Option<Value>,
}

impl SchemaSet {
    fn missing_schema_names(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if self.document.is_none() {
            missing.push("document.schema.json");
        }
        if self.spec.is_none() {
            missing.push("spec.schema.json");
        }
        if self.design.is_none() {
            missing.push("design.schema.json");
        }
        if self.adr.is_none() {
            missing.push("adr.schema.json");
        }
        if self.task.is_none() {
            missing.push("task.schema.json");
        }

        missing
    }
}

/// Error produced while reading schema files.
#[derive(Debug, Error)]
pub enum SchemaLoadError {
    #[error("failed to read schema {}: {source}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid schema JSON {}: {source}", path.display())]
    InvalidJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
}

/// Error produced while running repository validation.
#[derive(Debug, Error)]
pub enum ValidationRunError {
    #[error(transparent)]
    Schema(#[from] SchemaLoadError),
    #[error(transparent)]
    RepositoryScan(#[from] RepositoryScanError),
}

/// Load repository JSON Schema files from `docs/schemas/`.
pub fn load_schema_set(root: impl AsRef<Path>) -> Result<SchemaSet, SchemaLoadError> {
    let root = root.as_ref();

    Ok(SchemaSet {
        document: load_optional_schema(root, "document.schema.json")?,
        spec: load_optional_schema(root, "spec.schema.json")?,
        design: load_optional_schema(root, "design.schema.json")?,
        adr: load_optional_schema(root, "adr.schema.json")?,
        task: load_optional_schema(root, "task.schema.json")?,
    })
}

/// Validate a repository by loading schemas, scanning documents, and checking metadata rules.
pub fn validate_repository(root: impl AsRef<Path>) -> Result<ValidationReport, ValidationRunError> {
    let root = root.as_ref();
    let schemas = load_schema_set(root)?;

    match scan_repository(root) {
        Ok(documents) => Ok(validate_documents(root, &schemas, &documents)),
        Err(RepositoryScanError::Parse(error)) => Ok(ValidationReport {
            issues: vec![ValidationIssue::new(
                code_for_parse_error(&error),
                Some(relative_path(root, Path::new(error.source.as_str()))),
                error.message,
            )],
            incomplete: true,
        }),
        Err(error) => Err(ValidationRunError::RepositoryScan(error)),
    }
}

/// Run validation plus broader repository consistency checks.
pub fn check_repository(root: impl AsRef<Path>) -> Result<ValidationReport, ValidationRunError> {
    let root = root.as_ref();
    let validation_report = validate_repository(root)?;
    let incomplete = validation_report.incomplete;
    let mut issues = validation_report.issues;

    for readme in REQUIRED_READMES {
        let path = PathBuf::from(readme);
        if !root.join(&path).is_file() {
            issues.push(ValidationIssue::new(
                ValidationCode::ReadmeNotFound,
                Some(path.clone()),
                format!("README file {} was not found", path.display()),
            ));
        }
    }

    match scan_repository(root) {
        Ok(documents) => check_task_index(root, &documents, &mut issues),
        Err(RepositoryScanError::Parse(_)) => {}
        Err(error) => return Err(ValidationRunError::RepositoryScan(error)),
    }

    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });

    Ok(ValidationReport { issues, incomplete })
}

/// Validate already scanned documents against loaded schemas and built-in rules.
pub fn validate_documents(
    root: impl AsRef<Path>,
    schemas: &SchemaSet,
    documents: &[RepositoryDocument],
) -> ValidationReport {
    let root = root.as_ref();
    let mut issues = Vec::new();

    for schema_name in schemas.missing_schema_names() {
        issues.push(ValidationIssue::new(
            ValidationCode::SchemaNotFound,
            Some(PathBuf::from("docs/schemas").join(schema_name)),
            format!("schema file {schema_name} was not found"),
        ));
    }

    for duplicate in duplicate_document_ids(documents) {
        for path in duplicate.paths {
            issues.push(ValidationIssue::new(
                ValidationCode::DuplicateId,
                Some(relative_path(root, &path)),
                format!("document ID {} is used more than once", duplicate.id.get()),
            ));
        }
    }

    let ids_by_kind = ids_by_kind(documents);

    for document in documents {
        let path = relative_path(root, &document.path);
        let common = document.document.metadata.common();

        if common.title.trim().is_empty() {
            issues.push(ValidationIssue::new(
                ValidationCode::MissingRequiredField,
                Some(path.clone()),
                "required field title must not be empty",
            ));
        }

        if common.kind != document.expected_kind {
            issues.push(ValidationIssue::new(
                ValidationCode::InvalidKind,
                Some(path.clone()),
                format!(
                    "kind {} does not match expected kind {} for this location",
                    kind_name(common.kind),
                    kind_name(document.expected_kind)
                ),
            ));
        }

        validate_kind_specific_metadata(&mut issues, &path, &document.document.metadata);
        validate_references(
            &mut issues,
            &path,
            &document.document.metadata,
            &ids_by_kind,
        );
    }

    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });

    ValidationReport {
        issues,
        incomplete: false,
    }
}

fn load_optional_schema(root: &Path, file_name: &str) -> Result<Option<Value>, SchemaLoadError> {
    let path = root.join("docs/schemas").join(file_name);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(SchemaLoadError::ReadFile { path, source });
        }
    };

    serde_json::from_str(&raw)
        .map(Some)
        .map_err(|source| SchemaLoadError::InvalidJson { path, source })
}

const REQUIRED_READMES: &[&str] = &[
    "docs/README.md",
    "docs/specs/README.md",
    "docs/designs/README.md",
    "docs/adr/README.md",
    "docs/tasks/README.md",
];

fn check_task_index(
    root: &Path,
    documents: &[RepositoryDocument],
    issues: &mut Vec<ValidationIssue>,
) {
    let path = PathBuf::from("docs/tasks/index.md");
    let Some(index_document) = documents
        .iter()
        .find(|document| document.document.metadata.common().kind == DocumentKind::TaskIndex)
    else {
        issues.push(ValidationIssue::new(
            ValidationCode::IndexOutOfSync,
            Some(path),
            "task index document was not found",
        ));
        return;
    };

    let Ok(markdown) = fs::read_to_string(root.join(&path)) else {
        issues.push(ValidationIssue::new(
            ValidationCode::IndexOutOfSync,
            Some(path),
            "task index document could not be read",
        ));
        return;
    };

    let actual = task_index_entries(&markdown);
    let expected = expected_task_index_entries(documents);
    if actual != expected {
        issues.push(ValidationIssue::new(
            ValidationCode::IndexOutOfSync,
            Some(relative_path(root, &index_document.path)),
            "task index is out of sync with task documents",
        ));
    }
}

fn expected_task_index_entries(
    documents: &[RepositoryDocument],
) -> BTreeMap<String, Vec<(u64, String)>> {
    let mut entries = empty_task_index_entries();

    for document in documents {
        let DocumentMetadata::Task(task) = &document.document.metadata else {
            continue;
        };
        let section = match task.status {
            crate::TaskStatus::Doing => "Doing",
            crate::TaskStatus::Planned => "Planned",
            crate::TaskStatus::Blocked => "Blocked",
            crate::TaskStatus::Done | crate::TaskStatus::Dropped => "Done",
        };
        entries
            .entry(section.to_string())
            .or_default()
            .push((task.common.id.get(), task.common.title.clone()));
    }

    for values in entries.values_mut() {
        values.sort_by_key(|(id, _)| *id);
    }

    entries
}

fn task_index_entries(markdown: &str) -> BTreeMap<String, Vec<(u64, String)>> {
    let mut entries = empty_task_index_entries();
    let mut current_section: Option<String> = None;

    for line in markdown.lines() {
        if let Some(section) = line.strip_prefix("## ") {
            let section = section.trim();
            current_section = entries.contains_key(section).then(|| section.to_string());
            continue;
        }

        let Some(section) = &current_section else {
            continue;
        };
        let Some(item) = line.strip_prefix("- ") else {
            continue;
        };
        let Some((id, title)) = item.split_once(' ') else {
            continue;
        };
        let Ok(id) = id.parse::<u64>() else {
            continue;
        };
        entries
            .entry(section.clone())
            .or_default()
            .push((id, title.to_string()));
    }

    for values in entries.values_mut() {
        values.sort_by_key(|(id, _)| *id);
    }

    entries
}

fn empty_task_index_entries() -> BTreeMap<String, Vec<(u64, String)>> {
    ["Doing", "Planned", "Blocked", "Done"]
        .into_iter()
        .map(|section| (section.to_string(), Vec::new()))
        .collect()
}

fn code_for_parse_error(error: &crate::ParseError) -> ValidationCode {
    match error.kind {
        ParseErrorKind::MissingFrontmatter | ParseErrorKind::UnterminatedFrontmatter => {
            ValidationCode::BadFrontmatter
        }
        ParseErrorKind::InvalidFrontmatter => {
            let message = error.message.as_str();
            if message.contains("missing field") {
                ValidationCode::MissingRequiredField
            } else if message.contains("document ID must be a positive integer") {
                ValidationCode::InvalidId
            } else if message.contains("unknown variant") && message.contains("kind") {
                ValidationCode::InvalidKind
            } else if message.contains("unknown variant") && message.contains("status") {
                ValidationCode::InvalidStatus
            } else if message.contains("unknown variant") && message.contains("type") {
                ValidationCode::InvalidType
            } else if message.contains("unknown variant") && message.contains("priority") {
                ValidationCode::InvalidPriority
            } else {
                ValidationCode::BadFrontmatter
            }
        }
    }
}

fn ids_by_kind(documents: &[RepositoryDocument]) -> BTreeMap<DocumentId, DocumentKind> {
    documents
        .iter()
        .map(|document| {
            (
                document.document.metadata.common().id,
                document.document.metadata.common().kind,
            )
        })
        .collect()
}

fn validate_kind_specific_metadata(
    issues: &mut Vec<ValidationIssue>,
    path: &Path,
    metadata: &DocumentMetadata,
) {
    if let DocumentMetadata::Task(task) = metadata {
        let in_active = path.starts_with("docs/tasks/active");
        let in_done = path.starts_with("docs/tasks/done");
        let completed = matches!(
            task.status,
            crate::TaskStatus::Done | crate::TaskStatus::Dropped
        );

        if in_active && completed {
            issues.push(ValidationIssue::new(
                ValidationCode::TaskDoneInActive,
                Some(path.to_path_buf()),
                "done or dropped task is stored in the active task folder",
            ));
        }

        if in_done && !completed {
            issues.push(ValidationIssue::new(
                ValidationCode::TaskActiveInDone,
                Some(path.to_path_buf()),
                "active task status is stored in the done task folder",
            ));
        }
    }

    if let DocumentMetadata::Adr(adr) = metadata {
        if adr.status == AdrStatus::Superseded && adr.superseded_by.is_none() {
            issues.push(ValidationIssue::new(
                ValidationCode::AdrSupersededWithoutReplacement,
                Some(path.to_path_buf()),
                "superseded ADR must declare superseded_by",
            ));
        }
    }
}

fn validate_references(
    issues: &mut Vec<ValidationIssue>,
    path: &Path,
    metadata: &DocumentMetadata,
    ids_by_kind: &BTreeMap<DocumentId, DocumentKind>,
) {
    match metadata {
        DocumentMetadata::Spec(spec) => {
            validate_optional_reference(
                issues,
                path,
                "superseded_by",
                spec.superseded_by,
                DocumentKind::Spec,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
        }
        DocumentMetadata::Design(design) => {
            validate_references_to_kind(
                issues,
                path,
                "specs",
                &design.specs,
                DocumentKind::Spec,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_references_to_kind(
                issues,
                path,
                "adrs",
                &design.adrs,
                DocumentKind::Adr,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_optional_reference(
                issues,
                path,
                "superseded_by",
                design.superseded_by,
                DocumentKind::Design,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
        }
        DocumentMetadata::Adr(adr) => {
            validate_references_to_kind(
                issues,
                path,
                "related_designs",
                &adr.related_designs,
                DocumentKind::Design,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_references_to_kind(
                issues,
                path,
                "supersedes",
                &adr.supersedes,
                DocumentKind::Adr,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_optional_reference(
                issues,
                path,
                "superseded_by",
                adr.superseded_by,
                DocumentKind::Adr,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
        }
        DocumentMetadata::Task(task) => {
            validate_references_to_kind(
                issues,
                path,
                "specs",
                &task.specs,
                DocumentKind::Spec,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_references_to_kind(
                issues,
                path,
                "designs",
                &task.designs,
                DocumentKind::Design,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_references_to_kind(
                issues,
                path,
                "adrs",
                &task.adrs,
                DocumentKind::Adr,
                ids_by_kind,
                ValidationCode::BrokenReference,
            );
            validate_references_to_kind(
                issues,
                path,
                "depends_on",
                &task.depends_on,
                DocumentKind::Task,
                ids_by_kind,
                ValidationCode::MissingDependency,
            );
        }
        DocumentMetadata::TaskIndex(_) => {}
    }
}

fn validate_optional_reference(
    issues: &mut Vec<ValidationIssue>,
    path: &Path,
    field: &str,
    id: Option<DocumentId>,
    expected_kind: DocumentKind,
    ids_by_kind: &BTreeMap<DocumentId, DocumentKind>,
    code: ValidationCode,
) {
    if let Some(id) = id {
        validate_reference(issues, path, field, id, expected_kind, ids_by_kind, code);
    }
}

fn validate_references_to_kind(
    issues: &mut Vec<ValidationIssue>,
    path: &Path,
    field: &str,
    ids: &[DocumentId],
    expected_kind: DocumentKind,
    ids_by_kind: &BTreeMap<DocumentId, DocumentKind>,
    code: ValidationCode,
) {
    let mut seen = BTreeSet::new();

    for id in ids {
        if !seen.insert(*id) {
            issues.push(ValidationIssue::new(
                code,
                Some(path.to_path_buf()),
                format!("field {field} contains duplicate reference {}", id.get()),
            ));
            continue;
        }

        validate_reference(issues, path, field, *id, expected_kind, ids_by_kind, code);
    }
}

fn validate_reference(
    issues: &mut Vec<ValidationIssue>,
    path: &Path,
    field: &str,
    id: DocumentId,
    expected_kind: DocumentKind,
    ids_by_kind: &BTreeMap<DocumentId, DocumentKind>,
    code: ValidationCode,
) {
    match ids_by_kind.get(&id) {
        Some(actual_kind) if *actual_kind == expected_kind => {}
        Some(actual_kind) => issues.push(ValidationIssue::new(
            code,
            Some(path.to_path_buf()),
            format!(
                "field {field} references ID {}, which is kind {} instead of {}",
                id.get(),
                kind_name(*actual_kind),
                kind_name(expected_kind)
            ),
        )),
        None => issues.push(ValidationIssue::new(
            code,
            Some(path.to_path_buf()),
            format!("field {field} references missing ID {}", id.get()),
        )),
    }
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn kind_name(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Spec => "spec",
        DocumentKind::Design => "design",
        DocumentKind::Adr => "adr",
        DocumentKind::Task => "task",
        DocumentKind::TaskIndex => "task-index",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan_repository;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_schema_files_from_repository() {
        let repo = TestRepo::new("schemas");
        repo.write("docs/schemas/document.schema.json", "{}");
        repo.write("docs/schemas/spec.schema.json", "{}");
        repo.write("docs/schemas/design.schema.json", "{}");
        repo.write("docs/schemas/adr.schema.json", "{}");
        repo.write("docs/schemas/task.schema.json", "{}");

        let schemas = load_schema_set(repo.path()).unwrap();

        assert!(schemas.document.is_some());
        assert!(schemas.spec.is_some());
        assert!(schemas.design.is_some());
        assert!(schemas.adr.is_some());
        assert!(schemas.task.is_some());
    }

    #[test]
    fn reports_missing_schema_files_with_stable_code() {
        let repo = TestRepo::new("missing-schemas");
        repo.write("docs/specs/9-model.md", spec_markdown(9, "Model"));

        let schemas = load_schema_set(repo.path()).unwrap();
        let documents = scan_repository(repo.path()).unwrap();
        let report = validate_documents(repo.path(), &schemas, &documents);

        assert_eq!(report.issues.len(), 5);
        assert!(report
            .issues
            .iter()
            .all(|issue| issue.code == ValidationCode::SchemaNotFound));
    }

    #[test]
    fn valid_repository_has_no_validation_issues() {
        let repo = TestRepo::new("valid");
        write_schemas(&repo);
        repo.write("docs/specs/9-model.md", spec_markdown(9, "Model"));
        repo.write(
            "docs/designs/10-design.md",
            design_markdown(10, "Design", &[9], &[]),
        );
        repo.write(
            "docs/adr/11-decision.md",
            adr_markdown(11, "Decision", "accepted", &[10], &[], None),
        );
        repo.write(
            "docs/tasks/active/12-task.md",
            task_markdown(12, "Task", "planned", &[9], &[10], &[11], &[]),
        );
        repo.write(
            "docs/tasks/done/13-done.md",
            task_markdown(13, "Done", "done", &[], &[], &[], &[12]),
        );
        repo.write(
            "docs/tasks/index.md",
            "\
---
id: 14
title: Task Index
kind: task-index
---

# Tasks
",
        );

        let report = validate_repository(repo.path()).unwrap();

        assert!(report.is_valid(), "{:?}", report.issues);
    }

    #[test]
    fn readmes_and_agents_do_not_need_frontmatter() {
        let repo = TestRepo::new("unnumbered");
        write_schemas(&repo);
        repo.write("AGENTS.md", "# Agents\n");
        repo.write("docs/README.md", "# Docs\n");
        repo.write("docs/specs/README.md", "# Specs\n");

        let report = validate_repository(repo.path()).unwrap();

        assert!(report.is_valid(), "{:?}", report.issues);
    }

    #[test]
    fn validates_duplicates_kind_mismatch_references_and_task_folder_state() {
        let repo = TestRepo::new("invalid");
        write_schemas(&repo);
        repo.write("docs/specs/9-model.md", spec_markdown(9, "Model"));
        repo.write(
            "docs/tasks/active/10-wrong-kind.md",
            "\
---
id: 10
title: Wrong Kind
kind: spec
---

# Wrong Kind
",
        );
        repo.write(
            "docs/tasks/active/11-done-active.md",
            task_markdown(9, "Done Active", "done", &[999], &[], &[], &[998]),
        );
        repo.write(
            "docs/tasks/done/12-active-done.md",
            task_markdown(12, "Active Done", "planned", &[], &[], &[], &[]),
        );
        repo.write(
            "docs/adr/13-old.md",
            adr_markdown(13, "Old", "superseded", &[], &[], None),
        );

        let report = validate_repository(repo.path()).unwrap();
        let codes = report
            .issues
            .iter()
            .map(|issue| issue.code)
            .collect::<BTreeSet<_>>();

        assert!(codes.contains(&ValidationCode::DuplicateId));
        assert!(codes.contains(&ValidationCode::InvalidKind));
        assert!(codes.contains(&ValidationCode::BrokenReference));
        assert!(codes.contains(&ValidationCode::MissingDependency));
        assert!(codes.contains(&ValidationCode::TaskDoneInActive));
        assert!(codes.contains(&ValidationCode::TaskActiveInDone));
        assert!(codes.contains(&ValidationCode::AdrSupersededWithoutReplacement));
    }

    #[test]
    fn maps_missing_frontmatter_to_bad_frontmatter_code() {
        let repo = TestRepo::new("bad-frontmatter");
        write_schemas(&repo);
        repo.write("docs/specs/9-missing-frontmatter.md", "# Missing\n");

        let report = validate_repository(repo.path()).unwrap();

        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.issues[0].code, ValidationCode::BadFrontmatter);
    }

    #[test]
    fn maps_parse_failures_to_specific_validation_codes() {
        let cases = [
            (
                "missing-required-field",
                "docs/tasks/active/9-missing-status.md",
                "\
---
id: 9
title: Missing Status
kind: task
type: feature
---

# Missing Status
",
                ValidationCode::MissingRequiredField,
            ),
            (
                "invalid-id",
                "docs/specs/0-invalid-id.md",
                "\
---
id: 0
title: Invalid ID
kind: spec
---

# Invalid ID
",
                ValidationCode::InvalidId,
            ),
            (
                "invalid-kind",
                "docs/specs/9-invalid-kind.md",
                "\
---
id: 9
title: Invalid Kind
kind: unknown
---

# Invalid Kind
",
                ValidationCode::InvalidKind,
            ),
            (
                "invalid-type",
                "docs/tasks/active/9-invalid-type.md",
                "\
---
id: 9
title: Invalid Type
kind: task
type: unknown
status: planned
---

# Invalid Type
",
                ValidationCode::InvalidType,
            ),
            (
                "invalid-status",
                "docs/tasks/active/9-invalid-status.md",
                "\
---
id: 9
title: Invalid Status
kind: task
type: feature
status: unknown
---

# Invalid Status
",
                ValidationCode::InvalidStatus,
            ),
            (
                "invalid-priority",
                "docs/tasks/active/9-invalid-priority.md",
                "\
---
id: 9
title: Invalid Priority
kind: task
type: feature
status: planned
priority: unknown
---

# Invalid Priority
",
                ValidationCode::InvalidPriority,
            ),
        ];

        for (name, path, markdown, expected_code) in cases {
            let repo = TestRepo::new(name);
            write_schemas(&repo);
            repo.write(path, markdown);

            let report = validate_repository(repo.path()).unwrap();

            assert_eq!(report.issues.len(), 1, "{name}: {:?}", report.issues);
            assert_eq!(report.issues[0].code, expected_code, "{name}");
        }
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
            let root = std::env::temp_dir().join(format!("vibe-doc-validation-{name}-{unique}"));
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

    fn write_schemas(repo: &TestRepo) {
        repo.write("docs/schemas/document.schema.json", "{}");
        repo.write("docs/schemas/spec.schema.json", "{}");
        repo.write("docs/schemas/design.schema.json", "{}");
        repo.write("docs/schemas/adr.schema.json", "{}");
        repo.write("docs/schemas/task.schema.json", "{}");
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

    fn design_markdown(id: u64, title: &str, specs: &[u64], adrs: &[u64]) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: design
specs:{specs}
adrs:{adrs}
---

# {title}
",
            specs = yaml_id_list(specs),
            adrs = yaml_id_list(adrs),
        )
    }

    fn adr_markdown(
        id: u64,
        title: &str,
        status: &str,
        related_designs: &[u64],
        supersedes: &[u64],
        superseded_by: Option<u64>,
    ) -> String {
        let superseded_by = superseded_by
            .map(|id| format!("superseded_by: {id}\n"))
            .unwrap_or_default();

        format!(
            "\
---
id: {id}
title: {title}
kind: adr
status: {status}
related_designs:{related_designs}
supersedes:{supersedes}
{superseded_by}---

# {title}
",
            related_designs = yaml_id_list(related_designs),
            supersedes = yaml_id_list(supersedes),
        )
    }

    fn task_markdown(
        id: u64,
        title: &str,
        status: &str,
        specs: &[u64],
        designs: &[u64],
        adrs: &[u64],
        depends_on: &[u64],
    ) -> String {
        format!(
            "\
---
id: {id}
title: {title}
kind: task
type: feature
status: {status}
specs:{specs}
designs:{designs}
adrs:{adrs}
depends_on:{depends_on}
---

# {title}
",
            specs = yaml_id_list(specs),
            designs = yaml_id_list(designs),
            adrs = yaml_id_list(adrs),
            depends_on = yaml_id_list(depends_on),
        )
    }

    fn yaml_id_list(ids: &[u64]) -> String {
        if ids.is_empty() {
            " []\n".to_owned()
        } else {
            let mut raw = "\n".to_owned();
            for id in ids {
                raw.push_str(&format!("  - {id}\n"));
            }
            raw
        }
    }
}
