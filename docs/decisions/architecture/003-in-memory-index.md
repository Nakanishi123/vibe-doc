---
vibedoc: 1
id: DEC-003
kind: decision
decision_type: architecture
status: accepted
tags:
  - index
  - search
  - database
related:
  - ARCH-001
  - ARCH-002
  - DEC-001
---

# インデックスはメモリ上に構築する

## コンテキスト

タグ集計、検索、関連文書、依存関係、逆引きにはインデックスが必要になる。一方、Markdown以外の永続状態を持つと、同期や再構築の管理が増える。

## 決定

SQLiteなどのデータベースや永続インデックスを持たない。起動時に`./docs`配下を走査し、メモリ上へインデックスを構築する。ファイル変更時は対象文書を再解析して更新する。

## 結果

- Markdownだけが正本として残る。
- インデックスの移行や破損復旧が不要になる。
- プロセスを起動するたびに文書を走査する。
