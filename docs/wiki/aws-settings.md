# AWS設定・SSMパラメータ体系

> 最終更新: 2026-08-16 | ソース: raw/02_conventions/aws_iam.md, raw/01_requirements/system.md, raw/02_conventions/infrastructure.md

AWSのIAM設定、OIDC連携、SSMパラメータ命名規則をまとめる。

---

## 1. SSMパラメータ命名規則

```
/環境名/サービス名/サブシステム名/パラメータ名
```

| 項目 | 値 |
|---|---|
| 環境名 | `develop` または `release` |
| サービス名 | `contactform-api-public` |
| サブシステム名 | `api` / `db` / `frontend` / `review` / `retained` |

### パラメータ一覧

| 概要 | パス例 | 用途 |
|---|---|---|
| API GatewayエンドポイントURL | `/develop/contactform-api-public/api/ApiGatewayDomain` | CloudFrontのOrigin設定 |
| Aurora DSQLエンドポイント | `/develop/contactform-api-public/db/DSQLEndpoint` | Lambdaからの接続 |
| DB接続Lambda実行ロールARN | `/develop/contactform-api-public/db/DbConnectLambdaRole` | LambdaのIAMロール設定 |
| フロントエンドCloudFrontドメイン名 | `/develop/contactform-api-public/frontend/DomainName` | レビュー資料に記載 |
| フロントエンドCloudFront DistributionID | `/develop/contactform-api-public/frontend/DistributionID` | キャッシュ削除 |
| フロントエンドS3バケット名 | `/develop/contactform-api-public/retained/FrontendBucketName` | 静的ファイルのアップロード |
| フロントエンドS3 WebsiteURL | `/develop/contactform-api-public/retained/FrontendWebsiteURL` | CloudFrontのOrigin設定 |
| レビュー資料S3バケット名 | `/develop/contactform-api-public/retained/ReviewBucketName` | 静的ファイルのアップロード |
| レビュー資料S3 WebsiteURL | `/develop/contactform-api-public/retained/ReviewWebsiteURL` | CloudFrontのOrigin設定 |

---

## 2. GitHub Actions - AWS OIDC連携

### 前提条件

- AWSアカウントIDを確認
- GitHubリポジトリの Settings > Actions > OIDC > `Default subject claim prefix` 欄を確認
- SAMテンプレート配置用S3バケットを作成

### IAMロール（デプロイ用）の信頼関係

```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {
      "Federated": "arn:aws:iam::<AWS_ACCOUNT_ID>:oidc-provider/token.actions.githubusercontent.com"
    },
    "Action": "sts:AssumeRoleWithWebIdentity",
    "Condition": {
      "StringEquals": {
        "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
        "token.actions.githubusercontent.com:sub": [
          "repo:<OIDCサブクレームプレフィックス>:ref:refs/heads/develop",
          "repo:<OIDCサブクレームプレフィックス>:ref:refs/heads/release",
          "repo:<OIDCサブクレームプレフィックス>:pull_request"
        ]
      }
    }
  }]
}
```

### IAMポリシー方針（最小権限）

- 必要な権限のみを付与する（IAM Access Analyzerでのログ分析を推奨）
- リソース名指定（`/<service_name>-*` 等）や Tag 条件指定を活用
- release環境のリソースにはDelete権限を与えない
- **Aurora DSQL デプロイ用権限**: CloudFormation 実行ロール (`SAM_DEPLOY_ROLE_ARN`) には、Aurora DSQL サービスリンクロールを作成するため `iam:CreateServiceLinkedRole` (対象: `dsql.amazonaws.com`) の権限が必要です。

### GitHub Actionsでの利用方法

```yaml
- name: AWSクレデンシャルの設定
  uses: aws-actions/configure-aws-credentials@v6
  with:
    role-to-assume: ${{ secrets.AWS_DEPLOY_ROLE_ARN }}
    aws-region: ap-northeast-3
```

---

## 3. 推奨リージョン

- `ap-northeast-3`（大阪）

---

## 関連ページ

- [infrastructure.md](./infrastructure.md) — インフラ規約（SAM・CloudFront）
- [github-settings.md](./github-settings.md) — GitHub設定・Secrets
- [database.md](./database.md) — DB規約（Aurora DSQL）
