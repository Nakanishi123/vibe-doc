use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn init_creates_expected_files_in_empty_repository() {
    let repo = TestRepo::new("cli-init-empty");

    let output = run_vdoc(repo.path(), ["init"]);

    assert_success(&output);
    assert!(repo.path().join("AGENTS.md").is_file());
    assert!(repo.path().join("docs/README.md").is_file());
    assert!(repo.path().join("docs/specs/README.md").is_file());
    assert!(repo.path().join("docs/designs/README.md").is_file());
    assert!(repo.path().join("docs/adr/README.md").is_file());
    assert!(repo.path().join("docs/tasks/README.md").is_file());
    assert!(repo.path().join("docs/tasks/index.md").is_file());
    assert!(repo
        .path()
        .join("docs/schemas/document.schema.json")
        .is_file());
    assert!(repo.path().join("docs/schemas/spec.schema.json").is_file());
    assert!(repo
        .path()
        .join("docs/schemas/design.schema.json")
        .is_file());
    assert!(repo.path().join("docs/schemas/adr.schema.json").is_file());
    assert!(repo.path().join("docs/schemas/task.schema.json").is_file());
    assert!(repo.path().join("docs/tasks/active").is_dir());
    assert!(repo.path().join("docs/tasks/done").is_dir());

    let agents = fs::read_to_string(repo.path().join("AGENTS.md")).unwrap();
    let readme = fs::read_to_string(repo.path().join("docs/README.md")).unwrap();
    let index = fs::read_to_string(repo.path().join("docs/tasks/index.md")).unwrap();

    assert!(!agents.starts_with("---"));
    assert!(!readme.starts_with("---"));
    assert!(index.starts_with("---\nid: 1\ntitle: Task Index\nkind: task-index\n---"));
}

#[test]
fn init_refuses_existing_files_without_force() {
    let repo = TestRepo::new("cli-init-conflict");
    repo.write("AGENTS.md", "existing\n");

    let output = run_vdoc(repo.path(), ["init"]);

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(repo.path().join("AGENTS.md")).unwrap(),
        "existing\n"
    );
    assert!(stderr(&output).contains("init would overwrite existing files: AGENTS.md"));
}

#[test]
fn init_dry_run_reports_planned_writes_without_writing() {
    let repo = TestRepo::new("cli-init-dry-run");

    let output = run_vdoc(repo.path(), ["init", "--dry-run"]);

    assert_success(&output);
    assert!(!repo.path().join("AGENTS.md").exists());
    assert!(stdout(&output).contains("vdoc init dry-run:"));
    assert!(stdout(&output).contains("- create file AGENTS.md"));
}

#[test]
fn init_dry_run_reports_existing_files_as_kept() {
    let repo = TestRepo::new("cli-init-dry-run-existing");
    repo.write("AGENTS.md", "existing\n");

    let output = run_vdoc(repo.path(), ["init", "--dry-run", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value["changes"].as_array().unwrap().iter().any(|change| {
        change["path"] == "AGENTS.md" && change["kind"] == "file" && change["action"] == "keep"
    }));
    assert_eq!(
        fs::read_to_string(repo.path().join("AGENTS.md")).unwrap(),
        "existing\n"
    );
}

#[test]
fn init_json_output_is_stable() {
    let repo = TestRepo::new("cli-init-json");

    let output = run_vdoc(repo.path(), ["init", "--dry-run", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();

    assert_eq!(value["command"], "init");
    assert_eq!(value["dry_run"], true);
    assert_eq!(value["force"], false);
    assert_eq!(value["changes"][0]["path"], "docs");
    assert_eq!(value["changes"][0]["kind"], "directory");
    assert_eq!(value["changes"][0]["action"], "create");
    assert!(value["changes"].as_array().unwrap().iter().any(|change| {
        change["path"] == "docs/tasks/index.md"
            && change["kind"] == "file"
            && change["action"] == "create"
    }));
}

#[test]
fn init_force_overwrites_existing_files() {
    let repo = TestRepo::new("cli-init-force");
    repo.write("AGENTS.md", "existing\n");

    let output = run_vdoc(repo.path(), ["init", "--force", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(value["changes"].as_array().unwrap().iter().any(|change| {
        change["path"] == "AGENTS.md" && change["kind"] == "file" && change["action"] == "overwrite"
    }));
    assert!(fs::read_to_string(repo.path().join("AGENTS.md"))
        .unwrap()
        .starts_with("# Agent Instructions"));
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

    fn write(&self, relative_path: &str, content: impl AsRef<str>) {
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
