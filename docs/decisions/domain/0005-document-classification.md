---
vibedoc: 1
id: DEC-0005
kind: decision
decision_type: domain
status: accepted
tags:
  - decision
  - adr
  - task
  - taxonomy
related:
  - ARCH-0002
---

# Decision、ADR、Taskの分類を定める

## コンテキスト

すべての判断がアーキテクチャ判断とは限らない。また、Taskは進行状態をフォルダから把握できる必要がある。

## 決定

Decisionを上位概念とし、ADRは`architecture`種別のDecisionとして扱う。Decisionは`architecture`、`product`、`domain`、`operations`のフォルダへ分ける。

Taskは`todo`、`in-progress`、`done`、`wont-do`のフォルダへ分ける。

`wont-do`の理由が将来にも適用される方針ならDecisionを作る。Task固有の理由ならTask本文だけに書いてよい。どちらもlintでは強制しない。

## 結果

- ADR以外の判断もDecisionとして記録できる。
- GitHubやファイル一覧から種別とTask状態を把握できる。
- TaskとDecisionを必要に応じて`related`またはMarkdownリンクで結べる。
