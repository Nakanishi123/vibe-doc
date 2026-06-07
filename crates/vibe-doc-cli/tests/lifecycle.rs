use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use vibe_doc_core::{parse_numbered_document, DocumentMetadata, TaskStatus};

#[test]
fn start_task_updates_status_and_started_at() {
    let repo = setup_repo("cli-start-task");
    repo.write(
        "docs/tasks/active/2-start-me.md",
        &task_markdown(2, "Start Me", "planned"),
    );

    let output = run_vdoc(repo.path(), ["start", "task", "2", "--date", "2026-06-07"]);

    assert_success(&output);
    let content = fs::read_to_string(repo.path().join("docs/tasks/active/2-start-me.md")).unwrap();
    let document = parse_numbered_document("docs/tasks/active/2-start-me.md", &content).unwrap();
    let DocumentMetadata::Task(task) = document.metadata else {
        panic!("expected task metadata");
    };
    assert_eq!(task.status, TaskStatus::Doing);
    assert_eq!(task.started_at.as_deref(), Some("2026-06-07"));

    let index = fs::read_to_string(repo.path().join("docs/tasks/index.md")).unwrap();
    assert!(index.contains("## Doing\n\n- 2 Start Me\n\n## Planned"));
}

#[test]
fn complete_task_updates_status_result_and_moves_file() {
    let repo = setup_repo("cli-complete-task");
    repo.write(
        "docs/tasks/active/2-complete-me.md",
        &task_markdown(2, "Complete Me", "doing"),
    );

    let output = run_vdoc(
        repo.path(),
        [
            "complete",
            "task",
            "2",
            "--date",
            "2026-06-07",
            "--result",
            "Implemented lifecycle support.",
        ],
    );

    assert_success(&output);
    assert!(!repo
        .path()
        .join("docs/tasks/active/2-complete-me.md")
        .exists());
    let done_path = repo.path().join("docs/tasks/done/2-complete-me.md");
    assert!(done_path.is_file());

    let content = fs::read_to_string(done_path).unwrap();
    let document = parse_numbered_document("docs/tasks/done/2-complete-me.md", &content).unwrap();
    let DocumentMetadata::Task(task) = document.metadata else {
        panic!("expected task metadata");
    };
    assert_eq!(task.status, TaskStatus::Done);
    assert_eq!(task.completed_at.as_deref(), Some("2026-06-07"));
    assert!(content.contains("## Result\n\nImplemented lifecycle support.\n"));

    let index = fs::read_to_string(repo.path().join("docs/tasks/index.md")).unwrap();
    assert!(index.contains("## Done\n\n- 2 Complete Me\n"));
}

#[test]
fn lifecycle_dry_run_does_not_write_and_json_is_stable() {
    let repo = setup_repo("cli-lifecycle-dry-run");
    repo.write(
        "docs/tasks/active/2-dry-run.md",
        &task_markdown(2, "Dry Run", "planned"),
    );
    let before = fs::read_to_string(repo.path().join("docs/tasks/active/2-dry-run.md")).unwrap();

    let output = run_vdoc(
        repo.path(),
        [
            "start",
            "task",
            "2",
            "--date",
            "2026-06-07",
            "--dry-run",
            "--json",
        ],
    );

    assert_success(&output);
    let after = fs::read_to_string(repo.path().join("docs/tasks/active/2-dry-run.md")).unwrap();
    assert_eq!(after, before);

    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "start task");
    assert_eq!(value["dry_run"], true);
    assert_eq!(value["task_id"], 2);
    assert_eq!(
        value["changes"][0]["path"],
        "docs/tasks/active/2-dry-run.md"
    );
    assert_eq!(value["changes"][0]["action"], "overwrite");
    assert_eq!(value["changes"][1]["path"], "docs/tasks/index.md");
}

#[test]
fn lifecycle_reports_missing_task_and_invalid_status() {
    let repo = setup_repo("cli-lifecycle-errors");
    repo.write(
        "docs/tasks/active/2-planned.md",
        &task_markdown(2, "Planned", "planned"),
    );

    let missing = run_vdoc(repo.path(), ["start", "task", "999", "--json"]);
    assert!(!missing.status.success());
    let missing_json = stderr_json(&missing);
    assert_eq!(
        missing_json["error"]["code"],
        "TASK_LIFECYCLE_TASK_NOT_FOUND"
    );

    let invalid = run_vdoc(repo.path(), ["complete", "task", "2", "--json"]);
    assert!(!invalid.status.success());
    let invalid_json = stderr_json(&invalid);
    assert_eq!(
        invalid_json["error"]["code"],
        "TASK_LIFECYCLE_INVALID_STATUS"
    );
}

#[test]
fn lifecycle_rejects_invalid_date() {
    let repo = setup_repo("cli-lifecycle-invalid-date");
    repo.write(
        "docs/tasks/active/2-planned.md",
        &task_markdown(2, "Planned", "planned"),
    );

    let output = run_vdoc(repo.path(), ["start", "task", "2", "--date", "yesterday"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("expected YYYY-MM-DD"));
}

fn setup_repo(name: &str) -> TestRepo {
    let repo = TestRepo::new(name);
    assert_success(&run_vdoc(repo.path(), ["init"]));
    repo
}

fn task_markdown(id: u64, title: &str, status: &str) -> String {
    format!(
        "\
---
id: {id}
title: {title}
kind: task
type: feature
status: {status}
adrs: []
depends_on: []
---

## Goal

Do work.

## Result

Not implemented.
"
    )
}

fn run_vdoc<const N: usize>(cwd: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vdoc"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stderr_json(output: &Output) -> Value {
    let stderr = stderr(output);
    serde_json::from_str(stderr.lines().next().unwrap()).unwrap()
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
        let root = std::env::temp_dir().join(format!("vibe-doc-{name}-{unique}"));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
