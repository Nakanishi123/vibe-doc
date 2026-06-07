//! Shared core logic for vibe-doc.

mod allocation;
mod document;
mod init;
mod lifecycle;
mod metadata;
mod new;
mod parser;
mod repository;
mod task_index;
mod validation;

pub use allocation::{
    allocate_next_document_id, document_filename, document_relative_path, duplicate_document_ids,
    next_document_id, slugify_title, sorted_document_ids, DocumentLocation, DuplicateDocumentId,
    IdAllocationError,
};
pub use document::{
    FrontmatterBlock, MarkdownDocument, NumberedDocument, SourceId, SourceLocation, SourceSpan,
    UnnumberedDocument,
};
pub use init::{
    init_repository, InitChange, InitChangeAction, InitChangeKind, InitError, InitOptions, InitPlan,
};
pub use lifecycle::{
    complete_task, start_task, CompleteTaskOptions, TaskLifecycleAction, TaskLifecycleChange,
    TaskLifecycleError, TaskLifecycleOptions, TaskLifecyclePlan,
};
pub use metadata::{
    AdrMetadata, AdrStatus, CommonMetadata, DesignMetadata, DesignStatus, DocumentId, DocumentKind,
    DocumentMetadata, Priority, SpecMetadata, SpecStatus, TaskIndexMetadata, TaskMetadata,
    TaskStatus, TaskType,
};
pub use new::{
    new_adr, new_design, new_spec, new_task, NewAdrOptions, NewChange, NewChangeAction, NewError,
    NewOptions, NewPlan, NewTaskOptions,
};
pub use parser::{parse_markdown_document, parse_numbered_document, ParseError, ParseErrorKind};
pub use repository::{
    expected_kind_for_path, expected_kind_for_relative_path, scan_repository, RepositoryDocument,
    RepositoryScanError,
};
pub use task_index::{
    rebuild_task_index, TaskIndexRebuildAction, TaskIndexRebuildError, TaskIndexRebuildOptions,
    TaskIndexRebuildPlan,
};
pub use validation::{
    check_repository, load_schema_set, validate_documents, validate_repository, SchemaLoadError,
    SchemaSet, ValidationCode, ValidationIssue, ValidationReport, ValidationRunError,
};

/// Stable crate identifier used by workspace smoke tests.
pub const CRATE_NAME: &str = "vibe-doc-core";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_crate_name() {
        assert_eq!(CRATE_NAME, "vibe-doc-core");
    }
}
