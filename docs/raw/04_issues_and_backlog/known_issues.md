# 既知の課題・制約事項

## 1. AWS CloudFormation における制約
- **スタック名・リソース名の文字数制限**:
  - スタック名上限は最大 128 文字。
  - CloudFormationは自動命名時に `スタック名 + 論理ID + ランダム文字列` を生成するため、IAMロール (64文字) や EventBridge (64文字) の制限を超えないよう短く簡潔なスタック名にする。
- **IAM Policyの複雑性**:
  - ポリシー記述時は `!Sub` を活用してプレーンYAML形式で視認性を確保する。
  - CloudFormation実行は専用のサービスロール (`SAM_DEPLOY_ROLE_ARN`) に集約する。
- **Aurora DSQL デプロイ時の Service-Linked Role 作成権限エラー (`iam:CreateServiceLinkedRole`)**:
  - `AWS::DSQL::Cluster` デプロイ時に `AWSServiceRoleForDSQL` が必要となるため、CloudFormation実行ロールに `iam:CreateServiceLinkedRole` 権限を明示的に付与する。

---

## 2. データベース・ツール制約
- **Liquibase v5 JDBCドライバ同梱廃止**:
  - Liquibase v5 では JDBC ドライバが同梱されなくなったため、Liquibase Package Manager (`liquibase lpm add postgresql`) または Liquibase v4.33.0 を使用。
- **SeaORM `ActiveModel::insert` のエラー**:
  - `ActiveModel::insert` は内部で `RETURNING` 句（暗黙のSELECT）を呼ぶため、INSERT専用ロール (`insertonly`) では権限エラーとなる。
  - 対策: `exec_without_returning` または `execute_raw` を使用する。
- **sqlx / aurora-dsql-sqlx-connector バージョンピン**:
  - `aurora-dsql-sqlx-connector` と `sea-orm` 間の `sqlx` バージョン齟齬を防ぐため、互換性が確認されているバージョン (例: `sea-orm` v2.0.2 / `aurora-dsql-sqlx-connector` v0.2.2 / `sqlx` v0.9.0) に固定する。

---

## 3. Rust開発・クリーンアーキテクチャ制約
- **`let` 型省略によるAIパフォーマンス低下**:
  - 型省略（型推論頼み）はコンテキスト不足を生み、AIのハルシネーションやRAG検索コスト増加を招くため、`let` 宣言にも型アノテーションを明記する。
- **Orphan Rule（孤児規則）と型変換**:
  - クレート境界での `From` / `TryFrom` 実装制限を回避するため、`impl` ブロックに独自の名前付き変換メソッド（`from_xxx` / `to_xxx`）を実装する。
