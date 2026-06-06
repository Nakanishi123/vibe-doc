//! Shared core logic for vibe-doc.

mod allocation;
mod document;
mod metadata;
mod parser;
mod repository;
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
pub use metadata::{
    AdrMetadata, AdrStatus, CommonMetadata, DesignMetadata, DesignStatus, DocumentId, DocumentKind,
    DocumentMetadata, Priority, SpecMetadata, SpecStatus, TaskIndexMetadata, TaskMetadata,
    TaskStatus, TaskType,
};
pub use parser::{parse_markdown_document, parse_numbered_document, ParseError, ParseErrorKind};
pub use repository::{
    expected_kind_for_path, expected_kind_for_relative_path, scan_repository, RepositoryDocument,
    RepositoryScanError,
};
pub use validation::{
    load_schema_set, validate_documents, validate_repository, SchemaLoadError, SchemaSet,
    ValidationCode, ValidationIssue, ValidationReport, ValidationRunError,
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
