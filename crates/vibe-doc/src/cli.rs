use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::process::ExitCode;

use vibe_doc_core::index::DocumentIndex;
use vibe_doc_core::lint::{DiagnosticLevel, lint};
use vibe_doc_core::next_index::{IndexedKind, next_index};
use vibe_doc_core::parser::parse_document_tree;

const DOCUMENT_ROOT: &str = "docs";

/// vibe-docのコマンドライン引数。
#[derive(Debug, Parser)]
#[command(name = "vibe-doc", version, about = "Markdown documentation tools")]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// CLIから実行できる一回限りの操作。
#[derive(Debug, Subcommand)]
enum Command {
    /// 文書の構造上の問題を検査する。
    Lint,
    /// 重複のないタグ一覧を表示する。
    Tag,
    /// 次に使える文書番号を表示する。
    NextIndex { kind: CommandKind },
    /// 指定した文書を参照している文書を表示する。
    Refs {
        /// 文書ID(例: TASK-0030)またはdocs配下のファイルパス。
        target: String,
        /// 空のグループも含む構造化JSONで出力する。
        #[arg(long)]
        json: bool,
    },
    /// ローカルWeb UIを起動する。
    Serve,
}

/// `next-index`で指定できる文書種別。
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CommandKind {
    Decision,
    Task,
}

impl From<CommandKind> for IndexedKind {
    fn from(kind: CommandKind) -> Self {
        match kind {
            CommandKind::Decision => Self::Decision,
            CommandKind::Task => Self::Task,
        }
    }
}

/// 解釈済みのCLIコマンドを実行する。
pub(crate) fn run(cli: Cli) -> ExitCode {
    match cli.command {
        Some(Command::Lint) => run_lint(),
        Some(Command::Tag) => run_tag(),
        Some(Command::NextIndex { kind }) => run_next_index(kind.into()),
        Some(Command::Refs { target, json }) => run_refs(&target, json),
        Some(Command::Serve) => crate::server::serve(DOCUMENT_ROOT),
        None => {
            println!("{}", Cli::command().render_help());
            ExitCode::SUCCESS
        }
    }
}

fn run_lint() -> ExitCode {
    let tree = parse_document_tree(DOCUMENT_ROOT);
    let diagnostics = lint(&tree);
    for diagnostic in &diagnostics {
        println!(
            "{}: {}: {}",
            diagnostic.level.as_str(),
            diagnostic.path.display(),
            diagnostic.message
        );
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level == DiagnosticLevel::Error)
    {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run_tag() -> ExitCode {
    let tree = parse_document_tree(DOCUMENT_ROOT);
    let index = DocumentIndex::from_document_map(&tree.documents);
    for tag in index.tags() {
        println!("{tag}");
    }
    ExitCode::SUCCESS
}

fn run_refs(target: &str, json: bool) -> ExitCode {
    let tree = parse_document_tree(DOCUMENT_ROOT);
    let index = DocumentIndex::from_document_map(&tree.documents);
    match crate::refs::run_refs(target, json, &index, &tree.documents) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_next_index(kind: IndexedKind) -> ExitCode {
    let tree = parse_document_tree(DOCUMENT_ROOT);
    println!("{:04}", next_index(kind, tree.documents.into_values()));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command, CommandKind};
    use clap::Parser;

    #[test]
    fn parses_next_index_kind_as_a_constrained_value() {
        let cli = Cli::try_parse_from(["vibe-doc", "next-index", "task"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::NextIndex {
                kind: CommandKind::Task
            })
        ));
        assert!(Cli::try_parse_from(["vibe-doc", "next-index", "unknown"]).is_err());
    }
}
