---
vibedoc: 1
id: TASK-0080
kind: task
status: done
tags:
  - rust
  - cli
  - backlinks
related:
  - ARCH-0004
depends_on:
  - TASK-0020
  - TASK-0030
---

# refsコマンドを実装する

## 目的

指定した文書がどのファイルから参照されているかを、インデックス済みの3種の関係
(`related`、`depends_on` の逆向き、Markdownリンクのバックリンク)から逆引きして
CLIで表示できるようにする。

## 仕様

- `vibe-doc refs <ID|パス> [--json]`
- 引数はまずIDとして解決し、失敗したら `docs/` 配下のファイルパスとして解決する。
- パス指定の文書がIDを持たない場合はエラー。参照元もID持ち文書のみが対象。
- デフォルト出力は関係種別ごとにグループ化し、ID・タイトル・パスを表示する。
  空のグループは省略し、参照元ゼロ件は `no references` と表示する(正常終了)。
- `--json` は `id` / `path` / `refs.related` / `refs.dependents` / `refs.backlinks`
  を持つJSONを出力する。空グループも空配列として必ず含める。
- 終了コード: 正常は0(ゼロ件含む)、解決失敗・IDなし文書・読み込み失敗は1。

## スコープ外

- IDを持たないファイル(README等)からのリンク検出。
- outbound(その文書が参照している先)の表示。

## 完了条件

- ID指定とパス指定の両方で3種の参照元が表示される。
- `--json` が仕様どおりの構造を出力する。
- 未知のID・IDなし文書の指定が終了コード1でエラーになる。
