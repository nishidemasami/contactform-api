# CI/CD規約（GitHub Actions）

> 最終更新: 2026-08-16 | ソース: raw/02_conventions/cicd.md, raw/02_conventions/database.md, raw/02_conventions/frontend.md

GitHub ActionsによるCI/CDパイプラインの規約。

---

## 1. パイプライン構成概要

各サブシステムは独立したワークフローファイルを持つ。

```
.github/workflows/
├── api-cicd.yaml        APIサブシステム
├── db-cicd.yaml         DBサブシステム
├── frontend-cicd.yaml   フロントエンドサブシステム
└── review-cicd.yaml     レビュー資料サブシステム
```

### トリガー

| ブランチ | イベント | 動作 |
|---|---|---|
| `develop`, `release` | `push` | CI + CD（デプロイ） |
| 任意 | `pull_request` | CIのみ（検証） |

---

## 2. 基本構成（jobsの流れ）

### APIサブシステム

```
validate → deploy（pushのみ）
```

### DBサブシステム

```
validate → migrate（pushのみ）
```

### フロントエンドサブシステム

```
validate → build → deploy（pushのみ）
```

---

## 3. validateジョブ

各サブシステムで共通の検証を行う。

### Rust（API/DB）

```bash
sam validate --lint                                          # SAMテンプレート検証
cargo fmt -- --check                                        # フォーマット
cargo check --workspace --all-targets --all-features        # コンパイルチェック
cargo clippy --workspace --all-targets --all-features -- -D warnings  # 静的解析
cargo test --workspace --all-features -- --include-ignored  # テスト（ignore含む）
```

### フロントエンド（Next.js）

```bash
sam validate --lint                # SAMテンプレート検証
npx tsc --noEmit                   # コンパイルチェック
npm run lint                       # 静的解析
npm test                           # ユニットテスト
npm audit                          # 脆弱性診断
```

---

## 4. デプロイジョブ

### 共通設定

```yaml
permissions:
  id-token: write
  contents: read
```

- AWSクレデンシャルは `aws-actions/configure-aws-credentials` で設定（OIDC）
- `sam deploy` で各スタックをデプロイ

### ステージ判定

```yaml
if [ "${{ github.ref }}" = "refs/heads/develop" ]; then
  echo "stage=develop" >> "$GITHUB_OUTPUT"
else
  echo "stage=release" >> "$GITHUB_OUTPUT"
fi
```

---

## 5. DBマイグレーションジョブ

```bash
# DSQLエンドポイント取得
DSQL_ENDPOINT=$(aws ssm get-parameter --name "/${STAGE}/${SERVICE_NAME}/db/DSQLEndpoint" ...)

# 認証トークン取得
TOKEN=$(aws dsql generate-db-connect-admin-auth-token --hostname "$DSQL_ENDPOINT" ...)

# Liquibaseマイグレーション
docker run liquibase/liquibase:4.33.0 update ...
```

---

## 6. セキュリティ考慮事項

- ビルドjobの権限は最小限（`contents: read` 等）
- 値の受け渡しは `jobs.<job_id>.outputs` を使用
- ビルド成果物の受け渡しは `actions/upload-artifact` / `actions/download-artifact` を使用
- フロントエンドのビルドは `npm audit` による脆弱性診断を含む

---

## 7. 並列実行制御

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

---

## 関連ページ

- [review-process.md](./review-process.md) — レビュープロセス・PR自動化
- [infrastructure.md](./infrastructure.md) — インフラ規約
- [database.md](./database.md) — DB規約
- [github-settings.md](./github-settings.md) — GitHub設定
