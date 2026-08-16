# GitHub設定

> 最終更新: 2026-08-16 | ソース: raw/00000005_GitHub設定.md

GitHubリポジトリの設定・構成規約。

---

## 1. リポジトリ方針

- **モノレポ構成**: CI/CD・IaC・ドキュメント・プログラム・バックログをすべてテキストベースで1リポジトリに格納
- AIによるスムーズな開発と、人間の新規参画時の参入障壁最小化を目的とする

---

## 2. リポジトリ構成

| パス | 概要 |
|---|---|
| `/.github/workflows/` | CI/CDのGitHub Actions定義 |
| `/README.md` | リポジトリ概要 |
| `/AGENTS.md` | AI向けプロンプト（README.md参照を明記）|
| `/BACKLOG.md` | バックログ・懸念事項・アイデア |
| `/api/` | APIサブシステム |
| `/core/` | ビジネスロジック（shared library）|
| `/db/` | DBサブシステム（Liquibase + SAM）|
| `/docs/` | ドキュメント（raw + wiki）|
| `/frontend/` | フロントエンドサブシステム |
| `/retained/` | 永続的インフラリソース管理 |
| `/review/` | レビュー資料デプロイ用 |

---

## 3. ブランチ保護ルール

| 設定 | 値 | 説明 |
|---|---|---|
| Require signed commits | ON | 署名付きコミットのみマージ可 |
| Require a pull request before merging | ON | 直接push禁止、PR必須 |
| Require status checks to pass | ON | CI/CDのvalidateをパスが必須 |
| Require branches to be up to date | ON | 最新mainからの変更であること |
| Automatically request Copilot code review | ON | AI（GitHub Copilot）による自動レビュー |
| Restrict deletions | ON | ブランチ削除禁止 |
| Block force pushes | ON | 強制pushを禁止 |

**Publicリポジトリの場合**:
- `Require review from Code Owners`: ON（コードオーナーのレビュー必須）
- `Required reviewers`（Environment secrets の Deployment protection rules）: ON

---

## 4. GitHub Actions Secrets

`Environment secrets` として設定する。

| 設定名 | 説明 |
|---|---|
| `AWS_DEPLOY_ROLE_ARN` | OIDC認証用IAMロールのARN（`configure-aws-credentials`で使用）|
| `SAM_DEPLOY_ROLE_ARN` | `sam deploy --role-arn` で指定するCloudFormation実行ロールのARN |
| `SECRET_REFERER` | CloudFront↔S3間のRefererヘッダーに使用するランダム文字列 |

---

## 5. 必須ステータスチェック

各サブシステムのワークフローの `validate` ジョブをステータスチェックとして設定する。
形式: `<workflow name> / <job name>`

---

## 関連ページ

- [aws-settings.md](./aws-settings.md) — AWS設定・OIDC設定
- [cicd.md](./cicd.md) — CI/CD規約
- [review-process.md](./review-process.md) — レビュープロセス
