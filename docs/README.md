# ドキュメント (docs)

本ディレクトリ (`docs/`) は、`contactform-api` プロジェクトに関するすべてのドキュメントを管理します。

## ディレクトリ構成と概要

`docs/` ディレクトリは以下の2つのサブディレクトリで構成されています。

- **`raw/` (生情報)**: 人間が投入する一次情報（規約原文、要件定義、タスク管理など）。イミュータブル（読み取り専用）として扱います。「要件 (01_requirements)」「規約 (02_conventions)」「実装例 (03_examples)」「課題・バックログ (04_issues_and_backlog)」の3大分類＋課題に整理・分割されています。
- **`wiki/` (ウィキ)**: AIエージェントが `raw/` の情報を整理・統合・更新して維持管理するナレッジベース（LLM-WIKI）。

---

## wiki ドキュメント一覧

`./wiki/` ディレクトリ配下に格納されている整理・統合されたウィキページの一覧と要約です。

- [api.md](./wiki/api.md)
  - **概要**: バックエンドAPI（AWS Lambda）の設計規約、レイヤー構造、レスポンス・エラーハンドリング仕様およびパフォーマンス最適化方針。
- [aws-settings.md](./wiki/aws-settings.md)
  - **概要**: AWSのIAM権限設定、GitHub OIDC連携方針、SSMパラメータストアの階層構造と環境別パラメータ体系。
- [backlog.md](./wiki/backlog.md)
  - **概要**: バックログ・懸念事項・TODOの整理ページ。現時点の懸念事項や未解決課題の状況を管理。
- [cicd.md](./wiki/cicd.md)
  - **概要**: GitHub Actionsワークフロー（API, Frontend, Retained, Wiki Lint等）の設計とCI/CDパイプラインの全体像。
- [coding-conventions.md](./wiki/coding-conventions.md)
  - **概要**: Rust言語のコーディング規約、ドメイン駆動設計（DDD）のレイヤー構成、型明記ルールやエラー設計方針。
- [database.md](./wiki/database.md)
  - **概要**: Liquibaseを用いたDBマイグレーション規約、Aurora DSQL / PostgreSQL運用、SeaORMエンティティ定義規約。
- [frontend.md](./wiki/frontend.md)
  - **概要**: テスト用SPAフロントエンド（Next.js）のアーキテクチャ、ビルド・デプロイ設定（develop環境限定）。
- [github-settings.md](./wiki/github-settings.md)
  - **概要**: モノレポ構成方針、リポジトリのディレクトリ構造、GitHub上のアクセス権限や各種設定方針。
- [index.md](./wiki/index.md)
  - **概要**: ウィキ全ページの目次インデックスおよび最終更新日一覧。
- [infrastructure.md](./wiki/infrastructure.md)
  - **概要**: AWS SAMを用いたインフラ構築、サーバレスアーキテクチャ設計、自動命名ルールとスタック分離方針。
- [known-issues.md](./wiki/known-issues.md)
  - **概要**: CloudFormationスタック名長制限など、プロジェクトで把握されている既知の制約事項や注意点。
- [llm-wiki.md](./wiki/llm-wiki.md)
  - **概要**: AIエージェントと人間が協調してウィキを維持管理するためのLLM-WIKI運用ルール（スキーマ定義）。
- [log.md](./wiki/log.md)
  - **概要**: ウィキの更新・操作履歴（ingest, query, lint）を追記形式（append-only）で記録するログ。
- [overview.md](./wiki/overview.md)
  - **概要**: `contactform-api` システム全体の概要、目的、システム構成、およびサブシステム一覧。
- [requirements.md](./wiki/requirements.md)
  - **概要**: コンタクトフォームAPIの機能要件・非機能要件、全体アーキテクチャ要件の統合まとめ。
- [retained.md](./wiki/retained.md)
  - **概要**: フロントエンドSPAおよびレビュー資料配信用S3バケットを管理する永続インフラ（retainedサブシステム）の解説。
- [review-process.md](./wiki/review-process.md)
  - **概要**: プルリクエスト開発フロー、自動テスト・カバレッジ検証、プレビュー環境自動生成とレビュープロセス。

---

## raw ドキュメント一覧

`./raw/` ディレクトリ配下に格納されている生情報ファイルの一覧と要約です（詳細およびファイルツリーは [docs/raw/README.md](./raw/README.md) を参照のこと）。

### 1. 要件 (`./raw/01_requirements/`)
- [system.md](./raw/01_requirements/system.md): 全体システム要件・サービス名・開発フロー・SSMパラメータ命名方針
- [api.md](./raw/01_requirements/api.md): 問い合わせAPI機能要件
- [frontend_and_review.md](./raw/01_requirements/frontend_and_review.md): テスト用フロントエンドおよびレビュー資料要件

### 2. 規約 (`./raw/02_conventions/`)
- [coding.md](./raw/02_conventions/coding.md): Rustコーディング規約・DDD設計原則
- [api.md](./raw/02_conventions/api.md): バックエンドAPI設計・INSERT専用権限・コールドスタート最適化規約
- [infrastructure.md](./raw/02_conventions/infrastructure.md): AWS SAM・サーバレスアーキテクチャ設計規約
- [database.md](./raw/02_conventions/database.md): Liquibase・Aurora DSQL/PostgreSQL運用・ロール権限規約
- [cicd.md](./raw/02_conventions/cicd.md): GitHub Actions CI/CD パイプライン設計規約
- [aws_iam.md](./raw/02_conventions/aws_iam.md): AWS IAM & OIDC 連携規約
- [frontend.md](./raw/02_conventions/frontend.md): フロントエンド開発・セキュリティ規約
- [review_process.md](./raw/02_conventions/review_process.md): PR・自動レビュー資料作成運用規約
- [github.md](./raw/02_conventions/github.md): モノレポ構造・リポジトリ設定規約

### 3. 実装例 (`./raw/03_examples/`)
- [rust_ddd.md](./raw/03_examples/rust_ddd.md): Rust DDD レイヤー別実装例・`Cargo.toml`
- [lambda_handler.md](./raw/03_examples/lambda_handler.md): AWS Lambda エントリーポイント実装例
- [sam_templates.md](./raw/03_examples/sam_templates.md): AWS SAM `template.yaml` 各種実装例
- [liquibase_and_seaorm.md](./raw/03_examples/liquibase_and_seaorm.md): Liquibase changelog/sql & SeaORM Entity実装例
- [github_actions_workflows.md](./raw/03_examples/github_actions_workflows.md): GitHub Actions ワークフロー実装例
- [aws_oidc_policy.md](./raw/03_examples/aws_oidc_policy.md): AWS IAM OIDC ポリシー実装例

### 4. 課題・バックログ (`./raw/04_issues_and_backlog/`)
- [known_issues.md](./raw/04_issues_and_backlog/known_issues.md): 既知の課題・制約事項
- [backlog.md](./raw/04_issues_and_backlog/backlog.md): バックログ・TODO一覧
