# CI/CD (GitHub Actions) 規約

## 1. CI/CDパイプライン全体方針

- モノレポ構造を採用し、すべてのサブシステム（API, DB, フロントエンド, レビュー資料）のビルド・検証・デプロイをGitHub Actionsで統一制御する。
- 開発中の全コミット Push、PRの作成・更新、ブランチへのマージをトリガーとして実行する。
- セキュリティ対策として、フォークリポからのPR実行制御 (`if: github.event.pull_request.head.repo.fork == false`) を設定し、不必要なCI/CD走行を防ぐ。
- デプロイ対象環境は AWS `ap-northeast-3`（大阪リージョン）の `develop` および `release` ステージ。

---

## 2. パイプライン分類と役割

1. **CI検証パイプライン (`verify.yaml`)**:
   - `cargo fmt -- --check`, `cargo check`, `cargo clippy`, `sam validate --lint`
   - Postgresサービスコンテナ上での Liquibase ローカルマイグレーション検証
   - 全テスト実行 (`cargo test --all-features -- --include-ignored`)

2. **DB CI/CDパイプライン (`db-cicd.yaml`)**:
   - SAMによるDB基盤構築 (`db/template.yaml`)
   - DSQL管理者トークン動的発行による Liquibase マイグレーション実行
   - `sea-orm-cli` による Entity 自動生成および GitHub Actions ボットによる自動コミット・プッシュバック

3. **API CI/CDパイプライン (`api-cicd.yaml`)**:
   - Rustコード検証および `utoipa` による `openapi.yaml` 自動同期
   - `cargo-lambda` と `sam build` / `sam deploy` による Lambda 関数の更新デプロイ

4. **レビュー資料管理 CI/CDパイプライン (`review-cicd.yaml`)**:
   - `cargo tarpaulin` カバレッジ、`cargo doc` Rust-Doc、Storybook、TypeDoc、HonKit、Stoplight/RapiDoc のビルド
   - PR別 S3サブディレクトリ (`pull_request/{PR番号}/`) への同期および GitHub Deployment Status連携

---

## 3. セキュリティ・権限管理規約

- IAM Access Key の直埋め込みは**完全禁止**。AWS認証には必ず GitHub OIDC (`aws-actions/configure-aws-credentials`) を使用する。
- ログへの機密情報流出を防ぐため、トークンや一時パスワード取得時は `::add-mask::` を徹底する。
- ワークフローレベルで最小限の `permissions` を明示指定する (`id-token: write`, `contents: read` 等)。
- `release` ステージのデプロイでは、リソースの誤削除を防ぐため CloudFormation `DeletionPolicy` / `UpdateReplacePolicy` に `Retain` を指定する。
