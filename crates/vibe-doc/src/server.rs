//! ローカルHTTPサーバーと静的アセット配信を定義する。

use std::process::ExitCode;

use axum::Router;
use vibe_doc_core::parser::parse_document_tree;

const ADDRESS: &str = "127.0.0.1:3000";

/// 文書索引、JSON API、埋め込みWeb UIを同じローカルHTTPサーバーから配信する。
///
/// APIは起動時の文書スナップショットだけを公開し、書き込み用ルートを持たない。
/// UIはコンパイル時にバイナリへ取り込まれているため、起動時にNode.jsや
/// `frontend/dist`を探さず、SPAのクライアントルートも埋め込みHTMLへ戻す。
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
        let app = Router::new()
            .nest("/api", crate::api::router(state))
            .fallback(crate::embedded_ui::serve);
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
