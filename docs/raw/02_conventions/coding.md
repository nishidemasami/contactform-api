# Rust コーディング規約

## AI開発配慮方針
- AIの認識負荷を下げ、型解決やハルシネーションを防止するため、`let` 宣言にも型（Type Annotation）を省略せずに明記する。

## アーキテクチャ原則
- クリーンアーキテクチャとDDDを基本方針とする。
- **依存方向ルール**:
  - `domain` は他の業務層に依存してはならない。
  - `application` は `domain` にのみ依存する。
  - `infrastructure` は `domain` のリポジトリtraitを実装する。
  - `presentation` はHTTP、Lambda、OpenAPIなど外部入出力を担当する。
  - 外側のレイヤーの型を内側のレイヤーに持ち込んではならない（例: `domain` は `presentation` リクエスト/レスポンス型やSeaORM Modelに依存しない）。

## レイヤー間型変換方針
- レイヤー間マッピングは、依存方向を崩さない位置に実装する。
- 単純な構造変換は `From` トレイトで実装する。
- バリデーション、ドメイン値オブジェクト生成、外部データからの復元など、失敗可能性がある変換は `TryFrom` トレイトで実装する。
- **Orphan Rule（孤児規則）対策**:
  - RustのOrphan Ruleにより `From` / `TryFrom` トレイトで実装できない場合は、変換先または変換責務を持つ型の `impl` ブロックに、独自の名前付きメソッド（`from_xxx` / `to_xxx` / `try_from_xxx` / `try_to_xxx`）として実装する。
- **具体的な配置方針**:
  - `presentation request` → `application input`: `presentation` 側で変換
  - `application output` → `presentation response`: `presentation` 側で変換
  - `domain entity` → `SeaORM ActiveModel`: `infrastructure` 側で変換
  - `SeaORM Model` → `domain entity`: `infrastructure` 側で `TryFrom` または `try_from_model` により変換
  - `domain value object` の生成を伴う変換: `TryFrom` または `try_from_xxx`
  - 単純なDTO変換: `From` または `from_xxx`

## 品質・フォーマット・テスト規約
- **フォーマット**: `rustfmt` を使用。作業終了前に `cargo fmt -- --check` を実行。
- **静的解析**: `clippy` を使用。作業終了前に `cargo clippy --workspace --all-targets --all-features -- -D warnings` を実行。
- **コンパイルチェック**: `cargo check --workspace --all-targets --all-features` を実行。
- **テスト**:
  - 通常テスト: `cargo test --workspace --all-features`
  - 重いテスト、プロパティベーステスト、ローカルDB必須テスト: `#[ignore = "..."]` を付与。
  - ignore付き確認時: `cargo test --workspace --all-features -- --include-ignored`
  - **バグ修正時方針**: バグ修正時には、再発防止および修正の検証を目的として、対象バグに対するテストケースを必ず追加・更新する。
- **ドキュメント**:
  - public itemには原則として日本語のドキュメンテーションコメントを記載。
  - ドキュメント出力は `cargo doc --workspace --no-deps` を使用。
  - 内部実装もレビュー対象にする場合 `--document-private-items` を使用。
  - ドキュメンテーションコメントはレビュー資料となるため、Markdownの箇条書きやテーブル、`└─▶` 等を用いて分かりやすく記述する。
- **エラー処理**:
  - `domain` 層: ドメインルール違反エラーを定義。
  - `application` 層: ユースケース単位のエラーに変換。
  - `presentation` 層: アプリケーションエラーをHTTPステータスへ変換。
  - `infrastructure` 層の詳細な外部エラーをそのままAPIレスポンスへ返してはならない。
