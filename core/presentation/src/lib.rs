//! # プレゼンテーション層
//!
//! HTTP、Lambda、OpenAPIなど外部入出力を担当する。
//!
//! ## 形式手法による検証（Verus / Kani）
//!
//! AWS SDK・HTTP・JSONシリアライザ等の重い外部依存から切り離した純粋決定コア
//! （[`lambda::http_decision`]）に対し、CORS `OPTIONS` プリフライト優先・エンドポイント分離・
//! HTTPステータスコード範囲・内部リポジトリエラーの情報非漏洩（サニタイズ）を
//! Verus と Kani の双方で証明している。
//!
//! | 検証項目 | コマンド例 |
//! |---|---|
//! | **Verus**（演繹的検証・任意長パス） | `verus --crate-type=lib --no-cheating core/presentation/src/lambda/http_decision.rs` |
//! | **Kani**（有界モデル検査・標準スライス同値性） | `kani --output-format terse core/presentation/src/lambda/http_decision.rs` |

/// APIリクエスト・レスポンス定義
pub mod api;

/// Lambda実行関連
pub mod lambda;
