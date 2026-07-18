---
vibedoc: 1
id: DEC-006
kind: decision
decision_type: operations
status: accepted
tags:
  - lint
  - workflow
  - validation
related:
  - ARCH-002
  - ARCH-004
---

# lintを緩い検査に限定する

## コンテキスト

厳格な必須項目や関係の強制は、文書作成の負担を増やし、ツールを使わなくなる原因になる。

## 決定

lintは壊れたFront Matter、重複ID、見つからない関連・依存先ID、壊れた管理対象リンクなど、明らかな問題を中心に報告する。

終了理由、Decisionリンク、逆方向リンク、kindごとの本文セクション、タグ形式は強制しない。

## 結果

- 文書作成時の負担を小さく保てる。
- 内容面の不足は人やAIが判断する必要がある。
