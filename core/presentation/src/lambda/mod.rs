//! # Lambdaモジュール
//!
//! AWS Lambda実行関連の定義。

/// HTTPルーティング・エラー応答決定コア（Verus・Kani 形式検証対象）
pub mod http_decision;

/// Lambdaエグゼキューター
pub mod lambda_executor;

/// ルーター
pub mod router;

/// 問い合わせハンドラー
pub mod inquiry_handler;
