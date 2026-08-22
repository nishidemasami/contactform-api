# バックエンドAPI 設計・最適化規約

## 1. 開発基本方針

### 1.1. クリーンアーキテクチャとDDDの適用
ビジネスロジックとインフラストラクチャを厳密に分離するため、クリーンアーキテクチャおよびドメイン駆動設計（DDD）の設計思想を適用する。

### 1.2. データベース権限の最小化とSQLインジェクション対策
- 一般ユーザー向けの問い合わせ送信APIなどで使用するLambda実行ロールには、データベース（Amazon Aurora DSQL）の `inquiries` テーブルに対する **`INSERT` 権限のみ** を付与し、`SELECT` や `UPDATE`、`DELETE` 権限は一切付与しない。
- **実装上の制約**: SeaORMのデフォルトの `ActiveModel::insert` は自動生成キーやデフォルト値を取得するために `RETURNING` 句（暗黙的なSELECTを伴う）を実行するため、権限不足でエラーとなる。データ追加時には、プレースホルダーを用いた生SQLステートメント（`ConnectionTrait::execute_raw`）やRETURNINGを伴わないINSERT (`exec_without_returning`) を使用してINSERTを実行しなければならない。
- **セキュリティ対策**: SQLインジェクション攻撃を防止するため、文字列結合によるSQLの組み立ては厳禁とし、必ずプレースホルダー（`$1`, `$2`等）を用いてパラメータをバインドすること。
- **権限検証**: 実行時ロールには接続・スキーマ参照に必要な最小限の権限のみ付与し、runtime roleで `SELECT`, `UPDATE`, `DELETE` が失敗することを結合テストで確認する。

### 1.3. Fail-Fast原則
起動時に必要な環境変数（`DSQL_ENDPOINT`、`AWS_REGION`）が設定されていない場合、またはデータベース接続の初期化に失敗した場合は、その時点で即座に `panic!` させてプロセスを終了（Fail-Fast）させなければならない。

---

## 2. AWS Lambda最適化設計（Cold Start対策と接続共有）

### 2.1. INITフェーズ（Cold Start初期化）の有効活用
- INITフェーズの追加CPUブースト期間中に、重い初期化処理を完了させることでINVOKEフェーズの応答レイテンシを削減する。
- **INITフェーズ内で実行すべき処理**:
  1. ログ・トレーシング（`tracing_subscriber`）の初期化
  2. 環境変数（`DSQL_ENDPOINT`, `AWS_REGION`等）の読み込みとパース
  3. データベース（Amazon Aurora DSQL）のコネクションプールの作成と接続確立
- INITフェーズが10秒を超えると初期化失敗となるため、10秒以内で完結するよう実装する。

### 2.2. INVOKEフェーズのシングルスレッド最適化
割り当てメモリ128MB等の極小CPUリソース環境下でのスレッド切り替え・排他制御オーバーヘッドを防ぐため、`tokio` 非同期ランタイムのマクロ設定で明示的にシングルスレッド（`current_thread`）を指定する。
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() { ... }
```

### 2.3. 静的ライフサイクル（OnceCell）によるコネクション・環境変数の保持
Cold Start時に生成した環境変数およびデータベース接続プール（`DatabaseConnection`）は、複数のリクエスト（Hot Start）にわたって安全に使い回せるよう、`tokio::sync::OnceCell` を用いた静的ライフサイクル変数として保持する。
