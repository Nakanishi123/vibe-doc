---
vibedoc: 1
id: ARCH-0003
kind: architecture
tags:
  - vibe-doc
  - web-ui
  - react
  - tags
  - mermaid
related:
  - ARCH-0001
  - ARCH-0002
  - DEC-0002
  - DEC-0004
  - DEC-0007
---

# Web UI

## 画面

| 画面 | 内容 |
| --- | --- |
| Dashboard | 文書数、種別・状態ごとの件数、最近更新された文書、lint概要。 |
| Documents | 全文書の検索と、種別・状態・タグによる絞り込み。 |
| Decisions | Decisionの種別、状態、タグによる一覧。 |
| Tasks | `todo`、`in-progress`、`done`、`wont-do`で絞り込める一覧。 |
| Document Detail | Markdown本文、メタデータ、関連文書、Taskの依存関係、本文リンク、逆引き。 |
| Tags | `/tags`でタグ名と文書数を一覧表示。 |
| Tag Detail | `/tag/{tag}`で該当する全種別の文書を表示。 |
| Links | 関連文書、Taskの依存関係、本文リンクと逆引きの一覧。 |
| Lint | 最新の診断結果を重要度別に表示。 |

Front MatterはYAMLのまま本文へ表示せず、ID、kind、status、tags、追加メタデータをヘッダーやサイド情報として整形する。

## 検索

ID、タイトル、タグ、本文に対する、大文字・小文字を区別しない部分一致とする。検索には起動時に構築したメモリ上のインデックスを使う。

## タグ

タグは文書種別をまたいで集計する。`/tags`の一覧、文書一覧、文書詳細に表示されるタグはクリック可能にする。

タグを選択すると`/tag/{tag}`へ移動する。例えば`next-js`は`/tag/next-js`とする。タグ値にURL上で特別な意味を持つ文字があれば、パス部分をURLエンコードする。

タグ名は小文字のkebab-caseを推奨するが、lintでは強制しない。

## Mermaid

Markdown本文の`mermaid`コードブロックを図として描画する。

````markdown
```mermaid
graph LR
    TASK-0120 --> TASK-0150
    DEC-0007 --> TASK-0150
```
````

Markdownレンダリング後に`language-mermaid`のコードブロックを検出し、CDNから読み込んだMermaidでSVGへ変換する。

```text
https://cdn.jsdelivr.net/npm/mermaid@<固定バージョン>/dist/mermaid.esm.min.mjs
```

- 実装時に特定のMermaidバージョンへ固定する。
- `securityLevel: strict`で初期化する。
- CDNへ接続できない場合は元のコードブロックを表示する。
- Mermaidの構文解析に失敗してもページ全体を壊さず、コードと簡潔なエラーを表示する。

## API

```text
GET /api/documents
GET /api/documents/{id}
GET /api/tags
GET /api/tags/{tag}
GET /api/links
GET /api/lint
GET /api/next-index/{kind}
```
