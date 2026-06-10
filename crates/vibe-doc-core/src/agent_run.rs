use crate::DocumentId;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use thiserror::Error;

pub const VDOC_DIR: &str = ".vdoc";
pub const RUNS_DIR: &str = "runs";
pub const WORKTREES_DIR: &str = "worktrees";
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
    #[error("agent worktree path {} is outside {}", path.display(), allowed_dir.display())]
    UnsafeWorktreePath { path: PathBuf, allowed_dir: PathBuf },
    #[error("agent worktree path {} already exists", path.display())]
    WorktreePathExists { path: PathBuf },
    #[error("git worktree command failed: {message}")]
    GitWorktree { message: String },
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

pub fn agent_worktrees_dir(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(VDOC_DIR).join(WORKTREES_DIR)
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

pub fn agent_worktree_name(
    task_id: DocumentId,
    run_id: impl AsRef<str>,
) -> Result<String, AgentRunStorageError> {
    let run_id = run_id.as_ref();
    if !is_safe_run_id(run_id) {
        return Err(AgentRunStorageError::UnsafeRunId {
            run_id: run_id.to_string(),
        });
    }

    Ok(format!("task-{}-{run_id}", task_id.get()))
}

pub fn agent_worktree_path(
    root: impl AsRef<Path>,
    task_id: DocumentId,
    run_id: impl AsRef<str>,
) -> Result<PathBuf, AgentRunStorageError> {
    let name = agent_worktree_name(task_id, run_id)?;
    Ok(agent_worktrees_dir(root).join(name))
}

pub fn validate_agent_worktree_path(
    root: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<PathBuf, AgentRunStorageError> {
    let root = root.as_ref();
    let path = path.as_ref();
    let allowed_dir = agent_worktrees_dir(root);

    let relative_path = if path.is_absolute() {
        path.strip_prefix(root).ok()
    } else {
        Some(path)
    }
    .ok_or_else(|| AgentRunStorageError::UnsafeWorktreePath {
        path: path.to_path_buf(),
        allowed_dir: allowed_dir.clone(),
    })?;

    let mut normalized_relative = PathBuf::new();
    for component in relative_path.components() {
        match component {
            Component::Normal(value) => normalized_relative.push(value),
            Component::CurDir => {}
            _ => {
                return Err(AgentRunStorageError::UnsafeWorktreePath {
                    path: path.to_path_buf(),
                    allowed_dir,
                });
            }
        }
    }

    let required_prefix = Path::new(VDOC_DIR).join(WORKTREES_DIR);
    if !normalized_relative.starts_with(&required_prefix) || normalized_relative == required_prefix
    {
        return Err(AgentRunStorageError::UnsafeWorktreePath {
            path: path.to_path_buf(),
            allowed_dir,
        });
    }

    Ok(root.join(normalized_relative))
}

pub fn create_agent_run_worktree(
    root: impl AsRef<Path>,
    run: &mut AgentRun,
) -> Result<PathBuf, AgentRunStorageError> {
    let root = root.as_ref();
    let worktree_path = agent_worktree_path(root, run.task_id, &run.run_id)?;
    let worktree_path = validate_agent_worktree_path(root, &worktree_path)?;

    if worktree_path.exists() {
        return Err(AgentRunStorageError::WorktreePathExists {
            path: worktree_path,
        });
    }

    fs::create_dir_all(agent_worktrees_dir(root)).map_err(|source| {
        AgentRunStorageError::CreateDir {
            path: agent_worktrees_dir(root),
            source,
        }
    })?;

    run_git_worktree(root, ["worktree", "add", "--detach"], &worktree_path)?;
    run.worktree_path = Some(worktree_path.clone());
    run.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    write_agent_run_metadata(run)?;

    Ok(worktree_path)
}

pub fn cleanup_agent_run_worktree(
    root: impl AsRef<Path>,
    run: &mut AgentRun,
) -> Result<(), AgentRunStorageError> {
    let root = root.as_ref();
    let Some(worktree_path) = run.worktree_path.clone() else {
        return Ok(());
    };
    let worktree_path = validate_agent_worktree_path(root, worktree_path)?;

    if worktree_path.exists() {
        run_git_worktree(root, ["worktree", "remove", "--force"], &worktree_path)?;
    } else {
        let _ = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["worktree", "prune"])
            .output();
    }

    run.worktree_path = None;
    run.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    write_agent_run_metadata(run)
}

fn run_git_worktree<const N: usize>(
    root: &Path,
    args: [&str; N],
    path: &Path,
) -> Result<(), AgentRunStorageError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .arg(path)
        .output()
        .map_err(|source| AgentRunStorageError::GitWorktree {
            message: source.to_string(),
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };
    Err(AgentRunStorageError::GitWorktree { message })
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
        assert_eq!(saved.worktree_path, run.worktree_path);
        assert_eq!(saved.artifacts.prompt, run.artifacts.prompt);
    }

    #[test]
    fn creates_agent_run_worktree_and_records_metadata() {
        let repo = TestRepo::new("worktree-create");
        repo.init_git();
        let mut run = create_agent_run(
            repo.path(),
            CreateAgentRunOptions {
                task_id: DocumentId::new(38).unwrap(),
                agent_kind: "codex".to_string(),
                worktree_path: None,
            },
        )
        .unwrap();

        let worktree_path = create_agent_run_worktree(repo.path(), &mut run).unwrap();

        assert!(worktree_path.is_dir());
        assert_eq!(
            worktree_path,
            repo.path()
                .join(".vdoc/worktrees")
                .join(format!("task-38-{}", run.run_id))
        );
        assert_eq!(run.worktree_path, Some(worktree_path.clone()));

        let raw = fs::read_to_string(&run.artifacts.run_json).unwrap();
        let saved: AgentRun = serde_json::from_str(&raw).unwrap();
        assert_eq!(saved.worktree_path, Some(worktree_path));
    }

    #[test]
    fn rejects_agent_worktree_paths_outside_execution_area() {
        let repo = TestRepo::new("worktree-validation");

        for path in [
            repo.path().join(".vdoc/runs/run-38-example"),
            repo.path().join("../outside"),
            PathBuf::from("/tmp/vibe-doc-outside-worktree"),
            PathBuf::from(".vdoc/worktrees/../runs/example"),
        ] {
            let error = validate_agent_worktree_path(repo.path(), path).unwrap_err();
            assert!(matches!(
                error,
                AgentRunStorageError::UnsafeWorktreePath { .. }
            ));
        }

        let safe = validate_agent_worktree_path(
            repo.path(),
            PathBuf::from(".vdoc/worktrees/task-38-run-38-example"),
        )
        .unwrap();
        assert_eq!(
            safe,
            repo.path().join(".vdoc/worktrees/task-38-run-38-example")
        );
    }

    #[test]
    fn refuses_to_create_agent_worktree_when_path_exists() {
        let repo = TestRepo::new("worktree-conflict");
        repo.init_git();
        let mut run = create_agent_run(
            repo.path(),
            CreateAgentRunOptions {
                task_id: DocumentId::new(38).unwrap(),
                agent_kind: "codex".to_string(),
                worktree_path: None,
            },
        )
        .unwrap();
        let path = agent_worktree_path(repo.path(), run.task_id, &run.run_id).unwrap();
        fs::create_dir_all(&path).unwrap();

        let error = create_agent_run_worktree(repo.path(), &mut run).unwrap_err();

        assert!(matches!(
            error,
            AgentRunStorageError::WorktreePathExists { .. }
        ));
        assert_eq!(run.worktree_path, None);
    }

    #[test]
    fn cleanup_agent_run_worktree_removes_path_and_clears_metadata() {
        let repo = TestRepo::new("worktree-cleanup");
        repo.init_git();
        let mut run = create_agent_run(
            repo.path(),
            CreateAgentRunOptions {
                task_id: DocumentId::new(38).unwrap(),
                agent_kind: "codex".to_string(),
                worktree_path: None,
            },
        )
        .unwrap();
        let worktree_path = create_agent_run_worktree(repo.path(), &mut run).unwrap();

        cleanup_agent_run_worktree(repo.path(), &mut run).unwrap();

        assert!(!worktree_path.exists());
        assert_eq!(run.worktree_path, None);
        let raw = fs::read_to_string(&run.artifacts.run_json).unwrap();
        let saved: AgentRun = serde_json::from_str(&raw).unwrap();
        assert_eq!(saved.worktree_path, None);
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

        fn init_git(&self) {
            self.run_git(["init"]);
            self.run_git(["config", "user.email", "vibe-doc@example.invalid"]);
            self.run_git(["config", "user.name", "vibe-doc tests"]);
            self.run_git(["config", "commit.gpgsign", "false"]);
            self.write("README.md", "# Test repo\n");
            self.run_git(["add", "README.md"]);
            self.run_git(["commit", "-m", "Initial commit"]);
        }

        fn run_git<const N: usize>(&self, args: [&str; N]) {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.root)
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
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
