---
vibedoc: 1
id: DEC-0002
kind: decision
decision_type: product
status: accepted
tags:
  - scope
  - web-ui
  - read-only
related:
  - ARCH-0001
  - ARCH-0003
---

# Web UIを読み取り専用にする

## コンテキスト

文書の作成や編集はAIや既存のエディタで行える。Web UIへ編集機能を追加すると、競合、状態遷移、ファイル移動、認証などの責務が増える。

## 決定

Web UIは文書の閲覧、検索、絞り込み、タグ移動、関連・逆引き表示、lint結果の確認に限定する。

## 結果

- UIとAPIを小さく保てる。
- Markdownの編集方法を制限しない。
- Web UIだけでは文書を作成・更新できない。
