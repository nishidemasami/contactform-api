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

## [2026-11-27] update | 形式手法検証の `application` / `presentation` への拡大

- 更新したウィキページ: coding-conventions.md, cicd.md, overview.md, log.md（および raw/00000011_形式手法による検証.md）
- 概要: `domain` 層の追加証明（不正 UTF-8 生バイト耐性・エラー写像全単射・前後空白保持）に加え、`application` 層の `CreateInquiryUsecase::execute_with_metadata`（Kani + `Waker::noop()` による Fail-Fast・保存1回保証・リポジトリエラー伝播証明）および `presentation` 層の `http_decision.rs`（Verus + スタンドアロン Kani によるルーティング・CORS 最優先・内部リポジトリエラー非漏洩証明）を追加し、CI と規約・知見集に反映した。

## [2026-11-27] lint | ドメイン・API・OpenAPI・ドキュメントの不審仕様是正

- 修正したページ: requirements.md, api.md, database.md, frontend.md, coding-conventions.md, backlog.md, index.md, log.md（および raw/00000011_形式手法による検証.md）
- 発見した問題: メールアドレス上限が256文字かつ構文検証なし、氏名・本文への制御文字（NUL `\0` 等）混入によるDB 500リスク、`OPTIONS` 応答が `"\"{}\""` 文字列かつ未知パスでも200応答、リクエストボディ上限／Base64判定の欠落、DB一意制約エラーが `Duplicate` に写像されない問題、`api-cicd.yaml` の OpenAPI バイナリ名不一致、`api.md` の成功ステータス誤記（200 → 201）
- 概要: メールアドレスを前後空白 `trim` 後 254文字上限＋RFC 5321/5322/1123 構文検証とし、その他文字列（氏名100文字・本文5,000文字）は `trim` 前の文字列で文字数検証＋制御文字拒否に統一して Verus / Kani で形式証明を更新。あわせて Presentation・Infrastructure・Frontend・OpenAPI・CI・各ウィキ仕様書を整合させた。
