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

## 7. ドメイン層の `no_std` 化と形式手法検証（知見・ベストプラクティス・改善案）

### 7.1. 適用範囲と `no_std` 化の理由

- **なぜ `domain` 層のみを形式手法で検証するのか**
  - 外周レイヤー（`infrastructure` / `presentation` / `lambda`）は AWS Lambda・API Gateway・Aurora DSQL・`tokio`・`sea-orm` など外部世界や巨大クレートとの統合が主責務であり、形式モデルが存在しないため結合テスト（`sam local` + PostgreSQL + `pytest`）で検証する方が有効かつ現実的である。
  - 一方、`domain` 層のビジネスルール・バリデーション不変条件はシステムの根幹であり、境界値ミス・文字コード判定漏れ・優先順位バグ・パニックを数学的に根絶する投資対効果（ROI）が最も高い。
- **なぜ `#![no_std]` が必須なのか**
  1. I/O・システム時刻・スレッド・システムコール等の非決定的副作用の混入をコンパイラレベルで遮断するため。
  2. `std` が引き込むバックトレース・スレッドローカル変数・OS アロケータ等を排除し、Kani（CBMC）の状態空間を極小化するため。
  3. さらに検証コア（`core/domain/src/inquiry/validation.rs`）は `alloc` すら使わない純粋 `core` 実装とすることで、ヒープ再確保に伴う状態爆発も排除している。

### 7.2. 理想と現実：2つの設計パターンと採用したベストプラクティス

`domain` 層への Verus 導入には以下の 2 つのアプローチがある。

| 観点 | 【現実解・採用】Verified Core 分離方式（Functional Core, Imperative Shell） | 【理想形】クレート統合・インライン方式（Verus-First） |
|---|---|---|
| **構成** | 純粋アルゴリズムを外部クレート非依存の `validation.rs` に分離して Verus で検証し、`value_object.rs` は薄いラッパーとして Kani で検証する | `domain/Cargo.toml` に `vstd` と `[package.metadata.verus]` を追加し、`value_object.rs` 内の型に直接 `verus!` / `#[verifier::type_invariant]` を記述する |
| **強み** | ・本番 `Cargo.toml` / `Cargo.lock` に `vstd` 依存を持ち込まない<br>・既存 CI の Rust バージョン（`1.97.1`）や `x86_64-unknown-none`・`--all-features` と衝突しない<br>・外部型境界の公理が不要なため **`--no-cheating` で完全証明できる** | ・値オブジェクト定義の直下に形式仕様が書かれ局所性が高い<br>・`InquiryName` 構造体そのものの型不変条件まで Verus 単体で完結する |
| **弱み・制約** | ・型定義（`value_object.rs`）と形式仕様（`validation.rs`）のファイルが分かれる<br>・ラッパー内の数行のエラー変換 `match` は Verus ではなく Kani が担保する | ・ワークスペースの Rust バージョンが Verus 指定バージョン（例：`1.100.0`）に強く拘束される<br>・`Uuid` / `DateTime` / `Into<String>` 境界で `external_body` 等の信頼仮定が必要になる |

**本プロジェクトの方針**:
既存ワークスペースのビルド安定性・`no_std` 整合性・`--no-cheating`（公理追加禁止）を最優先し、**Verified Core 分離方式 ＋ Verus / Kani ハイブリッド検証**を標準ベストプラクティスとして採用する。
このとき、`validation.rs` は検証用コピーではなく**本番の値オブジェクトコンストラクタが直接呼び出す実コード**とし、自前実装した UTF-8 走査・Unicode 空白判定が Rust 標準ライブラリ（`char::is_whitespace` / `char::encode_utf8`）と全 Unicode スカラー値（1,112,064通り）で完全一致することを Kani で証明して両ツールの境界ギャップを塞ぐ。

### 7.3. 形式検証コードの実装ノウハウ・規約

#### Verus 記述ルール
- ファイル全体を `verus! { ... }` で囲むと `rustfmt` や通常 `clippy` と干渉しやすいため、実行関数は通常の Rust 構文で記述し、`#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(...))]` および `verus_proof_expr!` を使用する。
- `verus!` 外の定数には `#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]`、単純な列挙型には `#[verifier::external_type_specification]` を付与する。
- `forall` / `exists` 量化子には必ず `#[trigger]` を明示し、SMT ソルバのインスタンス化を安定させる。
- CI では必ず `--no-cheating` を付与し、`assume` / `admit` / `external_body` の混入を禁止する。

#### Kani 記述ルール（CBMC 状態爆発の回避）
- **避けるべきアンチパターン**:
  - 任意バイト配列への `core::str::from_utf8`（`run_utf8_validation` のチャンクループで爆発する）
  - シンボリック文字列への `str::trim()`（双方向探索 `DoubleEndedSearcher` でハングする）および `str::chars().count()`（SWAR ビット並列最適化で SAT 負荷が増大する）
  - `chrono::Utc.timestamp_opt` への広範囲なシンボリック秒入力（64ビット整数除算・剰余でハングする）
  - ループ内での `String::push` 繰り返しや、`Uuid` 同士の直接 `assert_eq!`（16バイト `memcmp` ループによる unwind 超過）
- **推奨パターン**:
  - `let ch: char = kani::any();` と `ch.encode_utf8(&mut buf)` でスタック上に妥当な UTF-8 文字列を構築する。
  - 日時生成は除算を含まない `NaiveTime::from_num_seconds_from_midnight_opt` と `FixedOffset::east_opt` を使う。
  - `Uuid` の等値検証は `as_u128()` で 128 ビット整数比較に落とす。
  - `kani::assume` の書きすぎによる「空疎な証明（Vacuous Truth）」を防ぐため、必ず具体値の通常単体テスト（`#[test]`）を併設する。

### 7.4. 今後のより良い改善案

1. **【短期】CI キャッシュによる高速化**: GitHub Actions で `~/.kani`・`~/.local/verus`・`cargo-kani` を `actions/cache` でキャッシュし、毎回のコンパイル・ダウンロード時間を短縮する。
2. **【中期】ドメイン仕様拡充への適用拡大**: メールアドレス構文ルール、制御文字禁止ルール、問い合わせ状態遷移などが追加される際、同じく純粋 `no_std` コアとして実装し Verus / Kani の証明対象に加える。
3. **【長期】インライン型不変条件（パターンB）と Wasm 共有の検討**: 将来ワークスペースの Rust ツールチェーン統一と `vstd` の `no_std` 対応が成熟した段階で、`#[verifier::type_invariant]` によるインライン方式への統合や、`domain` クレートの WebAssembly ビルドによるフロントエンドとの検証済みバリデーション共有を検討する。

---

## 関連ページ

- [api.md](./api.md) — API規約（Lambda最適化含む）
- [database.md](./database.md) — DB規約（SeaORM）
- [cicd.md](./cicd.md) — CI/CD規約（Validateジョブでの形式検証実行）
- [known-issues.md](./known-issues.md) — 既知の問題・制約事項
- [backlog.md](./backlog.md) — バックログ・今後の改善課題
- [overview.md](./overview.md) — プロジェクト全体概要
