use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Positive global document identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
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
