//! Viteの成果物を生成し、Rustから参照するアセット一覧を組み立てる。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let frontend_dir = manifest_dir.join("../../frontend");
    emit_rerun_directives(&frontend_dir);
    build_frontend(&frontend_dir);

    let dist_dir = frontend_dir.join("dist");
    let assets = collect_files(&dist_dir);
    if !assets.iter().any(|path| path == Path::new("index.html")) {
        panic!("Vite did not produce frontend/dist/index.html");
    }
    write_asset_module(&dist_dir, &assets);
}

/// フロントエンドの入力だけをCargoの再実行条件として登録する。
///
/// `dist` はこのビルドスクリプト自身が更新するため監視しない。監視するとCargoの
/// 実行ごとにビルドスクリプトが再起動する循環が生じる。Viteの設定、依存関係、
/// HTML、ソース、公開ファイルを入力として扱う。
fn emit_rerun_directives(frontend_dir: &Path) {
    for input in [
        "index.html",
        "package.json",
        "pnpm-lock.yaml",
        "tsconfig.json",
        "vite.config.ts",
        "src",
        "public",
    ] {
        println!(
            "cargo:rerun-if-changed={}",
            frontend_dir.join(input).display()
        );
    }
}

fn build_frontend(frontend_dir: &Path) {
    let status = Command::new("pnpm")
        .arg("build")
        .current_dir(frontend_dir)
        .status()
        .unwrap_or_else(|error| {
            panic!("failed to run pnpm; install pnpm and frontend dependencies first: {error}")
        });
    if !status.success() {
        panic!("frontend build failed with {status}");
    }
}

/// `dist` を再帰的に走査し、URLとして使う相対パスを安定した順序で返す。
///
/// Viteがハッシュ付きのサブディレクトリを生成しても扱えるよう、ディレクトリ階層を
/// 固定せずに収集する。シンボリックリンクは配布物へ意図しないファイルを混入させる
/// 可能性があるため追跡せず、通常ファイルだけを埋め込み対象にする。
fn collect_files(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!(
                    "failed to read an entry in {}: {error}",
                    directory.display()
                )
            });
            let file_type = entry.file_type().unwrap_or_else(|error| {
                panic!("failed to inspect {}: {error}", entry.path().display())
            });
            if file_type.is_dir() {
                visit(root, &entry.path(), files);
            } else if file_type.is_file() {
                files.push(entry.path().strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }

    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort();
    files
}

/// コンパイル時に各ファイルを取り込むRustコードを`OUT_DIR`へ生成する。
///
/// URLは常にスラッシュ区切りへ正規化し、実ファイルの絶対パスはRustの文字列
/// リテラルとしてエスケープする。生成物は`embedded_ui`モジュールからincludeされ、
/// 実行時には元の`frontend/dist`が存在しなくても静的ファイルを返せる。
fn write_asset_module(dist_dir: &Path, assets: &[PathBuf]) {
    let mut source = String::from("static ASSETS: &[EmbeddedAsset] = &[\n");
    for relative_path in assets {
        let url_path = relative_path
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let file_path = dist_dir.join(relative_path);
        source.push_str(&format!(
            "    EmbeddedAsset {{ path: {url_path:?}, bytes: include_bytes!({file_path:?}) }},\n",
        ));
    }
    source.push_str("];\n");

    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("embedded_assets.rs");
    fs::write(&output, source)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output.display()));
}
