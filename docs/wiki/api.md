# API規約

> 最終更新: 2026-08-16 | ソース: raw/00000002_API規約と実装例.md

バックエンドAPI（AWS Lambda）の設計および実装に関する規約。

---

## 1. 基本方針

- クリーンアーキテクチャ + DDD（詳細は [coding-conventions.md](./coding-conventions.md) を参照）
- Lambda（メモリ128MB）でのコールドスタート最小化・レイテンシ最小化

---

## 2. AWS Lambda最適化設計

### 2.1. INITフェーズの有効活用

Cold Start時のCPUブースト枠（最大10,000ms）を活用して重い初期化処理を完了させる。

**INITフェーズで実行すべき処理**:
1. ログ・トレーシング（`tracing_subscriber`）の初期化
2. 環境変数（`DSQL_ENDPOINT`, `AWS_REGION`等）の読み込みとパース
3. Aurora DSQLのコネクションプールの作成と接続確立

### 2.2. シングルスレッド最適化（INVOKEフェーズ）

メモリ128MB環境ではCPUが約0.08コア相当に制限される。マルチスレッドはオーバーヘッドが大きいため、シングルスレッドを使用する。

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> { ... }
```

### 2.3. OnceCell による静的ライフサイクル管理

Cold Start時に生成した環境変数・DB接続プールを複数リクエスト（Hot Start）にわたって再利用する。

```rust
static DSQL_ENDPOINT: OnceCell<String> = OnceCell::const_new();
static AWS_REGION: OnceCell<String> = OnceCell::const_new();
static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();
```

---

## 3. Fail-Fast原則

- 起動時に環境変数が設定されていない場合 → 即座に `panic!`
- DB接続の初期化に失敗した場合 → 即座に `panic!`
- 不完全な初期化状態のままリクエストを処理してはならない

---

## 4. DBアクセス制限（セキュリティ）

- Lambda実行ロールには `inquiries` テーブルへの **INSERTのみ** 付与
- SeaORMの `ActiveModel::insert` は `RETURNING` 句（暗黙的なSELECT）を含むため使用禁止
- 代わりに `ConnectionTrait::execute_raw` または `exec_without_returning` を使用
- SQLインジェクション対策: プレースホルダー（`$1`, `$2`等）を必ず使用。文字列結合によるSQL組み立て厳禁

---

## 5. エントリーポイント実装パターン

すべてのAPIハンドラーの `main.rs` は以下パターンを踏襲する。

```
main()
  └─ executor()
      ├─ [INITフェーズ] tracing初期化
      ├─ [INITフェーズ] 環境変数読み込み（DSQL_ENDPOINT, AWS_REGION）
      ├─ [INITフェーズ] DB接続確立（OnceCell）
      └─ [INVOKEフェーズ] lambda_executor(handler)
```

---

## 6. 環境変数

| 変数名 | 必須 | 説明 |
|---|---|---|
| `DSQL_ENDPOINT` | ✓ | Aurora DSQLクラスターのエンドポイント |
| `AWS_REGION` | ✓ | Aurora DSQLクラスターのAWSリージョン |

---

## 7. APIエンドポイント仕様

### POST /api/v1/inquiry

**リクエスト（JSON）**:
```json
{
  "name": "氏名",
  "email": "メールアドレス",
  "message": "本文"
}
```

**レスポンス**:
- 成功: HTTP 200
- バリデーションエラー: HTTP 400
- サーバーエラー: HTTP 500

---

## 関連ページ

- [coding-conventions.md](./coding-conventions.md) — コーディング規約（アーキテクチャ詳細）
- [database.md](./database.md) — DB規約
- [infrastructure.md](./infrastructure.md) — インフラ規約
