---
vibedoc: 1
id: TASK-0010
kind: task
status: done
tags:
  - rust
  - markdown
  - parser
related:
  - ARCH-0001
  - ARCH-0002
  - DEC-0001
---

# Markdown文書を解析する

## 目的

`./docs`配下を走査し、YAML Front Matter、タイトル、Markdown本文を共通Documentモデルへ変換する。

## 想定スコープ

- Decision、Task、Architecture文書の検出。
- Front MatterとMarkdown本文の分離。
- ID、kind、status、tags、related、depends_onの読込。
- 未知のFront Matter項目の許容。
- ファイル変更の検知と再解析。

## 完了条件

- 標準フォルダ構成のMarkdownを読み込める。
- 構文エラーを診断として保持できる。
