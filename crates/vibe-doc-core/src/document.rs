use crate::DocumentMetadata;
use std::path::Path;

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
    Numbered(Box<NumberedDocument>),
    Unnumbered(UnnumberedDocument),
}
