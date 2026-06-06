use crate::{
    AdrMetadata, CommonMetadata, DesignMetadata, DocumentKind, DocumentMetadata, FrontmatterBlock,
    MarkdownDocument, NumberedDocument, SourceId, SourceLocation, SourceSpan, SpecMetadata,
    TaskIndexMetadata, TaskMetadata, UnnumberedDocument,
};
use serde::Deserialize;
use std::fmt;

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
            Ok(MarkdownDocument::Numbered(Box::new(NumberedDocument {
                source,
                frontmatter: split.frontmatter,
                metadata,
                body: split.body,
            })))
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
        MarkdownDocument::Numbered(document) => Ok(*document),
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
    use crate::{
        AdrStatus, DocumentId, MarkdownDocument, Priority, SourceLocation, TaskStatus, TaskType,
    };

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
