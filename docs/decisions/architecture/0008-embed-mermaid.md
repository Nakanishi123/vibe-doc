---
vibedoc: 1
id: DEC-0008
kind: decision
decision_type: architecture
status: accepted
tags:
  - mermaid
  - markdown
  - offline
  - dark-mode
related:
  - ARCH-0003
  - DEC-0007
---

# MermaidをWeb UIへ埋め込みカラーテーマへ追従させる

## コンテキスト

MermaidをCDNから実行時に読み込む構成では、図の表示にインターネット接続と外部サービス
の可用性が必要になる。また、Mermaidを既定テーマで一度だけ初期化しているため、Web UIを
ダークテーマへ切り替えても図の配色はライトテーマのままで視認性が低い。

Web UIのVite成果物はすでにRustバイナリへ埋め込まれており、Mermaidも同じ配信経路へ
含められる。

## 決定

固定バージョンのMermaidをフロントエンドの依存関係として追加し、Viteで生成したチャンクを
Rustバイナリへ埋め込む。Mermaidは動的importで遅延読み込みし、図のない画面の初期ロードへ
含めない。

Web UIのカラーテーマをMermaid描画コンポーネントへ共有する。ライトテーマではMermaidの
`default`、ダークテーマでは`dark`を指定し、テーマ切り替え時には表示中の図を再描画する。
`securityLevel: strict`は維持する。複数の図が同時に描画されてもテーマ設定が混ざらないよう、
Mermaidの初期化と描画を直列に実行する。

## 結果

- インターネット接続やCDNの状態に依存せずMermaid図を表示できる。
- Mermaid図がWeb UIのライト・ダークテーマへ追従する。
- 図を含む画面でだけMermaidチャンクを読み込む。
- Mermaid本体と依存パッケージの分だけ配布バイナリが大きくなる。
- Mermaidの更新はフロントエンド依存関係とロックファイルを通じて管理する。
