---
vibedoc: 1
id: ARCH-0004
kind: architecture
tags:
  - vibe-doc
  - cli
  - lint
  - tags
  - next-index
related:
  - ARCH-0001
  - ARCH-0002
  - DEC-0006
---

# CLIとlint

## コマンド

```bash
vibe-doc serve
vibe-doc lint
vibe-doc tag
vibe-doc next-index decision
vibe-doc next-index task
```

- `serve`: ローカルのWeb UIとJSON APIを提供する。
- `lint`: 診断を標準出力へ表示する。
- `tag`: タグ一覧を標準出力へ表示する。
- `next-index`: 指定kindの次の採番候補を標準出力へ表示する。

## Tag

`vibe-doc tag`はタグ名を辞書順で、一行に一件ずつ表示する。重複は除去する。タグが存在しない場合は何も表示せず正常終了する。

```text
$ vibe-doc tag
hoge
next-js
rust
```

## Lint

lintは次を確認する。

- YAML Front Matterを構文として読めるか。
- `id`がある文書同士で重複していないか。
- `kind`と`status`が既知の標準値から明らかに逸脱していないか。
- `related`と`depends_on`の参照先IDが見つかるか。
- 管理対象文書へ向くMarkdownリンクのファイルパスが存在するか。
- Taskのフォルダ名と`status`が異なっていないか。これはwarningとする。

次はlint対象にしない。

- `wont-do` Taskに終了理由がないこと。
- TaskにDecisionへのリンクがないこと。
- `related`や`depends_on`の逆方向が明示されていないこと。
- kindごとの推奨本文セクションがないこと。
- 未知のtagsや追加Front Matter項目。

診断レベルは`error`と`warning`を使う。

## Next Index

対象文書のファイル名またはIDから最大の数値部分を読み取る。

- Decision: 最大値に`1`を加えて4桁で返す。
- Task: 最大値に`10`を加えて4桁で返す。
- 文書ファイルは作成しない。

```text
DEC-0001, DEC-0002, DEC-0007   → vibe-doc next-index decision → 0008
TASK-0120, TASK-0130, TASK-0140 → vibe-doc next-index task     → 0150
```

ブランチ間の採番衝突は許容し、重複IDをlintで検出する。ロックや中央採番サーバーは作らない。
