---
vibedoc: 1
id: TASK-0020
kind: task
status: todo
tags:
  - rust
  - index
  - backlinks
related:
  - ARCH-0002
  - DEC-0003
depends_on:
  - TASK-0010
---

# メモリ上のインデックスと逆引きを構築する

## 目的

検索、タグ集計、関連文書、Task依存関係、Markdownリンクの逆引きを提供する。

## 想定スコープ

- IDインデックス。
- タグインデックス。
- タイトルと本文の部分一致検索。
- `related`の対称表示。
- `depends_on`の逆引き。
- 管理対象Markdownリンクの逆引き。
- 重複する表示項目の除去。

## 完了条件

- データベースや永続インデックスを使わず、必要な一覧と逆引きを取得できる。
