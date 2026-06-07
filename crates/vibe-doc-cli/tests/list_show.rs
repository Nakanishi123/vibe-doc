use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn list_specs_sorts_by_numeric_id() {
    let repo = setup_repo("cli-list-specs");
    repo.write(
        "docs/specs/10-later.md",
        numbered_doc(10, "Later", "spec", ""),
    );
    repo.write(
        "docs/specs/2-earlier.md",
        numbered_doc(2, "Earlier", "spec", ""),
    );

    let output = run_vdoc(repo.path(), ["list", "specs"]);

    assert_success(&output);
    let lines: Vec<_> = stdout(&output).lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("2\tspec\t"));
    assert!(lines[1].starts_with("10\tspec\t"));
}

#[test]
fn list_tasks_applies_status_type_priority_and_tag_filters() {
    let repo = setup_repo("cli-list-tasks-filter");
    repo.write(
        "docs/tasks/active/2-bug.md",
        task_doc(2, "Bug Work", "bug", "planned", Some("high"), &["frontend"]),
    );
    repo.write(
        "docs/tasks/active/3-chore.md",
        task_doc(3, "Chore Work", "chore", "planned", Some("low"), &["ops"]),
    );

    let output = run_vdoc(
        repo.path(),
        [
            "list",
            "tasks",
            "--status",
            "planned",
            "--type",
            "bug",
            "--priority",
            "high",
            "--tag",
            "frontend",
            "--json",
        ],
    );

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let documents = value["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["id"], 2);
    assert_eq!(documents[0]["type"], "bug");
    assert_eq!(documents[0]["priority"], "high");
}

#[test]
fn list_adr_applies_status_and_tag_filters() {
    let repo = setup_repo("cli-list-adr-filter");
    repo.write(
        "docs/adr/2-accepted.md",
        adr_doc(2, "Accepted", "accepted", &["runtime"]),
    );
    repo.write(
        "docs/adr/3-rejected.md",
        adr_doc(3, "Rejected", "rejected", &["runtime"]),
    );

    let output = run_vdoc(
        repo.path(),
        ["list", "adr", "--status", "accepted", "--tag", "runtime"],
    );

    assert_success(&output);
    let output = stdout(&output);
    assert!(output.contains("2\tadr\tdocs/adr/2-accepted.md\tAccepted"));
    assert!(!output.contains("Rejected"));
}

#[test]
fn show_resolves_by_id_and_supports_path_and_frontmatter_modes() {
    let repo = setup_repo("cli-show-modes");
    repo.write(
        "docs/specs/2-show-me.md",
        numbered_doc(2, "Show Me", "spec", ""),
    );

    let path_output = run_vdoc(repo.path(), ["show", "2", "--path-only"]);
    assert_success(&path_output);
    assert_eq!(stdout(&path_output), "docs/specs/2-show-me.md\n");

    let frontmatter_output = run_vdoc(repo.path(), ["show", "spec", "2", "--frontmatter-only"]);
    assert_success(&frontmatter_output);
    let frontmatter = stdout(&frontmatter_output);
    assert!(frontmatter.starts_with("id: 2\ntitle: Show Me\nkind: spec\n"));
    assert!(!frontmatter.contains("# Show Me"));
}

#[test]
fn show_json_error_for_missing_id_is_machine_readable() {
    let repo = setup_repo("cli-show-missing-json");

    let output = run_vdoc(repo.path(), ["show", "99", "--json"]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    let first_line = stderr.lines().next().unwrap();
    let value: Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(value["error"]["code"], "DOCUMENT_NOT_FOUND");
    assert_eq!(value["error"]["id"], 99);
}

#[test]
fn show_json_output_is_stable() {
    let repo = setup_repo("cli-show-json");
    repo.write(
        "docs/designs/2-design.md",
        numbered_doc(2, "Design", "design", ""),
    );

    let output = run_vdoc(
        repo.path(),
        ["show", "design", "2", "--json", "--path-only"],
    );

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "show");
    assert_eq!(value["document"]["id"], 2);
    assert_eq!(value["document"]["kind"], "design");
    assert_eq!(value["document"]["path"], "docs/designs/2-design.md");
    assert_eq!(value["document"]["mode"], "path-only");
}

fn setup_repo(name: &str) -> TestRepo {
    let repo = TestRepo::new(name);
    run_vdoc(repo.path(), ["init"]);
    repo
}

fn numbered_doc(id: u64, title: &str, kind: &str, extra_frontmatter: &str) -> String {
    format!(
        "\
---
id: {id}
title: {title}
kind: {kind}
{extra_frontmatter}---

# {title}
"
    )
}

fn adr_doc(id: u64, title: &str, status: &str, tags: &[&str]) -> String {
    numbered_doc(
        id,
        title,
        "adr",
        &format!("status: {status}\n{}", tags_yaml(tags)),
    )
}

fn task_doc(
    id: u64,
    title: &str,
    task_type: &str,
    status: &str,
    priority: Option<&str>,
    tags: &[&str],
) -> String {
    let priority = priority
        .map(|value| format!("priority: {value}\n"))
        .unwrap_or_default();
    numbered_doc(
        id,
        title,
        "task",
        &format!(
            "type: {task_type}\nstatus: {status}\n{priority}{}",
            tags_yaml(tags)
        ),
    )
}

fn tags_yaml(tags: &[&str]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(
            "tags:\n{}\n",
            tags.iter()
                .map(|tag| format!("  - {tag}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
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
