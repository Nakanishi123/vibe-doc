---
vibedoc: 1
id: DEC-004
kind: decision
decision_type: architecture
status: accepted
tags:
  - rust
  - react
  - web-ui
  - distribution
related:
  - ARCH-001
  - ARCH-003
---

# React Web UIをRustバイナリへ埋め込む

## コンテキスト

文書をブラウザで読みやすく表示したい一方、利用時にRustサーバーとフロントエンドを別々に配布・起動したくない。

## 決定

React、TypeScript、ViteでWeb UIを実装し、ビルド済みの静的アセットをRustバイナリへ埋め込む。本番は`vibe-doc serve`の単一プロセスでUIとAPIを提供する。

## 結果

- 利用者へ単一バイナリとして配布できる。
- 開発時はViteとRust APIを分けて起動できる。
- フロントエンドのビルド工程はリリース作成時に必要になる。
