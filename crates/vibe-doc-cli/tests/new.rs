use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use vibe_doc_core::{parse_numbered_document, AdrStatus, DocumentMetadata, Priority, TaskType};

#[test]
fn new_spec_creates_file_with_correct_frontmatter() {
    let repo = setup_repo("cli-new-spec");

    let output = run_vdoc(repo.path(), ["new", "spec", "Test Spec"]);

    assert_success(&output);
    assert!(repo.path().join("docs/specs/2-test-spec.md").is_file());

    let content = fs::read_to_string(repo.path().join("docs/specs/2-test-spec.md")).unwrap();
    let document = parse_numbered_document("docs/specs/2-test-spec.md", &content).unwrap();
    let DocumentMetadata::Spec(spec) = document.metadata else {
        panic!("expected spec metadata");
    };
    assert_eq!(spec.common.id.get(), 2);
    assert_eq!(spec.common.title, "Test Spec");
}

#[test]
fn new_spec_quotes_yaml_sensitive_titles() {
    let repo = setup_repo("cli-new-spec-yaml-title");

    let output = run_vdoc(repo.path(), ["new", "spec", "Foo: Bar's Spec"]);

    assert_success(&output);
    let content = fs::read_to_string(repo.path().join("docs/specs/2-foo-bar-s-spec.md")).unwrap();
    let document = parse_numbered_document("docs/specs/2-foo-bar-s-spec.md", &content).unwrap();
    let DocumentMetadata::Spec(spec) = document.metadata else {
        panic!("expected spec metadata");
    };
    assert_eq!(spec.common.title, "Foo: Bar's Spec");
}

#[test]
fn new_design_creates_file_with_correct_frontmatter() {
    let repo = setup_repo("cli-new-design");

    let output = run_vdoc(repo.path(), ["new", "design", "Test Design"]);

    assert_success(&output);
    assert!(repo.path().join("docs/designs/2-test-design.md").is_file());
}

#[test]
fn new_adr_creates_file_with_correct_frontmatter() {
    let repo = setup_repo("cli-new-adr");

    let output = run_vdoc(
        repo.path(),
        [
            "new", "adr", "Test ADR", "--status", "accepted", "--tag", "test-tag",
        ],
    );

    assert_success(&output);
    assert!(repo.path().join("docs/adr/2-test-adr.md").is_file());

    let content = fs::read_to_string(repo.path().join("docs/adr/2-test-adr.md")).unwrap();
    let document = parse_numbered_document("docs/adr/2-test-adr.md", &content).unwrap();
    let DocumentMetadata::Adr(adr) = document.metadata else {
        panic!("expected adr metadata");
    };
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.common.tags, ["test-tag"]);
}

#[test]
fn new_task_creates_file_with_correct_frontmatter() {
    let repo = setup_repo("cli-new-task");

    let output = run_vdoc(
        repo.path(),
        [
            "new",
            "task",
            "Test Task",
            "--type",
            "bug",
            "--priority",
            "high",
        ],
    );

    assert_success(&output);
    assert!(repo
        .path()
        .join("docs/tasks/active/2-test-task.md")
        .is_file());

    let content = fs::read_to_string(repo.path().join("docs/tasks/active/2-test-task.md")).unwrap();
    let document = parse_numbered_document("docs/tasks/active/2-test-task.md", &content).unwrap();
    let DocumentMetadata::Task(task) = document.metadata else {
        panic!("expected task metadata");
    };
    assert_eq!(task.task_type, TaskType::Bug);
    assert_eq!(task.priority, Some(Priority::High));
}

#[test]
fn new_task_updates_task_index() {
    let repo = setup_repo("cli-new-task-index");

    let output = run_vdoc(repo.path(), ["new", "task", "Index Task"]);

    assert_success(&output);
    let index = fs::read_to_string(repo.path().join("docs/tasks/index.md")).unwrap();
    assert!(index.contains("## Planned\n\n- 2 Index Task\n\n## Blocked"));
}

#[test]
fn new_json_output_is_stable() {
    let repo = setup_repo("cli-new-json");

    let output = run_vdoc(repo.path(), ["new", "task", "Json Task", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(value["command"], "new task");
    assert_eq!(value["dry_run"], false);
    assert_eq!(value["force"], false);
    assert_eq!(
        value["changes"][0]["path"],
        "docs/tasks/active/2-json-task.md"
    );
    assert_eq!(value["changes"][0]["action"], "create");
    assert_eq!(value["changes"][1]["path"], "docs/tasks/index.md");
    assert_eq!(value["changes"][1]["action"], "overwrite");
}

fn setup_repo(name: &str) -> TestRepo {
    let repo = TestRepo::new(name);
    // Initialize repository so schemas exist for validation
    run_vdoc(repo.path(), ["init"]);
    repo
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
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
