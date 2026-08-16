# コーディング規約(retained)

## AWS SAM

- `Parameters` に `Stage`（`develop` or `release`）と `ServiceName` を必ず定義する。
- `Conditions: IsRelease: !Equals [!Ref Stage, release]` を定義する。
- 削除保護: `DeletionPolicy: !If [IsRelease, Retain, Delete]` を設定する。
- スタック間のリソース参照は `AWS::SSM::Parameter` で連携する（`Exports` / `ImportValue` は避ける）。
- SSMパラメータ命名規則: `/${Stage}/${ServiceName}/${SubSystem}/パラメータ名`。

## CI/CD

- **目的**: 永続的インフラリソース（S3バケット）のデプロイ（SAM）を自動制御する。
- **トリガー**: `retained/**` 配下の変更、または `.github/workflows/retained-cicd.yaml` 自体の変更。
- **主な実行内容**:
  1. **検証 (validate)**:
     - SAMテンプレートのバリデーション (`sam validate --lint --template-file retained/template.yaml`)
  2. **デプロイ (deploy)**:
     - `develop` 又は `release` へのPush時のみ動作。
     - AWS OIDCを介して、`sam deploy` を用い対象環境の永続的インフラリソースを更新。
