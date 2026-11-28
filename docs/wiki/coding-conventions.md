# コーディング規約（Rust）

> 最終更新: 2026-11-27 | ソース: raw/00000001_コーディング規約と実装例.md, raw/00000011_形式手法による検証.md, raw/20260816_要件.md

Rustの実装時のコーディング規約・アーキテクチャ方針・DDD実装方針・ドメイン層の形式手法検証規約を定める。

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
| `domain` | ドメインモデル・ビジネスルール・エンティティ・値オブジェクト（`#![no_std]`、検証コアは `alloc` 非依存） | なし（他の業務層および `std` に依存禁止） |
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

### 4.6. `no_std` チェック・形式手法検証（`domain` 層）

```bash
# domain クレートに std 依存が混入していないことの検証
cargo check -p domain --target x86_64-unknown-none

# Verus による演繹的検証（任意長入力・公理追加禁止）
verus --crate-type=lib --no-cheating core/domain/src/inquiry/validation.rs

# Kani による有界モデル検査（全 Unicode char 同値性・値オブジェクト・エンティティ）
cargo kani -p domain --output-format terse -j
```

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

`domain` クレートの `#![no_std]` を維持するため、`domain` でも利用する共通クレートはワークスペースルートで `default-features = false` とし、`std` やシステム時刻（`clock`）が必要な上位レイヤー（`application` / `infrastructure` / `presentation`）側で個別にフィーチャを有効化する。

```toml
[workspace.dependencies]
async-trait = "0.1"
chrono = { version = "0.4", default-features = false }
sea-orm = { version = "2", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros", "with-chrono", "with-uuid"] }
serde = { version = "1", features = ["derive"] }
thiserror = { version = "2", default-features = false }
tokio = { version = "1", features = ["macros", "rt"] }
uuid = { version = "1", default-features = false }
```

---

## 7. `no_std` 化と形式手法検証（知見・ベストプラクティス・改善案）

### 7.1. 適用範囲と `no_std` 化の理由

- **どのロジックを形式手法で検証し、何を結合テストに委ねるのか**
  - AWS Lambda ランタイム・API Gateway・Aurora DSQL・`tokio`・`sea-orm` など外部システムや巨大クレートの I/O 自体は形式モデルが存在しないため、結合テスト（`sam local` + PostgreSQL + `pytest`）で検証する。
  - 一方、各レイヤーから非決定的 I/O を剥がした**決定論的ロジック（Verified Core）**は形式手法の投資対効果（ROI）が極めて高く、本プロジェクトでは以下の 3 レイヤー横断で証明を行っている。
    1. **`domain` 層**: `validation.rs`（Verus + Kani：UTF-8 走査・空白判定・上限境界・不正バイト耐性）、`value_object.rs` / `entity.rs`（Kani：エラー全単射・前後空白保持・全フィールド保持）
    2. **`application` 層**: `CreateInquiryUsecase::execute_with_metadata`（Kani + `Waker::noop()`：不正入力時の DB 非アクセス保証［Fail-Fast］・正常保存1回保証・全リポジトリエラー伝播）
    3. **`presentation` 層**: `lambda/http_decision.rs`（Verus + スタンドアロン Kani：CORS `OPTIONS` 最優先・単数 `/api/v1/inquiry` と複数 `/api/v1/inquiries` の非混同・内部リポジトリエラー詳細の `500 internal server error` への完全秘匿［情報非干渉］）
- **なぜ `#![no_std]` が必須なのか**
  1. I/O・システム時刻・スレッド・システムコール等の非決定的副作用の混入をコンパイラレベルで遮断するため。
  2. `std` や AWS/DB SDK が引き込むバックトレース・スレッドローカル変数・暗号／通信スタック等を排除し、Kani（CBMC）の状態空間を極小化するため（`presentation` の `http_decision.rs` も `#![cfg_attr(any(kani, verus_keep_ghost), no_std)]` により単体ファイルで数秒検証可能にしている）。
  3. さらに純粋検証コア（`validation.rs`・`http_decision.rs`）は `alloc` すら使わない純粋 `core` 実装とすることで、ヒープ再確保に伴う状態爆発も排除している。

### 7.2. 理想と現実：2つの設計パターンと採用したベストプラクティス

Verus 導入には以下の 2 つのアプローチがある。

| 観点 | 【現実解・採用】Verified Core 分離方式（Functional Core, Imperative Shell） | 【理想形】クレート統合・インライン方式（Verus-First） |
|---|---|---|
| **構成** | 純粋アルゴリズムを外部クレート非依存のモジュール（`validation.rs`・`http_decision.rs`）に分離して Verus で検証し、ラッパーやユースケースは Kani で検証する | `Cargo.toml` に `vstd` と `[package.metadata.verus]` を追加し、各構造体・ハンドラに直接 `verus!` / `#[verifier::type_invariant]` を記述する |
| **強み** | ・本番 `Cargo.toml` / `Cargo.lock` に `vstd` 依存を持ち込まない<br>・既存 CI の Rust バージョン（`1.97.1`）や `x86_64-unknown-none`・`--all-features` と衝突しない<br>・重い依存を持つ `presentation` クレート内でも該当モジュールだけ 1 秒未満で Verus 検証できる<br>・外部型境界の公理が不要なため **`--no-cheating` で完全証明できる** | ・構造体定義の直下に形式仕様が書かれ局所性が高い<br>・`InquiryName` 構造体そのものの型不変条件まで Verus 単体で完結する |
| **弱み・制約** | ・型定義／ハンドラと形式仕様コアのファイルが分かれる<br>・ラッパーや `async` ユースケースの接続部は Verus ではなく Kani が担保する | ・ワークスペースの Rust バージョンが Verus 指定バージョン（例：`1.100.0`）に強く拘束される<br>・`Uuid` / `DateTime` / `http` / `lambda_runtime` 境界で `external_body` 等の信頼仮定が必要になる |

**本プロジェクトの方針**:
既存ワークスペースのビルド安定性・`no_std` 整合性・`--no-cheating`（公理追加禁止）を最優先し、**Verified Core 分離方式 ＋ Verus / Kani ハイブリッド検証**を標準ベストプラクティスとして採用する。
このとき、Verified Core は検証用コピーではなく**本番の値オブジェクトや HTTP ルーター／ハンドラが直接呼び出す実コード**とし、自前実装したバイト比較・UTF-8 走査・Unicode 空白判定が Rust 標準ライブラリ（`==` / `char::is_whitespace` / `char::encode_utf8`）と完全一致することを Kani で証明して両ツールの境界ギャップを塞ぐ。

### 7.3. 形式検証コードの実装ノウハウ・規約

#### Verus 記述ルール
- ファイル全体を `verus! { ... }` で囲むと `rustfmt` や通常 `clippy` と干渉しやすいため、実行関数は通常の Rust 構文で記述し、`#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(...))]` および `verus_proof_expr!` を使用する（`=~=` 等の ghost 等値表明は `proof! { ... }` で囲む）。
- `verus!` 外の定数には `#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]`、単純な列挙型・構造体には `#[verifier::external_type_specification]` を付与する。
- `forall` / `exists` 量化子には必ず `#[trigger]` を明示し、SMT ソルバのインスタンス化を安定させる。
- CI では必ず `--no-cheating` を付与し、`assume` / `admit` / `external_body` の混入を禁止する。

#### Kani 記述ルール（CBMC 状態爆発の回避）
- **避けるべきアンチパターン**:
  - 任意バイト配列への `core::str::from_utf8`（`run_utf8_validation` のコード生成とチャンクループでコンパイル・検証とも重くなる）
  - シンボリック文字列への `str::trim()`（双方向探索 `DoubleEndedSearcher` でハングする）および `str::chars().count()`（SWAR ビット並列最適化で SAT 負荷が増大する）
  - `chrono::Utc.timestamp_opt` への広範囲なシンボリック秒入力（64ビット整数除算・剰余でハングする）
  - ループ内での `String::push` 繰り返し、シンボリック長スライスからのヒープ確保、`match` / `if` 各分岐での重複 `String::from`（CBMC は全分岐を単一 SSA 論理式に展開するためヒープオブジェクト数が倍増する）、および `Uuid` 同士の直接 `assert_eq!`（16バイト `memcmp` ループによる unwind 超過）
  - 重い外部クレート（`sea-orm` / AWS SDK / `tokio` ランタイム）に依存するクレート全体への `cargo kani -p` 実行
- **推奨パターン**:
  - `let ch: char = kani::any();` と `ch.encode_utf8(&mut buf)` でスタック上に妥当な UTF-8 文字列を構築し、`String` 確保が必要な場合は分岐前に1回だけ確保してムーブ渡し（`Into<String>` 恒等変換）する。
  - `tokio` 不要の決定的 `async fn`（ユースケース等）は `core::task::Waker::noop()` で 1 ステップ `Future::poll` し、モックの状態保持は `Mutex` ではなく `AtomicUsize` 等の軽量プリミティブを使う。
  - 重い依存を持つクレート内の純粋モジュール（`http_decision.rs` 等）はスタンドアロン `kani <file.rs>` で直接検証する。
  - 日時生成は除算を含まない `NaiveTime::from_num_seconds_from_midnight_opt` と `FixedOffset::east_opt` を使い、`Uuid` 等値検証は `as_u128()` で 128 ビット整数比較に落とす。
  - `kani::assume` の書きすぎによる「空疎な証明（Vacuous Truth）」を防ぐため、必ず具体値の通常単体テスト（`#[test]`）を併設する。

### 7.4. 今後のより良い改善案

1. **【短期】CI キャッシュによる高速化**: GitHub Actions で `~/.kani`・`~/.local/verus`・`cargo-kani` を `actions/cache` でキャッシュし、毎回のコンパイル・ダウンロード時間を短縮する。
2. **【中期】ドメイン仕様・API エンドポイント拡充への適用拡大**: メールアドレス構文ルール、制御文字禁止ルール、新規 API ルートや認可ルールが追加される際、同じく純粋 `no_std` コア（`validation.rs` / `http_decision.rs`）として実装し Verus / Kani の証明対象に加える。
3. **【長期】インライン型不変条件（パターンB）と Wasm 共有の検討**: 将来ワークスペースの Rust ツールチェーン統一と `vstd` の `no_std` 対応が成熟した段階で、`#[verifier::type_invariant]` によるインライン方式への統合や、`domain` クレートの WebAssembly ビルドによるフロントエンドとの検証済みバリデーション共有を検討する。

---

## 関連ページ

- [api.md](./api.md) — API規約（Lambda最適化含む）
- [database.md](./database.md) — DB規約（SeaORM）
- [cicd.md](./cicd.md) — CI/CD規約（Validateジョブでの形式検証実行）
- [known-issues.md](./known-issues.md) — 既知の問題・制約事項
- [backlog.md](./backlog.md) — バックログ・今後の改善課題
- [overview.md](./overview.md) — プロジェクト全体概要
