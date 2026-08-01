---
vibedoc: 1
id: TASK-0100
kind: task
status: done
tags:
  - rust
  - cli
  - documentation
related:
  - ARCH-0004
depends_on:
  - TASK-0030
---

# initコマンドを追加する

## 目的

新しいプロジェクトで、AI向けの指示ファイルとvibe-docの標準文書構成をすぐに利用できるようにする。

## 完了条件

- `vibe-doc init`で`AGENTS.md`、それを指す`CLAUDE.md`シンボリックリンク、`docs/README.md`を作成する。
- Architecture、Decision、Research、Task用の標準ディレクトリを作成する。
- 既存ファイルを上書きせず、繰り返し安全に実行できる。
- Unix系とWindowsの両方でビルドでき、各OSのAPIでシンボリックリンクを作成する。
