---
vibedoc: 1
id: TASK-060
kind: task
status: todo
tags:
  - rust
  - react
  - distribution
related:
  - ARCH-001
  - DEC-004
depends_on:
  - TASK-030
  - TASK-040
  - TASK-050
---

# 単一バイナリとして配布する

## 目的

React Web UIとRustサーバーを、利用時に一つの実行ファイルとして扱えるようにする。

## 想定スコープ

- ViteでReactの静的アセットをビルドする。
- ビルド済みアセットをRustバイナリへ埋め込む。
- `vibe-doc serve`でUIとAPIを配信する。

## 完了条件

- Node.jsを別途起動せず、生成したバイナリだけでWeb UIを利用できる。
