use clap::{CommandFactory, Parser, Subcommand};
use std::net::{IpAddr, Ipv4Addr};
use std::process::ExitCode;

use vibe_doc_core::index::DocumentIndex;
use vibe_doc_core::lint::{DiagnosticLevel, lint};
use vibe_doc_core::next_index::{IndexedKind, InvalidIndexedKind, next_index};
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
    /// プロジェクト用のAI指示ファイルと文書ディレクトリを作成する。
    Init,
    /// 文書の構造上の問題を検査する。
    Lint,
    /// 重複のないタグ一覧を表示する。
    Tag,
    /// 次に使える文書番号を表示する。
    NextIndex {
        /// 採番対象の文書種別。decision・task・researchのいずれかを指定する。
        #[arg(value_parser = parse_indexed_kind)]
        kind: IndexedKind,
    },
    /// 指定した文書を参照している文書を表示する。
    Refs {
        /// 文書ID(例: TASK-0030)またはdocs配下のファイルパス。
        target: String,
        /// 空のグループも含む構造化JSONで出力する。
        #[arg(long)]
        json: bool,
    },
    /// Web UIを起動する。
    Serve {
        /// 待ち受けるIPアドレス。
        #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
        host: IpAddr,
        /// 待ち受けるポート。0を指定すると空きポートを自動選択する。
        #[arg(long, default_value_t = 0)]
        port: u16,
    },
}

/// clapの引数文字列を`IndexedKind`へ変換する。`IndexedKind`の`FromStr`を唯一の
/// 変換元として再利用し、CLI・API・lintで受け付ける種別の定義が分散しないようにする。
fn parse_indexed_kind(value: &str) -> Result<IndexedKind, InvalidIndexedKind> {
    value.parse()
}

/// 解釈済みのCLIコマンドを実行する。
pub(crate) fn run(cli: Cli) -> ExitCode {
    match cli.command {
        Some(Command::Init) => crate::init::run_init(),
        Some(Command::Lint) => run_lint(),
        Some(Command::Tag) => run_tag(),
        Some(Command::NextIndex { kind }) => run_next_index(kind),
        Some(Command::Refs { target, json }) => run_refs(&target, json),
        Some(Command::Serve { host, port }) => crate::server::serve(DOCUMENT_ROOT, host, port),
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
    use super::{Cli, Command};
    use clap::Parser;
    use std::net::{IpAddr, Ipv4Addr};
    use vibe_doc_core::next_index::IndexedKind;

    #[test]
    fn parses_init_command() {
        let cli = Cli::try_parse_from(["vibe-doc", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Init)));
    }

    #[test]
    fn parses_next_index_kind_as_a_constrained_value() {
        let cli = Cli::try_parse_from(["vibe-doc", "next-index", "task"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::NextIndex {
                kind: IndexedKind::Task
            })
        ));
        let cli = Cli::try_parse_from(["vibe-doc", "next-index", "research"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::NextIndex {
                kind: IndexedKind::Research
            })
        ));
        assert!(Cli::try_parse_from(["vibe-doc", "next-index", "unknown"]).is_err());
    }

    #[test]
    fn parses_serve_network_options() {
        let cli = Cli::try_parse_from(["vibe-doc", "serve", "--host", "0.0.0.0", "--port", "3000"])
            .unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Serve {
                host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                port: 3000
            })
        ));
    }

    #[test]
    fn serve_defaults_to_loopback_and_an_automatic_port() {
        let cli = Cli::try_parse_from(["vibe-doc", "serve"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Serve {
                host: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0
            })
        ));
    }
}
