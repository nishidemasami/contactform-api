# インフラ規約（AWS SAM・サーバレスアーキテクチャ）

> 最終更新: 2026-08-16 | ソース: raw/00000003_インフラ規約と実装例.md

AWS SAMを用いたインフラ構築のベストプラクティスおよびサーバレスアーキテクチャ設計規約。

---

## 1. AWS SAMベストプラクティス

### 1.1. 物理名の自動命名

- Lambda FunctionName、S3 BucketName、IAM RoleName 等の物理名は**明示指定せず**、CloudFormation/SAMによる自動命名を使用する
- 例外: `DeletionPolicy: Retain` リソースのみ、運用上の追跡しやすさのために物理名を指定することがある

### 1.2. スタックの疎結合化（SSM Parameter Store活用）

- スタック間のリソース参照は `AWS::SSM::Parameter` で連携する（`Exports` / `ImportValue` は避ける）
- これにより、スタックを独立してデプロイ・削除できる

### 1.3. SSMパラメータ命名規則

```
/環境名/サービス名/サブシステム名/パラメータ名
```

詳細は [aws-settings.md](./aws-settings.md) を参照。

### 1.4. マルチスタック（サブシステム別）構成

| ディレクトリ | スタック名例 | 概要 |
|---|---|---|
| `api/` | `contactform-api-public-api-develop` | API Gateway + Lambda |
| `db/` | `contactform-api-public-db-develop` | Aurora DSQL + IAMロール |
| `frontend/` | `contactform-api-public-frontend-develop` | CloudFront + S3（developのみ） |
| `review/` | `contactform-api-public-review-develop` | レビュー資料S3（developのみ） |

### 1.5. 全template.yaml共通仕様

- `Parameters` に `Stage`（`develop` or `release`）と `ServiceName` を必ず定義する
- `Conditions: IsRelease: !Equals [!Ref Stage, release]` を定義する
- releaseデプロイ不要リソースには `Condition: !Not [IsRelease]` を付与する
- 削除保護: `DeletionPolicy: !If [IsRelease, Retain, Delete]` を設定する

---

## 2. サーバレスアーキテクチャ構成

### 2.1. インフラ選択基準

| レイヤー | 選定サービス | 理由 |
|---|---|---|
| CDN | Amazon CloudFront | グローバル配信・HTTPSリダイレクト・パスルーティング |
| API | Amazon API Gateway HTTP API + AWS Lambda | サーバレス・低コスト |
| DB | Amazon Aurora DSQL | PostgreSQL互換・サーバレス・ゼロ管理 |
| 静的ホスティング | Amazon S3 | 低コスト・CDN連携 |
| IaC | AWS SAM | Lambda/API Gateway特化・CloudFormation拡張 |

### 2.2. Lambdaメモリ設定とCPU性能

| メモリ | CPU相当 | 推奨 tokio設定 |
|---|---|---|
| 128MB | 約0.08コア | `current_thread`（シングルスレッド）|
| 512MB以上 | 約0.32コア〜 | マルチスレッド検討可 |

詳細は [api.md](./api.md) を参照。

### 2.3. CloudFront パスルーティング

| パスパターン | オリジン | 用途 |
|---|---|---|
| `/api/*` | API Gateway | APIリクエスト |
| `/review/*` | S3（レビュー資料）| レビュー資料（developのみ）|
| `/*`（デフォルト）| S3（フロントエンドSPA）| SPAコンテンツ（developのみ）|

### 2.4. CloudFrontキャッシュポリシー

| パス | CachePolicyId | ポリシー名 |
|---|---|---|
| `/api/*` | `4135ea2d-6df8-44a3-9df3-4b5a84be39ad` | CachingDisabled |
| それ以外 | `658327ea-f89d-4fab-a63d-7e88639e58f6` | CachingOptimized |

### 2.5. CloudFrontとS3のセキュリティ構成

- **推奨**: CloudFront + S3 REST API origin + OAC（Origin Access Control）
- **簡易構成（developのみ）**: S3 Website Endpoint + Referer制御
  - Refererヘッダーによる直接アクセス抑止（完全なセキュリティは保証しない）

---

## 3. IaCベストプラクティス

### 3.1. モノレポ一元管理

- CI/CD・IaC・ドキュメント・プログラム・バックログをすべてテキストベースで1リポジトリに格納
- AIによるスムーズな開発と、人間の新規参画時の参入障壁最小化を目的とする

### 3.2. CI/CDパイプライン構成概要

詳細は [cicd.md](./cicd.md) を参照。

---

## 関連ページ

- [aws-settings.md](./aws-settings.md) — AWS設定・SSMパラメータ詳細
- [api.md](./api.md) — API規約
- [database.md](./database.md) — DB規約
- [cicd.md](./cicd.md) — CI/CD規約
- [frontend.md](./frontend.md) — フロントエンド規約
