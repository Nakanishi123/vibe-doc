use crate::DocumentId;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub const VDOC_DIR: &str = ".vdoc";
pub const RUNS_DIR: &str = "runs";
pub const RUN_JSON_FILE: &str = "run.json";
pub const PROMPT_FILE: &str = "prompt.md";
pub const EVENTS_FILE: &str = "events.ndjson";
pub const TERMINAL_LOG_FILE: &str = "terminal.log";
pub const DIFF_FILE: &str = "diff.patch";
pub const REVIEW_FILE: &str = "review.md";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRun {
    pub task_id: DocumentId,
    pub run_id: String,
    pub agent_kind: String,
    pub status: AgentRunStatus,
    pub worktree_path: Option<PathBuf>,
    pub created_at: String,
    pub updated_at: String,
    pub exit_result: Option<AgentRunExitResult>,
    pub artifacts: AgentRunArtifacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRunStatus {
    Prepared,
    PromptApproved,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Rejected,
    Accepted,
}

impl AgentRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::PromptApproved => "prompt-approved",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Rejected => "rejected",
            Self::Accepted => "accepted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunExitResult {
    pub code: Option<i32>,
    pub signal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunArtifacts {
    pub directory: PathBuf,
    pub run_json: PathBuf,
    pub prompt: PathBuf,
    pub events: PathBuf,
    pub terminal_log: PathBuf,
    pub diff: PathBuf,
    pub review: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentRunOptions {
    pub task_id: DocumentId,
    pub agent_kind: String,
    pub worktree_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum AgentRunStorageError {
    #[error("could not locate vibe-doc repository root from {}", start.display())]
    RepositoryRootNotFound { start: PathBuf },
    #[error("agent run ID `{run_id}` is not path-safe")]
    UnsafeRunId { run_id: String },
    #[error("failed to create directory {}: {source}", path.display())]
    CreateDir {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write {}: {source}", path.display())]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize agent run metadata: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("could not allocate a unique agent run ID under {}", runs_dir.display())]
    ExhaustedRunIds { runs_dir: PathBuf },
}

pub fn find_repository_root(start: impl AsRef<Path>) -> Result<PathBuf, AgentRunStorageError> {
    let start = start.as_ref();
    let mut current = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if is_vibe_doc_root(&current) {
            return Ok(current);
        }

        if !current.pop() {
            return Err(AgentRunStorageError::RepositoryRootNotFound {
                start: start.to_path_buf(),
            });
        }
    }
}

pub fn agent_runs_dir(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(VDOC_DIR).join(RUNS_DIR)
}

pub fn allocate_agent_run_id(
    root: impl AsRef<Path>,
    task_id: DocumentId,
) -> Result<String, AgentRunStorageError> {
    let runs_dir = agent_runs_dir(root);
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");

    for counter in 1..=1000 {
        let run_id = format!("run-{}-{timestamp}-{counter:03}", task_id.get());
        if !runs_dir.join(&run_id).exists() {
            return Ok(run_id);
        }
    }

    Err(AgentRunStorageError::ExhaustedRunIds { runs_dir })
}

pub fn agent_run_artifacts(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
) -> Result<AgentRunArtifacts, AgentRunStorageError> {
    let run_id = run_id.as_ref();
    if !is_safe_run_id(run_id) {
        return Err(AgentRunStorageError::UnsafeRunId {
            run_id: run_id.to_string(),
        });
    }

    let directory = agent_runs_dir(root).join(run_id);
    Ok(AgentRunArtifacts {
        run_json: directory.join(RUN_JSON_FILE),
        prompt: directory.join(PROMPT_FILE),
        events: directory.join(EVENTS_FILE),
        terminal_log: directory.join(TERMINAL_LOG_FILE),
        diff: directory.join(DIFF_FILE),
        review: directory.join(REVIEW_FILE),
        directory,
    })
}

pub fn create_agent_run_artifact_dir(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
) -> Result<AgentRunArtifacts, AgentRunStorageError> {
    let artifacts = agent_run_artifacts(root, run_id)?;
    fs::create_dir_all(&artifacts.directory).map_err(|source| AgentRunStorageError::CreateDir {
        path: artifacts.directory.clone(),
        source,
    })?;
    Ok(artifacts)
}

pub fn create_agent_run(
    root: impl AsRef<Path>,
    options: CreateAgentRunOptions,
) -> Result<AgentRun, AgentRunStorageError> {
    let root = root.as_ref();
    let run_id = allocate_agent_run_id(root, options.task_id)?;
    let artifacts = create_agent_run_artifact_dir(root, &run_id)?;
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let run = AgentRun {
        task_id: options.task_id,
        run_id,
        agent_kind: options.agent_kind,
        status: AgentRunStatus::Prepared,
        worktree_path: options.worktree_path,
        created_at: now.clone(),
        updated_at: now,
        exit_result: None,
        artifacts,
    };
    write_agent_run_metadata(&run)?;
    Ok(run)
}

pub fn write_agent_run_metadata(run: &AgentRun) -> Result<(), AgentRunStorageError> {
    let content = serde_json::to_string_pretty(run).map_err(AgentRunStorageError::Serialize)?;
    fs::write(&run.artifacts.run_json, format!("{content}\n")).map_err(|source| {
        AgentRunStorageError::WriteFile {
            path: run.artifacts.run_json.clone(),
            source,
        }
    })
}

fn is_vibe_doc_root(path: &Path) -> bool {
    path.join("docs").is_dir() && (path.join("AGENTS.md").is_file() || path.join(VDOC_DIR).is_dir())
}

fn is_safe_run_id(run_id: &str) -> bool {
    !run_id.is_empty()
        && run_id != "."
        && run_id != ".."
        && run_id
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_' | '.'))
        && Path::new(run_id)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan_repository;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn locates_non_worktree_repository_root() {
        let repo = TestRepo::new("root");
        repo.write("AGENTS.md", "# Agents\n");
        repo.write("docs/tasks/active/37-task.md", "# Task\n");
        repo.mkdir("docs/specs/nested");

        let root = find_repository_root(repo.path().join("docs/specs/nested")).unwrap();

        assert_eq!(root, repo.path());
    }

    #[test]
    fn allocates_run_ids_without_colliding_with_existing_directories() {
        let repo = TestRepo::new("allocation");
        let task_id = DocumentId::new(37).unwrap();
        let first = allocate_agent_run_id(repo.path(), task_id).unwrap();
        repo.mkdir(format!(".vdoc/runs/{first}"));

        let second = allocate_agent_run_id(repo.path(), task_id).unwrap();

        assert_ne!(first, second);
        assert!(second.starts_with("run-37-"));
    }

    #[test]
    fn creates_expected_artifact_paths_safely() {
        let repo = TestRepo::new("artifacts");

        let artifacts = create_agent_run_artifact_dir(repo.path(), "run-37-example").unwrap();

        assert!(artifacts.directory.is_dir());
        assert_eq!(
            artifacts.directory,
            repo.path().join(".vdoc/runs/run-37-example")
        );
        assert_eq!(artifacts.run_json, artifacts.directory.join(RUN_JSON_FILE));
        assert_eq!(artifacts.prompt, artifacts.directory.join(PROMPT_FILE));
        assert_eq!(artifacts.events, artifacts.directory.join(EVENTS_FILE));
        assert_eq!(
            artifacts.terminal_log,
            artifacts.directory.join(TERMINAL_LOG_FILE)
        );
        assert_eq!(artifacts.diff, artifacts.directory.join(DIFF_FILE));
        assert_eq!(artifacts.review, artifacts.directory.join(REVIEW_FILE));
    }

    #[test]
    fn rejects_unsafe_run_ids() {
        let repo = TestRepo::new("unsafe");

        for run_id in ["", ".", "..", "../outside", "nested/run", "nested\\run"] {
            let error = agent_run_artifacts(repo.path(), run_id).unwrap_err();
            assert!(matches!(error, AgentRunStorageError::UnsafeRunId { .. }));
        }
    }

    #[test]
    fn writes_run_json_metadata() {
        let repo = TestRepo::new("metadata");
        let run = create_agent_run(
            repo.path(),
            CreateAgentRunOptions {
                task_id: DocumentId::new(37).unwrap(),
                agent_kind: "codex".to_string(),
                worktree_path: Some(repo.path().join(".vdoc/worktrees/example")),
            },
        )
        .unwrap();

        let raw = fs::read_to_string(&run.artifacts.run_json).unwrap();
        let saved: AgentRun = serde_json::from_str(&raw).unwrap();

        assert_eq!(saved.task_id, DocumentId::new(37).unwrap());
        assert_eq!(saved.run_id, run.run_id);
        assert_eq!(saved.agent_kind, "codex");
        assert_eq!(saved.status, AgentRunStatus::Prepared);
        assert_eq!(saved.artifacts.prompt, run.artifacts.prompt);
    }

    #[test]
    fn run_artifacts_are_not_scanned_as_documents() {
        let repo = TestRepo::new("scan");
        repo.write("AGENTS.md", "# Agents\n");
        repo.write(
            "docs/specs/12-spec.md",
            "\
---
id: 12
title: Spec
kind: spec
---

# Spec
",
        );
        repo.write(
            ".vdoc/runs/run-37-example/999-runtime.md",
            "\
---
id: 999
title: Runtime
kind: spec
---

# Runtime
",
        );

        let documents = scan_repository(repo.path()).unwrap();

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].document.metadata.common().id.get(), 12);
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
            let root = std::env::temp_dir().join(format!("vibe-doc-agent-run-{name}-{unique}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn mkdir(&self, relative_path: impl AsRef<Path>) {
            fs::create_dir_all(self.root.join(relative_path)).unwrap();
        }

        fn write(&self, relative_path: impl AsRef<Path>, content: impl AsRef<str>) {
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
}
