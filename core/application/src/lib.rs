//! # アプリケーション層
//!
//! ユースケースを定義する。domain層にのみ依存する。
//! SeaORM、Lambda、HTTP等の外部技術には依存しない。
//!
//! ## 形式手法による検証（Kani）
//!
//! 非決定的なID採番・現在時刻取得を境界に分離した決定論的コア
//! （[`inquiry::create::CreateInquiryUsecase::execute_with_metadata`]）に対し、
//! Kani（`core::task::Waker::noop()` による非同期ステートマシン検査）を用いて
//! バリデーション失敗時のリポジトリ非呼び出し（Fail-Fast）・正常保存時のID整合・
//! 全リポジトリエラー伝播を証明している。
//!
//! | 検証項目 | コマンド例 |
//! |---|---|
//! | **Kani**（ユースケース制御フロー検証） | `cargo kani -p application --output-format terse -j` |

/// 問い合わせユースケース
pub mod inquiry;
