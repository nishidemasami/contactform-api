//! # 問い合わせリポジトリトレイト
//!
//! 問い合わせの永続化処理を定義するトレイト。
//! infrastructure層でSeaORMを使用して実装する。

use async_trait::async_trait;

use super::{entity::Inquiry, repository_error::InquiryRepositoryError};

/// 問い合わせリポジトリトレイト
///
/// ## フロー
///
/// `application` └─▶ `InquiryRepository::save` └─▶ `infrastructure`
#[async_trait]
pub trait InquiryRepository: Send + Sync {
    /// 問い合わせを保存する
    ///
    /// ## 引数
    ///
    /// - `inquiry`: 保存する問い合わせエンティティ
    ///
    /// ## 戻り値
    ///
    /// - `Ok(Inquiry)`: 保存された問い合わせエンティティ
    /// - `Err(InquiryRepositoryError)`: リポジトリエラー
    async fn save(&self, inquiry: Inquiry) -> Result<Inquiry, InquiryRepositoryError>;
}
