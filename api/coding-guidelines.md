# コーディング規約(Rust)

- AI開発
  - AIの認識負荷を下げるため、 let 宣言にも型（Type Annotation）を省略せずに明記する

- アーキテクチャ
  - クリーンアーキテクチャとDDDを基本方針とする。
  - `domain` は他の業務層に依存してはならない。
  - `application` は `domain` にのみ依存する。
  - `infrastructure` は `domain` のリポジトリtraitを実装する。
  - `presentation` はHTTP、Lambda、OpenAPIなど外部入出力を担当する。
  - レイヤー間マッピングは、依存方向を崩さない位置に実装する。
  - 単純な構造変換は `From` トレイトで実装する。
  - バリデーション、ドメイン値オブジェクト生成、外部データからの復元など、失敗可能性がある変換は `TryFrom` トレイトで実装する。
  - ただし、RustのOrphan Ruleにより `From` / `TryFrom` トレイトで実装できない場合は、変換先または変換責務を持つ型の `impl` ブロックに、独自の名前付きメソッド（`from_xxx` / `to_xxx` / `try_from_xxx` / `try_to_xxx`）として実装する。
  - 具体的な配置方針は以下の通り。
    - `presentation request` → `application input` は `presentation` 側で変換する。
    - `application output` → `presentation response` は `presentation` 側で変換する。
    - `domain entity` → `SeaORM ActiveModel` は `infrastructure` 側で変換する。
    - `SeaORM Model` → `domain entity` は `infrastructure` 側で `TryFrom` または `try_from_model` により変換する。
    - `domain value object` の生成を伴う変換は `TryFrom` または `try_from_xxx` とする。
    - 単純なDTO変換は `From` または `from_xxx` とする。
  - 外側のレイヤーの型を内側のレイヤーに持ち込んではならない。
    - 例: `domain` は `presentation` のrequest/response型、SeaORMのModel/ActiveModel、Lambda/API Gateway型に依存してはならない。
    - 例: `application` はSeaORM、Lambda、HTTP、OpenAPIに依存してはならない。

- フォーマット
  - `rustfmt` を使用する。
  - 作業終了前に `cargo fmt -- --check` を実行する。

- 静的解析
  - `clippy` を使用する。
  - 作業終了前に以下を実行する。
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings`

- コンパイルチェック
  - `cargo check --workspace --all-targets --all-features` を実行する。

- テスト
  - 通常テストは `cargo test --workspace --all-features` を実行する。
  - 重いテスト、プロパティベーステスト、ローカルDB必須テストは `#[ignore = "..."]` を付与する。
  - ignore付きテストを含めた確認時のみ `cargo test --workspace --all-features -- --include-ignored` を実行する。
  - バグ修正時には、再発防止および修正の検証を目的として、対象バグに対するテストケースを必ず追加・更新する。

- ドキュメント
  - public itemには原則として日本語のドキュメンテーションコメントを記載する。
  - ドキュメント出力は `cargo doc --workspace --no-deps` を使用する。
  - 内部実装もレビュー対象にする場合、 `--document-private-items` を使用する。
  - ドキュメンテーションコメントは資料として出力され関係者各位からのレビューを受けるため、Markdownで箇条書きやテーブル表記でわかりやすく記述したり、フローは`└─▶`などの記号を用いて、レビュー者の負担を軽減する。


- エラー処理
  - domain層ではドメインルール違反を表すエラーを定義する。
  - application層ではユースケース単位のエラーに変換する。
  - presentation層ではアプリケーションエラーをHTTPステータスへ変換する。
  - infrastructure層の詳細な外部エラーをそのままAPIレスポンスへ返してはならない。
