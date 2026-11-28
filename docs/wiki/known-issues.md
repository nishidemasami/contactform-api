# 既知の問題

> 最終更新: 2026-11-27 | ソース: raw/00000010_既知の問題.md, raw/00000011_形式手法による検証.md

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

## 3. 形式手法検証（Verus / Kani）におけるツールチェーン制約と状態爆発

### 問題

1. **Verus のツールチェーン・`vstd` 制約**
   - `cargo verus` でクレート全体を検証する場合、`Cargo.toml` への `vstd` 追加と Verus 指定の特定 Rust バージョン（例：`1.100.0`）が必須となるが、既存 CI の Rust バージョン（`1.97.1`）や `no_std`（`x86_64-unknown-none`）・`--all-features` ビルドと衝突する。
2. **Kani（CBMC）における標準ライブラリ最適化起因の状態爆発**
   - シンボリック入力に対して `core::str::from_utf8`・`str::trim()`・`str::chars().count()`・`chrono::Utc.timestamp_opt` を呼ぶと、標準ライブラリ内部のチャンク処理・双方向探索・SWAR ビット演算・64ビット除算により SAT ソルバがハングする。

### 対策

- 純粋な検証ロジックは外部クレート・`alloc` 非依存のコアモジュール（`core/domain/src/inquiry/validation.rs`）に分離し、`Cargo.toml` に `vstd` を追加せず `verus --crate-type=lib --no-cheating` で直接検証する。
- Kani ハーネスでは `kani::any::<char>()` と `encode_utf8`、`NaiveTime::from_num_seconds_from_midnight_opt`、`Uuid::as_u128()` を用いて不要な状態空間爆発を回避する（詳細は [coding-conventions.md](./coding-conventions.md) を参照）。

---

## 関連ページ

- [coding-conventions.md](./coding-conventions.md) — コーディング規約・形式手法ノウハウ
- [infrastructure.md](./infrastructure.md) — インフラ規約（SAM・スタック設計）
- [backlog.md](./backlog.md) — バックログ・未解決問題
