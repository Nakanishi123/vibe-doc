use crate::{
    guard_task, scan_repository, task_context, DocumentId, DocumentMetadata, Priority, TaskContext,
    TaskContextError, TaskContextItemKind, TaskGuardReport, TaskMetadata, TaskStatus,
};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
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
const AGENT_OUTPUT_CHANNEL_BOUND: usize = 64;

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

impl std::fmt::Display for AgentRunStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareAgentRunOptions {
    pub task_id: DocumentId,
    pub agent_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAgentRun {
    pub run: AgentRun,
    pub guard: TaskGuardReport,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommand {
    pub name: String,
    pub agent_kind: String,
    pub program: String,
    pub args: Vec<String>,
    pub prompt_stdin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunExecution {
    pub run: AgentRun,
    pub terminal_log: String,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum AgentRunStreamEvent {
    Terminal {
        data: String,
    },
    Error {
        message: String,
    },
    Completed {
        status: AgentRunStatus,
        exit_result: Option<AgentRunExitResult>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRunEvent {
    pub event: String,
    pub run_id: String,
    pub task_id: DocumentId,
    pub status: AgentRunStatus,
    pub created_at: String,
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
    #[error("failed to read {}: {source}", path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize agent run metadata: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to parse agent run metadata from {}: {source}", path.display())]
    Deserialize {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not allocate a unique agent run ID under {}", runs_dir.display())]
    ExhaustedRunIds { runs_dir: PathBuf },
    #[error("task guard failed for task {}", report.task_id.get())]
    GuardFailed { report: TaskGuardReport },
    #[error(transparent)]
    TaskContext(#[from] TaskContextError),
    #[error("agent run `{run_id}` has status {status}; expected {expected}")]
    InvalidRunStatus {
        run_id: String,
        status: AgentRunStatus,
        expected: &'static str,
    },
    #[error("agent worktree path {} is outside {}", path.display(), allowed_dir.display())]
    UnsafeWorktreePath { path: PathBuf, allowed_dir: PathBuf },
    #[error("agent worktree path {} already exists", path.display())]
    WorktreePathExists { path: PathBuf },
    #[error("git worktree command failed: {message}")]
    GitWorktree { message: String },
    #[error("agent command `{command}` is not supported for run `{run_id}`")]
    UnsupportedAgentCommand { run_id: String, command: String },
    #[error("agent run `{run_id}` does not have an execution worktree")]
    MissingWorktree { run_id: String },
    #[error("failed to spawn agent command `{command}`: {source}")]
    SpawnAgentCommand {
        command: String,
        #[source]
        source: io::Error,
    },
    #[error("failed while reading agent command output: {0}")]
    ReadAgentOutput(#[source] io::Error),
    #[error("failed while writing agent command input: {0}")]
    WriteAgentInput(#[source] io::Error),
    #[error("agent command input writer panicked")]
    AgentInputWriterPanicked,
    #[error("agent command output reader panicked")]
    AgentOutputReaderPanicked,
    #[error("failed to capture agent run diff: {message}")]
    CaptureDiff { message: String },
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

pub fn prepare_agent_run(
    root: impl AsRef<Path>,
    options: PrepareAgentRunOptions,
) -> Result<PreparedAgentRun, AgentRunStorageError> {
    let root = root.as_ref();
    let guard = guard_task(root, options.task_id)?;
    if !guard.ready {
        return Err(AgentRunStorageError::GuardFailed { report: guard });
    }

    let context = task_context(root, options.task_id)?;
    let metadata = load_task_metadata(root, options.task_id)?;
    let prompt = generate_agent_prompt(root, &metadata, &guard, &context);
    let run = create_agent_run(
        root,
        CreateAgentRunOptions {
            task_id: options.task_id,
            agent_kind: options.agent_kind,
            worktree_path: None,
        },
    )?;
    write_agent_run_prompt(&run, &prompt)?;

    Ok(PreparedAgentRun { run, guard, prompt })
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

pub fn read_agent_run_metadata(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
) -> Result<AgentRun, AgentRunStorageError> {
    let artifacts = agent_run_artifacts(root, run_id)?;
    let raw = fs::read_to_string(&artifacts.run_json).map_err(|source| {
        AgentRunStorageError::ReadFile {
            path: artifacts.run_json.clone(),
            source,
        }
    })?;
    serde_json::from_str(&raw).map_err(|source| AgentRunStorageError::Deserialize {
        path: artifacts.run_json,
        source,
    })
}

pub fn write_agent_run_prompt(run: &AgentRun, prompt: &str) -> Result<(), AgentRunStorageError> {
    fs::write(&run.artifacts.prompt, prompt).map_err(|source| AgentRunStorageError::WriteFile {
        path: run.artifacts.prompt.clone(),
        source,
    })
}

pub fn approve_agent_run_prompt(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
) -> Result<AgentRun, AgentRunStorageError> {
    let mut run = read_agent_run_metadata(root, run_id)?;
    if run.status != AgentRunStatus::Prepared {
        return Err(AgentRunStorageError::InvalidRunStatus {
            run_id: run.run_id,
            status: run.status,
            expected: AgentRunStatus::Prepared.as_str(),
        });
    }

    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    run.status = AgentRunStatus::PromptApproved;
    run.updated_at = now.clone();
    let event = AgentRunEvent {
        event: "prompt-approved".to_owned(),
        run_id: run.run_id.clone(),
        task_id: run.task_id,
        status: run.status,
        created_at: now,
    };
    append_agent_run_event(&run, &event)?;
    write_agent_run_metadata(&run)?;

    Ok(run)
}

pub fn append_agent_run_event(
    run: &AgentRun,
    event: &AgentRunEvent,
) -> Result<(), AgentRunStorageError> {
    let content = serde_json::to_string(event).map_err(AgentRunStorageError::Serialize)?;
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&run.artifacts.events)
        .and_then(|mut file| {
            use std::io::Write;
            writeln!(file, "{content}")
        })
        .map_err(|source| AgentRunStorageError::WriteFile {
            path: run.artifacts.events.clone(),
            source,
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

pub fn execute_agent_run<F>(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
    command: &AgentCommand,
    mut on_stream_event: F,
) -> Result<AgentRunExecution, AgentRunStorageError>
where
    F: FnMut(&AgentRunStreamEvent),
{
    let root = root.as_ref();
    let mut run = preflight_agent_run_execution(root, run_id, command)?;

    if run.worktree_path.is_none() {
        create_agent_run_worktree(root, &mut run)?;
    }
    let worktree_path =
        run.worktree_path
            .clone()
            .ok_or_else(|| AgentRunStorageError::MissingWorktree {
                run_id: run.run_id.clone(),
            })?;
    let worktree_path = validate_agent_worktree_path(root, worktree_path)?;

    transition_agent_run(&mut run, AgentRunStatus::Running, "started")?;

    let mut terminal_log = File::options()
        .create(true)
        .append(true)
        .open(&run.artifacts.terminal_log)
        .map_err(|source| AgentRunStorageError::WriteFile {
            path: run.artifacts.terminal_log.clone(),
            source,
        })?;

    let command_result = run_agent_process(
        command,
        &worktree_path,
        &run.artifacts.prompt,
        &run.artifacts.terminal_log,
        &mut terminal_log,
        &mut on_stream_event,
    );

    let exit_result = match command_result {
        Ok(exit_result) => exit_result,
        Err(error) => return fail_agent_run(&mut run, error),
    };

    run.exit_result = Some(exit_result.clone());
    let final_status = if exit_result.code == Some(0) {
        AgentRunStatus::Succeeded
    } else {
        AgentRunStatus::Failed
    };

    let diff = match capture_agent_run_diff(&worktree_path) {
        Ok(diff) => diff,
        Err(error) => return fail_agent_run(&mut run, error),
    };
    fs::write(&run.artifacts.diff, &diff).map_err(|source| AgentRunStorageError::WriteFile {
        path: run.artifacts.diff.clone(),
        source,
    })?;
    transition_agent_run(&mut run, final_status, final_status.as_str())?;

    let terminal_log_content =
        fs::read_to_string(&run.artifacts.terminal_log).map_err(|source| {
            AgentRunStorageError::ReadFile {
                path: run.artifacts.terminal_log.clone(),
                source,
            }
        })?;

    Ok(AgentRunExecution {
        run,
        terminal_log: terminal_log_content,
        diff,
    })
}

pub fn preflight_agent_run_execution(
    root: impl AsRef<Path>,
    run_id: impl AsRef<str>,
    command: &AgentCommand,
) -> Result<AgentRun, AgentRunStorageError> {
    let run = read_agent_run_metadata(root, run_id)?;

    if run.status != AgentRunStatus::PromptApproved {
        return Err(AgentRunStorageError::InvalidRunStatus {
            run_id: run.run_id,
            status: run.status,
            expected: AgentRunStatus::PromptApproved.as_str(),
        });
    }

    if command.agent_kind != run.agent_kind {
        return Err(AgentRunStorageError::UnsupportedAgentCommand {
            run_id: run.run_id,
            command: command.name.clone(),
        });
    }

    Ok(run)
}

fn transition_agent_run(
    run: &mut AgentRun,
    status: AgentRunStatus,
    event_name: impl Into<String>,
) -> Result<(), AgentRunStorageError> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    run.status = status;
    run.updated_at = now.clone();
    let event = AgentRunEvent {
        event: event_name.into(),
        run_id: run.run_id.clone(),
        task_id: run.task_id,
        status: run.status,
        created_at: now,
    };
    append_agent_run_event(run, &event)?;
    write_agent_run_metadata(run)
}

fn fail_agent_run<T>(
    run: &mut AgentRun,
    error: AgentRunStorageError,
) -> Result<T, AgentRunStorageError> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let event = AgentRunEvent {
        event: format!("error: {error}"),
        run_id: run.run_id.clone(),
        task_id: run.task_id,
        status: AgentRunStatus::Failed,
        created_at: now,
    };
    append_agent_run_event(run, &event)?;
    run.status = AgentRunStatus::Failed;
    run.updated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    write_agent_run_metadata(run)?;
    Err(error)
}

fn run_agent_process<F>(
    command: &AgentCommand,
    worktree_path: &Path,
    prompt_path: &Path,
    terminal_log_path: &Path,
    terminal_log: &mut File,
    on_stream_event: &mut F,
) -> Result<AgentRunExitResult, AgentRunStorageError>
where
    F: FnMut(&AgentRunStreamEvent),
{
    let prompt = if command.prompt_stdin {
        Some(
            fs::read(prompt_path).map_err(|source| AgentRunStorageError::ReadFile {
                path: prompt_path.to_path_buf(),
                source,
            })?,
        )
    } else {
        None
    };

    let mut child = Command::new(&command.program)
        .args(&command.args)
        .current_dir(worktree_path)
        .stdin(if command.prompt_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| AgentRunStorageError::SpawnAgentCommand {
            command: command.name.clone(),
            source,
        })?;

    let (sender, receiver) = mpsc::sync_channel(AGENT_OUTPUT_CHANNEL_BOUND);
    let mut readers = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        readers.push(spawn_output_reader(stdout, sender.clone()));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(spawn_output_reader(stderr, sender.clone()));
    }
    drop(sender);

    let input_writer = match (prompt, child.stdin.take()) {
        (Some(prompt), Some(stdin)) => Some(spawn_input_writer(stdin, prompt)),
        _ => None,
    };
    let mut utf8_decoder = Utf8StreamDecoder::default();

    for chunk in receiver {
        terminal_log
            .write_all(&chunk)
            .map_err(|source| AgentRunStorageError::WriteFile {
                path: terminal_log_path.to_path_buf(),
                source,
            })?;
        utf8_decoder.push(&chunk, on_stream_event);
    }
    utf8_decoder.finish(on_stream_event);

    for reader in readers {
        match reader.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(AgentRunStorageError::ReadAgentOutput(error)),
            Err(_) => return Err(AgentRunStorageError::AgentOutputReaderPanicked),
        }
    }
    if let Some(input_writer) = input_writer {
        match input_writer.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(AgentRunStorageError::WriteAgentInput(error)),
            Err(_) => return Err(AgentRunStorageError::AgentInputWriterPanicked),
        }
    }

    let status = child
        .wait()
        .map_err(AgentRunStorageError::ReadAgentOutput)?;
    Ok(AgentRunExitResult {
        code: status.code(),
        signal: exit_signal(&status),
    })
}

fn spawn_input_writer(
    mut stdin: std::process::ChildStdin,
    prompt: Vec<u8>,
) -> thread::JoinHandle<io::Result<()>> {
    thread::spawn(move || stdin.write_all(&prompt))
}

fn spawn_output_reader<R>(
    mut reader: R,
    sender: mpsc::SyncSender<Vec<u8>>,
) -> thread::JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            if sender.send(buffer[..read].to_vec()).is_err() {
                break;
            }
        }
        Ok(())
    })
}

#[derive(Default)]
struct Utf8StreamDecoder {
    pending: Vec<u8>,
}

impl Utf8StreamDecoder {
    fn push<F>(&mut self, chunk: &[u8], on_stream_event: &mut F)
    where
        F: FnMut(&AgentRunStreamEvent),
    {
        self.pending.extend_from_slice(chunk);
        self.emit_complete_chunks(on_stream_event);
    }

    fn finish<F>(&mut self, on_stream_event: &mut F)
    where
        F: FnMut(&AgentRunStreamEvent),
    {
        if self.pending.is_empty() {
            return;
        }
        let data = String::from_utf8_lossy(&self.pending).into_owned();
        self.pending.clear();
        emit_terminal_event(data, on_stream_event);
    }

    fn emit_complete_chunks<F>(&mut self, on_stream_event: &mut F)
    where
        F: FnMut(&AgentRunStreamEvent),
    {
        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(data) => {
                    if !data.is_empty() {
                        emit_terminal_event(data.to_owned(), on_stream_event);
                    }
                    self.pending.clear();
                    break;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    if valid_up_to > 0 {
                        let data =
                            String::from_utf8_lossy(&self.pending[..valid_up_to]).into_owned();
                        self.pending.drain(..valid_up_to);
                        emit_terminal_event(data, on_stream_event);
                    }

                    match error.error_len() {
                        Some(error_len) => {
                            let data =
                                String::from_utf8_lossy(&self.pending[..error_len]).into_owned();
                            self.pending.drain(..error_len);
                            emit_terminal_event(data, on_stream_event);
                        }
                        None => break,
                    }
                }
            }
        }
    }
}

fn emit_terminal_event<F>(data: String, on_stream_event: &mut F)
where
    F: FnMut(&AgentRunStreamEvent),
{
    if !data.is_empty() {
        on_stream_event(&AgentRunStreamEvent::Terminal { data });
    }
}

#[cfg(unix)]
fn exit_signal(status: &ExitStatus) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;

    status.signal().map(|signal| signal.to_string())
}

#[cfg(not(unix))]
fn exit_signal(_status: &ExitStatus) -> Option<String> {
    None
}

fn capture_agent_run_diff(worktree_path: &Path) -> Result<String, AgentRunStorageError> {
    let add_intent = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .args(["add", "-N", "--", "."])
        .output()
        .map_err(|source| AgentRunStorageError::CaptureDiff {
            message: source.to_string(),
        })?;
    if !add_intent.status.success() {
        return Err(AgentRunStorageError::CaptureDiff {
            message: command_output_message(&add_intent),
        });
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .args(["diff", "--binary"])
        .output()
        .map_err(|source| AgentRunStorageError::CaptureDiff {
            message: source.to_string(),
        })?;
    if !output.status.success() {
        return Err(AgentRunStorageError::CaptureDiff {
            message: command_output_message(&output),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn command_output_message(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.trim().is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    } else {
        stderr.trim().to_owned()
    }
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

fn load_task_metadata(
    root: &Path,
    task_id: DocumentId,
) -> Result<TaskMetadata, AgentRunStorageError> {
    let documents = scan_repository(root).map_err(TaskContextError::RepositoryScan)?;
    documents
        .into_iter()
        .find_map(|document| match document.document.metadata {
            DocumentMetadata::Task(metadata) if metadata.common.id == task_id => Some(metadata),
            _ => None,
        })
        .ok_or(TaskContextError::TaskNotFound { id: task_id })
        .map_err(AgentRunStorageError::TaskContext)
}

fn generate_agent_prompt(
    root: &Path,
    task_metadata: &TaskMetadata,
    guard: &TaskGuardReport,
    context: &TaskContext,
) -> String {
    let mut prompt = String::new();
    prompt.push_str("# Codex Task Run\n\n");
    prompt
        .push_str("Implement the documented vibe-doc task using the repository context below.\n\n");
    prompt.push_str("## Repository\n\n");
    prompt.push_str(&format!("- Root: `{}`\n", root.display()));
    prompt.push_str("- Entry point: task ID only\n");
    prompt.push_str(
        "- Execution mode: prepare prompt for explicit approval before running an agent\n\n",
    );

    prompt.push_str("## Task\n\n");
    prompt.push_str(&format!("- ID: {}\n", task_metadata.common.id.get()));
    prompt.push_str(&format!("- Title: {}\n", task_metadata.common.title));
    prompt.push_str(&format!(
        "- Status: {}\n",
        task_status_str(task_metadata.status)
    ));
    prompt.push_str(&format!(
        "- Priority: {}\n\n",
        task_metadata.priority.map(priority_str).unwrap_or("medium")
    ));

    prompt.push_str("## Guard\n\n");
    prompt.push_str(&format!("- Ready: {}\n", guard.ready));
    if guard.issues.is_empty() {
        prompt.push_str("- Issues: none\n\n");
    } else {
        prompt.push_str("- Issues:\n");
        for issue in &guard.issues {
            prompt.push_str(&format!("  - {}: {}\n", issue.code.as_str(), issue.message));
        }
        prompt.push('\n');
    }

    prompt.push_str("## Context\n\n");
    for item in &context.items {
        let role = match item.kind {
            TaskContextItemKind::Task => "Task",
            TaskContextItemKind::Spec => "Spec",
            TaskContextItemKind::Design => "Design",
            TaskContextItemKind::Adr => "ADR",
        };
        let id = item
            .document_id
            .map(|id| id.get().to_string())
            .unwrap_or_else(|| "unnumbered".to_owned());
        let title = item.title.as_deref().unwrap_or("Untitled");
        prompt.push_str(&format!(
            "### {role} {id}: {title}\n\nPath: `{}`\n\n",
            item.path.display()
        ));
        prompt.push_str(item.content.trim());
        prompt.push_str("\n\n");
    }

    prompt
}

fn task_status_str(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Doing => "doing",
        TaskStatus::Blocked => "blocked",
        TaskStatus::Done => "done",
        TaskStatus::Dropped => "dropped",
    }
}

fn priority_str(priority: Priority) -> &'static str {
    match priority {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Critical => "critical",
    }
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
    fn prepares_agent_run_with_prompt_and_metadata_artifacts() {
        let repo = TestRepo::new("prepare");
        repo.seed_ready_task();

        let prepared = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "codex".to_owned(),
            },
        )
        .unwrap();

        assert!(prepared.guard.ready);
        assert_eq!(prepared.run.status, AgentRunStatus::Prepared);
        assert_eq!(prepared.run.agent_kind, "codex");
        assert!(prepared.prompt.contains("# Codex Task Run"));
        assert!(prepared.prompt.contains("- ID: 39"));
        assert!(prepared.prompt.contains("Implement agent run APIs"));
        assert!(prepared.prompt.contains("### Spec 12: Agent Spec"));
        assert!(prepared.run.artifacts.run_json.is_file());
        assert_eq!(
            fs::read_to_string(&prepared.run.artifacts.prompt).unwrap(),
            prepared.prompt
        );
    }

    #[test]
    fn prepare_agent_run_does_not_create_run_when_guard_fails() {
        let repo = TestRepo::new("prepare-guard-fails");
        repo.write(
            "docs/tasks/done/39-agent.md",
            "\
---
id: 39
title: Agent APIs
kind: task
type: feature
status: done
depends_on: []
---

# Agent APIs
",
        );

        let error = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "codex".to_owned(),
            },
        )
        .unwrap_err();

        let AgentRunStorageError::GuardFailed { report } = error else {
            panic!("expected guard failure");
        };
        assert!(!report.ready);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code.as_str() == "TASK_NOT_ACTIVE"));
        assert!(!agent_runs_dir(repo.path()).exists());
    }

    #[test]
    fn approving_prompt_records_event_and_updates_status() {
        let repo = TestRepo::new("approve");
        repo.seed_ready_task();
        let prepared = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "codex".to_owned(),
            },
        )
        .unwrap();

        let approved = approve_agent_run_prompt(repo.path(), &prepared.run.run_id).unwrap();

        assert_eq!(approved.status, AgentRunStatus::PromptApproved);
        let saved = read_agent_run_metadata(repo.path(), &prepared.run.run_id).unwrap();
        assert_eq!(saved.status, AgentRunStatus::PromptApproved);
        let events = fs::read_to_string(&approved.artifacts.events).unwrap();
        assert!(events.contains("\"event\":\"prompt-approved\""));
        assert!(matches!(
            approve_agent_run_prompt(repo.path(), &prepared.run.run_id),
            Err(AgentRunStorageError::InvalidRunStatus { .. })
        ));
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
    fn executes_approved_agent_run_and_captures_logs_and_diff() {
        let repo = TestRepo::new("execute");
        repo.seed_ready_task();
        repo.init_git();
        let prepared = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "fixture".to_owned(),
            },
        )
        .unwrap();
        approve_agent_run_prompt(repo.path(), &prepared.run.run_id).unwrap();
        let command = AgentCommand {
            name: "fixture".to_owned(),
            agent_kind: "fixture".to_owned(),
            program: "sh".to_owned(),
            args: vec![
                "-c".to_owned(),
                "printf 'hello stdout\\n'; printf 'hello stderr\\n' >&2; printf 'changed\\n' > agent-output.txt"
                    .to_owned(),
            ],
            prompt_stdin: false,
        };
        let mut streamed = Vec::new();

        let execution = execute_agent_run(repo.path(), &prepared.run.run_id, &command, |event| {
            streamed.push(event.clone());
        })
        .unwrap();

        assert_eq!(execution.run.status, AgentRunStatus::Succeeded);
        assert_eq!(
            execution.run.exit_result,
            Some(AgentRunExitResult {
                code: Some(0),
                signal: None,
            })
        );
        assert!(execution.run.worktree_path.as_ref().unwrap().is_dir());
        assert!(execution.terminal_log.contains("hello stdout"));
        assert!(execution.terminal_log.contains("hello stderr"));
        assert_eq!(
            fs::read_to_string(&execution.run.artifacts.terminal_log).unwrap(),
            execution.terminal_log
        );
        assert!(streamed.iter().any(|event| matches!(
            event,
            AgentRunStreamEvent::Terminal { data } if data.contains("hello stdout")
        )));
        assert!(execution.diff.contains("agent-output.txt"));
        assert!(execution.diff.contains("+changed"));
        assert_eq!(
            fs::read_to_string(&execution.run.artifacts.diff).unwrap(),
            execution.diff
        );
        let events = fs::read_to_string(&execution.run.artifacts.events).unwrap();
        assert!(events.contains("\"event\":\"started\""));
        assert!(events.contains("\"event\":\"succeeded\""));
    }

    #[cfg(unix)]
    #[test]
    fn records_signal_exit_results_on_unix() {
        let repo = TestRepo::new("execute-signal");
        repo.seed_ready_task();
        repo.init_git();
        let prepared = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "fixture".to_owned(),
            },
        )
        .unwrap();
        approve_agent_run_prompt(repo.path(), &prepared.run.run_id).unwrap();
        let command = AgentCommand {
            name: "fixture".to_owned(),
            agent_kind: "fixture".to_owned(),
            program: "sh".to_owned(),
            args: vec!["-c".to_owned(), "kill -TERM $$".to_owned()],
            prompt_stdin: false,
        };

        let execution =
            execute_agent_run(repo.path(), &prepared.run.run_id, &command, |_| {}).unwrap();

        assert_eq!(execution.run.status, AgentRunStatus::Failed);
        assert_eq!(
            execution.run.exit_result,
            Some(AgentRunExitResult {
                code: None,
                signal: Some("15".to_owned()),
            })
        );
    }

    #[test]
    fn utf8_stream_decoder_carries_split_multibyte_sequences() {
        let mut decoder = Utf8StreamDecoder::default();
        let mut events = Vec::new();
        let bytes = "あ".as_bytes();

        decoder.push(&bytes[..1], &mut |event| events.push(event.clone()));
        assert!(events.is_empty());
        decoder.push(&bytes[1..], &mut |event| events.push(event.clone()));

        assert_eq!(
            events,
            vec![AgentRunStreamEvent::Terminal {
                data: "あ".to_owned(),
            }]
        );
    }

    #[test]
    fn rejects_unapproved_agent_run_execution() {
        let repo = TestRepo::new("execute-unapproved");
        let run = create_agent_run(
            repo.path(),
            CreateAgentRunOptions {
                task_id: DocumentId::new(40).unwrap(),
                agent_kind: "fixture".to_owned(),
                worktree_path: None,
            },
        )
        .unwrap();
        let command = AgentCommand {
            name: "fixture".to_owned(),
            agent_kind: "fixture".to_owned(),
            program: "sh".to_owned(),
            args: vec!["-c".to_owned(), "true".to_owned()],
            prompt_stdin: false,
        };

        let error = execute_agent_run(repo.path(), &run.run_id, &command, |_| {}).unwrap_err();

        assert!(matches!(
            error,
            AgentRunStorageError::InvalidRunStatus { .. }
        ));
        let saved = read_agent_run_metadata(repo.path(), &run.run_id).unwrap();
        assert_eq!(saved.status, AgentRunStatus::Prepared);
    }

    #[test]
    fn rejects_unsupported_agent_command() {
        let repo = TestRepo::new("execute-unsupported");
        repo.seed_ready_task();
        let prepared = prepare_agent_run(
            repo.path(),
            PrepareAgentRunOptions {
                task_id: DocumentId::new(39).unwrap(),
                agent_kind: "codex".to_owned(),
            },
        )
        .unwrap();
        approve_agent_run_prompt(repo.path(), &prepared.run.run_id).unwrap();
        let command = AgentCommand {
            name: "fixture".to_owned(),
            agent_kind: "fixture".to_owned(),
            program: "sh".to_owned(),
            args: vec!["-c".to_owned(), "true".to_owned()],
            prompt_stdin: false,
        };

        let error =
            execute_agent_run(repo.path(), &prepared.run.run_id, &command, |_| {}).unwrap_err();

        assert!(matches!(
            error,
            AgentRunStorageError::UnsupportedAgentCommand { .. }
        ));
        let saved = read_agent_run_metadata(repo.path(), &prepared.run.run_id).unwrap();
        assert_eq!(saved.status, AgentRunStatus::PromptApproved);
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

        fn seed_ready_task(&self) {
            self.write(
                "docs/specs/12-agent.md",
                "\
---
id: 12
title: Agent Spec
kind: spec
---

# Agent Spec

Use task IDs as the entry point.
",
            );
            self.write(
                "docs/designs/35-agent.md",
                "\
---
id: 35
title: Agent Design
kind: design
specs:
  - 12
---

# Agent Design

Generate a prompt before execution.
",
            );
            self.write(
                "docs/tasks/active/39-agent.md",
                "\
---
id: 39
title: Implement agent run APIs
kind: task
type: feature
status: planned
priority: high
specs:
  - 12
designs:
  - 35
depends_on: []
---

# Implement agent run APIs

Prepare and approve a prompt.
",
            );
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
