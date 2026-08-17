# 既知の問題

> 最終更新: 2026-08-16 | ソース: raw/00000010_既知の問題.md

現時点で把握している既知の問題・制約事項をまとめる。

---

## 1. AWS CloudFormationスタック名の制限

### 問題

- CloudFormationのスタック名の最大文字数は**128文字**
- 使用可能な文字: 英数字（大文字小文字区別）とハイフン（`-`）のみ
- 先頭文字は必ずアルファベット

### 影響

CloudFormationがリソース物理名を自動生成する場合、`スタック名 + 論理ID + ランダム文字列` で生成される。
AWSリソースには物理名の文字数制限が厳しいものがあるため、**スタック名は実際にはかなり短くする必要がある**。

**例**: IAMロール名の上限は64文字。スタック名が長すぎると自動生成ロール名が上限を超えてデプロイ失敗する。

### 対策

- スタック名は短く設計する（`<service>-<subsystem>-<stage>` 形式で30文字以内を目安）
- デプロイ前に自動生成名の文字数を事前確認する

---

## 2. Aurora DSQL デプロイ時の Service-Linked Role 作成権限エラー

### 問題

CloudFormation で `AWS::DSQL::Cluster` リソースをデプロイする際、Amazon Aurora DSQL 用の Service-Linked Role（`AWSServiceRoleForDSQL`）の自動作成が必要です。
CloudFormation 実行ロール（`SAM_DEPLOY_ROLE_ARN`）に `iam:CreateServiceLinkedRole` の権限が付与されていない場合、以下のエラーが発生してデプロイが失敗（`CREATE_FAILED`）します。

```text
CREATE_FAILED AWS::DSQL::Cluster DSQLCluster Resource handler returned message:
"Insufficient permissions to create service-linked role. Add the iam:CreateServiceLinkedRole permission to your IAM policy. (Service: Dsql, Status Code: 403...)"
```

### 対策

CloudFormation 実行用 IAM ロール（`SAM_DEPLOY_ROLE_ARN`）の IAM ポリシーに `iam:CreateServiceLinkedRole`（対象サービス: `dsql.amazonaws.com`）の許可ステートメントを追加します。

```json
{
    "Sid": "AllowCreateSLRForDSQL",
    "Effect": "Allow",
    "Action": "iam:CreateServiceLinkedRole",
    "Resource": "arn:aws:iam::*:role/aws-service-role/dsql.amazonaws.com/AWSServiceRoleForDSQL*",
    "Condition": {
        "StringEquals": {
            "iam:AWSServiceName": "dsql.amazonaws.com"
        }
    }
}
```

---

## 関連ページ

- [infrastructure.md](./infrastructure.md) — インフラ規約（SAM・スタック設計）
- [backlog.md](./backlog.md) — バックログ・未解決問題
