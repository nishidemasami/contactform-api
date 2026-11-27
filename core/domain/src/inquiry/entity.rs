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

#[cfg(kani)]
mod kani_proofs {
    use alloc::string::String;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use uuid::Uuid;

    use super::*;

    /// 証明: 任意の有効な入力（任意の UUID・任意の非空白 ASCII 文字・任意の時刻とオフセット）から
    /// 生成された `Inquiry` エンティティが全フィールドを欠損なく保持すること
    #[kani::proof]
    #[kani::unwind(4)]
    fn proof_inquiry_entity_preserves_all_fields() {
        let uuid_bytes: [u8; 16] = kani::any();
        let expected_uuid: Uuid = Uuid::from_bytes(uuid_bytes);
        let id: InquiryId = InquiryId::new(expected_uuid);

        let name_byte: u8 = kani::any();
        kani::assume(name_byte.is_ascii_graphic());
        let name: InquiryName = InquiryName::new(String::from(name_byte as char)).unwrap();

        let email_byte: u8 = kani::any();
        kani::assume(email_byte.is_ascii_graphic());
        let email: InquiryEmail = InquiryEmail::new(String::from(email_byte as char)).unwrap();

        let msg_byte: u8 = kani::any();
        kani::assume(msg_byte.is_ascii_graphic());
        let message: InquiryMessage = InquiryMessage::new(String::from(msg_byte as char)).unwrap();

        let secs: u32 = kani::any();
        kani::assume(secs < 86_400);
        let nsecs: u32 = kani::any();
        kani::assume(nsecs < 1_000_000_000);
        let offset_secs: i32 = kani::any();
        kani::assume((-43_200..=50_400).contains(&offset_secs));

        let date: NaiveDate = NaiveDate::from_ymd_opt(2026, 8, 16).unwrap();
        let time: NaiveTime = NaiveTime::from_num_seconds_from_midnight_opt(secs, nsecs).unwrap();
        let naive: NaiveDateTime = NaiveDateTime::new(date, time);
        let offset: FixedOffset = FixedOffset::east_opt(offset_secs).unwrap();
        let created_at: DateTime<FixedOffset> =
            DateTime::<FixedOffset>::from_naive_utc_and_offset(naive, offset);

        let inquiry: Inquiry = Inquiry::new(id, name, email, message, created_at);

        assert_eq!(inquiry.id().value().as_u128(), expected_uuid.as_u128());
        assert_eq!(inquiry.name().value().as_bytes(), &[name_byte]);
        assert_eq!(inquiry.email().value().as_bytes(), &[email_byte]);
        assert_eq!(inquiry.message().value().as_bytes(), &[msg_byte]);
        assert_eq!(*inquiry.created_at(), created_at);
        assert_eq!(inquiry.created_at().offset(), created_at.offset());
    }
}
