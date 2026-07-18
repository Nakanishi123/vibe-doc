---
vibedoc: 1
id: TASK-0050
kind: task
status: todo
tags:
  - react
  - mermaid
  - cdn
related:
  - ARCH-0003
  - DEC-0007
depends_on:
  - TASK-0040
---

# Mermaid図を描画する

## 目的

Markdownの`mermaid`コードブロックをWeb UIで図として表示する。

## 想定スコープ

- 固定バージョンのMermaid ES ModuleをCDNから読み込む。
- `securityLevel: strict`で初期化する。
- `language-mermaid`コードブロックをSVGへ変換する。
- CDN読込失敗と構文エラー時にコード表示へフォールバックする。

## 完了条件

- 正しいMermaid記法がSVGとして表示される。
- オフライン時や不正なMermaid記法があっても文書ページ全体は表示できる。
