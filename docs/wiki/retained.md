# retained サブシステム

> 最終更新: 2026-08-16 | ソース: raw/00000009_フロントエンド規約と実装例.md, raw/00000003_インフラ規約と実装例.md

フロントエンドSPA・レビュー資料配信用の永続的インフラリソース（S3バケット）を管理するサブシステム。

---

## 1. 概要

| 項目 | 内容 |
|---|---|
| ディレクトリ | `retained/` |
| スタック名例 | `contactform-api-public-retained-develop` |
| デプロイ環境 | `develop` / `release` |
| 主なリソース | S3バケット×2（フロントエンドSPA用・レビュー資料用）|

`retained` サブシステムは、`frontend` および `review` サブシステムが参照するS3バケットを提供する。
バケット名・WebsiteURLはSSM Parameter Storeに登録し、他スタックから参照する。

---

## 2. SSMパラメータ

| パラメータパス | 概要 |
|---|---|
| `/${Stage}/contactform-api-public/retained/FrontendBucketName` | フロントエンドSPA用S3バケット名 |
| `/${Stage}/contactform-api-public/retained/FrontendWebsiteURL` | フロントエンドSPA用S3 WebsiteURL（ドメイン部分）|
| `/${Stage}/contactform-api-public/retained/ReviewBucketName` | レビュー資料用S3バケット名 |
| `/${Stage}/contactform-api-public/retained/ReviewWebsiteURL` | レビュー資料用S3 WebsiteURL（ドメイン部分）|

---

## 3. インフラ構成

### 3.1. S3バケット仕様

| バケット | 用途 | WebsiteConfiguration |
|---|---|---|
| `FrontendBucket` | フロントエンドSPA配信 | IndexDocument: `index.html`, ErrorDocument: `index.html` |
| `ReviewBucket` | レビュー資料配信 | IndexDocument: `index.html`, ErrorDocument: `error.html` |

- 両バケットともに `PublicAccessBlock` はバケットポリシーによる公開に必要な項目のみ部分解除
- CloudFrontからのアクセスはRefererヘッダーにより制御（`SECRET_REFERER` シークレットを使用）
- `DeletionPolicy: !If [IsRelease, Retain, Delete]` により、releaseスタックではバケットの誤削除を防止

### 3.2. 注意事項

- 本構成はWebsiteEndpointをCustom Originとして使う簡易構成（Referer制御による直接アクセス抑止）
- 本番運用では CloudFront + S3 REST API origin + OAC を推奨する

---

## 4. CI/CD

- **ワークフロー**: `.github/workflows/retained-cicd.yaml`
- **トリガー**: `retained/**` または `.github/workflows/retained-cicd.yaml` の変更

### ジョブ構成

```
validate → deploy（pushのみ）
```

| ジョブ | 概要 |
|---|---|
| `validate` | `sam validate --lint` でSAMテンプレートを検証 |
| `deploy` | `develop` / `release` への Push 時のみ `sam deploy` を実行 |

---

## 関連ページ

- [aws-settings.md](./aws-settings.md) — AWS設定・SSMパラメータ詳細
- [infrastructure.md](./infrastructure.md) — インフラ規約（SAM・CloudFront）
- [frontend.md](./frontend.md) — フロントエンド規約
- [cicd.md](./cicd.md) — CI/CD規約
