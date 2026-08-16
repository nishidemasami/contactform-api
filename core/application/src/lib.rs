//! # アプリケーション層
//!
//! ユースケースを定義する。domain層にのみ依存する。
//! SeaORM、Lambda、HTTP等の外部技術には依存しない。

/// 問い合わせユースケース
pub mod inquiry;
