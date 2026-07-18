---
vibedoc: 1
id: ARCH-002
kind: architecture
tags:
  - vibe-doc
  - markdown
  - front-matter
  - document-model
related:
  - ARCH-001
  - DEC-001
  - DEC-005
  - DEC-006
---

# ドキュメントモデル

## ドキュメントルート

初期版は、カレントディレクトリ直下の`./docs`を固定のドキュメントルートとして扱う。

```text
docs/
├── decisions/
│   ├── architecture/
│   ├── product/
│   ├── domain/
│   └── operations/
├── tasks/
│   ├── todo/
│   ├── in-progress/
│   ├── done/
│   └── wont-do/
└── architecture/
```

## 共通Front Matter

```yaml
---
vibedoc: 1
id: DEC-007
kind: decision
status: accepted
tags:
  - api
  - error-handling
related:
  - ARCH-001
---
```

| 項目 | 説明 |
| --- | --- |
| `vibedoc` | 文書スキーマのバージョン。初期値は`1`。 |
| `id` | 文書を識別する一意ID。 |
| `kind` | 初期版では`decision`、`task`、`architecture`。 |
| `status` | kindごとの状態。 |
| `tags` | 任意のタグ配列。 |
| `related` | 関連文書のID。相手側への記載は不要。 |
| `depends_on` | Taskが依存する別TaskのID。 |

日付や優先度などの追加項目は許容する。未知の項目はエラーにしない。

## DecisionとADR

Decisionを上位概念とし、ADRは`architecture`種別のDecisionとして扱う。

| 保存場所 | `decision_type` | 用途 |
| --- | --- | --- |
| `decisions/architecture/` | `architecture` | システム構造、技術選定、セキュリティ。ADRを含む。 |
| `decisions/product/` | `product` | 機能、対象、提供方針。 |
| `decisions/domain/` | `domain` | 業務ルール、用語、モデル上の判断。 |
| `decisions/operations/` | `operations` | 運用、開発プロセス、保守方針。 |

Decision本文では、`コンテキスト`、`決定`、`結果`の見出しを推奨するが、lintでは強制しない。

## Task

| 保存場所 | `status` | 意味 |
| --- | --- | --- |
| `tasks/todo/` | `todo` | 未着手・実施候補。 |
| `tasks/in-progress/` | `in-progress` | 実施中。 |
| `tasks/done/` | `done` | 実施済み。 |
| `tasks/wont-do/` | `wont-do` | 実施しないと決めて終了。 |

フォルダは人が状態を把握するために使う。Front Matterの`status`はUIの絞り込みやAPIに使う。両者の不一致は警告にできるが、エラーにはしない。

`wont-do`の理由がTask固有ならTask本文へ記載する。同じ判断が将来にも適用される方針なら、対応するDecisionを作り、`related`またはMarkdownリンクで参照する。理由やDecisionリンクは必須にしない。

## 関連文書と逆引き

汎用的な`relations`は設けない。

```yaml
related:
  - DEC-007
depends_on:
  - TASK-120
```

- `related`は対称的な関係として表示する。どちらか一方にだけ記述する。
- `depends_on`は方向を持つ。記述元では「依存先」、参照先では「このTaskに依存しているTask」と表示する。
- 管理対象文書へのMarkdownリンクは、参照先で「この文書を参照している文書」と表示する。

逆引きはMarkdownへ保存しない。起動時に全対象文書を走査してメモリ上に生成し、ファイル変更時に更新する。

| Markdownに書く情報 | 記述元 | 参照先 |
| --- | --- | --- |
| `related: [DEC-007]` | 関連文書 | 関連文書 |
| `depends_on: [TASK-120]` | 依存先 | このTaskに依存しているTask |
| Markdownリンク | 本文中のリンク | この文書を参照している文書 |

両側への重複記述やFront Matterと本文リンクの重複は、表示時に除去する。
