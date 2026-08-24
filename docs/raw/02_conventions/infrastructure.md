# インフラ (AWS SAM & サーバレスアーキテクチャ) 設計規約

## 1. AWS SAMによるインフラ構築のベストプラクティス

- **自動命名の利用**: 原則として Lambda FunctionName、S3 BucketName、IAM RoleName などの物理名は明示指定せず、CloudFormation/SAMによる自動命名を利用する。リソース置換時の名前衝突を避けデプロイ失敗リスクを低減する。
- **疎結合マルチスタックとSSM Parameter Store連携**:
  - スタックは `AWS::Serverless::Application` で親子関係にせず、スタック間の値の受け渡しは `AWS::SSM::Parameter` を経由する。
  - `Fn::ImportValue` によるクロススタック参照を原則として避け、スタック同士のデプロイ・削除の独立性を保つ。

### SSMパラメータ命名規則
パラメータ名は以下の階層構造で厳格に定義する：
`/{環境名}/{サービス名}/{サブシステム名}/{パラメータ名}`

- **環境名 (Stage)**: `develop` または `release`
- **サービス名**: `contactform-api-public`
- **サブシステム名**: `api`, `db`, `frontend`, `review`, `retained`
- **パラメータ名**: キャメルケース表記（例: `ApiGatewayDomain`, `DSQLEndpoint`）

### マルチスタック構成
モノレポ構成を採用し、`template.yaml` は役割ごとにサブシステム配下に分割管理する（`api/`, `db/`, `frontend/`, `retained/` 等）。

---

## 2. サーバレスアーキテクチャ方針

### 2.1. インフラ選択基準
- **採用サービス**: EC2、RDS、Lightsail、ECS、EKSなどの「OSやパッチ管理や運用保守を必要とするコンテナ・VMサービス」は**一切使用しない**。
- サーバレスサービス（API Gateway, AWS Lambda, Aurora DSQL, S3, CloudFront）のみで構成し、運用負荷とコストを最小化する。

### 2.2. 各コンポーネントの選定・設計方針
- **AWS Lambda**: **Rust (provided.al2023) × ARM64 アーキテクチャ** を採用。初期メモリ設定は128MB。
- **API Gateway**: Amazon API Gateway v2 (HTTP API) を採用。
- **CDN / 静的サイト配信 (CloudFront + S3)**:
  - 静的SPAおよびレビュー資料の配信は、原則として **CloudFront + S3 REST API origin + Origin Access Control (OAC)** または Referer 制御を標準とする。
  - S3 Bucket は Public Access Block を有効にし、インターネットからの直接 `s3:GetObject` を禁止する。
  - プロトコルは HTTPS リダイレクト (`redirect-to-https`)、HTTP/2 および HTTP/3 を有効化 (`http2and3`)。
  - 価格クラスは日本を含む `PriceClass_200` を採用。
- **Amazon Aurora DSQL**:
  - 物理的単一障害点がなく、完全マネージドなオートスケール・Scale to Zeroデータベース。
  - 分離レベルは Repeatable Read 固定。
