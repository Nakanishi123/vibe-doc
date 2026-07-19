---
vibedoc: 1
id: TASK-0070
kind: task
status: done
tags:
  - react
  - mermaid
  - offline
  - dark-mode
related:
  - ARCH-0003
  - DEC-0008
---

# Mermaidを埋め込みダークモードへ対応する

## 目的

Mermaidの実行時CDN依存をなくし、Web UIのテーマに合った読みやすい図を表示する。

## 実施内容

- 固定バージョンのMermaidをフロントエンド依存関係へ追加する。
- Mermaidを遅延ロード可能なViteチャンクとしてRustバイナリへ埋め込む。
- Web UIのカラーテーマをMermaid図へ共有する。
- テーマ切り替え時にライトまたはダークテーマで図を再描画する。
- 読み込み失敗と構文エラー時のコード表示フォールバックを維持する。

## 完了条件

- 外部ネットワークへ接続せずMermaid図を表示できる。
- ライト・ダークテーマの切り替えが表示中のMermaid図へ反映される。
- Mermaidの構文エラーが文書ページ全体の表示を妨げない。
