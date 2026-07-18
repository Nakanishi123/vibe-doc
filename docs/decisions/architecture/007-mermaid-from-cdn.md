---
vibedoc: 1
id: DEC-007
kind: decision
decision_type: architecture
status: accepted
tags:
  - mermaid
  - markdown
  - cdn
related:
  - ARCH-003
---

# MermaidをCDNから読み込んで描画する

## コンテキスト

Markdown内のMermaid記法をWeb UIで図として表示したい。Mermaid本体を埋め込むと、フロントエンド成果物とRustバイナリが大きくなる。

## 決定

固定バージョンのMermaid ES ModuleをCDNから実行時に読み込み、`mermaid`コードブロックをSVGへ変換する。`securityLevel: strict`で初期化する。

## 結果

- Mermaidをバイナリへ含めず図を表示できる。
- 図の表示にはネットワーク接続が必要になる。
- CDN読込や構文解析に失敗した場合は元のコードブロックへフォールバックする。
