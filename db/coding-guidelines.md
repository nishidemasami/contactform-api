# コーディング規約(DB)

## Liquibase

### 開発ルール

- `context`は必ず指定する。
  - `local` ： ローカル開発時に使用するPostgreSQL用のSQL。
  - `develop` ： テスト環境のAmazon Aurora DSQL用のSQL。編集可
  - `release` ： 本番環境に入ったSQL。編集不可。
- `context`に`release`を追加した場合、以降編集不可。`release`入りしていないものはまだ変更される可能性があることを考慮する。

### 注意事項

- `db/liquibase/changes/*.sql`の注意
  - ファイル名は先頭を`0001_`から辞書順で並ぶように作成する。ただし数字部の桁数はDBの複雑さによって考慮する。
  - Amazon Aurora DSQLの特殊仕様
    - DDL と DML の混在不可  
      1つのトランザクション内でテーブル定義変更（DDL）とデータ操作（DML）は同時実行不可
    - 1つのトランザクションに含められる DDL は1文のみ  
      COMMENTすらsqlファイルを分ける必要がある。  
      そのため`*.sql`ファイルが大量にできる予定だが、仕様通り。

###　ロール・権限対応表

| ロール名 | inquiriesテーブル |
|---|---|
| **`insertonly`** | INSERTのみ |


## SeaORM

CI/CDによりSeaORMのDB定義のEntityを自動生成する。

以下はSeaORMによって自動生成される。

```text
db
└── sea_orm_entities
    └── src
        ├── libs.rs
        └── entities
            ├── mod.rs
            └── inquiries.rs
```

## Amazon Aurora DSQL

ローカルの開発環境ではPostgreSQL、develop環境およびrelease環境はAmazon Aurora DSQLをDBに利用する。

Amazon Aurora DSQLは接続インターフェースレベルではPostgreSQL互換だが、実装はPostgreSQLとは異なるため、以下に注意。

* 外部キー（FOREIGN KEY）が使えない : 参照整合性の担保はアプリケーション層で実装するか、非正規化して持たせる必要あり。 
* ON DELETE CASCADE が使えない : 依存データの削除は、アプリ側で複数クエリを発行するかソフトデリート（論理削除）で対応。
* SERIAL / BIGSERIAL が使えない : 自動採番は `AS IDENTITY`もしくはシーケンスオブジェクトを利用。
* JSONB / JSON 型の直接定義が不可 : カラムとしては TEXT 型で保存し、クエリ実行時に JSON/JSONB へキャストして処理。
* PL/pgSQL・トリガー・ストアドファンクションが使えない : DB側のロジック（手続き型処理）は、すべてアプリケーション層か AWS Lambda 等へ追い出す必要あり。
* 拡張機能（Extensions）はサポート外 : PostGIS、pgvector などの一般的な拡張モジュールは利用不可。
* 一時テーブル（CREATE TEMP TABLE）が使えない : 複雑な中間データ処理は、共通テーブル式（CTE）やサブクエリで代替。
* DDL と DML の混在不可 : 1つのトランザクション内でテーブル定義変更（DDL）とデータ操作（DML）は同時実行不可。また、1つのトランザクションに含められる DDL は1文のみ。
* 分離レベルは Repeatable Read 固定。
* ロック動作の変更（楽観的同時実行制御 / OCC） : SELECT ... FOR UPDATE などの構文は使用可能だが行はロックされず、競合はコミット時に検出されコミット時にシリアライズエラーとなるため、アプリ側に「エラー時のリトライ処理」の実装が必須。
* 分散型の原理上、ユニークインデックスを使用するとパフォーマンスが大幅に低下するため、性能に影響するため必要な箇所のみ利用。
* `GENERATED ALWAYS AS IDENTITY (CACHE 1)`はパフォーマンスが低下するため、大量のINSERTが発生するテーブルでは使用しない。
* `CREATE SEQUENCE ~~~ CACHE 1`はパフォーマンスが低下するため、同時に何回も参照される場合は使用しない。

# CI/CD

- **目的**: データベースの定義変更（Liquibase）、データベーススタックのインフラ配備（SAM）、およびSeaORMの実体定義（Entity）の最新同期を自動制御する。
- **トリガー**: `db/**` 配下の変更、または `.github/workflows/db-cicd.yaml` 自体の変更。
- **主な実行内容**:
  1. **検証 (validate)**:
     - SAMテンプレートのバリデーション (`sam validate --lint --template-file db/template.yaml`)
     - DB Entityプロジェクトのフォーマット、チェック、Linter検証。
     - ローカルPostgresコンテナを起動し、Liquibaseによるローカルマイグレーションがエラーなく実行できるか検証。
     - DB Entityの自動テストを実行。
  2. **デプロイ・マイグレーション (migrate)**:
     - `develop` 又は `release` へのPush時のみ動作。
     - AWS OIDCを介して、`sam deploy` を用い対象環境のDB基盤（DSQLクラスター、Lambda用IAMロール等）を更新。
     - DSQLの管理者認証トークンを動的に発行し、Docker経由でLiquibaseを実行し、対象環境のコンテキスト（`develop` 又は `release`）でマイグレーションSQLを適用。
  3. **ORM定義生成 (generate_orm)**:
     - マイグレーション完了後の最新DSQLから、`sea-orm-cli` を使用してRustのEntityコードをリバース生成。
     - 差分が存在する場合のみ、`github-actions[bot]` 名義でリポジトリに自動でコミット＆プッシュバック。


## FAQ

## なぜLiquibaseのバージョンはv4.33.0なのか？

Liquibase v5ではJDBCドライバが同梱されなくなってしまったため。  
Liquibaseのドキュメントによると`lpm`コマンドを実行することでJDBCドライバをダウンロードできるようになるとのことだが動作未確認。

- 将来的な修正対象ファイル
  - `/db/README.md`
  - `/.github/workflows/db-cicd.yaml`
