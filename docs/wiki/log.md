# 操作ログ

append-onlyの操作記録。既存エントリは編集・削除しない。

`grep "^## \[" log.md` でエントリ一覧を取得できる。

---

## [2026-08-16] ingest | docs/raw/*.md（初回）

- 更新したウィキページ: llm-wiki.md（新規）, index.md（新規）, log.md（新規）, overview.md（新規）, requirements.md（新規）, coding-conventions.md（新規）, api.md（新規）, infrastructure.md（新規）, database.md（新規）, frontend.md（新規）, cicd.md（新規）, review-process.md（新規）, github-settings.md（新規）, aws-settings.md（新規）, known-issues.md（新規）, backlog.md（新規）
- 概要: `docs/raw/` 配下の全ファイルを初回取り込み。LLM-WIKI運用ルールを策定し、全規約・要件をウィキに整理・統合した。

## [2026-11-27] ingest | docs/raw/00000011_形式手法による検証.md

- 更新したウィキページ: coding-conventions.md, cicd.md, known-issues.md, backlog.md, overview.md, index.md, log.md
- 概要: `domain` 層の `#![no_std]` 化および Verus / Kani による形式手法検証の知見・ノウハウ、理想（クレート統合インライン方式）と現実（Verified Core 分離方式）のトレードオフ、CBMC 状態爆発回避パターン、今後の改善案を整理・統合した。
