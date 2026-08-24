# データベース 運用・設計規約

## 1. Liquibase 開発ルール

### 1.1. Context の指定
- `context` は必ず指定して変更セットを作成する。
  - `local`: ローカル開発時に使用するPostgreSQL用のSQL。
  - `develop`: テスト環境のAmazon Aurora DSQL用のSQL。編集可。
  - `release`: 本番環境に入ったSQL。編集不可。
- `release` に追加された変更セットは以降変更不可。

### 1.2. Aurora DSQLの特殊仕様・注意事項
- **DDL と DML の混在不可**: 1つのトランザクション内でテーブル定義変更（DDL）とデータ操作（DML）は同時実行不可。
- **1つのトランザクションに含められる DDL は1文のみ**: COMMENTすら別SQLファイルに分ける必要がある。
- **ファイル命名規則**: `db/liquibase/changes/*.sql` は先頭を `0001_` から辞書順で並ぶように作成する。

---

## 2. データベース ロール・権限管理

| ロール名 | `inquiries` テーブル権限 | 用途 |
|---|---|---|
| **`insertonly`** | **INSERT のみ** (`GRANT INSERT ON public.inquiries TO insertonly;`) | 一般ユーザー向け問い合わせ送信Lambda実行ロール |
| **`admin`** | 全権限 | DBマイグレーションおよび管理者直接アクセス用 |

- 問い合わせ送信API用Lambdaロールには `SELECT`, `UPDATE`, `DELETE` 権限を一切与えない。

---

## 3. Amazon Aurora DSQL 固有の制約事項

PostgreSQL互換であるが、分散アーキテクチャのため以下の制約がある：
- 外部キー (FOREIGN KEY) が使用不可。参照整合性はアプリ層で担保。
- `ON DELETE CASCADE` が使用不可。
- `SERIAL` / `BIGSERIAL` が使用不可。自動採番は `AS IDENTITY` やシーケンスを利用。
- `JSONB` / `JSON` 型の直接定義不可。`TEXT` 型で保存しクエリ時キャスト。
- ストアドファンクション、トリガー、PL/pgSQL、一時テーブル (`CREATE TEMP TABLE`) はサポート外。
- 分離レベルは Repeatable Read 固定。競合はコミット時に検出されるため、アプリ側でリトライ処理が必要。
