//! # 問い合わせリポジトリエラー
//!
//! リポジトリ操作時のエラー定義。

use thiserror::Error;

/// 問い合わせリポジトリエラー
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InquiryRepositoryError {
    /// 重複エラー
    #[error("inquiry already exists")]
    Duplicate,

    /// 見つからないエラー
    #[error("inquiry not found")]
    NotFound,

    /// インフラエラー
    #[error("infrastructure error")]
    Infrastructure,

    /// データ破損エラー
    #[error("stored inquiry data is corrupted")]
    CorruptedData,
}
