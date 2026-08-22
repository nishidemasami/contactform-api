# AWS IAM & OIDC 連携規約

## 1. OIDC（OpenID Connect）信頼関係方針

- GitHub ActionsとAWS間の認証には、永続的なIAM Access Keyを使用せず、OIDCフェデレーション (`token.actions.githubusercontent.com`) を使用する。
- 信頼関係条件 (`Condition`) において、対象リポジトリおよび対象ブランチ (`refs/heads/develop`, `refs/heads/release`, `pull_request`) を厳格に制限する。

---

## 2. CloudFormation実行権限の最小化

- **最小権限原則**: デプロイ実行ロール (`AWS_DEPLOY_ROLE_ARN` / `SAM_DEPLOY_ROLE_ARN`) には過剰な管理者権限を与えず、必要最小限のアクションおよびリソース範囲 (`/<service_name>-*`) のみを許可する。
- **IAM PassRole の制限**: `iam:PassRole` は CloudFormation サービス (`cloudformation.amazonaws.com`) に対して指定されたIAMロールのみ許可する。
- **Aurora DSQL 用 Service-Linked Role**: CloudFormation による Aurora DSQL クラスタ構築時に自動作成される Service-Linked Role に対応するため、`iam:CreateServiceLinkedRole` (対象: `dsql.amazonaws.com`) 許可を付与する。
