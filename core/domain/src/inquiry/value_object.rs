//! # 問い合わせ値オブジェクト
//!
//! 問い合わせドメインで使用する値オブジェクトを定義する。

use uuid::Uuid;

use super::domain_error::InquiryDomainError;

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
/// - 空文字不可
/// - 最大100文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryName(String);

impl InquiryName {
    /// 氏名値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyName`: 氏名が空
    /// - `InquiryDomainError::NameTooLong`: 氏名が100文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        if value.trim().is_empty() {
            return Err(InquiryDomainError::EmptyName);
        }

        if value.chars().count() > 100 {
            return Err(InquiryDomainError::NameTooLong);
        }

        Ok(Self(value))
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
/// - 空文字不可
/// - 最大256文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryEmail(String);

impl InquiryEmail {
    /// メールアドレス値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyEmail`: メールアドレスが空
    /// - `InquiryDomainError::InvalidEmail`: メールアドレスが256文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        if value.trim().is_empty() {
            return Err(InquiryDomainError::EmptyEmail);
        }

        if value.chars().count() > 256 {
            return Err(InquiryDomainError::InvalidEmail);
        }

        Ok(Self(value))
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
/// - 空文字不可
/// - 最大5000文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryMessage(String);

impl InquiryMessage {
    /// 問い合わせ本文値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyMessage`: 本文が空
    /// - `InquiryDomainError::MessageTooLong`: 本文が5000文字超
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        if value.trim().is_empty() {
            return Err(InquiryDomainError::EmptyMessage);
        }

        if value.chars().count() > 5000 {
            return Err(InquiryDomainError::MessageTooLong);
        }

        Ok(Self(value))
    }

    /// 問い合わせ本文の値を返す
    pub fn value(&self) -> &str {
        &self.0
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
    fn 氏名が100文字以内の場合成功する() {
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("山田太郎");
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
    fn メールアドレスが257文字以上の場合エラーになる() {
        let value: String = "a".repeat(257);
        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new(value);
        assert_eq!(result, Err(InquiryDomainError::InvalidEmail));
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
    fn 本文が5001文字以上の場合エラーになる() {
        let value: String = "あ".repeat(5001);
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(value);
        assert_eq!(result, Err(InquiryDomainError::MessageTooLong));
    }
}
