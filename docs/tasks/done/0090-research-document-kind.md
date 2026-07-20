---
vibedoc: 1
id: TASK-0090
kind: task
status: done
tags:
  - research
  - lint
  - web-ui
  - document-model
related:
  - DEC-0005
  - ARCH-0002
  - ARCH-0004
  - ARCH-0003
---

# kind: research を正式なドキュメント種別として追加する

## 目的

決定にもタスクにも属さない参照用の調査記録を `docs/research/` 配下で正式に管理できるようにし、
ID採番・lint・Web UI の分類を既存3種別と同等に扱えるようにする。

## 仕様

- ドキュメント種別として `kind: research` を追加する。
- 配置ディレクトリは `docs/research/` とする。
- IDプレフィックスは `RES-` に4桁の連番を続ける(例: `RES-0001`)。
- 採番は Decision と同じく +1 ずつ増やす。`vibe-doc next-index research` で次候補を取得できるようにする。
- research 文書は status(ライフサイクル)を持たない。lint は research 文書の status を要求も検証もしない。
- Front Matter の `tags` / `related` および Markdown リンクの索引・バックリンクは既存種別と同様に機能させる。

## 実施内容

- `vibe-doc-core` の lint の既知 kind 許可リストへ `research` を追加する。
- `next_index` の `IndexedKind` に research を追加し、IDプレフィックス `RES-`・ディレクトリ名 `research`・増分 1 を定義する。
- CLI の `next-index` サブコマンドで `research` を受け付ける。
- Web UI の kind 別分類・フィルタ表示へ research を追加する。
- `docs/research/` ディレクトリを作成する。
- `docs/README.md` のディレクトリ構成・ID規約・Front Matter の説明へ research を追記する。
  あわせて「調査から決定が生まれたら結論を Decision へ昇格させ、research からリンクする」という運用ルールを明記する。

## 完了条件

- `kind: research` の文書が `vibe-doc lint` でエラーにならない。
- `vibe-doc next-index research` が既存の `RES-` ID とファイル名から次番号(+1)を出力する。
- Web UI で research 文書が一覧・分類・フィルタに表示され、バックリンクが機能する。
- `docs/README.md` に research の規約が記載されている。
