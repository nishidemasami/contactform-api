//! # 問い合わせ値オブジェクト
//!
//! 問い合わせドメインで使用する値オブジェクトを定義する。

use alloc::string::String;

use uuid::Uuid;

use super::{
    domain_error::InquiryDomainError,
    validation::{InquiryField, TextValidationError, validate_field_str},
};

/// 問い合わせID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryId(Uuid);

impl InquiryId {
    /// 問い合わせIDを生成する
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    /// 問い合わせIDの値を返す
    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

/// 氏名値オブジェクト
///
/// ## バリデーション
///
/// - 空文字不可（空白文字のみも不可）
/// - 最大100文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryName(String);

impl InquiryName {
    /// 氏名値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyName`: 氏名が空または空白のみ
    /// - `InquiryDomainError::NameTooLong`: 氏名が100文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_field_str(InquiryField::Name, &value) {
            Ok(()) => Ok(Self(value)),
            Err(TextValidationError::Empty) => Err(InquiryDomainError::EmptyName),
            Err(TextValidationError::TooLong) => Err(InquiryDomainError::NameTooLong),
        }
    }

    /// 氏名の値を返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// メールアドレス値オブジェクト
///
/// ## バリデーション
///
/// - 空文字不可（空白文字のみも不可）
/// - 最大256文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryEmail(String);

impl InquiryEmail {
    /// メールアドレス値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyEmail`: メールアドレスが空または空白のみ
    /// - `InquiryDomainError::EmailTooLong`: メールアドレスが256文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_field_str(InquiryField::Email, &value) {
            Ok(()) => Ok(Self(value)),
            Err(TextValidationError::Empty) => Err(InquiryDomainError::EmptyEmail),
            Err(TextValidationError::TooLong) => Err(InquiryDomainError::EmailTooLong),
        }
    }

    /// メールアドレスの値を返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// 問い合わせ本文値オブジェクト
///
/// ## バリデーション
///
/// - 空文字不可（空白文字のみも不可）
/// - 最大5000文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryMessage(String);

impl InquiryMessage {
    /// 問い合わせ本文値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyMessage`: 本文が空または空白のみ
    /// - `InquiryDomainError::MessageTooLong`: 本文が5000文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_field_str(InquiryField::Message, &value) {
            Ok(()) => Ok(Self(value)),
            Err(TextValidationError::Empty) => Err(InquiryDomainError::EmptyMessage),
            Err(TextValidationError::TooLong) => Err(InquiryDomainError::MessageTooLong),
        }
    }

    /// 問い合わせ本文の値を返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::inquiry::validation::{
        INQUIRY_EMAIL_MAX_CHARS, INQUIRY_MESSAGE_MAX_CHARS, INQUIRY_NAME_MAX_CHARS,
        validate_field_utf8,
    };

    /// 証明: 任意の16バイトから生成した `Uuid` を `InquiryId` に格納し、値を完全に保持すること
    #[kani::proof]
    fn proof_inquiry_id_roundtrip() {
        let bytes: [u8; 16] = kani::any();
        let uuid: Uuid = Uuid::from_bytes(bytes);
        let id: InquiryId = InquiryId::new(uuid);
        assert_eq!(id.value().as_u128(), uuid.as_u128());
    }

    /// 証明: 空文字または任意の1文字 Unicode 入力（全 `char`: ASCII・日本語・全角空白・4バイト絵文字を網羅）に対し、
    /// `InquiryName::new` が空判定・値保持・ドメイン不変条件を満たしパニックしないこと
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_inquiry_name_symbolic_unicode() {
        let is_empty: bool = kani::any();
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let text: &str = if is_empty {
            ""
        } else {
            ch.encode_utf8(&mut buf)
        };

        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(text);

        if is_empty || ch.is_whitespace() {
            assert_eq!(result, Err(InquiryDomainError::EmptyName));
        } else {
            let name: InquiryName = result.expect("1文字の非空白文字列は常に100文字以内");
            assert_eq!(name.value(), text);
            assert!(1 <= INQUIRY_NAME_MAX_CHARS);
        }
    }

    /// 証明: 空文字または任意の1文字 Unicode 入力（全 `char` 網羅）に対し、
    /// `InquiryEmail::new` が空判定・値保持・ドメイン不変条件を満たしパニックしないこと
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_inquiry_email_symbolic_unicode() {
        let is_empty: bool = kani::any();
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let text: &str = if is_empty {
            ""
        } else {
            ch.encode_utf8(&mut buf)
        };

        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new(text);

        if is_empty || ch.is_whitespace() {
            assert_eq!(result, Err(InquiryDomainError::EmptyEmail));
        } else {
            let email: InquiryEmail = result.expect("1文字の非空白文字列は常に256文字以内");
            assert_eq!(email.value(), text);
            assert!(1 <= INQUIRY_EMAIL_MAX_CHARS);
        }
    }

    /// 証明: 空文字または任意の1文字 Unicode 入力（全 `char` 網羅）に対し、
    /// `InquiryMessage::new` が空判定・値保持・ドメイン不変条件を満たしパニックしないこと
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_inquiry_message_symbolic_unicode() {
        let is_empty: bool = kani::any();
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let text: &str = if is_empty {
            ""
        } else {
            ch.encode_utf8(&mut buf)
        };

        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(text);

        if is_empty || ch.is_whitespace() {
            assert_eq!(result, Err(InquiryDomainError::EmptyMessage));
        } else {
            let message: InquiryMessage = result.expect("1文字の非空白文字列は常に5000文字以内");
            assert_eq!(message.value(), text);
            assert!(1 <= INQUIRY_MESSAGE_MAX_CHARS);
        }
    }

    /// 証明: `InquiryName` の上限境界（100文字・101文字）および空白優先順位（101文字の空白は `EmptyName`）
    #[kani::proof]
    #[kani::unwind(104)]
    fn proof_inquiry_name_boundary_and_precedence() {
        let exact_bytes: [u8; INQUIRY_NAME_MAX_CHARS] = [b'a'; INQUIRY_NAME_MAX_CHARS];
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &exact_bytes),
            Ok(())
        );

        let over_bytes: [u8; INQUIRY_NAME_MAX_CHARS + 1] = [b'a'; INQUIRY_NAME_MAX_CHARS + 1];
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &over_bytes),
            Err(TextValidationError::TooLong)
        );

        let spaces_bytes: [u8; INQUIRY_NAME_MAX_CHARS + 1] = [b' '; INQUIRY_NAME_MAX_CHARS + 1];
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &spaces_bytes),
            Err(TextValidationError::Empty)
        );
    }

    /// 証明: `InquiryEmail` の上限境界（256文字・257文字）および空白優先順位（257文字の空白は `EmptyEmail`）
    #[kani::proof]
    #[kani::unwind(260)]
    fn proof_inquiry_email_boundary_and_precedence() {
        let exact_bytes: [u8; INQUIRY_EMAIL_MAX_CHARS] = [b'a'; INQUIRY_EMAIL_MAX_CHARS];
        assert_eq!(
            validate_field_utf8(InquiryField::Email, &exact_bytes),
            Ok(())
        );

        let over_bytes: [u8; INQUIRY_EMAIL_MAX_CHARS + 1] = [b'a'; INQUIRY_EMAIL_MAX_CHARS + 1];
        assert_eq!(
            validate_field_utf8(InquiryField::Email, &over_bytes),
            Err(TextValidationError::TooLong)
        );

        let spaces_bytes: [u8; INQUIRY_EMAIL_MAX_CHARS + 1] = [b' '; INQUIRY_EMAIL_MAX_CHARS + 1];
        assert_eq!(
            validate_field_utf8(InquiryField::Email, &spaces_bytes),
            Err(TextValidationError::Empty)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 氏名が空の場合エラーになる() {
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("");
        assert_eq!(result, Err(InquiryDomainError::EmptyName));
    }

    #[test]
    fn 氏名が空白のみの場合エラーになる() {
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("   ");
        assert_eq!(result, Err(InquiryDomainError::EmptyName));
    }

    #[test]
    fn 氏名が全角空白のみの場合エラーになる() {
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("　　　");
        assert_eq!(result, Err(InquiryDomainError::EmptyName));
    }

    #[test]
    fn 氏名が101文字の空白のみの場合空エラーが優先される() {
        let value: String = " ".repeat(101);
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(value);
        assert_eq!(result, Err(InquiryDomainError::EmptyName));
    }

    #[test]
    fn 氏名が100文字以内の場合成功する() {
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("山田太郎");
        assert!(result.is_ok());
    }

    #[test]
    fn 氏名がちょうど100文字の場合成功する() {
        let value: String = "あ".repeat(100);
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(value);
        assert!(result.is_ok());
    }

    #[test]
    fn 氏名が101文字以上の場合エラーになる() {
        let value: String = "あ".repeat(101);
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(value);
        assert_eq!(result, Err(InquiryDomainError::NameTooLong));
    }

    #[test]
    fn メールアドレスが空の場合エラーになる() {
        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new("");
        assert_eq!(result, Err(InquiryDomainError::EmptyEmail));
    }

    #[test]
    fn メールアドレスが256文字以内の場合成功する() {
        let result: Result<InquiryEmail, InquiryDomainError> =
            InquiryEmail::new("test@example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn メールアドレスがちょうど256文字の場合成功する() {
        let value: String = "a".repeat(256);
        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new(value);
        assert!(result.is_ok());
    }

    #[test]
    fn メールアドレスが257文字以上の場合エラーになる() {
        let value: String = "a".repeat(257);
        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new(value);
        assert_eq!(result, Err(InquiryDomainError::EmailTooLong));
    }

    #[test]
    fn 本文が空の場合エラーになる() {
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new("");
        assert_eq!(result, Err(InquiryDomainError::EmptyMessage));
    }

    #[test]
    fn 本文が5000文字以内の場合成功する() {
        let result: Result<InquiryMessage, InquiryDomainError> =
            InquiryMessage::new("お問い合わせです");
        assert!(result.is_ok());
    }

    #[test]
    fn 本文がちょうど5000文字の場合成功する() {
        let value: String = "あ".repeat(5000);
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(value);
        assert!(result.is_ok());
    }

    #[test]
    fn 本文が5001文字以上の場合エラーになる() {
        let value: String = "あ".repeat(5001);
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(value);
        assert_eq!(result, Err(InquiryDomainError::MessageTooLong));
    }
}
