# ドキュメント (docs)

本ディレクトリ (`docs/`) は、`contactform-api` プロジェクトに関するすべてのドキュメントを管理します。

## ディレクトリ構成と概要

`docs/` ディレクトリは以下の2つのサブディレクトリで構成されています。

- **`raw/` (生情報)**: 人間が投入する一次情報（規約原文、要件定義、タスク管理など）。イミュータブル（読み取り専用）として扱います。
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

`./raw/` ディレクトリ配下に格納されている生情報ファイルの一覧と要約です。

- [00000001_コーディング規約と実装例.md](./raw/00000001_コーディング規約と実装例.md)
  - **概要**: Rustの実装におけるコーディング規約、オニオン/クリーンアーキテクチャおよびDDD実装方針、型明記ルールなどを定めたドキュメント。
- [00000002_API規約と実装例.md](./raw/00000002_API規約と実装例.md)
  - **概要**: バックエンドAPI（AWS Lambda）の設計・実装規約。コールドスタート時間の極小化や128MBメモリ制限下での最適化設計について規定。
- [00000003_インフラ規約と実装例.md](./raw/00000003_インフラ規約と実装例.md)
  - **概要**: AWS SAMを用いたインフラ構築、サーバレスアーキテクチャ設計、CloudFormationリソース命名規則、IaCのベストプラクティスを定義。
- [00000004_レビュー規約.md](./raw/00000004_レビュー規約.md)
  - **概要**: プルリクエスト（PR）運用、自動テスト・カバレッジ計測、レビュー用プレビュー環境（S3静的配信）のデプロイプロセスに関する規約。
- [00000005_GitHub設定.md](./raw/00000005_GitHub設定.md)
  - **概要**: モノレポ構成の目的・構成案、AIによるスムーズなシステム開発と人間の開発参入障壁の最小化を目指すリポジトリ設定規約。
- [00000006_DB規約と実装例.md](./raw/00000006_DB規約と実装例.md)
  - **概要**: LiquibaseによるDBマイグレーション管理、Amazon Aurora DSQLおよびローカルPostgreSQLの環境別設定、SeaORM利用規約。
- [00000007_CICD規約と実装例.md](./raw/00000007_CICD規約と実装例.md)
  - **概要**: GitHub Actionsを用いたCI/CDパイプラインの全体設計、各種ワークフロー、自動テスト・品質検証・自動デプロイ規約。
- [00000008_AWS規約と実装例.md](./raw/00000008_AWS規約と実装例.md)
  - **概要**: AWS IAM設定、GitHub ActionsとのOIDC連携、SSMパラメータストアの命名規則・パラメータ体系を定めたドキュメント。
- [00000009_フロントエンド規約と実装例.md](./raw/00000009_フロントエンド規約と実装例.md)
  - **概要**: テスト用の静的SPAサイト（Next.js / TypeScript）の実装規約およびデプロイ方針（develop環境限定）。
- [00000010_既知の問題.md](./raw/00000010_既知の問題.md)
  - **概要**: AWS CloudFormationのスタック名文字数制限（最大128文字）や命名ルール等、システム構築における既知の制約事項。
- [20260816_要件.md](./raw/20260816_要件.md)
  - **概要**: コンタクトフォームからのPOSTを受け付ける問い合わせAPIの要件定義、およびテスト用静的SPAサイトの要件。
- [BACKLOG.md](./raw/BACKLOG.md)
  - **概要**: 考慮不足・未解決問題・外部パッケージ修正待ち等のTODOおよび課題を追跡するためのバックログ。
