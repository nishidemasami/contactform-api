# レビュープロセス・PR運用規約

> 最終更新: 2026-08-16 | ソース: raw/00000004_レビュー規約.md

プルリクエスト（PR）作成・自動テスト・カバレッジ計測・レビュー資料の自動生成・デプロイまでのプロセス規約。

---

## 1. 基本方針

- すべての機能変更・リファクタリング・バグ修正は**プルリクエスト経由**でレビュー・マージする
- PRを起点として、CI/CDパイプラインが自動的にレビュー資料を生成・デプロイする
- レビュアーはURLから資料を参照して判断する

---

## 2. レビュー資料として自動生成されるコンテンツ

| コンテンツ | ツール | パス |
|---|---|---|
| エンジニアリング仕様書 | HonKit | `/index.html` |
| テスト・カバレッジレポート | cargo-tarpaulin | `/coverage/tarpaulin-report.html` |
| バックエンドAPIドキュメント | cargo doc | `/rust-doc/` |
| インタラクティブAPIドキュメント（Stoplight Elements）| OpenAPI → HTML | `/stoplight/index.html` |
| インタラクティブAPIドキュメント（RapiDoc）| OpenAPI → HTML | `/rapidoc/index.html` |
| フロントエンド検証環境 | Storybook | `/storybook/` |
| フロントエンドAPIドキュメント | TypeDoc | `/typedoc/` |

---

## 3. CIパイプライン概要

### 3.1. トリガー

- `pull_request` イベントで自動起動
- 変更ファイルパスによりjobが分岐（`api/**`, `core/**`, `db/**`, `frontend/**`, `docs/**` 等）

### 3.2. 主なjobs

```
validate
  ├── SAMテンプレート検証（sam validate --lint）
  ├── cargo fmt / clippy / check / test（Rust）
  ├── TypeScript型チェック / lint / test / audit（フロントエンド）
  └── カバレッジ計測（cargo-tarpaulin）

build
  ├── HonKit（docs → HTML）
  ├── cargo doc（Rust APIリファレンス）
  ├── Storybook（フロントエンドコンポーネント）
  └── TypeDoc（フロントエンドAPI）

deploy
  ├── S3へのアップロード（aws s3 sync）
  └── CloudFrontキャッシュ削除（CloudFront invalidation）
```

### 3.3. テスト環境

- GitHub Actionsのサービスコンテナ機能でPostgreSQL（`postgres:16-alpine`）を立ち上げ
- インメモリDB相当（ジョブ終了時に自動クリーンアップ）

---

## 4. ブランチ保護ルール

詳細は [github-settings.md](./github-settings.md) を参照。

- `Require a pull request before merging` ON（直接pushを禁止）
- `Require status checks to pass before merging` ON（CI必須）
- `Automatically request Copilot code review` ON（AI自動レビュー）

---

## 5. レビュー承認フロー

```
[AI] 実装 → プルリクエスト作成
    ↓
[CI/CD] 自動テスト・カバレッジ・レビュー資料の生成・デプロイ
    ↓
[人間] レビュー資料URL + コード + テスト結果を確認
    ↓
[人間] 承認 → マージ
    ↓
[CI/CD] 各環境（develop/release）へデプロイ
```

---

## 関連ページ

- [cicd.md](./cicd.md) — CI/CD規約詳細
- [github-settings.md](./github-settings.md) — GitHub設定
- [overview.md](./overview.md) — 開発フロー全体
