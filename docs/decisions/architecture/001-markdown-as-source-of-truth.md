---
vibedoc: 1
id: DEC-001
kind: decision
decision_type: architecture
status: accepted
tags:
  - markdown
  - storage
  - git
related:
  - ARCH-001
  - ARCH-002
---

# Markdownを文書の正本にする

## コンテキスト

文書は人、AI、テキストエディタ、GitHubから編集される。Web UIでの表示やCLIでの検査も必要だが、特定アプリだけが読める保存形式にはしたくない。

## 決定

Markdown本文とYAML Front Matterを文書の正本にする。JSONはAPI応答にのみ使い、XMLやデータベースを正本にしない。

## 結果

- Gitのdiffと通常のMarkdown編集を利用できる。
- vibe-docがなくても文書を読める。
- Web UI向けの検索・逆引き情報は起動時に再構築する必要がある。
