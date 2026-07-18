---
vibedoc: 1
id: TASK-040
kind: task
status: todo
tags:
  - react
  - web-ui
  - tags
  - search
related:
  - ARCH-003
  - DEC-002
  - DEC-004
depends_on:
  - TASK-010
  - TASK-020
---

# 読み取り専用Web UIを実装する

## 目的

Markdown文書を人が読みやすく閲覧、検索、横断参照できるようにする。

## 想定スコープ

- Dashboard、Documents、Decisions、Tasks。
- Document Detail、Links、Lint。
- `/tags`のタグ一覧。
- `/tag/{tag}`のタグ別文書一覧。
- 文書一覧と詳細にあるタグからの画面遷移。
- ID、タイトル、タグ、本文の部分一致検索。

## 完了条件

- Front Matterを専用UIとして表示できる。
- タグ、関連文書、依存関係、逆引きから文書間を移動できる。
- Web UIから文書を変更できない。
