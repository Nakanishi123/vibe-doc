use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Options for creating the initial vibe-doc documentation layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InitOptions {
    pub dry_run: bool,
    pub force: bool,
}

/// A planned or applied init filesystem change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitChange {
    pub path: PathBuf,
    pub kind: InitChangeKind,
    pub action: InitChangeAction,
}

/// The kind of filesystem target touched by init.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitChangeKind {
    Directory,
    File,
}

impl InitChangeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::File => "file",
        }
    }
}

/// The action init plans or performs for a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitChangeAction {
    Create,
    Overwrite,
    Keep,
}

impl InitChangeAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Overwrite => "overwrite",
            Self::Keep => "keep",
        }
    }
}

/// The complete init plan, returned for dry-runs and successful writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitPlan {
    pub changes: Vec<InitChange>,
}

/// Error produced while planning or applying `vdoc init`.
#[derive(Debug)]
pub enum InitError {
    Conflicts { paths: Vec<PathBuf> },
    CreateDir { path: PathBuf, source: io::Error },
    WriteFile { path: PathBuf, source: io::Error },
}

impl fmt::Display for InitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflicts { paths } => {
                write!(
                    formatter,
                    "init would overwrite existing files: {}",
                    paths
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::CreateDir { path, source } => {
                write!(
                    formatter,
                    "failed to create directory {}: {source}",
                    path.display()
                )
            }
            Self::WriteFile { path, source } => {
                write!(
                    formatter,
                    "failed to write file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for InitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Conflicts { .. } => None,
            Self::CreateDir { source, .. } => Some(source),
            Self::WriteFile { source, .. } => Some(source),
        }
    }
}

/// Create the initial vibe-doc documentation layout under `root`.
pub fn init_repository(
    root: impl AsRef<Path>,
    options: InitOptions,
) -> Result<InitPlan, InitError> {
    let root = root.as_ref();
    let changes = plan_init(root, options.force, options.dry_run)?;

    if options.dry_run {
        return Ok(InitPlan { changes });
    }

    for change in &changes {
        let path = root.join(&change.path);
        match (change.kind, change.action) {
            (InitChangeKind::Directory, InitChangeAction::Create) => fs::create_dir_all(&path)
                .map_err(|source| InitError::CreateDir {
                    path: change.path.clone(),
                    source,
                })?,
            (InitChangeKind::File, InitChangeAction::Create | InitChangeAction::Overwrite) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|source| InitError::CreateDir {
                        path: relative_to_root(root, parent),
                        source,
                    })?;
                }
                fs::write(
                    &path,
                    template_for(&change.path).expect("planned files have templates"),
                )
                .map_err(|source| InitError::WriteFile {
                    path: change.path.clone(),
                    source,
                })?;
            }
            (_, InitChangeAction::Keep)
            | (InitChangeKind::Directory, InitChangeAction::Overwrite) => {}
        }
    }

    Ok(InitPlan { changes })
}

fn plan_init(root: &Path, force: bool, dry_run: bool) -> Result<Vec<InitChange>, InitError> {
    let mut changes = Vec::new();
    let mut conflicts = Vec::new();

    for relative_path in INIT_DIRECTORIES {
        let path = root.join(relative_path);
        changes.push(InitChange {
            path: PathBuf::from(relative_path),
            kind: InitChangeKind::Directory,
            action: if path.exists() {
                InitChangeAction::Keep
            } else {
                InitChangeAction::Create
            },
        });
    }

    for relative_path in INIT_FILES {
        let path = root.join(relative_path);
        let action = if path.exists() {
            if force {
                InitChangeAction::Overwrite
            } else if dry_run {
                InitChangeAction::Keep
            } else {
                conflicts.push(PathBuf::from(relative_path));
                InitChangeAction::Keep
            }
        } else {
            InitChangeAction::Create
        };

        changes.push(InitChange {
            path: PathBuf::from(relative_path),
            kind: InitChangeKind::File,
            action,
        });
    }

    if !conflicts.is_empty() {
        return Err(InitError::Conflicts { paths: conflicts });
    }

    Ok(changes)
}

fn relative_to_root(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn template_for(path: &Path) -> Option<&'static str> {
    match path.to_str()? {
        "AGENTS.md" => Some(AGENTS),
        "docs/README.md" => Some(DOCS_README),
        "docs/specs/README.md" => Some(SPECS_README),
        "docs/designs/README.md" => Some(DESIGNS_README),
        "docs/adr/README.md" => Some(ADR_README),
        "docs/tasks/README.md" => Some(TASKS_README),
        "docs/tasks/index.md" => Some(TASK_INDEX),
        "docs/schemas/document.schema.json" => Some(DOCUMENT_SCHEMA),
        "docs/schemas/spec.schema.json" => Some(SPEC_SCHEMA),
        "docs/schemas/design.schema.json" => Some(DESIGN_SCHEMA),
        "docs/schemas/adr.schema.json" => Some(ADR_SCHEMA),
        "docs/schemas/task.schema.json" => Some(TASK_SCHEMA),
        _ => None,
    }
}

const INIT_DIRECTORIES: &[&str] = &[
    "docs",
    "docs/schemas",
    "docs/specs",
    "docs/designs",
    "docs/adr",
    "docs/tasks",
    "docs/tasks/active",
    "docs/tasks/done",
];

const INIT_FILES: &[&str] = &[
    "AGENTS.md",
    "docs/README.md",
    "docs/specs/README.md",
    "docs/designs/README.md",
    "docs/adr/README.md",
    "docs/tasks/README.md",
    "docs/tasks/index.md",
    "docs/schemas/document.schema.json",
    "docs/schemas/spec.schema.json",
    "docs/schemas/design.schema.json",
    "docs/schemas/adr.schema.json",
    "docs/schemas/task.schema.json",
];

const AGENTS: &str = "\
# Agent Instructions

This repository is managed as a vibe-doc project.

## Operating Rules

- Treat repository Markdown as the source of truth.
- Maintain vibe-doc frontmatter for specs, designs, ADRs, tasks, and the task index.
- `AGENTS.md` and README files do not use frontmatter.
- Keep document references ID-based, not path-based.
- Keep operational documentation English-first. Frontmatter keys and enum values must remain stable English identifiers.

## Managed Documents

Repository documentation files are:

- `docs/README.md`
- `docs/specs/README.md`
- `docs/specs/*.md`
- `docs/designs/README.md`
- `docs/designs/*.md`
- `docs/adr/README.md`
- `docs/adr/*.md`
- `docs/tasks/README.md`
- `docs/tasks/index.md`
- `docs/tasks/active/*.md`
- `docs/tasks/done/*.md`

## Task Lifecycle

- Create new tasks in `docs/tasks/active/`.
- Use task statuses `planned`, `doing`, `blocked`, `done`, or `dropped`.
- Move completed or dropped tasks to `docs/tasks/done/`.
- Record implementation results in the task body, not in frontmatter.

## Validation Expectations

Before finishing documentation work, check that:

- IDs are unique positive integers.
- `kind` matches the file location.
- ADR and task statuses use allowed values.
- Task references point to existing document IDs.
- The task index reflects current active and done tasks.
";
const DOCS_README: &str = "\
# Docs

This directory contains vibe-doc-managed project documentation.

The source of truth is Markdown. Schemas describe frontmatter expectations for numbered vibe-doc documents.

## Structure

- `schemas/` contains JSON Schema files for vibe-doc frontmatter.
- `specs/` contains product requirements and externally observable behavior.
- `designs/` contains implementation designs for specs.
- `adr/` contains architectural decision records.
- `tasks/` contains active and completed implementation work.

## Frontmatter

Specs, designs, ADRs, tasks, and the task index must include:

```yaml
---
id: 1
title: Example Title
kind: spec
---
```

`AGENTS.md` and README files do not use frontmatter.

IDs are global across all numbered vibe-doc documents. Do not create separate ID ranges per document kind.
";
const SPECS_README: &str = "\
# Specs

Specs define what the product must do.

A spec should focus on externally observable behavior, user needs, constraints, API contracts, error cases, and acceptance criteria. It should avoid detailed implementation choices unless they are part of the product contract.

## Frontmatter

```yaml
---
id: 2
title: Example Spec
kind: spec
tags:
  - example
---
```

`status` is optional for specs. Existing specs are considered active unless explicitly marked deprecated.
";
const DESIGNS_README: &str = "\
# Designs

Design documents describe how the project should be built to satisfy one or more specs.

A design may cover components, data flow, data models, error handling, testing strategy, and alternatives considered.

## Frontmatter

```yaml
---
id: 3
title: Example Design
kind: design
specs:
  - 2
adrs: []
tags:
  - example
---
```

Use ID-based references for related specs and ADRs.
";
const ADR_README: &str = "\
# ADR

Architectural decision records explain why important technical decisions were made.

ADRs are intended to be durable. Prefer creating a new ADR that supersedes an accepted ADR instead of heavily rewriting the old one.

## Frontmatter

```yaml
---
id: 4
title: Example Decision
kind: adr
status: proposed
tags: []
related_designs: []
supersedes: []
superseded_by: null
---
```

Allowed ADR statuses are:

- `proposed`
- `accepted`
- `rejected`
- `deprecated`
- `superseded`
";
const TASKS_README: &str = "\
# Tasks

Tasks define implementation work units.

Tasks are mutable documents. Their frontmatter captures lifecycle state and relationships; their body captures goal, scope, checklist, done criteria, and result.

## Locations

- Active work belongs in `docs/tasks/active/`.
- Completed or dropped work belongs in `docs/tasks/done/`.
- `docs/tasks/index.md` is the task index.

## Frontmatter

```yaml
---
id: 5
title: Example Task
kind: task
type: feature
status: planned
priority: medium
specs: []
designs: []
adrs: []
depends_on: []
---
```

Allowed task types are `feature`, `bug`, `refactor`, `chore`, `docs`, `test`, and `spike`.

Allowed task statuses are `planned`, `doing`, `blocked`, `done`, and `dropped`.

Allowed priorities are `low`, `medium`, `high`, and `critical`.
";
const TASK_INDEX: &str = "\
---
id: 1
title: Task Index
kind: task-index
---

This index is generated by `vdoc rebuild index`.

## Doing

No tasks.

## Planned

No tasks.

## Blocked

No tasks.

## Done

No tasks.
";

const DOCUMENT_SCHEMA: &str = include_str!("../../../docs/schemas/document.schema.json");
const SPEC_SCHEMA: &str = include_str!("../../../docs/schemas/spec.schema.json");
const DESIGN_SCHEMA: &str = include_str!("../../../docs/schemas/design.schema.json");
const ADR_SCHEMA: &str = include_str!("../../../docs/schemas/adr.schema.json");
const TASK_SCHEMA: &str = include_str!("../../../docs/schemas/task.schema.json");

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_creates_expected_layout() {
        let repo = TestRepo::new("init-layout");

        let plan = init_repository(repo.path(), InitOptions::default()).unwrap();

        assert!(plan
            .changes
            .iter()
            .any(|change| change.path == Path::new("docs/tasks/index.md")));
        assert!(repo.path().join("AGENTS.md").is_file());
        assert!(repo.path().join("docs/specs").is_dir());
        assert!(repo.path().join("docs/tasks/active").is_dir());
        assert!(repo.path().join("docs/tasks/done").is_dir());
        assert!(repo
            .path()
            .join("docs/schemas/document.schema.json")
            .is_file());
        assert!(fs::read_to_string(repo.path().join("AGENTS.md"))
            .unwrap()
            .starts_with("# Agent Instructions"));
        assert!(fs::read_to_string(repo.path().join("docs/README.md"))
            .unwrap()
            .starts_with("# Docs"));
        assert!(fs::read_to_string(repo.path().join("docs/tasks/index.md"))
            .unwrap()
            .starts_with("---\nid: 1\ntitle: Task Index\nkind: task-index\n---"));
    }

    #[test]
    fn init_refuses_to_overwrite_files_without_force() {
        let repo = TestRepo::new("init-conflict");
        repo.write("AGENTS.md", "keep me\n");

        let error = init_repository(repo.path(), InitOptions::default()).unwrap_err();

        let InitError::Conflicts { paths } = error else {
            panic!("expected conflict");
        };
        assert_eq!(paths, [PathBuf::from("AGENTS.md")]);
        assert_eq!(
            fs::read_to_string(repo.path().join("AGENTS.md")).unwrap(),
            "keep me\n"
        );
    }

    #[test]
    fn dry_run_reports_create_actions_without_writing() {
        let repo = TestRepo::new("init-dry-run");

        let plan = init_repository(
            repo.path(),
            InitOptions {
                dry_run: true,
                force: false,
            },
        )
        .unwrap();

        assert!(plan
            .changes
            .iter()
            .all(|change| change.action == InitChangeAction::Create));
        assert!(!repo.path().join("AGENTS.md").exists());
    }

    #[test]
    fn dry_run_reports_existing_files_as_kept_without_force() {
        let repo = TestRepo::new("init-dry-run-existing");
        repo.write("AGENTS.md", "old\n");

        let plan = init_repository(
            repo.path(),
            InitOptions {
                dry_run: true,
                force: false,
            },
        )
        .unwrap();

        assert!(plan.changes.iter().any(|change| {
            change.path == Path::new("AGENTS.md") && change.action == InitChangeAction::Keep
        }));
        assert_eq!(
            fs::read_to_string(repo.path().join("AGENTS.md")).unwrap(),
            "old\n"
        );
    }

    #[test]
    fn force_overwrites_existing_files() {
        let repo = TestRepo::new("init-force");
        repo.write("AGENTS.md", "old\n");

        let plan = init_repository(
            repo.path(),
            InitOptions {
                dry_run: false,
                force: true,
            },
        )
        .unwrap();

        assert!(plan.changes.iter().any(|change| {
            change.path == Path::new("AGENTS.md") && change.action == InitChangeAction::Overwrite
        }));
        assert!(fs::read_to_string(repo.path().join("AGENTS.md"))
            .unwrap()
            .starts_with("# Agent Instructions"));
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
}
