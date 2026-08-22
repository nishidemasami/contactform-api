# DB規約（Liquibase・Aurora DSQL・SeaORM）

> 最終更新: 2026-08-16 | ソース: raw/02_conventions/database.md

DBマイグレーション・スキーマ管理・SeaORMエンティティの規約。

---

## 1. Liquibaseマイグレーション規約

### 1.1. contextの使い分け

| context | 対象 | 編集可否 |
|---|---|---|
| `local` | ローカル開発時（PostgreSQL）| ✅ 編集可 |
| `develop` | テスト環境（Aurora DSQL）| ✅ 編集可 |
| `release` | 本番環境（Aurora DSQL）| ❌ 編集不可 |

- `context` は必ず指定する
- `release` を追加した場合、以降はそのファイルを編集してはならない

### 1.2. ファイル命名規則

- `db/liquibase/changes/*.sql`
- ファイル名は先頭を `0001_` から辞書順で並ぶように作成する（桁数はDB複雑さによって調整）

### 1.3. Aurora DSQLの特殊仕様

| 制約 | 内容 |
|---|---|
| DDLとDMLの混在不可 | 1トランザクション内でDDLとDMLを同時実行不可 |
| 1トランザクションのDDLは1文のみ | COMMENT文でさえ別ファイルに分ける必要がある |

> そのため `*.sql` ファイルが大量になるが、仕様通りである。

---

## 2. ロール・権限設計

### ロール対応表

| ロール名 | inquiriesテーブル |
|---|---|
| `insertonly` | INSERTのみ |

- runtime roleで `SELECT`, `UPDATE`, `DELETE` が成功しないことを結合テストで確認する
- migration role と runtime role は分離する

---

## 3. SeaORMの使用方針

### 3.1. INSERTの実装制約

SeaORMの `ActiveModel::insert` は `RETURNING` 句（暗黙的なSELECT）を実行するため、INSERT専用ロール環境では**使用禁止**。

**代替手段**:
- `ConnectionTrait::execute_raw`（プレースホルダー使用必須）
- `exec_without_returning`

### 3.2. SQLインジェクション対策

- プレースホルダー（`$1`, `$2`等）を必ず使用する
- 文字列結合によるSQL組み立ては厳禁

---

## 4. ディレクトリ構成

```
db/
├── liquibase/
│   ├── changelog.xml         Liquibaseチェンジログ
│   └── changes/
│       ├── 0001_create_inquiries_table.sql
│       ├── 0002_add_comment_to_id.sql
│       └── ...
├── sea_orm_entities/         SeaORMエンティティ定義
│   ├── Cargo.toml
│   └── src/
│       └── inquiries.rs
└── template.yaml             SAMテンプレート（IAMロール・DSQLクラスター等）
```

---

## 5. CI/CDでの実行コマンド

```bash
# ローカルマイグレーション（PostgreSQL）
docker run --rm --network host \
  -v "$PWD:/workspace" \
  liquibase/liquibase:4.33.0 \
  update \
  --changelog-file=db/liquibase/changelog.xml \
  --context-filter=local \
  --url=jdbc:postgresql://localhost:5432/postgres \
  --username=postgres \
  --******

# SeaORMエンティティ検証
cargo fmt -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features -- --include-ignored
```

---

## 関連ページ

- [api.md](./api.md) — API規約（SeaORMの使用方針含む）
- [infrastructure.md](./infrastructure.md) — インフラ規約
- [cicd.md](./cicd.md) — CI/CD規約
- [aws-settings.md](./aws-settings.md) — AWS設定
