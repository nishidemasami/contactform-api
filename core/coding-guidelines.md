## ビジネスロジック

Rustのワークスペースにより循環参照を未然に防ぐことでクリーンアーキテクチャを実装しやすくする。

```text
core
├── domain
│   ├── Cargo.toml
│   └── src
│       └── inquiry
│           ├── entity.rs
│           ├── repository.rs
│           ├── validation.rs
│           ├── value_object.rs
│           ├── domain_error.rs
│           └── repository_error.rs
│
├── application
│   ├── Cargo.toml
│   └── src
│       └── inquiry
│           ├── dto.rs
│           ├── create.rs
│           └── error.rs
│
├── infrastructure
│   ├── Cargo.toml
│   └── src
│       ├── db
│       │   ├── config.rs
│       │   ├── connection.rs
│       │   ├── aurora_dsql.rs
│       │   └── postgres.rs
│       │
│       └── repository
│           └── inquiry
│               └── seaorm_inquiry_repository.rs
│
└── presentation
    ├── Cargo.toml
    └── src
        ├── api
        └── lambda
```

# コーディング規約(Rust)

- AI開発
  - AIの認識負荷を下げるため、 let 宣言にも型（Type Annotation）を省略せずに明記する

- アーキテクチャ
  - クリーンアーキテクチャとDDDを基本方針とする。
  - `domain` は他の業務層に依存してはならず、`#![no_std]`（`alloc` 使用、純粋検証コアは `alloc` 非依存）を維持する。
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
  - `domain` 変更時は `cargo check -p domain --target x86_64-unknown-none` で `no_std` 整合性を確認する。

- 形式手法検証（`domain` 層）
  - 既存のワークスペースビルドや `Cargo.toml` 依存グラフへの影響を避けつつ `--no-cheating` で厳格に証明するため、純粋なドメイン検証アルゴリズムは外部クレート・`alloc` 非依存のコアモジュール（例: `inquiry/validation.rs`）に切り出す **Verified Core（Functional Core, Imperative Shell）パターン**を採用する。
  - 値オブジェクトは検証用コピーを持たず、Verified Core の本番関数を直接呼び出す薄いラッパーとして実装する。
  - **Verus**（任意長・演繹的検証）:
    - `#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(...))]` および `verus_proof_expr!` を用い、通常ビルドでゼロオーバーヘッドかつ `rustfmt` / `clippy` が通る形で記述する。
    - `verus --crate-type=lib --no-cheating core/domain/src/inquiry/validation.rs` で公理追加（`assume` / `admit` / `external_body`）なしに検証通過させる。
  - **Kani**（有界モデル検査）:
    - Verified Core と Rust 標準ライブラリ（`char::is_whitespace` 等）の同値性、および外側の値オブジェクト・エンティティを `cargo kani -p domain --output-format terse -j` で検証する。
    - CBMC の状態爆発を避けるため、ハーネス内での `core::str::from_utf8`・`str::trim()`・`str::chars().count()`・`chrono::Utc::timestamp_opt`・ループ内の `String::push` を避け、`kani::any::<char>()` と `encode_utf8` 等による軽量なシンボリック入力構築を用いる。

- テスト
  - 通常テストは `cargo test --workspace --all-features` を実行する。
  - 重いテスト、プロパティベーステスト、ローカルDB必須テストは `#[ignore = "..."]` を付与する。
  - ignore付きテストを含めた確認時のみ `cargo test --workspace --all-features -- --include-ignored` を実行する。

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
