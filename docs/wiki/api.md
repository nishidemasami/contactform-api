# API規約

> 最終更新: 2026-11-28 | ソース: raw/00000002_API規約と実装例.md, raw/00000001_コーディング規約と実装例.md

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

Cold Start時に生成したDB接続プールを複数リクエスト（Hot Start）にわたって再利用する。

```rust
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
- 一意制約違反（`UniqueConstraintViolation`）は `InquiryRepositoryError::Duplicate`（HTTP 409）へ変換し、それ以外の内部DBエラーは外部レスポンスへ詳細を漏洩せず HTTP 500 `"internal server error"` に統一する

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

| 変数名 | 区分 | 説明 |
|---|---|---|
| `DSQL_ENDPOINT` | 必須（SAMテンプレートで注入） | Aurora DSQLクラスターのエンドポイント |
| `AWS_REGION` | 必須（Lambdaランタイム定義済み変数） | Aurora DSQLクラスターのAWSリージョン |

---

## 7. APIエンドポイント仕様

### POST /api/v1/inquiry

**リクエスト制約**:
- Base64エンコードされていない UTF-8 JSON であること
- リクエストボディ上限: **64 KiB（65,536 bytes）**

**リクエスト（JSON）**:
```json
{
  "name": "山田 太郎",
  "email": "yamada@example.com",
  "message": "お問い合わせ本文"
}
```

**フィールドバリデーション規則**:

| フィールド | 前後空白 | 文字数上限の検証対象 | 最大文字数（Unicodeコードポイント） | 構文・禁止文字ルール |
|---|---|---|---|---|
| `name` | 保持 | **`trim`前の文字列** | `100` 文字 | 空・空白のみ不可、C0/C1制御文字（`\0`, `\r`, `\n`, `\t` 等）禁止 |
| `email` | `trim` 正規化して保持 | **`trim`後の文字列** | `254` 文字 | 空不可、RFC 5321/5322 `dot-atom`（`local` 1..=64文字）+ RFC 1123 ドメイン（`domain` 4..=253文字、2ラベル以上、各ラベル 1..=63文字、TLD 2文字以上かつ英字含有） |
| `message` | 保持 | **`trim`前の文字列** | `5000` 文字 | 空・空白のみ不可、`\t`・`\n`・`\r` 以外のC0/C1制御文字（NUL `\0` 等）禁止 |

> **OpenAPIスキーマ（`api/openapi.yaml`）上の表現に関する注意**:
> OpenAPIのJSONスキーマ制約は `trim` 前の生文字列に適用されるため、`CreateInquiryRequest.email` に `maxLength: 254` や `format: email` を直接付与すると、APIが受理する「有効な254文字のメールアドレス＋前後空白」がスキーマ検証で誤拒否される。そのため生JSONスキーマでは `minLength: 6` のみを課し、`trim` 正規化後の `6..=254` 文字・RFC構文制約を説明文およびドメイン層で担保する。

**レスポンス**:
- 作成成功: **HTTP 201 Created**（`{"id": "<UUID v7>"}`）
- バリデーション／JSON不正／ボディ過大エラー: **HTTP 400 Bad Request**
- ID重複エラー: **HTTP 409 Conflict**
- サーバー内部エラー: **HTTP 500 Internal Server Error**

### その他のルーティング仕様（Lambdaハンドラー到達リクエスト）

| メソッド・パス | ステータス | 説明 |
|---|---|---|
| `OPTIONS /api/v1/inquiry`, `OPTIONS /api/v1/inquiries` | **HTTP 204 No Content** | 既知APIパスへの `OPTIONS` 応答（空ボディ） |
| `GET /api/v1/inquiries` | **HTTP 501 Not Implemented** | 問い合わせ一覧（未実装スタブ） |
| 既知パスに対する上記以外のメソッド | **HTTP 405 Method Not Allowed** | 未許可メソッド |
| 未定義パス（全メソッド） | **HTTP 404 Not Found** | 存在しないエンドポイント（Lambda到達時） |

> **API Gateway HTTP API の CORS 自動応答との境界**:
> AWSデプロイ環境では `api/template.yaml` の `AWS::Serverless::HttpApi` に `CorsConfiguration` が設定されているため、許可 `Origin` 等を伴う CORS プリフライト `OPTIONS` リクエストは未知パスを含め **API Gateway エッジで自動応答** され、Lambda には到達しない。上表の `OPTIONS`（`204`）および未定義パス（`404`）のルーティング保証は、`sam local`・直接呼び出し・非プリフライト `OPTIONS` など **Lambda ハンドラーが受け取って処理するリクエスト** に適用される。

---

## 関連ページ

- [coding-conventions.md](./coding-conventions.md) — コーディング規約（アーキテクチャ詳細）
- [database.md](./database.md) — DB規約
- [infrastructure.md](./infrastructure.md) — インフラ規約
