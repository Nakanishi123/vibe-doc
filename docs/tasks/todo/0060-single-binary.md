---
vibedoc: 1
id: TASK-0060
kind: task
status: todo
tags:
  - rust
  - react
  - distribution
related:
  - ARCH-0001
  - DEC-0004
depends_on:
  - TASK-0030
  - TASK-0040
  - TASK-0050
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
