use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn context_task_returns_related_documents_in_deterministic_order() {
    let repo = setup_repo("cli-context-order");
    repo.write(
        "docs/specs/10-later.md",
        numbered_doc(10, "Later Spec", "spec", ""),
    );
    repo.write(
        "docs/specs/2-earlier.md",
        numbered_doc(2, "Earlier Spec", "spec", ""),
    );
    repo.write(
        "docs/designs/11-later.md",
        numbered_doc(11, "Later Design", "design", "specs: []\nadrs: []\n"),
    );
    repo.write(
        "docs/designs/3-earlier.md",
        numbered_doc(3, "Earlier Design", "design", "specs: []\nadrs: []\n"),
    );
    repo.write("docs/adr/12-later.md", adr_doc(12, "Later ADR", "accepted"));
    repo.write(
        "docs/adr/4-earlier.md",
        adr_doc(4, "Earlier ADR", "accepted"),
    );
    repo.write(
        "docs/tasks/active/20-context.md",
        task_doc(20, "Context", "planned", &[10, 2], &[11, 3], &[12, 4], &[]),
    );

    let output = run_vdoc(repo.path(), ["context", "task", "20", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "context task");
    let items = value["items"].as_array().unwrap();
    let kinds_and_ids: Vec<_> = items
        .iter()
        .map(|item| {
            (
                item["kind"].as_str().unwrap().to_string(),
                item["id"].as_u64(),
            )
        })
        .collect();
    assert_eq!(
        kinds_and_ids,
        [
            ("task".to_string(), Some(20)),
            ("spec".to_string(), Some(2)),
            ("spec".to_string(), Some(10)),
            ("design".to_string(), Some(3)),
            ("design".to_string(), Some(11)),
            ("adr".to_string(), Some(4)),
            ("adr".to_string(), Some(12)),
        ]
    );
    assert_eq!(items[0]["path"], "docs/tasks/active/20-context.md");
    let task_content = items[0]["content"].as_str().unwrap();
    assert!(task_content.contains("## Goal"));
    assert!(!task_content.contains("kind: task"));
}

#[test]
fn context_task_rejects_missing_related_documents() {
    let repo = setup_repo("cli-context-missing-related");
    repo.write(
        "docs/tasks/active/20-missing-context.md",
        task_doc(20, "Missing Context", "planned", &[99], &[], &[], &[]),
    );

    let output = run_vdoc(repo.path(), ["context", "task", "20", "--json"]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    let value = stderr_json(&output);
    assert_eq!(
        value["error"]["code"],
        "TASK_CONTEXT_MISSING_RELATED_DOCUMENT"
    );
    assert_eq!(value["error"]["kind"], "spec");
    assert_eq!(value["error"]["id"], 99);
}

#[test]
fn guard_task_reports_ready_for_active_task_with_complete_dependencies() {
    let repo = setup_repo("cli-guard-ready");
    repo.write(
        "docs/tasks/done/2-dependency.md",
        task_doc(2, "Dependency", "done", &[], &[], &[], &[]),
    );
    repo.write("docs/specs/3-spec.md", numbered_doc(3, "Spec", "spec", ""));
    repo.write(
        "docs/designs/4-design.md",
        numbered_doc(4, "Design", "design", "specs: []\nadrs: []\n"),
    );
    repo.write("docs/adr/5-adr.md", adr_doc(5, "ADR", "accepted"));
    repo.write(
        "docs/tasks/active/6-ready.md",
        task_doc(6, "Ready", "planned", &[3], &[4], &[5], &[2]),
    );

    let output = run_vdoc(repo.path(), ["guard", "task", "6", "--json"]);

    assert_success(&output);
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["command"], "guard task");
    assert_eq!(value["ready"], true);
    assert_eq!(value["issue_count"], 0);
}

#[test]
fn guard_task_reports_blocked_and_invalid_references() {
    let repo = setup_repo("cli-guard-blocked");
    repo.write(
        "docs/tasks/active/2-dependency.md",
        task_doc(2, "Dependency", "doing", &[], &[], &[], &[]),
    );
    repo.write("docs/adr/5-rejected.md", adr_doc(5, "Rejected", "rejected"));
    repo.write(
        "docs/tasks/active/6-blocked.md",
        task_doc(6, "Blocked", "blocked", &[99], &[], &[5], &[2, 42]),
    );

    let output = run_vdoc(repo.path(), ["guard", "task", "6", "--json"]);

    assert!(!output.status.success());
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(value["ready"], false);
    let codes: Vec<_> = value["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"INVALID_TASK_STATUS"));
    assert!(codes.contains(&"INCOMPLETE_DEPENDENCY"));
    assert!(codes.contains(&"MISSING_DEPENDENCY"));
    assert!(codes.contains(&"MISSING_RELATED_DOCUMENT"));
    assert!(codes.contains(&"INVALID_RELATED_ADR_STATUS"));
}

#[test]
fn guard_task_rejects_done_tasks_outside_active_work() {
    let repo = setup_repo("cli-guard-done");
    repo.write(
        "docs/tasks/done/2-done.md",
        task_doc(2, "Done", "done", &[], &[], &[], &[]),
    );

    let output = run_vdoc(repo.path(), ["guard", "task", "2", "--json"]);

    assert!(!output.status.success());
    let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
    let codes: Vec<_> = value["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"TASK_NOT_ACTIVE"));
    assert!(codes.contains(&"INVALID_TASK_STATUS"));
}

fn setup_repo(name: &str) -> TestRepo {
    let repo = TestRepo::new(name);
    assert_success(&run_vdoc(repo.path(), ["init"]));
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

fn adr_doc(id: u64, title: &str, status: &str) -> String {
    numbered_doc(
        id,
        title,
        "adr",
        &format!("status: {status}\nrelated_designs: []\n"),
    )
}

fn task_doc(
    id: u64,
    title: &str,
    status: &str,
    specs: &[u64],
    designs: &[u64],
    adrs: &[u64],
    depends_on: &[u64],
) -> String {
    format!(
        "\
---
id: {id}
title: {title}
kind: task
type: feature
status: {status}
specs:{specs}
designs:{designs}
adrs:{adrs}
depends_on:{depends_on}
---

## Goal

Do work.

## Result

Not implemented.
",
        specs = yaml_id_list(specs),
        designs = yaml_id_list(designs),
        adrs = yaml_id_list(adrs),
        depends_on = yaml_id_list(depends_on)
    )
}

fn yaml_id_list(ids: &[u64]) -> String {
    if ids.is_empty() {
        " []\n".to_string()
    } else {
        format!(
            "\n{}\n",
            ids.iter()
                .map(|id| format!("  - {id}"))
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

fn stderr_json(output: &Output) -> Value {
    serde_json::from_str(stderr(output).lines().next().unwrap()).unwrap()
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

    fn write(&self, path: &str, content: impl AsRef<str>) {
        let path = self.root.join(path);
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
