use std::collections::BTreeMap;
use std::path::PathBuf;

/// A Markdown file managed by vibe-doc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub path: PathBuf,
    pub metadata: Metadata,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Metadata {
    pub schema_version: Option<u32>,
    pub id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub related: Vec<String>,
    pub depends_on: Vec<String>,
    pub extra: BTreeMap<String, String>,
}
