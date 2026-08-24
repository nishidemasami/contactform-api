# プロジェクト全体概要

> 最終更新: 2026-08-16 | ソース: raw/01_requirements/system.md, raw/02_conventions/github.md, raw/02_conventions/infrastructure.md

`contactform-api` は、コンタクトフォーム（<https://nishidemasami.github.io/contact/index.html>）からのPOSTを受け付ける問い合わせAPIである。
AIによる開発を前提に、モノレポ構成でインフラ・アプリ・ドキュメントを一元管理する。

---

## サービス概要

- **サービス名**: `contactform-api-public`
- **目的**: コンタクトフォームからの問い合わせを受け付け、DBに保存する
- **環境**: `develop`（テスト環境）/ `release`（本番環境）

### 主要コンポーネント

| コンポーネント | 概要 | releaseデプロイ |
|---|---|---|
| API | 問い合わせ受付API（AWS Lambda + API Gateway） | ✅ |
| DB | Amazon Aurora DSQL + Liquibase | ✅ |
| フロントエンド | テスト用静的SPA（Next.js）| ❌（developのみ）|
| レビュー資料 | カバレッジ・ドキュメント等 | ❌（developのみ）|

---

## システムアーキテクチャ

```
クライアント（ブラウザ）
  └─▶ Amazon CloudFront
        ├─▶ /api/*        → Amazon API Gateway HTTP API → AWS Lambda → Amazon Aurora DSQL
        ├─▶ /review/*     → S3（レビュー資料）※develop環境のみ
        └─▶ /             → S3（フロントエンドSPA）※develop環境のみ
```

- **API**: Rust + AWS Lambda（メモリ128MB、シングルスレッド）
- **DB**: Amazon Aurora DSQL（PostgreSQL互換・サーバレス）
- **インフラ**: AWS SAM（IaC）
- **CI/CD**: GitHub Actions
- **フロントエンド**: Next.js + TypeScript + Tailwind CSS（テスト用）

---

## 開発フロー

```
1. [人間] docs/raw/*.md に規約・要件・設計方針等を投入
2. [AI] docs/wiki/*.md へドキュメント整理・統合 → プルリクエスト作成
3. [人間] レビュー・承認・マージ
4. [AI] docs/wiki/*.md に従い実装 → プルリクエスト作成
5. [CI/CD] テスト・カバレッジ・レビュー資料の自動生成・デプロイ
6. [人間] 実装・レビュー資料・カバレッジレポートをレビュー → 承認・マージ
7. [CI/CD] テスト環境へデプロイ
8. [人間] 手動テスト → 本番環境へデプロイ
```

---

## リポジトリ構成

```
/ （リポジトリルート）
  ├── .github/workflows/  CI/CD定義
  ├── README.md           概要
  ├── AGENTS.md           AI向けプロンプト
  ├── docs/raw/04_issues_and_backlog/backlog.md バックログ・懸念事項
  ├── api/                APIサブシステム
  ├── core/               ビジネスロジック（共有ライブラリ）
  │   ├── domain/
  │   ├── application/
  │   ├── infrastructure/
  │   └── presentation/
  ├── db/                 DBサブシステム（Liquibase + SAM）
  ├── frontend/           フロントエンドサブシステム（Next.js）
  ├── review/             レビュー資料サブシステム
  └── docs/
      ├── raw/            生情報（人間投入、AI読み取り専用）
      └── wiki/           ウィキ（AI管理）
```

---

## SSMパラメータ体系

命名規則: `/環境名/サービス名/サブシステム名/パラメータ名`

詳細は [aws-settings.md](./aws-settings.md) を参照。

---

## 関連ページ

- [requirements.md](./requirements.md) — 要件定義詳細
- [coding-conventions.md](./coding-conventions.md) — コーディング規約
- [api.md](./api.md) — API規約
- [infrastructure.md](./infrastructure.md) — インフラ規約
- [database.md](./database.md) — DB規約
- [frontend.md](./frontend.md) — フロントエンド規約
- [cicd.md](./cicd.md) — CI/CD規約
- [review-process.md](./review-process.md) — レビュープロセス
- [github-settings.md](./github-settings.md) — GitHub設定
- [aws-settings.md](./aws-settings.md) — AWS設定
- [known-issues.md](./known-issues.md) — 既知の問題
