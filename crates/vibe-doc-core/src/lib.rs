//! vibe-docの文書ツリーを読み取るためのドメイン型とサービス。
//!
//! このクレートはHTTPやUIに依存しない。コマンドラインアプリケーションと、将来の
//! JSON APIから共有して利用する。

pub mod document;
pub mod index;
pub mod links;
pub mod lint;
pub mod next_index;
pub mod parser;
