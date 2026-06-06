//! Shared core logic for vibe-doc.

use serde::de::{self, Visitor};
use serde::Deserialize;
use std::fmt;
use std::path::Path;

/// Stable crate identifier used by workspace smoke tests.
pub const CRATE_NAME: &str = "vibe-doc-core";

/// Positive global document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(u64);

impl DocumentId {
    /// Create a document ID when the value is positive.
    pub fn new(value: u64) -> Option<Self> {
        (value > 0).then_some(Self(value))
    }

    /// Return the numeric ID value.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for DocumentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DocumentIdVisitor;

        impl Visitor<'_> for DocumentIdVisitor {
            type Value = DocumentId;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a positive integer document ID")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= 0 {
                    return Err(E::custom("document ID must be a positive integer"));
                }

                Ok(DocumentId(value as u64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DocumentId::new(value)
                    .ok_or_else(|| E::custom("document ID must be a positive integer"))
            }
        }

        deserializer.deserialize_any(DocumentIdVisitor)
    }
}

/// VDoc document kind values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentKind {
    Spec,
    Design,
    Adr,
    Task,
    TaskIndex,
}

/// Metadata shared by every numbered VDoc document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommonMetadata {
    pub id: DocumentId,
    pub title: String,
    pub kind: DocumentKind,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Spec lifecycle values currently recognized by the document model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecStatus {
    Deprecated,
}

/// Design lifecycle values currently recognized by the document model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesignStatus {
    Deprecated,
}

/// ADR status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

/// Task type values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskType {
    Feature,
    Bug,
    Refactor,
    Chore,
    Docs,
    Test,
    Spike,
}

/// Task status values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Planned,
    Doing,
    Blocked,
    Done,
    Dropped,
}

/// Task priority values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Frontmatter for spec documents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SpecMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
    pub status: Option<SpecStatus>,
    pub superseded_by: Option<DocumentId>,
}

/// Frontmatter for design documents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DesignMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
    #[serde(default)]
    pub specs: Vec<DocumentId>,
    #[serde(default)]
    pub adrs: Vec<DocumentId>,
    pub status: Option<DesignStatus>,
    pub superseded_by: Option<DocumentId>,
}

/// Frontmatter for ADR documents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AdrMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
    pub status: AdrStatus,
    pub date: Option<String>,
    #[serde(default)]
    pub related_designs: Vec<DocumentId>,
    #[serde(default)]
    pub supersedes: Vec<DocumentId>,
    pub superseded_by: Option<DocumentId>,
}

/// Frontmatter for task documents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TaskMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: Option<Priority>,
    #[serde(default)]
    pub specs: Vec<DocumentId>,
    #[serde(default)]
    pub designs: Vec<DocumentId>,
    #[serde(default)]
    pub adrs: Vec<DocumentId>,
    #[serde(default)]
    pub depends_on: Vec<DocumentId>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Frontmatter for the task index document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TaskIndexMetadata {
    #[serde(flatten)]
    pub common: CommonMetadata,
}

/// Typed metadata for any numbered VDoc document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentMetadata {
    Spec(SpecMetadata),
    Design(DesignMetadata),
    Adr(AdrMetadata),
    Task(TaskMetadata),
    TaskIndex(TaskIndexMetadata),
}

impl DocumentMetadata {
    /// Return the common metadata regardless of document kind.
    pub fn common(&self) -> &CommonMetadata {
        match self {
            Self::Spec(metadata) => &metadata.common,
            Self::Design(metadata) => &metadata.common,
            Self::Adr(metadata) => &metadata.common,
            Self::Task(metadata) => &metadata.common,
            Self::TaskIndex(metadata) => &metadata.common,
        }
    }
}

/// Source identifier used in parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SourceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SourceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&Path> for SourceId {
    fn from(value: &Path) -> Self {
        Self::new(value.display().to_string())
    }
}

/// One-based source location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// One-based source span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub const fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub const fn point(location: SourceLocation) -> Self {
        Self {
            start: location,
            end: location,
        }
    }
}

/// Parsed YAML frontmatter and source context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterBlock {
    pub raw: String,
    pub span: SourceSpan,
    pub content_start: SourceLocation,
}

/// A parsed numbered VDoc Markdown document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberedDocument {
    pub source: SourceId,
    pub frontmatter: FrontmatterBlock,
    pub metadata: DocumentMetadata,
    pub body: String,
}

/// An unnumbered Markdown document such as `README.md` or `AGENTS.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnnumberedDocument {
    pub source: SourceId,
    pub body: String,
}

/// Parsed Markdown document classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownDocument {
    Numbered(NumberedDocument),
    Unnumbered(UnnumberedDocument),
}

/// Parser error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    MissingFrontmatter,
    UnterminatedFrontmatter,
    InvalidFrontmatter,
}

/// Error produced while parsing Markdown frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub source: SourceId,
    pub kind: ParseErrorKind,
    pub span: SourceSpan,
    pub message: String,
}

impl ParseError {
    fn new(
        source: SourceId,
        kind: ParseErrorKind,
        span: SourceSpan,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source,
            kind,
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}: {}",
            self.source.as_str(),
            self.span.start.line,
            self.span.start.column,
            self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parse Markdown and classify documents without frontmatter as unnumbered.
pub fn parse_markdown_document(
    source: impl Into<SourceId>,
    markdown: &str,
) -> Result<MarkdownDocument, ParseError> {
    let source = source.into();

    match split_frontmatter(source.clone(), markdown)? {
        Some(split) => {
            let metadata = parse_metadata(source.clone(), &split.frontmatter)?;
            Ok(MarkdownDocument::Numbered(NumberedDocument {
                source,
                frontmatter: split.frontmatter,
                metadata,
                body: split.body,
            }))
        }
        None => Ok(MarkdownDocument::Unnumbered(UnnumberedDocument {
            source,
            body: markdown.to_owned(),
        })),
    }
}

/// Parse Markdown that must be a numbered VDoc document.
pub fn parse_numbered_document(
    source: impl Into<SourceId>,
    markdown: &str,
) -> Result<NumberedDocument, ParseError> {
    let source = source.into();

    match parse_markdown_document(source.clone(), markdown)? {
        MarkdownDocument::Numbered(document) => Ok(document),
        MarkdownDocument::Unnumbered(_) => Err(ParseError::new(
            source,
            ParseErrorKind::MissingFrontmatter,
            SourceSpan::point(SourceLocation::new(1, 1)),
            "missing YAML frontmatter",
        )),
    }
}

struct SplitFrontmatter {
    frontmatter: FrontmatterBlock,
    body: String,
}

fn split_frontmatter(
    source: SourceId,
    markdown: &str,
) -> Result<Option<SplitFrontmatter>, ParseError> {
    let Some(first_line) = read_line(markdown, 0) else {
        return Ok(None);
    };

    if first_line.content != "---" {
        return Ok(None);
    }

    let mut cursor = first_line.next_index;
    let mut line_number = 2;

    while let Some(line) = read_line(markdown, cursor) {
        if line.content == "---" {
            let raw = markdown[first_line.next_index..line.start_index].to_owned();
            let body = markdown[line.next_index..].to_owned();
            let span = SourceSpan::new(
                SourceLocation::new(1, 1),
                SourceLocation::new(line_number, line.content.len() + 1),
            );

            return Ok(Some(SplitFrontmatter {
                frontmatter: FrontmatterBlock {
                    raw,
                    span,
                    content_start: SourceLocation::new(2, 1),
                },
                body,
            }));
        }

        cursor = line.next_index;
        line_number += 1;
    }

    Err(ParseError::new(
        source,
        ParseErrorKind::UnterminatedFrontmatter,
        SourceSpan::point(SourceLocation::new(1, 1)),
        "frontmatter opening delimiter has no closing delimiter",
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Line<'a> {
    content: &'a str,
    start_index: usize,
    next_index: usize,
}

fn read_line(markdown: &str, start_index: usize) -> Option<Line<'_>> {
    if start_index >= markdown.len() {
        return None;
    }

    let remainder = &markdown[start_index..];
    let newline_offset = remainder.find('\n');
    let (line_with_possible_cr, next_index) = match newline_offset {
        Some(offset) => (&remainder[..offset], start_index + offset + 1),
        None => (remainder, markdown.len()),
    };
    let content = line_with_possible_cr
        .strip_suffix('\r')
        .unwrap_or(line_with_possible_cr);

    Some(Line {
        content,
        start_index,
        next_index,
    })
}

fn parse_metadata(
    source: SourceId,
    frontmatter: &FrontmatterBlock,
) -> Result<DocumentMetadata, ParseError> {
    let common = deserialize_frontmatter::<CommonMetadata>(source.clone(), frontmatter)?;

    match common.kind {
        DocumentKind::Spec => {
            deserialize_frontmatter::<SpecMetadata>(source, frontmatter).map(DocumentMetadata::Spec)
        }
        DocumentKind::Design => deserialize_frontmatter::<DesignMetadata>(source, frontmatter)
            .map(DocumentMetadata::Design),
        DocumentKind::Adr => {
            deserialize_frontmatter::<AdrMetadata>(source, frontmatter).map(DocumentMetadata::Adr)
        }
        DocumentKind::Task => {
            deserialize_frontmatter::<TaskMetadata>(source, frontmatter).map(DocumentMetadata::Task)
        }
        DocumentKind::TaskIndex => {
            deserialize_frontmatter::<TaskIndexMetadata>(source, frontmatter)
                .map(DocumentMetadata::TaskIndex)
        }
    }
}

fn deserialize_frontmatter<T>(
    source: SourceId,
    frontmatter: &FrontmatterBlock,
) -> Result<T, ParseError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_yaml::from_str::<T>(&frontmatter.raw).map_err(|error| {
        let location = error
            .location()
            .map(|location| {
                SourceLocation::new(
                    frontmatter.content_start.line + location.line() - 1,
                    location.column(),
                )
            })
            .unwrap_or(frontmatter.content_start);

        ParseError::new(
            source,
            ParseErrorKind::InvalidFrontmatter,
            SourceSpan::point(location),
            format!("invalid YAML frontmatter: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_crate_name() {
        assert_eq!(CRATE_NAME, "vibe-doc-core");
    }

    #[test]
    fn parses_spec_frontmatter() {
        let markdown = "\
---
id: 9
title: Document Model
kind: spec
tags:
  - vibe-doc
---

# Document Model
";

        let document = parse_numbered_document("docs/specs/9-document-model.md", markdown).unwrap();

        assert_eq!(
            document.frontmatter.content_start,
            SourceLocation::new(2, 1)
        );
        assert_eq!(document.body, "\n# Document Model\n");

        let DocumentMetadata::Spec(metadata) = document.metadata else {
            panic!("expected spec metadata");
        };

        assert_eq!(metadata.common.id, DocumentId::new(9).unwrap());
        assert_eq!(metadata.common.title, "Document Model");
        assert_eq!(metadata.common.kind, DocumentKind::Spec);
        assert_eq!(metadata.common.tags, ["vibe-doc"]);
    }

    #[test]
    fn parses_design_frontmatter() {
        let markdown = "\
---
id: 10
title: CLI Design
kind: design
specs:
  - 9
adrs:
  - 12
---

# CLI Design
";

        let document = parse_numbered_document("docs/designs/10-cli-design.md", markdown).unwrap();
        let DocumentMetadata::Design(metadata) = document.metadata else {
            panic!("expected design metadata");
        };

        assert_eq!(metadata.specs, [DocumentId::new(9).unwrap()]);
        assert_eq!(metadata.adrs, [DocumentId::new(12).unwrap()]);
    }

    #[test]
    fn parses_adr_frontmatter() {
        let markdown = "\
---
id: 12
title: Use Rust
kind: adr
status: accepted
related_designs:
  - 10
---

# Use Rust
";

        let document = parse_numbered_document("docs/adr/12-use-rust.md", markdown).unwrap();
        let DocumentMetadata::Adr(metadata) = document.metadata else {
            panic!("expected ADR metadata");
        };

        assert_eq!(metadata.status, AdrStatus::Accepted);
        assert_eq!(metadata.related_designs, [DocumentId::new(10).unwrap()]);
    }

    #[test]
    fn parses_task_frontmatter() {
        let markdown = "\
---
id: 16
title: Implement parser
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
depends_on:
  - 15
---

# Implement parser
";

        let document = parse_numbered_document("docs/tasks/active/16-parser.md", markdown).unwrap();
        let DocumentMetadata::Task(metadata) = document.metadata else {
            panic!("expected task metadata");
        };

        assert_eq!(metadata.task_type, TaskType::Feature);
        assert_eq!(metadata.status, TaskStatus::Planned);
        assert_eq!(metadata.priority, Some(Priority::High));
        assert_eq!(metadata.specs, [DocumentId::new(9).unwrap()]);
        assert_eq!(metadata.depends_on, [DocumentId::new(15).unwrap()]);
    }

    #[test]
    fn parses_task_index_frontmatter() {
        let markdown = "\
---
id: 7
title: Task Index
kind: task-index
---

# Task Index
";

        let document = parse_numbered_document("docs/tasks/index.md", markdown).unwrap();
        let DocumentMetadata::TaskIndex(metadata) = document.metadata else {
            panic!("expected task index metadata");
        };

        assert_eq!(metadata.common.id, DocumentId::new(7).unwrap());
        assert_eq!(metadata.common.kind, DocumentKind::TaskIndex);
    }

    #[test]
    fn classifies_markdown_without_frontmatter_as_unnumbered() {
        let document = parse_markdown_document("docs/README.md", "# Docs\n").unwrap();

        let MarkdownDocument::Unnumbered(document) = document else {
            panic!("expected unnumbered document");
        };

        assert_eq!(document.source.as_str(), "docs/README.md");
        assert_eq!(document.body, "# Docs\n");
    }

    #[test]
    fn reports_missing_frontmatter_for_numbered_parser() {
        let error = parse_numbered_document("docs/specs/missing.md", "# Missing\n").unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::MissingFrontmatter);
        assert_eq!(error.span.start, SourceLocation::new(1, 1));
    }

    #[test]
    fn reports_unterminated_frontmatter() {
        let error = parse_numbered_document("docs/specs/broken.md", "---\nid: 1\n").unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::UnterminatedFrontmatter);
        assert_eq!(error.span.start, SourceLocation::new(1, 1));
    }

    #[test]
    fn reports_malformed_yaml_with_markdown_location() {
        let markdown = "\
---
id: 1
title: Broken
kind: spec
tags: [
---

# Broken
";

        let error = parse_numbered_document("docs/specs/broken.md", markdown).unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::InvalidFrontmatter);
        assert!(error.span.start.line >= 2);
    }

    #[test]
    fn reports_missing_required_task_status() {
        let markdown = "\
---
id: 16
title: Implement parser
kind: task
type: feature
---

# Implement parser
";

        let error =
            parse_numbered_document("docs/tasks/active/16-parser.md", markdown).unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::InvalidFrontmatter);
        assert!(error.message.contains("status"));
    }

    #[test]
    fn rejects_non_positive_document_ids() {
        let markdown = "\
---
id: 0
title: Invalid
kind: spec
---

# Invalid
";

        let error = parse_numbered_document("docs/specs/0-invalid.md", markdown).unwrap_err();

        assert_eq!(error.kind, ParseErrorKind::InvalidFrontmatter);
        assert!(error.message.contains("positive integer"));
    }
}
