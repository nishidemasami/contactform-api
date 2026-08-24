# GitHub モノレポ・リポジトリ設定規約

## 1. モノレポ構造方針
ソースコード、インフラ定義 (IaC)、DBマイグレーション (Liquibase)、CI/CDワークフロー、自動テスト、ドキュメントをすべて単一のGitHubリポジトリで管理する。
コミットハッシュひとつで「コード・DB・インフラ構成」が同期し、再現性およびロールバック確実性を担保する。

---

## 2. ディレクトリ構造規約
- `/` : リポジトリルート
  - `.github/workflows/` : GitHub Actions 定義
  - `README.md` : プロジェクト概要
  - `AGENTS.md` : AI向け指示・ルール
  - `api/` : バックエンドAPI Lambdaサブシステム
  - `core/` : DDDビジネスロジック (domain, application, infrastructure, presentation)
  - `db/` : DBマイグレーション & SeaORM Entities
  - `docs/` : ドキュメント (`raw/` および `wiki/`)
  - `frontend/` : テスト用 Next.js SPA
  - `retained/` : 永続インフラ管理
  - `review/` : レビュー資料配信用設定

---

## 3. ブランチ保護およびSecrets規約

### ブランチ保護ルール
- `Require signed commits`: ON
- `Require a pull request before merging`: ON
- `Require status checks to pass before merging`: ON
- `Automatically request Copilot code review`: ON
- `Restrict deletions`: ON
- `Block force pushes`: ON

### Secrets 設定一覧 (`Environment secrets`)
- `AWS_DEPLOY_ROLE_ARN`: GitHub Actions AWS OIDC Role ARN
- `SAM_DEPLOY_ROLE_ARN`: CloudFormation 実行 Role ARN
- `SECRET_REFERER`: CloudFront - S3 間 Referer 認証用シークレット文字列
