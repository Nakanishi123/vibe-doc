//! ローカルHTTPサーバーと静的アセット配信を定義する。

use std::path::PathBuf;
use std::process::ExitCode;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use vibe_doc_core::parser::parse_document_tree;

const ADDRESS: &str = "127.0.0.1:3000";

/// 文書索引と静的UIを同じローカルHTTPサーバーから配信する。
///
/// 現段階では後続の単一バイナリ化タスクと責務を分け、`frontend/dist` にあるViteの
/// ビルド成果物をファイルシステムから提供する。SPAのクライアント側ルートへ直接
/// アクセスした場合は `index.html` にフォールバックする。APIは起動時の文書
/// スナップショットだけを公開し、書き込み用ルートを一切持たない。
pub(crate) fn serve(document_root: &str) -> ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("failed to start async runtime: {error}");
            return ExitCode::FAILURE;
        }
    };
    runtime.block_on(async move {
        let state = crate::api::ApiState::new(parse_document_tree(document_root));
        let dist = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../frontend/dist");
        let index = dist.join("index.html");
        if !index.is_file() {
            eprintln!("frontend build was not found; run `pnpm --dir frontend build`");
            return ExitCode::FAILURE;
        }
        let app = Router::new()
            .nest("/api", crate::api::router(state))
            .fallback_service(ServeDir::new(dist).not_found_service(ServeFile::new(index)));
        let listener = match tokio::net::TcpListener::bind(ADDRESS).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("failed to listen on {ADDRESS}: {error}");
                return ExitCode::FAILURE;
            }
        };
        println!("vibe-doc is available at http://{ADDRESS}");
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("server failed: {error}");
            return ExitCode::FAILURE;
        }
        ExitCode::SUCCESS
    })
}
