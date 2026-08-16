//! # 問い合わせエンティティ
//!
//! DDDにおける問い合わせの集約ルートを定義する。

use chrono::{DateTime, FixedOffset};

use super::value_object::{InquiryEmail, InquiryId, InquiryMessage, InquiryName};

/// 問い合わせエンティティ
///
/// ## フィールド
///
/// | フィールド | 型 | 説明 |
/// |---|---|---|
/// | `id` | `InquiryId` | 問い合わせID |
/// | `name` | `InquiryName` | 氏名 |
/// | `email` | `InquiryEmail` | 連絡先メールアドレス |
/// | `message` | `InquiryMessage` | 問い合わせ本文 |
/// | `created_at` | `DateTime<FixedOffset>` | 作成日時 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inquiry {
    id: InquiryId,
    name: InquiryName,
    email: InquiryEmail,
    message: InquiryMessage,
    created_at: DateTime<FixedOffset>,
}

impl Inquiry {
    /// 問い合わせエンティティを生成する
    pub fn new(
        id: InquiryId,
        name: InquiryName,
        email: InquiryEmail,
        message: InquiryMessage,
        created_at: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            id,
            name,
            email,
            message,
            created_at,
        }
    }

    /// 問い合わせIDを返す
    pub fn id(&self) -> &InquiryId {
        &self.id
    }

    /// 氏名を返す
    pub fn name(&self) -> &InquiryName {
        &self.name
    }

    /// 連絡先メールアドレスを返す
    pub fn email(&self) -> &InquiryEmail {
        &self.email
    }

    /// 問い合わせ本文を返す
    pub fn message(&self) -> &InquiryMessage {
        &self.message
    }

    /// 作成日時を返す
    pub fn created_at(&self) -> &DateTime<FixedOffset> {
        &self.created_at
    }
}
