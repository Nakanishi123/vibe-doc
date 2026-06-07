use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn validate_reports_success_for_initialized_repository() {
    let repo = setup_repo("cli-validate-ok");

    let output = run_vdoc(repo.path(), ["validate"]);

    assert_success(&output);
    assert_eq!(stdout(&output), "vdoc validate: ok\n");
}

#[test]
fn validate_json_reports_stable_issue_codes_and_exit_failure() {
    let repo = setup_repo("cli-validate-json-fail");
    repo.write(
        "docs/tasks/active/2-broken.md",
        task_doc(2, "Broken", "planned", &[999]),
    );

    let output = run_vdoc(repo.path(), ["validate", "--json"]);

    assert!(!output.status.success());
    assert_eq!(stderr(&output), "");
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "validate");
    assert_eq!(value["valid"], false);
    assert_eq!(value["issue_count"], 1);
    assert_eq!(value["issues"][0]["code"], "BROKEN_REFERENCE");
    assert_eq!(value["issues"][0]["path"], "docs/tasks/active/2-broken.md");
}

#[test]
fn validate_path_argument_filters_reported_issues() {
    let repo = setup_repo("cli-validate-path");
    repo.write(
        "docs/tasks/active/2-broken.md",
        task_doc(2, "Broken", "planned", &[999]),
    );
    repo.write(
        "docs/tasks/active/3-clean.md",
        task_doc(3, "Clean", "planned", &[]),
    );

    let output = run_vdoc(
        repo.path(),
        ["validate", "docs/tasks/active/3-clean.md", "--json"],
    );

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["valid"], true);
    assert_eq!(value["issue_count"], 0);
}

#[test]
fn validate_path_argument_does_not_hide_out_of_scope_parse_failure() {
    let repo = setup_repo("cli-validate-path-parse-failure");
    repo.write("docs/specs/2-broken.md", "# Broken\n");
    repo.write(
        "docs/tasks/active/3-clean.md",
        task_doc(3, "Clean", "planned", &[]),
    );

    let output = run_vdoc(
        repo.path(),
        ["validate", "docs/tasks/active/3-clean.md", "--json"],
    );

    assert!(!output.status.success());
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["valid"], false);
    assert_eq!(value["incomplete"], true);
    assert_eq!(value["issue_count"], 1);
    assert_eq!(value["issues"][0]["code"], "BAD_FRONTMATTER");
    assert_eq!(value["issues"][0]["path"], "docs/specs/2-broken.md");
}

#[test]
fn check_reports_missing_readmes_and_task_index_drift() {
    let repo = setup_repo("cli-check-fail");
    fs::remove_file(repo.path().join("docs/README.md")).unwrap();
    repo.write(
        "docs/tasks/active/2-new-task.md",
        task_doc(2, "New Task", "planned", &[]),
    );

    let output = run_vdoc(repo.path(), ["check", "--json"]);

    assert!(!output.status.success());
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let codes: Vec<_> = value["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"README_NOT_FOUND"));
    assert!(codes.contains(&"INDEX_OUT_OF_SYNC"));
}

fn setup_repo(name: &str) -> TestRepo {
    let repo = TestRepo::new(name);
    let output = run_vdoc(repo.path(), ["init"]);
    assert_success(&output);
    repo
}

fn task_doc(id: u64, title: &str, status: &str, specs: &[u64]) -> String {
    let specs = if specs.is_empty() {
        String::new()
    } else {
        format!(
            "specs:\n{}\n",
            specs
                .iter()
                .map(|id| format!("  - {id}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    format!(
        "\
---
id: {id}
title: {title}
kind: task
type: feature
status: {status}
{specs}---

# {title}
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
