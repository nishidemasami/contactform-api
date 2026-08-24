# raw ディレクトリ概要 (生情報)

本ディレクトリ (`docs/raw/`) には、システム要件・各種規約・実装例・既知の課題およびバックログの一次情報が整理されて格納されています。
AIエージェントおよび人間が開発タスクを行う際、関連する領域の情報を最小限のトークン消費で参照できるよう、以下のカテゴリおよびファイル構成で分割・整理されています。

## ディレクトリ・ファイル構成

### 1. 要件 (`01_requirements/`)
システム全体の要件定義、個別機能要件、開発フローなどを定義します。
- `system.md` : 全体システム要件、サービス名 (`contactform-api-public`)、開発フロー、SSMパラメータ設定方針
- `api.md` : 問い合わせAPI機能要件（エンドポイント仕様、リクエスト/レスポンス要件）
- `frontend_and_review.md` : テスト用静的SPAサイトおよびレビュー資料の機能要件

### 2. 規約 (`02_conventions/`)
開発・運用時に遵守すべき各種設計原則・コーディング規約・セキュリティ方針を定義します。
- `coding.md` : Rust言語コーディング規約（型アノテーション明記ルール、DDDレイヤー構成・型変換方針）
- `api.md` : バックエンドAPI (AWS Lambda) 設計規約、データベース権限最小化 (INSERT専用ロール)、コールドスタート対策・シングルスレッド最適化原則
- `infrastructure.md` : AWS SAM / サーバレスアーキテクチャ設計規約（自動命名、疎結合スタック、SSMパラメータ連携）
- `database.md` : DBマイグレーション（Liquibase）運用規約、Amazon Aurora DSQL / ローカルPostgreSQL運用規約、ロール権限表
- `cicd.md` : CI/CD (GitHub Actions) パイプライン設計規約、セキュリティ・権限管理方針
- `aws_iam.md` : GitHub Actions-AWS間 OIDC連携規約、CloudFormation用IAMロール権限最小化方針
- `frontend.md` : フロントエンド (Next.js/TypeScript) 実装・セキュリティ規約、サプライチェーン攻撃対策
- `review_process.md` : プルリクエスト (PR) 運用・自動レビュー資料作成・プレビュー環境自動デプロイ規約
- `github.md` : モノレポ構造方針、リポジトリ構成、ブランチ保護ルール、Secrets設定規約

### 3. 実装例 (`03_examples/`)
各実装タスクで参照可能な標準実装コード・テンプレート・構成定義です。
- `rust_ddd.md` : RustにおけるDDDレイヤー別（domain, application, infrastructure, presentation）実装例、`Cargo.toml` 設定例
- `lambda_handler.md` : AWS Lambdaエントリーポイント (`main.rs`) 実装例、`OnceCell` 接続保持・ハンドラー実装例
- `sam_templates.md` : 各サブシステム (`api`, `db`, `frontend`, `retained`) の `template.yaml` 実装例
- `liquibase_and_seaorm.md` : Liquibase `changelog.xml` / `changes/*.sql` および SeaORM Entity実装例
- `github_actions_workflows.md` : GitHub Actions ワークフロー YAML (Review, DB CI/CD) 実装例
- `aws_oidc_policy.md` : AWS IAM OIDC 信頼関係ポリシーおよび許可ポリシー JSON実装例

### 4. 既知の課題・バックログ (`04_issues_and_backlog/`)
システム開発上把握されている制約事項や、今後のTODO・懸念事項を管理します。
- `known_issues.md` : AWS CloudFormationスタック名長制限、IAMポリシー難易度、Liquibase v5 JDBC変更、Rust型省略/Orphan Rule問題、sqlx/ActiveModel制約
- `backlog.md` : プロジェクトのバックログ・検討事項・TODO一覧
