# コーディング規約（Rust）

> 最終更新: 2026-08-16 | ソース: raw/02_conventions/coding.md, raw/01_requirements/system.md

Rustの実装時のコーディング規約・アーキテクチャ方針・DDD実装方針を定める。

---

## 1. AI開発における注意事項

- `let` 宣言にも型（Type Annotation）を省略せずに明記する（AIの認識負荷を下げるため）

---

## 2. アーキテクチャ方針（クリーンアーキテクチャ + DDD）

### 2.1. レイヤー構成と依存ルール

```
presentation  ←──  application  ←──  domain
     ↓                  ↑
infrastructure  ─────────┘
```

| レイヤー | 役割 | 依存先 |
|---|---|---|
| `domain` | ドメインモデル・ビジネスルール・エンティティ・値オブジェクト | なし（他の業務層に依存禁止） |
| `application` | ユースケース・DTO | `domain` のみ |
| `infrastructure` | DBアクセス（SeaORM）・外部サービス連携 | `domain` のリポジトリtraitを実装 |
| `presentation` | HTTP・Lambda・OpenAPI入出力 | `application` |

### 2.2. レイヤー間マッピング規則

| 変換方向 | 実装場所 | トレイト |
|---|---|---|
| `presentation request` → `application input` | `presentation` 側 | `From` / `TryFrom` |
| `application output` → `presentation response` | `presentation` 側 | `From` / `TryFrom` |
| `domain entity` → `SeaORM ActiveModel` | `infrastructure` 側 | `From` |
| `SeaORM Model` → `domain entity` | `infrastructure` 側 | `TryFrom` / `try_from_model` |
| domain value object生成を伴う変換 | 変換先の `impl` | `TryFrom` / `try_from_xxx` |
| 単純なDTO変換 | 変換先の `impl` | `From` / `from_xxx` |

> **注意**: RustのOrphan Ruleにより `From` / `TryFrom` が使えない場合は、独自メソッド（`from_xxx` / `to_xxx` / `try_from_xxx` / `try_to_xxx`）として実装する。

### 2.3. レイヤー間の型の持ち込み禁止

- `domain` は `presentation` のrequest/response型、SeaORMのModel/ActiveModel、Lambda/API Gateway型に依存してはならない
- `application` はSeaORM、Lambda、HTTP、OpenAPIに依存してはならない
- 外側のレイヤーの型を内側のレイヤーに持ち込んではならない

---

## 3. Rustワークスペース構成

```text
core
├── domain         （他業務層への依存禁止）
├── application    （domainのみ依存）
├── infrastructure （domainのリポジトリtraitを実装）
└── presentation   （HTTP・Lambda・OpenAPI入出力）

api
└── lambda         （エントリーポイント）

db
└── sea_orm_entities （SeaORMエンティティ定義）
```

---

## 4. フォーマット・静的解析・テスト

### 4.1. フォーマット

```bash
cargo fmt -- --check
```

### 4.2. 静的解析（Clippy）

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### 4.3. コンパイルチェック

```bash
cargo check --workspace --all-targets --all-features
```

### 4.4. テスト

```bash
# 通常テスト
cargo test --workspace --all-features

# ignore付きテスト（ローカルDB必須・プロパティベーステスト等）を含む場合
cargo test --workspace --all-features -- --include-ignored
```

- 重いテスト、プロパティベーステスト、ローカルDB必須テストには `#[ignore = "..."]` を付与する

### 4.5. ドキュメント生成

```bash
# 通常
cargo doc --workspace --no-deps

# 内部実装もレビュー対象とする場合
cargo doc --workspace --no-deps --document-private-items
```

- public itemには原則として**日本語**のドキュメンテーションコメントを記載する
- Markdownで箇条書き・テーブル表記・`└─▶` 等の記号を使い、レビュー者の負担を軽減する

---

## 5. エラー処理方針

| レイヤー | エラー定義 |
|---|---|
| `domain` | ドメインルール違反を表すエラーを定義 |
| `application` | ユースケース単位のエラーに変換 |
| `presentation` | アプリケーションエラーをHTTPステータスへ変換 |
| `infrastructure` | 詳細な外部エラーをそのままAPIレスポンスへ返してはならない |

---

## 6. 主要Cargo.toml設定

### Releaseプロフィール

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
panic = "abort"
strip = true
```

### ワークスペース共通依存（抜粋）

```toml
[workspace.dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["clock", "serde"] }
sea-orm = { version = "2", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros", "with-chrono", "with-uuid"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt"] }
uuid = { version = "1", features = ["serde", "v7"] }
```

---

## 関連ページ

- [api.md](./api.md) — API規約（Lambda最適化含む）
- [database.md](./database.md) — DB規約（SeaORM）
- [overview.md](./overview.md) — プロジェクト全体概要
