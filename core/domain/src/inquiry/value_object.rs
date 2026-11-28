//! # 問い合わせ値オブジェクト
//!
//! 問い合わせドメインで使用する値オブジェクトを定義する。
//!
//! ## バリデーション要約
//!
//! | 値オブジェクト | 前後空白 | 文字数上限の検証対象 | 上限値 | 構文・文字種制約 |
//! |---|---|---|---|---|
//! | `InquiryName` | 保持 | **`trim`前の文字列** | `100` 文字 | 空白のみ不可、全 C0/C1 制御文字（`\0`, `\r`, `\n`, `\t` 等）禁止 |
//! | `InquiryEmail` | `trim` 正規化して保持 | **`trim`後の文字列** | `254` 文字 | 空白のみ不可、RFC 5321/5322 `dot-atom` + RFC 1123 ドメイン形式 |
//! | `InquiryMessage` | 保持 | **`trim`前の文字列** | `5000` 文字 | 空白のみ不可、`\t`・`\n`・`\r` 以外の C0/C1 制御文字（`\0` 等）禁止 |

use alloc::string::String;

use uuid::Uuid;

use super::{
    domain_error::InquiryDomainError,
    validation::{InquiryField, TextValidationError, validate_email_utf8, validate_field_str},
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

/// 検証コアのエラーを対応するドメインエラーへ変換する
///
/// `(InquiryField, TextValidationError)` の全9通りの組合せを
/// `InquiryDomainError` の全9バリアントへ1対1（全単射）で写像する。
pub const fn map_validation_error(
    field: InquiryField,
    error: TextValidationError,
) -> InquiryDomainError {
    match (field, error) {
        (InquiryField::Name, TextValidationError::Empty) => InquiryDomainError::EmptyName,
        (InquiryField::Name, TextValidationError::TooLong) => InquiryDomainError::NameTooLong,
        (InquiryField::Name, TextValidationError::InvalidFormat) => InquiryDomainError::InvalidName,
        (InquiryField::Email, TextValidationError::Empty) => InquiryDomainError::EmptyEmail,
        (InquiryField::Email, TextValidationError::TooLong) => InquiryDomainError::EmailTooLong,
        (InquiryField::Email, TextValidationError::InvalidFormat) => {
            InquiryDomainError::InvalidEmail
        }
        (InquiryField::Message, TextValidationError::Empty) => InquiryDomainError::EmptyMessage,
        (InquiryField::Message, TextValidationError::TooLong) => InquiryDomainError::MessageTooLong,
        (InquiryField::Message, TextValidationError::InvalidFormat) => {
            InquiryDomainError::InvalidMessage
        }
    }
}

/// 氏名値オブジェクト
///
/// ## バリデーション
///
/// - 空文字不可（空白文字のみも不可）
/// - 最大100文字（**`trim`前の文字列**で検証）
/// - C0/C1 制御文字（`\0`, `\r`, `\n`, `\t` 等）不可
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryName(String);

impl InquiryName {
    /// 氏名値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyName`: 氏名が空または空白のみ
    /// - `InquiryDomainError::NameTooLong`: 氏名が `trim` 前で100文字超
    /// - `InquiryDomainError::InvalidName`: 氏名に制御文字が含まれる
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_field_str(InquiryField::Name, &value) {
            Ok(()) => Ok(Self(value)),
            Err(err) => Err(map_validation_error(InquiryField::Name, err)),
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
/// - 前後の Unicode 空白を `trim` して正規化する
/// - 空文字不可（空白文字のみも不可）
/// - 最大254文字（**`trim`後の文字列**で検証、RFC 5321 §4.5.3.1.3 / Errata 1690 準拠）
/// - RFC 5321/5322 `dot-atom` + RFC 1123 ドメイン構文を満たすこと
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryEmail(String);

impl InquiryEmail {
    /// メールアドレス値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyEmail`: メールアドレスが空または空白のみ
    /// - `InquiryDomainError::EmailTooLong`: `trim` 後のメールアドレスが254文字超
    /// - `InquiryDomainError::InvalidEmail`: メールアドレスの構文が不正
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_email_utf8(value.as_bytes()) {
            Ok((start, end)) => {
                if start == 0 && end == value.len() {
                    Ok(Self(value))
                } else {
                    let normalized: String = String::from(&value[start..end]);
                    Ok(Self(normalized))
                }
            }
            Err(err) => Err(map_validation_error(InquiryField::Email, err)),
        }
    }

    /// 正規化済みメールアドレスの値を返す
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// 問い合わせ本文値オブジェクト
///
/// ## バリデーション
///
/// - 空文字不可（空白文字のみも不可）
/// - 最大5000文字（**`trim`前の文字列**で検証）
/// - `\t`・`\n`・`\r` 以外の C0/C1 制御文字（NUL `\0` 等）不可
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryMessage(String);

impl InquiryMessage {
    /// 問い合わせ本文値オブジェクトを生成する
    ///
    /// ## エラー
    ///
    /// - `InquiryDomainError::EmptyMessage`: 本文が空または空白のみ
    /// - `InquiryDomainError::MessageTooLong`: 本文が `trim` 前で5000文字超
    /// - `InquiryDomainError::InvalidMessage`: 本文に禁止制御文字が含まれる
    pub fn new(value: impl Into<String>) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        match validate_field_str(InquiryField::Message, &value) {
            Ok(()) => Ok(Self(value)),
            Err(err) => Err(map_validation_error(InquiryField::Message, err)),
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
        INQUIRY_EMAIL_DOMAIN_MAX_CHARS, INQUIRY_EMAIL_LABEL_MAX_CHARS,
        INQUIRY_EMAIL_LOCAL_MAX_CHARS, INQUIRY_EMAIL_MAX_CHARS, INQUIRY_MESSAGE_MAX_CHARS,
        INQUIRY_NAME_MAX_CHARS, is_control_char, is_valid_email_domain_part,
        is_valid_email_local_part, is_valid_email_syntax_bytes, validate_field_utf8,
    };

    /// 証明: `(InquiryField, TextValidationError)` の全9通りから
    /// `InquiryDomainError` の全9バリアントへの変換が全単射（1対1対応）であること
    #[kani::proof]
    #[kani::unwind(12)]
    fn proof_map_validation_error_bijective() {
        let pairs: [(InquiryField, TextValidationError); 9] = [
            (InquiryField::Name, TextValidationError::Empty),
            (InquiryField::Name, TextValidationError::TooLong),
            (InquiryField::Name, TextValidationError::InvalidFormat),
            (InquiryField::Email, TextValidationError::Empty),
            (InquiryField::Email, TextValidationError::TooLong),
            (InquiryField::Email, TextValidationError::InvalidFormat),
            (InquiryField::Message, TextValidationError::Empty),
            (InquiryField::Message, TextValidationError::TooLong),
            (InquiryField::Message, TextValidationError::InvalidFormat),
        ];

        let mut i: usize = 0;
        while i < pairs.len() {
            let mut j: usize = i + 1;
            while j < pairs.len() {
                let lhs: InquiryDomainError = map_validation_error(pairs[i].0, pairs[i].1);
                let rhs: InquiryDomainError = map_validation_error(pairs[j].0, pairs[j].1);
                assert_ne!(lhs, rhs);
                j += 1;
            }
            i += 1;
        }
    }

    /// 証明: `InquiryName` および `InquiryMessage` は前後の非制御空白（全角空白 `U+3000`・半角空白 `' '`）を
    /// トリムせずそのまま保持すること
    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_name_and_message_preserve_surrounding_whitespace() {
        let core_byte: u8 = kani::any();
        kani::assume(core_byte.is_ascii_graphic());

        let buf: [u8; 5] = [0xE3, 0x80, 0x80, core_byte, b' '];
        // SAFETY: `U+3000` (`[0xE3, 0x80, 0x80]`) + 印字可能 ASCII + `' '` は常に正当な UTF-8。
        let text: &str = unsafe { core::str::from_utf8_unchecked(&buf) };
        let owned: String = String::from(text);

        let check_name: bool = kani::any();
        if check_name {
            let vo: InquiryName = InquiryName::new(owned).unwrap();
            assert_eq!(vo.value().as_bytes(), &buf);
        } else {
            let vo: InquiryMessage = InquiryMessage::new(owned).unwrap();
            assert_eq!(vo.value().as_bytes(), &buf);
        }
    }

    /// 証明: `InquiryEmail` は前後の空白（全角空白 `U+3000`・ASCII 空白・タブ・改行）を `trim` して正規化し、
    /// 正規化後のメールアドレスのみを保持すること
    #[kani::proof]
    #[kani::unwind(14)]
    fn proof_email_trims_surrounding_whitespace() {
        let local_ch: u8 = kani::any();
        kani::assume((b'a'..=b'z').contains(&local_ch) || (b'0'..=b'9').contains(&local_ch));

        let ws_trail: u8 = kani::any();
        kani::assume(matches!(ws_trail, b' ' | b'\t' | b'\n' | b'\r'));

        let buf: [u8; 10] = [
            0xE3, 0x80, 0x80, local_ch, b'@', b'b', b'.', b'c', b'o', ws_trail,
        ];
        // SAFETY: `U+3000` + `"x@b.co"` + ASCII 空白文字は常に正当な UTF-8。
        let text: &str = unsafe { core::str::from_utf8_unchecked(&buf) };
        let email: InquiryEmail = InquiryEmail::new(String::from(text)).unwrap();

        let expected: [u8; 6] = [local_ch, b'@', b'b', b'.', b'c', b'o'];
        assert_eq!(email.value().as_bytes(), &expected);
    }

    /// 証明: 任意の16バイトから生成した `Uuid` を `InquiryId` に格納し、値を完全に保持すること
    #[kani::proof]
    fn proof_inquiry_id_roundtrip() {
        let bytes: [u8; 16] = kani::any();
        let uuid: Uuid = Uuid::from_bytes(bytes);
        let id: InquiryId = InquiryId::new(uuid);
        assert_eq!(id.value().as_u128(), uuid.as_u128());
    }

    /// 証明: 空文字または任意の1文字 Unicode 入力（全 `char` 網羅）に対し、
    /// `InquiryName::new` が空白判定・制御文字拒否・値保持を満たしパニックしないこと
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
        } else if is_control_char(ch) {
            assert_eq!(result, Err(InquiryDomainError::InvalidName));
        } else {
            let name: InquiryName = result.expect("非空白・非制御の1文字は常に妥当");
            assert_eq!(name.value(), text);
            assert!(1 <= INQUIRY_NAME_MAX_CHARS);
        }
    }

    /// 証明: 空文字または任意の1文字 Unicode 入力（全 `char` 網羅）に対し、
    /// `InquiryMessage::new` が空白判定・制御文字拒否（`\0` 等）・値保持を満たしパニックしないこと
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
        } else if is_control_char(ch) {
            assert_eq!(result, Err(InquiryDomainError::InvalidMessage));
        } else {
            let message: InquiryMessage = result.expect("非空白・非制御の1文字は常に妥当");
            assert_eq!(message.value(), text);
            assert!(1 <= INQUIRY_MESSAGE_MAX_CHARS);
        }
    }

    /// 証明: `InquiryName` の上限境界（`trim`前 100文字 OK・`trim`前 101文字／`trim`後 100文字 NG・空白101文字 Empty）
    #[kani::proof]
    #[kani::unwind(104)]
    fn proof_inquiry_name_boundary_and_pre_trim_precedence() {
        let mut exact_with_space: [u8; INQUIRY_NAME_MAX_CHARS] = [b'a'; INQUIRY_NAME_MAX_CHARS];
        exact_with_space[0] = b' ';
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &exact_with_space),
            Ok(())
        );

        // trim後は100文字でも、trim前が101文字なら TooLong になること
        let mut over_due_to_leading_space: [u8; INQUIRY_NAME_MAX_CHARS + 1] =
            [b'a'; INQUIRY_NAME_MAX_CHARS + 1];
        over_due_to_leading_space[0] = b' ';
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &over_due_to_leading_space),
            Err(TextValidationError::TooLong)
        );

        let spaces_bytes: [u8; INQUIRY_NAME_MAX_CHARS + 1] = [b' '; INQUIRY_NAME_MAX_CHARS + 1];
        assert_eq!(
            validate_field_utf8(InquiryField::Name, &spaces_bytes),
            Err(TextValidationError::Empty)
        );
    }

    /// 証明: RFC 5321 / 1035 に基づくメールアドレス各構成要素の境界値
    /// （全体254/255文字、ローカル部64/65文字、ラベル63/64文字、ドメイン部253/254文字）が厳密に判定されること
    #[kani::proof]
    #[kani::unwind(70)]
    fn proof_inquiry_email_rfc_component_boundaries() {
        assert_eq!(INQUIRY_EMAIL_MAX_CHARS, 254);
        assert_eq!(INQUIRY_EMAIL_DOMAIN_MAX_CHARS, 253);

        let local_64: [u8; INQUIRY_EMAIL_LOCAL_MAX_CHARS] = [b'a'; INQUIRY_EMAIL_LOCAL_MAX_CHARS];
        let local_65: [u8; INQUIRY_EMAIL_LOCAL_MAX_CHARS + 1] =
            [b'a'; INQUIRY_EMAIL_LOCAL_MAX_CHARS + 1];
        assert!(is_valid_email_local_part(&local_64));
        assert!(!is_valid_email_local_part(&local_65));

        // ラベル上限: 63文字 + ".com" = 67文字 (OK) vs 64文字 + ".com" = 68文字 (NG)
        let mut label_63_domain: [u8; INQUIRY_EMAIL_LABEL_MAX_CHARS + 4] =
            [b'a'; INQUIRY_EMAIL_LABEL_MAX_CHARS + 4];
        label_63_domain[63] = b'.';
        label_63_domain[64] = b'c';
        label_63_domain[65] = b'o';
        label_63_domain[66] = b'm';
        assert!(is_valid_email_domain_part(&label_63_domain));

        let mut label_64_domain: [u8; INQUIRY_EMAIL_LABEL_MAX_CHARS + 5] =
            [b'a'; INQUIRY_EMAIL_LABEL_MAX_CHARS + 5];
        label_64_domain[64] = b'.';
        label_64_domain[65] = b'c';
        label_64_domain[66] = b'o';
        label_64_domain[67] = b'm';
        assert!(!is_valid_email_domain_part(&label_64_domain));

        // ドメイン全体上限超過（254文字）およびメール全体上限超過（255文字）の即時拒否
        let domain_254: [u8; INQUIRY_EMAIL_DOMAIN_MAX_CHARS + 1] =
            [b'a'; INQUIRY_EMAIL_DOMAIN_MAX_CHARS + 1];
        assert!(!is_valid_email_domain_part(&domain_254));
        assert!(!is_valid_email_syntax_bytes(
            &[b'a'; INQUIRY_EMAIL_MAX_CHARS + 1]
        ));
    }

    /// 証明: `trim` 後 255文字のメールアドレスは `TooLong`（254文字上限超過）となり、
    /// 255文字すべて空白の場合は `Empty` が優先されること
    #[kani::proof]
    #[kani::unwind(257)]
    fn proof_inquiry_email_255_too_long_and_empty_precedence() {
        let over_255: [u8; INQUIRY_EMAIL_MAX_CHARS + 1] = [b'a'; INQUIRY_EMAIL_MAX_CHARS + 1];
        assert_eq!(
            validate_email_utf8(&over_255),
            Err(TextValidationError::TooLong)
        );

        let spaces_255: [u8; INQUIRY_EMAIL_MAX_CHARS + 1] = [b' '; INQUIRY_EMAIL_MAX_CHARS + 1];
        assert_eq!(
            validate_email_utf8(&spaces_255),
            Err(TextValidationError::Empty)
        );
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::*;

    /// 指定文字数の正当なテスト用メールアドレス（`local` 64文字 + `@` + ドメインラベル分割 + `.com`）を構築する
    fn build_valid_email_of_len(total_len: usize) -> String {
        assert!(total_len >= 70);
        let local: String = "a".repeat(64);
        let domain_len: usize = total_len - 65; // '@' を除いた長さ
        // 末尾 ".com" (4文字) を除いた残りを 63 文字以下のラベルに分割
        let rem: usize = domain_len - 4;
        let label1: String = "b".repeat(63);
        let label2: String = "c".repeat(63);
        let label3_len: usize = rem - 63 - 1 - 63 - 1;
        let label3: String = "d".repeat(label3_len);
        format!("{local}@{label1}.{label2}.{label3}.com")
    }

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
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new("山田 太郎");
        assert!(result.is_ok());
    }

    #[test]
    fn 氏名がtrim前ちょうど100文字の場合成功する() {
        let value: String = format!(" {}", "あ".repeat(99));
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(value.clone());
        assert_eq!(result.unwrap().value(), value);
    }

    #[test]
    fn 氏名がtrim後100文字でもtrim前101文字の場合エラーになる() {
        let value: String = format!(" {}", "あ".repeat(100));
        let result: Result<InquiryName, InquiryDomainError> = InquiryName::new(value);
        assert_eq!(result, Err(InquiryDomainError::NameTooLong));
    }

    #[test]
    fn 氏名に改行やnul等の制御文字が含まれる場合エラーになる() {
        assert_eq!(
            InquiryName::new("山田\n太郎"),
            Err(InquiryDomainError::InvalidName)
        );
        assert_eq!(
            InquiryName::new("山田\0太郎"),
            Err(InquiryDomainError::InvalidName)
        );
        assert_eq!(
            InquiryName::new("山田\t太郎"),
            Err(InquiryDomainError::InvalidName)
        );
    }

    #[test]
    fn メールアドレスが空または空白のみの場合エラーになる() {
        assert_eq!(InquiryEmail::new(""), Err(InquiryDomainError::EmptyEmail));
        assert_eq!(
            InquiryEmail::new("  \u{3000}\t\n "),
            Err(InquiryDomainError::EmptyEmail)
        );
    }

    #[test]
    fn メールアドレスの前後空白はtrimされて正規化される() {
        let email: InquiryEmail = InquiryEmail::new("  \u{3000}test@example.com\n ").unwrap();
        assert_eq!(email.value(), "test@example.com");
    }

    #[test]
    fn メールアドレスがtrim後ちょうど254文字の場合成功する() {
        let raw_254: String = build_valid_email_of_len(254);
        assert_eq!(raw_254.chars().count(), 254);

        let padded: String = format!("  {raw_254}  ");
        let email: InquiryEmail = InquiryEmail::new(padded).unwrap();
        assert_eq!(email.value(), raw_254);
    }

    #[test]
    fn メールアドレスがtrim後255文字以上の場合エラーになる() {
        let raw_255: String = build_valid_email_of_len(255);
        assert_eq!(raw_255.chars().count(), 255);

        let result: Result<InquiryEmail, InquiryDomainError> = InquiryEmail::new(raw_255);
        assert_eq!(result, Err(InquiryDomainError::EmailTooLong));
    }

    #[test]
    fn メールアドレスの構文が不正な場合エラーになる() {
        let invalid_cases: [&str; 12] = [
            "invalid-email",
            "@example.com",
            "user@",
            "user@@example.com",
            ".user@example.com",
            "user.@example.com",
            "u..ser@example.com",
            "user@localhost",
            "user@-example.com",
            "user@example-.com",
            "user@example.c",
            "user@192.168.1.1",
        ];

        for case in invalid_cases {
            assert_eq!(
                InquiryEmail::new(case),
                Err(InquiryDomainError::InvalidEmail),
                "case `{case}` should be rejected as InvalidEmail"
            );
        }

        let local_65: String = format!("{}@example.com", "a".repeat(65));
        assert_eq!(
            InquiryEmail::new(local_65),
            Err(InquiryDomainError::InvalidEmail)
        );
    }

    #[test]
    fn 本文が空の場合エラーになる() {
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new("");
        assert_eq!(result, Err(InquiryDomainError::EmptyMessage));
    }

    #[test]
    fn 本文が改行やタブを含み5000文字以内の場合成功する() {
        let result: Result<InquiryMessage, InquiryDomainError> =
            InquiryMessage::new("お問い合わせです。\n\tどうぞよろしくお願いいたします。");
        assert!(result.is_ok());
    }

    #[test]
    fn 本文にnul等の禁止制御文字が含まれる場合エラーになる() {
        assert_eq!(
            InquiryMessage::new("本文\0テスト"),
            Err(InquiryDomainError::InvalidMessage)
        );
        assert_eq!(
            InquiryMessage::new("本文\x07テスト"),
            Err(InquiryDomainError::InvalidMessage)
        );
    }

    #[test]
    fn 本文がtrim前ちょうど5000文字の場合成功する() {
        let value: String = format!(" {}", "あ".repeat(4999));
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(value);
        assert!(result.is_ok());
    }

    #[test]
    fn 本文がtrim後5000文字でもtrim前5001文字の場合エラーになる() {
        let value: String = format!(" {}", "あ".repeat(5000));
        let result: Result<InquiryMessage, InquiryDomainError> = InquiryMessage::new(value);
        assert_eq!(result, Err(InquiryDomainError::MessageTooLong));
    }
}
