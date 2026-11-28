//! # 問い合わせドメインエラー
//!
//! ドメインルール違反を表すエラー定義。

use thiserror::Error;

/// 問い合わせドメインエラー
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InquiryDomainError {
    /// 氏名が空（または空白文字のみ）
    #[error("name is required")]
    EmptyName,

    /// 氏名が長すぎる（trim前100文字超）
    #[error("name is too long")]
    NameTooLong,

    /// 氏名に禁止文字（制御文字・改行等）が含まれている
    #[error("name contains invalid characters")]
    InvalidName,

    /// メールアドレスが空（または空白文字のみ）
    #[error("email is required")]
    EmptyEmail,

    /// メールアドレスが長すぎる（trim後254文字超）
    #[error("email is too long")]
    EmailTooLong,

    /// メールアドレスの形式が不正
    #[error("invalid email")]
    InvalidEmail,

    /// 問い合わせ本文が空（または空白文字のみ）
    #[error("message is required")]
    EmptyMessage,

    /// 問い合わせ本文が長すぎる（trim前5000文字超）
    #[error("message is too long")]
    MessageTooLong,

    /// 問い合わせ本文に禁止文字（改行・タブ以外の制御文字、NUL等）が含まれている
    #[error("message contains invalid characters")]
    InvalidMessage,
}
