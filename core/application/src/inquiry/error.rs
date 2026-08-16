//! # 問い合わせアプリケーションエラー
//!
//! ユースケース単位のエラー定義。

use domain::inquiry::{domain_error::InquiryDomainError, repository_error::InquiryRepositoryError};

/// 問い合わせ作成エラー
#[derive(Debug, thiserror::Error)]
pub enum CreateInquiryError {
    /// ドメインバリデーションエラー
    #[error(transparent)]
    Domain(#[from] InquiryDomainError),

    /// リポジトリエラー
    #[error(transparent)]
    Repository(#[from] InquiryRepositoryError),
}
