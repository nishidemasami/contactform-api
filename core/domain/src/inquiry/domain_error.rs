//! # 問い合わせドメインエラー
//!
//! ドメインルール違反を表すエラー定義。

use thiserror::Error;

/// 問い合わせドメインエラー
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InquiryDomainError {
    /// 氏名が空
    #[error("name is required")]
    EmptyName,

    /// 氏名が長すぎる（100文字超）
    #[error("name is too long")]
    NameTooLong,

    /// メールアドレスが空
    #[error("email is required")]
    EmptyEmail,

    /// メールアドレスが無効（256文字超）
    #[error("invalid email")]
    EmailTooLong,

    /// 問い合わせ本文が空
    #[error("message is required")]
    EmptyMessage,

    /// 問い合わせ本文が長すぎる（5000文字超）
    #[error("message is too long")]
    MessageTooLong,
}
