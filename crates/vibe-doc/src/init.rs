use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const AGENTS_TEMPLATE: &str = include_str!("templates/AGENTS.md");
const DOCS_README_TEMPLATE: &str = include_str!("templates/docs-README.md");

const DOCUMENT_DIRECTORIES: &[&str] = &[
    "docs/architecture",
    "docs/decisions/architecture",
    "docs/decisions/product",
    "docs/decisions/domain",
    "docs/decisions/operations",
    "docs/research",
    "docs/tasks/todo",
    "docs/tasks/in-progress",
    "docs/tasks/done",
    "docs/tasks/wont-do",
];

#[derive(Debug, Eq, PartialEq)]
enum FileStatus {
    Created,
    AlreadyExists,
}

#[derive(Debug, Eq, PartialEq)]
struct InitReport {
    created_files: Vec<PathBuf>,
    existing_files: Vec<PathBuf>,
}

/// カレントディレクトリにvibe-docの標準構成を初期化する。
///
/// 既存プロジェクトでも安全に実行できるよう、ディレクトリは不足分だけを作り、
/// ファイルの作成には`create_new`を使い、シンボリックリンクも存在確認をOSに任せて
/// 既存内容を決して上書きしない。途中で別のプロセスが同名の項目を作った場合も、
/// その項目を既存扱いとして保持する。
pub(crate) fn run_init() -> ExitCode {
    match initialize_project(Path::new(".")) {
        Ok(report) => {
            for path in report.created_files {
                println!("created {}", path.display());
            }
            for path in report.existing_files {
                println!("skipped {} (already exists)", path.display());
            }
            println!("initialized vibe-doc project");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: failed to initialize project: {error}");
            ExitCode::FAILURE
        }
    }
}

/// 指定したルートへ標準ディレクトリとテンプレートを作成し、ファイルごとの結果を返す。
///
/// テストから任意の一時ディレクトリを渡せるよう、カレントディレクトリの変更には
/// 依存しない。先にすべてのディレクトリを作ることで、各テンプレートの作成処理を
/// 単純化している。既存ファイルは正常系として報告し、それ以外のI/Oエラーだけを
/// 呼び出し元へ伝播する。
fn initialize_project(root: &Path) -> io::Result<InitReport> {
    for directory in DOCUMENT_DIRECTORIES {
        fs::create_dir_all(root.join(directory))?;
    }

    let mut report = InitReport {
        created_files: Vec::new(),
        existing_files: Vec::new(),
    };

    for (relative_path, contents) in [
        (Path::new("AGENTS.md"), AGENTS_TEMPLATE),
        (Path::new("docs/README.md"), DOCS_README_TEMPLATE),
    ] {
        match write_new_file(&root.join(relative_path), contents)? {
            FileStatus::Created => report.created_files.push(relative_path.to_path_buf()),
            FileStatus::AlreadyExists => report.existing_files.push(relative_path.to_path_buf()),
        }
    }

    let claude_path = Path::new("CLAUDE.md");
    match create_new_symlink(Path::new("AGENTS.md"), &root.join(claude_path))? {
        FileStatus::Created => report.created_files.push(claude_path.to_path_buf()),
        FileStatus::AlreadyExists => report.existing_files.push(claude_path.to_path_buf()),
    }

    Ok(report)
}

fn write_new_file(path: &Path, contents: &str) -> io::Result<FileStatus> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            file.write_all(contents.as_bytes())?;
            Ok(FileStatus::Created)
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(FileStatus::AlreadyExists),
        Err(error) => Err(error),
    }
}

fn create_new_symlink(target: &Path, link: &Path) -> io::Result<FileStatus> {
    match create_file_symlink(target, link) {
        Ok(()) => Ok(FileStatus::Created),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(FileStatus::AlreadyExists),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(test)]
mod tests {
    use super::{DOCUMENT_DIRECTORIES, initialize_project};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("vibe-doc-init-{}-{id}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn creates_instruction_files_and_document_tree() {
        let root = TempDirectory::new();

        let report = initialize_project(&root.0).unwrap();

        assert_eq!(report.created_files.len(), 3);
        assert!(report.existing_files.is_empty());
        assert!(
            fs::read_to_string(root.0.join("AGENTS.md"))
                .unwrap()
                .contains("vibe-doc lint")
        );
        let claude_path = root.0.join("CLAUDE.md");
        assert!(
            fs::symlink_metadata(&claude_path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_link(claude_path).unwrap(),
            PathBuf::from("AGENTS.md")
        );
        assert!(
            fs::read_to_string(root.0.join("docs/README.md"))
                .unwrap()
                .contains("Front Matter")
        );
        for directory in DOCUMENT_DIRECTORIES {
            assert!(root.0.join(directory).is_dir(), "missing {directory}");
        }
    }

    #[test]
    fn preserves_existing_files_when_run_again() {
        let root = TempDirectory::new();
        initialize_project(&root.0).unwrap();
        fs::write(root.0.join("AGENTS.md"), "custom instructions\n").unwrap();

        let report = initialize_project(&root.0).unwrap();

        assert!(report.created_files.is_empty());
        assert_eq!(report.existing_files.len(), 3);
        assert_eq!(
            fs::read_to_string(root.0.join("AGENTS.md")).unwrap(),
            "custom instructions\n"
        );
        assert_eq!(
            fs::read_to_string(root.0.join("CLAUDE.md")).unwrap(),
            "custom instructions\n"
        );
    }
}
