//! Viteで生成してバイナリへ埋め込んだWeb UIを配信する。

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode, Uri};
use axum::response::Response;

struct EmbeddedAsset {
    path: &'static str,
    bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

/// 要求された静的アセット、またはSPAのエントリーポイントを返す。
///
/// Viteが`index.html`から参照するハッシュ付きアセットは完全一致で返す。React側の
/// クライアントルートは実ファイルを持たないため、見つからないパスを`index.html`へ
/// フォールバックさせる。アセットはすべてコンパイル時に埋め込まれており、実行時に
/// `frontend/dist`やNode.jsを必要としない。
pub(crate) async fn serve(uri: Uri) -> Response<Body> {
    let requested_path = uri.path().trim_start_matches('/');
    let asset = find_asset(requested_path).or_else(|| find_asset("index.html"));
    match asset {
        Some(asset) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type(asset.path))
            .header(CACHE_CONTROL, cache_control(asset.path))
            .body(Body::from(asset.bytes))
            .expect("embedded asset response contains valid static headers"),
        None => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("embedded UI is unavailable"))
            .expect("static error response is valid"),
    }
}

fn find_asset(path: &str) -> Option<&'static EmbeddedAsset> {
    let path = if path.is_empty() { "index.html" } else { path };
    ASSETS.iter().find(|asset| asset.path == path)
}

fn content_type(path: &str) -> HeaderValue {
    let value = match PathExtension::from_path(path) {
        PathExtension::Css => "text/css; charset=utf-8",
        PathExtension::Html => "text/html; charset=utf-8",
        PathExtension::JavaScript => "text/javascript; charset=utf-8",
        PathExtension::Json => "application/json",
        PathExtension::Svg => "image/svg+xml",
        PathExtension::Png => "image/png",
        PathExtension::Jpeg => "image/jpeg",
        PathExtension::Gif => "image/gif",
        PathExtension::Icon => "image/x-icon",
        PathExtension::Webp => "image/webp",
        PathExtension::Woff => "font/woff",
        PathExtension::Woff2 => "font/woff2",
        PathExtension::Other => "application/octet-stream",
    };
    HeaderValue::from_static(value)
}

fn cache_control(path: &str) -> HeaderValue {
    if path == "index.html" {
        HeaderValue::from_static("no-cache")
    } else {
        HeaderValue::from_static("public, max-age=31536000, immutable")
    }
}

enum PathExtension {
    Css,
    Html,
    JavaScript,
    Json,
    Svg,
    Png,
    Jpeg,
    Gif,
    Icon,
    Webp,
    Woff,
    Woff2,
    Other,
}

impl PathExtension {
    fn from_path(path: &str) -> Self {
        match path.rsplit_once('.').map(|(_, extension)| extension) {
            Some("css") => Self::Css,
            Some("html") => Self::Html,
            Some("js" | "mjs") => Self::JavaScript,
            Some("json" | "map") => Self::Json,
            Some("svg") => Self::Svg,
            Some("png") => Self::Png,
            Some("jpg" | "jpeg") => Self::Jpeg,
            Some("gif") => Self::Gif,
            Some("ico") => Self::Icon,
            Some("webp") => Self::Webp,
            Some("woff") => Self::Woff,
            Some("woff2") => Self::Woff2,
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ASSETS, find_asset};

    #[test]
    fn embeds_vite_index_and_static_assets() {
        let index = find_asset("index.html").expect("index.html should be embedded");
        assert!(index.bytes.starts_with(b"<!doctype html>"));
        assert!(ASSETS.iter().any(|asset| asset.path.starts_with("assets/")));
    }

    #[test]
    fn resolves_root_to_index() {
        assert_eq!(find_asset("").unwrap().path, "index.html");
    }
}
