use crate::error::CliError;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use vibe_doc_core::{AdrStatus, DocumentMetadata, Priority, TaskStatus, TaskType};

#[derive(Debug, Parser)]
#[command(name = "vdoc")]
#[command(about = "Manage vibe-doc repository documentation")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Init(InitCommandOptions),
    New(NewCommand),
    Rebuild(RebuildCommand),
    List(ListCommand),
    Show(ShowCommand),
    Validate(ValidationCommand),
    Check(ValidationCommand),
    Start(StartCommand),
    Complete(CompleteCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct RebuildCommand {
    #[command(subcommand)]
    pub(crate) target: RebuildTargetCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RebuildTargetCommand {
    Index(RebuildIndexCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct RebuildIndexCommand {
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct InitCommandOptions {
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) force: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct NewCommand {
    #[command(flatten)]
    pub(crate) options: NewCommandOptions,
    #[command(subcommand)]
    pub(crate) kind: NewKindCommand,
}

#[derive(Debug, Parser, Clone, Copy)]
pub(crate) struct NewCommandOptions {
    #[arg(long, global = true)]
    pub(crate) dry_run: bool,
    #[arg(long, global = true)]
    pub(crate) json: bool,
    #[arg(long, global = true)]
    pub(crate) force: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum NewKindCommand {
    Spec {
        title: String,
    },
    Design {
        title: String,
    },
    Adr {
        title: String,
        #[arg(long)]
        status: Option<AdrStatusArg>,
        #[arg(long = "tag")]
        tag: Vec<String>,
        #[arg(long = "related-design")]
        related_design: Vec<u64>,
    },
    Task {
        title: String,
        #[arg(long = "type")]
        task_type: Option<TaskTypeArg>,
        #[arg(long)]
        priority: Option<PriorityArg>,
        #[arg(long = "spec")]
        spec: Vec<u64>,
        #[arg(long = "design")]
        design: Vec<u64>,
        #[arg(long = "adr")]
        adr: Vec<u64>,
        #[arg(long = "depends-on")]
        depends_on: Vec<u64>,
    },
}

#[derive(Debug, Parser)]
pub(crate) struct ListCommand {
    pub(crate) kind: ListKindArg,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) status: Option<StatusFilterArg>,
    #[arg(long = "type")]
    pub(crate) task_type: Option<TaskTypeArg>,
    #[arg(long)]
    pub(crate) priority: Option<PriorityArg>,
    #[arg(long)]
    pub(crate) tag: Option<String>,
}

#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("show_mode")
        .args(["path_only", "frontmatter_only"])
        .multiple(false)
))]
pub(crate) struct ShowCommand {
    #[arg(required = true, num_args = 1..=2)]
    target: Vec<String>,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    path_only: bool,
    #[arg(long)]
    frontmatter_only: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ValidationCommand {
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(value_name = "PATH")]
    pub(crate) paths: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct StartCommand {
    #[command(subcommand)]
    pub(crate) target: StartTargetCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum StartTargetCommand {
    Task(StartTaskCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct StartTaskCommand {
    pub(crate) id: u64,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) date: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct CompleteCommand {
    #[command(subcommand)]
    pub(crate) target: CompleteTargetCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CompleteTargetCommand {
    Task(CompleteTaskCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct CompleteTaskCommand {
    pub(crate) id: u64,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long)]
    pub(crate) date: Option<String>,
    #[arg(long)]
    pub(crate) result: Option<String>,
}

impl ShowCommand {
    pub(crate) fn target(&self) -> Result<(Option<ShowKindArg>, u64), CliError> {
        match self.target.as_slice() {
            [id] => Ok((None, parse_raw_id(id)?)),
            [kind, id] => Ok((Some(parse_show_kind(kind)?), parse_raw_id(id)?)),
            _ => Err(CliError::Usage(
                "usage: vdoc show [spec|design|adr|task] <id>".to_string(),
            )),
        }
    }

    pub(crate) fn mode(&self) -> ShowMode {
        if self.path_only {
            ShowMode::PathOnly
        } else if self.frontmatter_only {
            ShowMode::FrontmatterOnly
        } else {
            ShowMode::Full
        }
    }
}

fn parse_raw_id(value: &str) -> Result<u64, CliError> {
    value
        .parse()
        .map_err(|_| CliError::Usage(format!("invalid document ID: {value}")))
}

fn parse_show_kind(value: &str) -> Result<ShowKindArg, CliError> {
    match value {
        "spec" => Ok(ShowKindArg::Spec),
        "design" => Ok(ShowKindArg::Design),
        "adr" => Ok(ShowKindArg::Adr),
        "task" => Ok(ShowKindArg::Task),
        unknown => Err(CliError::Usage(format!(
            "unknown show document kind `{unknown}`"
        ))),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShowMode {
    Full,
    PathOnly,
    FrontmatterOnly,
}

impl ShowMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::PathOnly => "path-only",
            Self::FrontmatterOnly => "frontmatter-only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ListKindArg {
    Specs,
    Designs,
    Adr,
    Tasks,
}

impl ListKindArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Specs => "specs",
            Self::Designs => "designs",
            Self::Adr => "adr",
            Self::Tasks => "tasks",
        }
    }

    pub(crate) fn matches_document(self, metadata: &DocumentMetadata) -> bool {
        matches!(
            (self, metadata),
            (Self::Specs, DocumentMetadata::Spec(_))
                | (Self::Designs, DocumentMetadata::Design(_))
                | (Self::Adr, DocumentMetadata::Adr(_))
                | (Self::Tasks, DocumentMetadata::Task(_))
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ShowKindArg {
    Spec,
    Design,
    Adr,
    Task,
}

impl ShowKindArg {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Design => "design",
            Self::Adr => "adr",
            Self::Task => "task",
        }
    }

    pub(crate) fn matches_document(self, metadata: &DocumentMetadata) -> bool {
        matches!(
            (self, metadata),
            (Self::Spec, DocumentMetadata::Spec(_))
                | (Self::Design, DocumentMetadata::Design(_))
                | (Self::Adr, DocumentMetadata::Adr(_))
                | (Self::Task, DocumentMetadata::Task(_))
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum StatusFilterArg {
    Deprecated,
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Planned,
    Doing,
    Blocked,
    Done,
    Dropped,
}

impl StatusFilterArg {
    pub(crate) fn matches_adr(self, status: AdrStatus) -> bool {
        matches!(
            (self, status),
            (Self::Proposed, AdrStatus::Proposed)
                | (Self::Accepted, AdrStatus::Accepted)
                | (Self::Rejected, AdrStatus::Rejected)
                | (Self::Deprecated, AdrStatus::Deprecated)
                | (Self::Superseded, AdrStatus::Superseded)
        )
    }

    pub(crate) fn matches_task(self, status: TaskStatus) -> bool {
        matches!(
            (self, status),
            (Self::Planned, TaskStatus::Planned)
                | (Self::Doing, TaskStatus::Doing)
                | (Self::Blocked, TaskStatus::Blocked)
                | (Self::Done, TaskStatus::Done)
                | (Self::Dropped, TaskStatus::Dropped)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum AdrStatusArg {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

impl AdrStatusArg {
    pub(crate) fn into_core(self) -> AdrStatus {
        match self {
            Self::Proposed => AdrStatus::Proposed,
            Self::Accepted => AdrStatus::Accepted,
            Self::Rejected => AdrStatus::Rejected,
            Self::Deprecated => AdrStatus::Deprecated,
            Self::Superseded => AdrStatus::Superseded,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum TaskTypeArg {
    Feature,
    Bug,
    Refactor,
    Chore,
    Docs,
    Test,
    Spike,
}

impl TaskTypeArg {
    pub(crate) fn into_core(self) -> TaskType {
        match self {
            Self::Feature => TaskType::Feature,
            Self::Bug => TaskType::Bug,
            Self::Refactor => TaskType::Refactor,
            Self::Chore => TaskType::Chore,
            Self::Docs => TaskType::Docs,
            Self::Test => TaskType::Test,
            Self::Spike => TaskType::Spike,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum PriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

impl PriorityArg {
    pub(crate) fn into_core(self) -> Priority {
        match self {
            Self::Low => Priority::Low,
            Self::Medium => Priority::Medium,
            Self::High => Priority::High,
            Self::Critical => Priority::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_init_options_with_clap() {
        let cli = Cli::try_parse_from(["vdoc", "init", "--dry-run", "--json", "--force"]).unwrap();
        let Some(Command::Init(options)) = cli.command else {
            panic!("expected init command");
        };

        assert!(options.dry_run);
        assert!(options.json);
        assert!(options.force);
    }

    #[test]
    fn parses_show_path_only_mode_with_clap() {
        let cli = Cli::try_parse_from(["vdoc", "show", "task", "22", "--path-only"]).unwrap();
        let Some(Command::Show(command)) = cli.command else {
            panic!("expected show command");
        };

        assert_eq!(command.target().unwrap(), (Some(ShowKindArg::Task), 22));
        assert_eq!(command.mode(), ShowMode::PathOnly);
    }

    #[test]
    fn parses_validation_paths_with_clap() {
        let cli =
            Cli::try_parse_from(["vdoc", "validate", "--json", "docs/tasks/active/23-task.md"])
                .unwrap();
        let Some(Command::Validate(command)) = cli.command else {
            panic!("expected validate command");
        };

        assert!(command.json);
        assert_eq!(
            command.paths,
            [PathBuf::from("docs/tasks/active/23-task.md")]
        );
    }
}
