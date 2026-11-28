//! # 問い合わせテキスト検証コア（`no_std` / 形式検証対象）
//!
//! 問い合わせドメインの値オブジェクト（氏名・メールアドレス・本文）のバリデーションルールを、
//! `std` および `alloc` に一切依存しない純粋な `no_std` コアロジックとして提供する。
//!
//! ## 項目別バリデーション仕様
//!
//! | 項目 | 前後空白の扱い | 文字数上限の検証対象 | 上限値 | 形式・禁止文字ルール |
//! |---|---|---|---|---|
//! | `Name`（氏名） | 保持（非トリム） | **`trim`前の文字列** | `100` 文字 | 空・空白のみ不可、全 C0/C1 制御文字（`\0`, `\r`, `\n`, `\t` 等）禁止 |
//! | `Email`（メールアドレス） | 前後空白を `trim` 正規化 | **`trim`後の文字列** | `254` 文字 | 空不可、RFC 5321/5322 `dot-atom` + RFC 1123 ドメイン構文（`local` 1..=64文字、`domain` 1..=253文字・2ラベル以上・各ラベル 1..=63文字・TLD 2文字以上かつ英字含有） |
//! | `Message`（本文） | 保持（非トリム） | **`trim`前の文字列** | `5000` 文字 | 空・空白のみ不可、`\t`・`\n`・`\r` 以外の全 C0/C1 制御文字（NUL `\0` 等）禁止 |
//!
//! ## 形式検証（Formal Verification）
//!
//! | ツール | 手法 | 主な検証内容 |
//! |---|---|---|
//! | **Verus** | SMT（Z3）による演繹的検証 | 任意長入力に対する UTF-8 走査・単一パス `trim` 範囲計算・制御文字排除・254文字メール構文検証の安全性・停止性・境界厳密性（`--no-cheating`） |
//! | **Kani** | CBMC によるビット精密モデル検査 | 全 Unicode スカラー値（`char`）での `char::is_whitespace` / `char::is_control` 完全一致、不正 UTF-8 生バイト耐性、メール構文および `trim` 前後境界の網羅検証 |

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

/// 氏名の最大文字数（`trim`前の Unicode スカラー値単位）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_NAME_MAX_CHARS: usize = 100;

/// メールアドレスの最大文字数（`trim`後の文字数・RFC 5321 §4.5.3.1.3 / Errata 1690 準拠）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_EMAIL_MAX_CHARS: usize = 254;

/// メールアドレスのローカル部（`@` より前）の最大文字数（RFC 5321 §4.5.3.1.1）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_EMAIL_LOCAL_MAX_CHARS: usize = 64;

/// メールアドレスのドメイン部（`@` より後）の最大文字数
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_EMAIL_DOMAIN_MAX_CHARS: usize = 253;

/// メールアドレスのドメイン各ラベルの最大文字数（RFC 1035 §2.3.4）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_EMAIL_LABEL_MAX_CHARS: usize = 63;

/// 問い合わせ本文の最大文字数（`trim`前の Unicode スカラー値単位）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_MESSAGE_MAX_CHARS: usize = 5000;

/// 問い合わせテキスト項目の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InquiryField {
    /// 氏名（`trim`前 最大100文字）
    Name,
    /// メールアドレス（`trim`後 最大254文字）
    Email,
    /// 問い合わせ本文（`trim`前 最大5000文字）
    Message,
}

/// テキストバリデーションエラー（値オブジェクト共通コア）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextValidationError {
    /// 空文字または空白文字のみ
    Empty,
    /// 最大文字数超過
    TooLong,
    /// 形式不正または禁止制御文字の混入
    InvalidFormat,
}

#[cfg(verus_keep_ghost)]
verus! {

#[verifier::external_type_specification]
pub struct ExInquiryField(InquiryField);

#[verifier::external_type_specification]
pub struct ExTextValidationError(TextValidationError);

/// 問い合わせ項目ごとの最大文字数仕様
pub open spec fn field_max_chars_spec(field: InquiryField) -> usize {
    match field {
        InquiryField::Name => INQUIRY_NAME_MAX_CHARS,
        InquiryField::Email => INQUIRY_EMAIL_MAX_CHARS,
        InquiryField::Message => INQUIRY_MESSAGE_MAX_CHARS,
    }
}

/// Unicode `White_Space` プロパティ仕様（Rust `char::is_whitespace` と同一集合）
pub open spec fn is_unicode_whitespace_codepoint_spec(codepoint: u32) -> bool {
    (0x0009u32 <= codepoint && codepoint <= 0x000Du32)
        || codepoint == 0x0020u32
        || codepoint == 0x0085u32
        || codepoint == 0x00A0u32
        || codepoint == 0x1680u32
        || (0x2000u32 <= codepoint && codepoint <= 0x200Au32)
        || codepoint == 0x2028u32
        || codepoint == 0x2029u32
        || codepoint == 0x202Fu32
        || codepoint == 0x205Fu32
        || codepoint == 0x3000u32
}

/// Unicode `Cc`（C0/C1 制御文字）仕様（Rust `char::is_control` と同一集合）
pub open spec fn is_control_codepoint_spec(codepoint: u32) -> bool {
    codepoint <= 0x001Fu32 || (0x007Fu32 <= codepoint && codepoint <= 0x009Fu32)
}

/// 項目別に禁止される制御文字の仕様
pub open spec fn is_disallowed_control_for_field_spec(field: InquiryField, codepoint: u32) -> bool {
    match field {
        InquiryField::Name => is_control_codepoint_spec(codepoint),
        InquiryField::Email => is_control_codepoint_spec(codepoint),
        InquiryField::Message => {
            is_control_codepoint_spec(codepoint)
                && codepoint != 0x0009u32
                && codepoint != 0x000Au32
                && codepoint != 0x000Du32
        }
    }
}

/// メトリクス単位のテキスト妥当性仕様（`trim`前文字数・禁止制御文字考慮）
pub open spec fn is_valid_text_metrics_spec(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
    has_disallowed_control: bool,
) -> bool {
    has_non_whitespace && char_count > 0 && char_count <= max_chars && !has_disallowed_control
}

/// 指定位置からの1文字分の UTF-8 デコード仕様（コードポイントと消費バイト数）
pub open spec fn decode_utf8_codepoint_spec(bytes: Seq<u8>, index: int) -> (u32, nat)
    recommends
        0 <= index < bytes.len(),
{
    let b0: u8 = bytes[index];
    if b0 < 0x80u8 {
        (b0 as u32, 1nat)
    } else if (b0 & 0xE0u8) == 0xC0u8 && bytes.len() - index >= 2 {
        let b1: u8 = bytes[index + 1];
        let cp: u32 = (((b0 & 0x1Fu8) as u32) << 6) | ((b1 & 0x3Fu8) as u32);
        (cp, 2nat)
    } else if (b0 & 0xF0u8) == 0xE0u8 && bytes.len() - index >= 3 {
        let b1: u8 = bytes[index + 1];
        let b2: u8 = bytes[index + 2];
        let cp: u32 = (((b0 & 0x0Fu8) as u32) << 12)
            | (((b1 & 0x3Fu8) as u32) << 6)
            | ((b2 & 0x3Fu8) as u32);
        (cp, 3nat)
    } else if (b0 & 0xF8u8) == 0xF0u8 && bytes.len() - index >= 4 {
        let b1: u8 = bytes[index + 1];
        let b2: u8 = bytes[index + 2];
        let b3: u8 = bytes[index + 3];
        let cp: u32 = (((b0 & 0x07u8) as u32) << 18)
            | (((b1 & 0x3Fu8) as u32) << 12)
            | (((b2 & 0x3Fu8) as u32) << 6)
            | ((b3 & 0x3Fu8) as u32);
        (cp, 4nat)
    } else {
        (0xFFFDu32, 1nat)
    }
}

/// 指定位置以降の UTF-8 文字数仕様
pub open spec fn utf8_char_count_from_spec(bytes: Seq<u8>, index: int) -> nat
    decreases bytes.len() - index,
{
    if 0 <= index < bytes.len() {
        let (_cp, step): (u32, nat) = decode_utf8_codepoint_spec(bytes, index);
        1nat + utf8_char_count_from_spec(bytes, index + step)
    } else {
        0nat
    }
}

/// 指定位置以降に非空白文字が存在するかの仕様
pub open spec fn utf8_has_non_whitespace_from_spec(bytes: Seq<u8>, index: int) -> bool
    decreases bytes.len() - index,
{
    if 0 <= index < bytes.len() {
        let (cp, step): (u32, nat) = decode_utf8_codepoint_spec(bytes, index);
        !is_unicode_whitespace_codepoint_spec(cp)
            || utf8_has_non_whitespace_from_spec(bytes, index + step)
    } else {
        false
    }
}

/// 指定位置以降に項目別の禁止制御文字が存在するかの仕様
pub open spec fn utf8_has_disallowed_control_from_spec(
    field: InquiryField,
    bytes: Seq<u8>,
    index: int,
) -> bool
    decreases bytes.len() - index,
{
    if 0 <= index < bytes.len() {
        let (cp, step): (u32, nat) = decode_utf8_codepoint_spec(bytes, index);
        is_disallowed_control_for_field_spec(field, cp)
            || utf8_has_disallowed_control_from_spec(field, bytes, index + step)
    } else {
        false
    }
}

/// `char` スライス内に非空白文字が存在するかの仕様
pub open spec fn chars_has_non_whitespace_spec(chars: Seq<char>) -> bool {
    exists|i: int| 0 <= i < chars.len() && !is_unicode_whitespace_codepoint_spec(#[trigger] chars[i] as u32)
}

}

impl InquiryField {
    /// 項目ごとの最大文字数を返す
    #[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
        ensures
            res == field_max_chars_spec(self),
            res > 0,
    ))]
    pub const fn max_chars(self) -> usize {
        match self {
            Self::Name => INQUIRY_NAME_MAX_CHARS,
            Self::Email => INQUIRY_EMAIL_MAX_CHARS,
            Self::Message => INQUIRY_MESSAGE_MAX_CHARS,
        }
    }
}

/// コードポイントが Unicode `White_Space` プロパティに該当するか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> is_unicode_whitespace_codepoint_spec(codepoint),
))]
pub const fn is_unicode_whitespace_codepoint(codepoint: u32) -> bool {
    matches!(
        codepoint,
        0x0009..=0x000D
            | 0x0020
            | 0x0085
            | 0x00A0
            | 0x1680
            | 0x2000..=0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
    )
}

/// `char` が Unicode 空白文字か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> is_unicode_whitespace_codepoint_spec(ch as u32),
))]
pub const fn is_unicode_whitespace(ch: char) -> bool {
    let codepoint: u32 = ch as u32;
    is_unicode_whitespace_codepoint(codepoint)
}

/// コードポイントが C0/C1 制御文字（Unicode `Cc`、`char::is_control` 相当）か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> is_control_codepoint_spec(codepoint),
))]
pub const fn is_control_codepoint(codepoint: u32) -> bool {
    codepoint <= 0x001F || (0x007F <= codepoint && codepoint <= 0x009F)
}

/// `char` が C0/C1 制御文字か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> is_control_codepoint_spec(ch as u32),
))]
pub const fn is_control_char(ch: char) -> bool {
    let codepoint: u32 = ch as u32;
    is_control_codepoint(codepoint)
}

/// 指定した問い合わせ項目で禁止されている制御文字か判定する
///
/// - `Name` / `Email`: 全 C0/C1 制御文字（`\0`, `\r`, `\n`, `\t` を含む）を禁止
/// - `Message`: `\t` (`0x09`), `\n` (`0x0A`), `\r` (`0x0D`) 以外の全 C0/C1 制御文字（`\0` 等）を禁止
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> is_disallowed_control_for_field_spec(field, codepoint),
))]
pub const fn is_disallowed_control_for_field(field: InquiryField, codepoint: u32) -> bool {
    match field {
        InquiryField::Name | InquiryField::Email => is_control_codepoint(codepoint),
        InquiryField::Message => {
            is_control_codepoint(codepoint)
                && codepoint != 0x0009
                && codepoint != 0x000A
                && codepoint != 0x000D
        }
    }
}

/// 算出済みメトリクス（非空白文字の有無・`trim`前文字数・禁止制御文字の有無）に対してドメイン規則を検証する
///
/// ## 優先順位
///
/// 1. 空文字または空白文字のみ → `Err(TextValidationError::Empty)`
/// 2. `char_count > max_chars` → `Err(TextValidationError::TooLong)`
/// 3. `has_disallowed_control` → `Err(TextValidationError::InvalidFormat)`
/// 4. 上記以外 → `Ok(())`
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.is_ok()
            <==> is_valid_text_metrics_spec(
                has_non_whitespace,
                char_count,
                max_chars,
                has_disallowed_control,
            ),
        res == Err(TextValidationError::Empty) <==> (!has_non_whitespace || char_count == 0),
        res == Err(TextValidationError::TooLong)
            <==> (has_non_whitespace && char_count > 0 && char_count > max_chars),
        res == Err(TextValidationError::InvalidFormat) <==> (
            has_non_whitespace
                && char_count > 0
                && char_count <= max_chars
                && has_disallowed_control
        ),
))]
pub const fn validate_text_metrics(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
    has_disallowed_control: bool,
) -> Result<(), TextValidationError> {
    if !has_non_whitespace || char_count == 0 {
        return Err(TextValidationError::Empty);
    }

    if char_count > max_chars {
        return Err(TextValidationError::TooLong);
    }

    if has_disallowed_control {
        return Err(TextValidationError::InvalidFormat);
    }

    Ok(())
}

/// UTF-8 バイト列の指定位置から 1 文字分のコードポイントと消費バイト数をデコードする
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    requires
        index < bytes.len(),
    ensures
        res.0 == decode_utf8_codepoint_spec(bytes@, index as int).0,
        res.1 as nat == decode_utf8_codepoint_spec(bytes@, index as int).1,
        1 <= res.1 && res.1 <= 4,
        index + res.1 <= bytes.len(),
))]
pub fn decode_utf8_codepoint_at(bytes: &[u8], index: usize) -> (u32, usize) {
    let remaining: usize = bytes.len() - index;
    let b0: u8 = bytes[index];

    if b0 < 0x80u8 {
        return (b0 as u32, 1);
    }

    if (b0 & 0xE0u8) == 0xC0u8 && remaining >= 2 {
        let b1: u8 = bytes[index + 1];
        let codepoint: u32 = (((b0 & 0x1Fu8) as u32) << 6) | ((b1 & 0x3Fu8) as u32);
        return (codepoint, 2);
    }

    if (b0 & 0xF0u8) == 0xE0u8 && remaining >= 3 {
        let b1: u8 = bytes[index + 1];
        let b2: u8 = bytes[index + 2];
        let codepoint: u32 =
            (((b0 & 0x0Fu8) as u32) << 12) | (((b1 & 0x3Fu8) as u32) << 6) | ((b2 & 0x3Fu8) as u32);
        return (codepoint, 3);
    }

    if (b0 & 0xF8u8) == 0xF0u8 && remaining >= 4 {
        let b1: u8 = bytes[index + 1];
        let b2: u8 = bytes[index + 2];
        let b3: u8 = bytes[index + 3];
        let codepoint: u32 = (((b0 & 0x07u8) as u32) << 18)
            | (((b1 & 0x3Fu8) as u32) << 12)
            | (((b2 & 0x3Fu8) as u32) << 6)
            | ((b3 & 0x3Fu8) as u32);
        return (codepoint, 4);
    }

    (0xFFFDu32, 1)
}

/// UTF-8 バイト列を単一パスで走査し、`（非空白文字の有無, 文字数）` を算出する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.0 == utf8_has_non_whitespace_from_spec(bytes@, 0),
        res.1 as nat == utf8_char_count_from_spec(bytes@, 0),
        res.1 <= bytes.len(),
        res.0 ==> res.1 > 0,
        bytes.len() == 0 ==> (!res.0 && res.1 == 0),
))]
pub fn scan_utf8_bytes(bytes: &[u8]) -> (bool, usize) {
    let mut index: usize = 0;
    let mut has_non_whitespace: bool = false;
    let mut char_count: usize = 0;

    while index < bytes.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                0 <= index <= bytes.len(),
                char_count <= index,
                has_non_whitespace ==> char_count > 0,
                char_count as nat + utf8_char_count_from_spec(bytes@, index as int)
                    == utf8_char_count_from_spec(bytes@, 0),
                (has_non_whitespace || utf8_has_non_whitespace_from_spec(bytes@, index as int))
                    == utf8_has_non_whitespace_from_spec(bytes@, 0),
            ]);
            decreases(bytes.len() - index);
        });

        let decoded: (u32, usize) = decode_utf8_codepoint_at(bytes, index);
        let codepoint: u32 = decoded.0;
        let step: usize = decoded.1;

        if !is_unicode_whitespace_codepoint(codepoint) {
            has_non_whitespace = true;
        }

        char_count += 1;
        index += step;
    }

    (has_non_whitespace, char_count)
}

/// 項目別に `trim` 前の UTF-8 バイト列を単一パスで走査し、
/// `(非空白文字の有無, trim前文字数, 禁止制御文字の有無)` を算出する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.0 == utf8_has_non_whitespace_from_spec(bytes@, 0),
        res.1 as nat == utf8_char_count_from_spec(bytes@, 0),
        res.2 == utf8_has_disallowed_control_from_spec(field, bytes@, 0),
        res.1 <= bytes.len(),
        res.0 ==> res.1 > 0,
))]
pub fn scan_utf8_for_field(field: InquiryField, bytes: &[u8]) -> (bool, usize, bool) {
    let mut index: usize = 0;
    let mut has_non_whitespace: bool = false;
    let mut char_count: usize = 0;
    let mut has_disallowed_control: bool = false;

    while index < bytes.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                0 <= index <= bytes.len(),
                char_count <= index,
                has_non_whitespace ==> char_count > 0,
                char_count as nat + utf8_char_count_from_spec(bytes@, index as int)
                    == utf8_char_count_from_spec(bytes@, 0),
                (has_non_whitespace || utf8_has_non_whitespace_from_spec(bytes@, index as int))
                    == utf8_has_non_whitespace_from_spec(bytes@, 0),
                (has_disallowed_control
                    || utf8_has_disallowed_control_from_spec(field, bytes@, index as int))
                    == utf8_has_disallowed_control_from_spec(field, bytes@, 0),
            ]);
            decreases(bytes.len() - index);
        });

        let decoded: (u32, usize) = decode_utf8_codepoint_at(bytes, index);
        let codepoint: u32 = decoded.0;
        let step: usize = decoded.1;

        if !is_unicode_whitespace_codepoint(codepoint) {
            has_non_whitespace = true;
        }
        if is_disallowed_control_for_field(field, codepoint) {
            has_disallowed_control = true;
        }

        char_count += 1;
        index += step;
    }

    (has_non_whitespace, char_count, has_disallowed_control)
}

/// 単一の前方 UTF-8 走査によって、前後の Unicode 空白を除いたバイト範囲 `(start, end)` を算出する
///
/// 非空白文字が存在しない場合は `(0, 0)` を返す。
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.0 <= res.1,
        res.1 <= bytes.len(),
        (res.0 == res.1) <==> !utf8_has_non_whitespace_from_spec(bytes@, 0),
        utf8_has_non_whitespace_from_spec(bytes@, 0) ==> (
            res.0 < res.1
                && !is_unicode_whitespace_codepoint_spec(
                    decode_utf8_codepoint_spec(bytes@, res.0 as int).0,
                )
        ),
))]
pub fn trim_utf8_range(bytes: &[u8]) -> (usize, usize) {
    let mut index: usize = 0;
    let mut has_non_whitespace: bool = false;
    let mut first_start: usize = 0;
    let mut last_end: usize = 0;

    while index < bytes.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                0 <= index <= bytes.len(),
                !has_non_whitespace ==> (first_start == 0 && last_end == 0),
                has_non_whitespace ==> (first_start < last_end && last_end <= index),
                (has_non_whitespace || utf8_has_non_whitespace_from_spec(bytes@, index as int))
                    == utf8_has_non_whitespace_from_spec(bytes@, 0),
                has_non_whitespace ==> (
                    first_start < bytes.len()
                        && !is_unicode_whitespace_codepoint_spec(
                            decode_utf8_codepoint_spec(bytes@, first_start as int).0,
                        )
                ),
            ]);
            decreases(bytes.len() - index);
        });

        let decoded: (u32, usize) = decode_utf8_codepoint_at(bytes, index);
        let codepoint: u32 = decoded.0;
        let step: usize = decoded.1;
        let next_index: usize = index + step;

        if !is_unicode_whitespace_codepoint(codepoint) {
            if !has_non_whitespace {
                first_start = index;
                has_non_whitespace = true;
            }
            last_end = next_index;
        }

        index = next_index;
    }

    (first_start, last_end)
}

/// ASCII 英字（`A-Z` または `a-z`）か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> ((0x41u8 <= b && b <= 0x5Au8) || (0x61u8 <= b && b <= 0x7Au8)),
))]
pub const fn is_ascii_alpha_byte(b: u8) -> bool {
    (b'A' <= b && b <= b'Z') || (b'a' <= b && b <= b'z')
}

/// ASCII 数字（`0-9`）か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res <==> (0x30u8 <= b && b <= 0x39u8),
))]
pub const fn is_ascii_digit_byte(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

/// RFC 5322 §3.2.3 の `atext`（ローカル部で使用可能な非ドット ASCII 文字）か判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res ==> (0x21u8 <= b && b <= 0x7Eu8 && b != 0x40u8 && b != 0x2Eu8),
))]
pub const fn is_email_atext_byte(b: u8) -> bool {
    is_ascii_alpha_byte(b)
        || is_ascii_digit_byte(b)
        || matches!(
            b,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'/'
                | b'='
                | b'?'
                | b'^'
                | b'_'
                | b'`'
                | b'{'
                | b'|'
                | b'}'
                | b'~'
        )
}

/// 全バイトが空白を含まない印字可能 ASCII（`0x21..=0x7E`）であり、
/// かつ `@`（`0x40`）がちょうど1つだけ存在する場合にその位置を返す
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        match res {
            Some(pos) => {
                pos < bytes.len()
                    && bytes@[pos as int] == 0x40u8
                    && (forall|k: int|
                        0 <= k < bytes@.len()
                            ==> (0x21u8 <= #[trigger] bytes@[k] && bytes@[k] <= 0x7Eu8))
                    && (forall|k: int|
                        0 <= k < bytes@.len() && k != pos as int
                            ==> #[trigger] bytes@[k] != 0x40u8)
            }
            None => true,
        },
))]
pub fn find_unique_at_and_check_printable_ascii(bytes: &[u8]) -> Option<usize> {
    let mut index: usize = 0;
    let mut found_at: bool = false;
    let mut at_pos: usize = 0;

    while index < bytes.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                0 <= index <= bytes.len(),
                forall|k: int|
                    0 <= k < index ==> (0x21u8 <= #[trigger] bytes@[k] && bytes@[k] <= 0x7Eu8),
                !found_at ==> forall|k: int| 0 <= k < index ==> #[trigger] bytes@[k] != 0x40u8,
                found_at ==> (
                    at_pos < index
                        && bytes@[at_pos as int] == 0x40u8
                        && forall|k: int|
                            0 <= k < index && k != at_pos as int
                                ==> #[trigger] bytes@[k] != 0x40u8
                ),
            ]);
            decreases(bytes.len() - index);
        });

        let b: u8 = bytes[index];
        if !matches!(b, 0x21u8..=0x7Eu8) {
            return None;
        }

        if b == b'@' {
            if found_at {
                return None;
            }
            found_at = true;
            at_pos = index;
        }

        index += 1;
    }

    if found_at { Some(at_pos) } else { None }
}

/// メールアドレスのローカル部（`dot-atom-text`、`1..=64` 文字）を検証する
///
/// - 長さ `1..=64` バイト
/// - `atext` または `.` のみで構成
/// - 先頭 `.`、末尾 `.`、連続する `..` を禁止
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res ==> (
            1 <= local.len()
                && local.len() <= INQUIRY_EMAIL_LOCAL_MAX_CHARS
                && local@[0] != 0x2Eu8
                && local@[local.len() - 1] != 0x2Eu8
        ),
))]
pub fn is_valid_email_local_part(local: &[u8]) -> bool {
    let len: usize = local.len();
    if len == 0 || len > INQUIRY_EMAIL_LOCAL_MAX_CHARS {
        return false;
    }

    if local[0] == b'.' || local[len - 1] == b'.' {
        return false;
    }

    let mut index: usize = 0;
    while index < len {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                len == local.len(),
                0 <= index <= len,
                1 <= len <= INQUIRY_EMAIL_LOCAL_MAX_CHARS,
            ]);
            decreases(len - index);
        });

        let b: u8 = local[index];
        if b == b'.' {
            if index + 1 < len && local[index + 1] == b'.' {
                return false;
            }
        } else if !is_email_atext_byte(b) {
            return false;
        }

        index += 1;
    }

    true
}

/// メールアドレスのドメイン部（RFC 1035 / RFC 1123 ラベル列、`4..=253` 文字）を検証する
///
/// - 長さ `4..=253` バイト（最小構成 `a.bc`）
/// - `.` 区切りで **2ラベル以上** 存在すること
/// - 各ラベルは `1..=63` 文字の ASCII 英数字または `-` であり、先頭・末尾の `-` を禁止
/// - 最上位ラベル（末尾 TLD）は **2文字以上** かつ **ASCII 英字を1文字以上含む** こと
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res ==> (
            4 <= domain.len()
                && domain.len() <= INQUIRY_EMAIL_DOMAIN_MAX_CHARS
                && domain@[0] != 0x2Eu8
                && domain@[0] != 0x2Du8
        ),
))]
pub fn is_valid_email_domain_part(domain: &[u8]) -> bool {
    let len: usize = domain.len();
    if !matches!(len, 4..=INQUIRY_EMAIL_DOMAIN_MAX_CHARS) {
        return false;
    }

    if domain[0] == b'.' || domain[0] == b'-' {
        return false;
    }

    let mut index: usize = 0;
    let mut dot_count: usize = 0;
    let mut label_len: usize = 0;
    let mut label_has_alpha: bool = false;
    let mut prev_was_hyphen: bool = false;

    while index < len {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                len == domain.len(),
                0 <= index <= len,
                4 <= len <= INQUIRY_EMAIL_DOMAIN_MAX_CHARS,
                dot_count <= index,
                label_len <= INQUIRY_EMAIL_LABEL_MAX_CHARS,
            ]);
            decreases(len - index);
        });

        let b: u8 = domain[index];
        if b == b'.' {
            if label_len == 0 || prev_was_hyphen {
                return false;
            }
            dot_count += 1;
            label_len = 0;
            label_has_alpha = false;
            prev_was_hyphen = false;
        } else if is_ascii_alpha_byte(b) {
            if label_len >= INQUIRY_EMAIL_LABEL_MAX_CHARS {
                return false;
            }
            label_len += 1;
            label_has_alpha = true;
            prev_was_hyphen = false;
        } else if is_ascii_digit_byte(b) {
            if label_len >= INQUIRY_EMAIL_LABEL_MAX_CHARS {
                return false;
            }
            label_len += 1;
            prev_was_hyphen = false;
        } else if b == b'-' {
            if label_len == 0 || label_len >= INQUIRY_EMAIL_LABEL_MAX_CHARS {
                return false;
            }
            label_len += 1;
            prev_was_hyphen = true;
        } else {
            return false;
        }

        index += 1;
    }

    dot_count >= 1 && label_len >= 2 && !prev_was_hyphen && label_has_alpha
}

/// トリム済みバイト列が RFC 5321 / 5322 実運用プロファイルのメールアドレス構文を満たすか判定する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res ==> (
            6 <= bytes.len()
                && bytes.len() <= INQUIRY_EMAIL_MAX_CHARS
                && forall|k: int|
                    0 <= k < bytes@.len()
                        ==> (0x21u8 <= #[trigger] bytes@[k] && bytes@[k] <= 0x7Eu8)
        ),
))]
pub fn is_valid_email_syntax_bytes(bytes: &[u8]) -> bool {
    if bytes.len() > INQUIRY_EMAIL_MAX_CHARS {
        return false;
    }

    let at_pos: usize = match find_unique_at_and_check_printable_ascii(bytes) {
        Some(pos) => pos,
        None => return false,
    };

    let local: &[u8] = &bytes[..at_pos];
    let domain: &[u8] = &bytes[at_pos + 1..];

    is_valid_email_local_part(local) && is_valid_email_domain_part(domain)
}

/// メールアドレスの UTF-8 バイト列を前後 `trim` のうえ検証し、正規化済みスライス範囲 `(start, end)` を返す
///
/// ## 優先順位
///
/// 1. `trim` 後に空文字 → `Err(TextValidationError::Empty)`
/// 2. `trim` 後の文字数が `254` 文字超 → `Err(TextValidationError::TooLong)`
/// 3. メールアドレス構文違反 → `Err(TextValidationError::InvalidFormat)`
/// 4. 上記すべて通過 → `Ok((start, end))`（`6 <= end - start <= 254` かつ全バイトが ASCII `0x21..=0x7E`）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res == Err(TextValidationError::Empty)
            <==> !utf8_has_non_whitespace_from_spec(bytes@, 0),
        match res {
            Ok((start, end)) => {
                start < end
                    && end <= bytes.len()
                    && 6 <= end - start
                    && end - start <= INQUIRY_EMAIL_MAX_CHARS
                    && forall|k: int|
                        start as int <= k < end as int
                            ==> (0x21u8 <= #[trigger] bytes@[k] && bytes@[k] <= 0x7Eu8)
            }
            Err(_) => true,
        },
))]
pub fn validate_email_utf8(bytes: &[u8]) -> Result<(usize, usize), TextValidationError> {
    let range: (usize, usize) = trim_utf8_range(bytes);
    let start: usize = range.0;
    let end: usize = range.1;

    if start == end {
        return Err(TextValidationError::Empty);
    }

    let trimmed: &[u8] = &bytes[start..end];
    let trimmed_metrics: (bool, usize) = scan_utf8_bytes(trimmed);
    let trimmed_char_count: usize = trimmed_metrics.1;

    if trimmed_char_count > INQUIRY_EMAIL_MAX_CHARS {
        return Err(TextValidationError::TooLong);
    }

    if !is_valid_email_syntax_bytes(trimmed) {
        return Err(TextValidationError::InvalidFormat);
    }

    #[cfg(verus_keep_ghost)]
    verus_builtin_macros::verus_proof_expr!({
        proof! {
            assert forall|k: int| start as int <= k < end as int implies (
                0x21u8 <= #[trigger] bytes@[k] && bytes@[k] <= 0x7Eu8
            ) by {
                let idx: int = k - start as int;
                assert(0 <= idx && idx < trimmed@.len());
                assert(trimmed@[idx] == bytes@[k]);
            }
        }
    });

    Ok((start, end))
}

/// メールアドレス文字列を前後 `trim` のうえ検証し、正規化済み `&str` スライスを返す
pub fn validate_and_trim_email_str(value: &str) -> Result<&str, TextValidationError> {
    let range: (usize, usize) = validate_email_utf8(value.as_bytes())?;
    Ok(&value[range.0..range.1])
}

/// UTF-8 バイト列に対して汎用テキストバリデーション（`trim`前文字数上限・制御文字検証なし）を実行する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.is_ok() <==> (
            utf8_has_non_whitespace_from_spec(bytes@, 0)
                && utf8_char_count_from_spec(bytes@, 0) <= max_chars as nat
        ),
        res == Err(TextValidationError::Empty)
            <==> !utf8_has_non_whitespace_from_spec(bytes@, 0),
        res == Err(TextValidationError::TooLong) <==> (
            utf8_has_non_whitespace_from_spec(bytes@, 0)
                && utf8_char_count_from_spec(bytes@, 0) > max_chars as nat
        ),
))]
pub fn validate_utf8_bytes(bytes: &[u8], max_chars: usize) -> Result<(), TextValidationError> {
    let metrics: (bool, usize) = scan_utf8_bytes(bytes);
    let has_non_whitespace: bool = metrics.0;
    let char_count: usize = metrics.1;

    validate_text_metrics(has_non_whitespace, char_count, max_chars, false)
}

/// `char` スライスに対して汎用テキストバリデーションを実行する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.is_ok() <==> (chars_has_non_whitespace_spec(chars@) && chars@.len() <= max_chars as nat),
        res == Err(TextValidationError::Empty) <==> !chars_has_non_whitespace_spec(chars@),
        res == Err(TextValidationError::TooLong)
            <==> (chars_has_non_whitespace_spec(chars@) && chars@.len() > max_chars as nat),
))]
pub fn validate_chars(chars: &[char], max_chars: usize) -> Result<(), TextValidationError> {
    let mut index: usize = 0;
    let mut has_non_whitespace: bool = false;

    while index < chars.len() {
        #[cfg(verus_keep_ghost)]
        verus_builtin_macros::verus_proof_expr!({
            invariant([
                0 <= index <= chars.len(),
                has_non_whitespace <==> exists|j: int|
                    0 <= j < index && !is_unicode_whitespace_codepoint_spec(#[trigger] chars@[j] as u32),
            ]);
            decreases(chars.len() - index);
        });

        let ch: char = chars[index];
        if !is_unicode_whitespace(ch) {
            has_non_whitespace = true;
        }
        index += 1;
    }

    validate_text_metrics(has_non_whitespace, chars.len(), max_chars, false)
}

/// 問い合わせ項目種別と UTF-8 バイト列に対してドメインバリデーションを実行する
///
/// - `Name` / `Message`: **`trim`前の文字列** の文字数を上限（`100` / `5000`）と比較し、禁止制御文字を拒否する
/// - `Email`: 前後の Unicode 空白を `trim` した文字列の文字数を上限（`254`）と比較し、メール構文を検証する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res == Err(TextValidationError::Empty)
            <==> !utf8_has_non_whitespace_from_spec(bytes@, 0),
        (field == InquiryField::Name || field == InquiryField::Message) ==> (
            res.is_ok() <==> (
                utf8_has_non_whitespace_from_spec(bytes@, 0)
                    && utf8_char_count_from_spec(bytes@, 0) <= field_max_chars_spec(field) as nat
                    && !utf8_has_disallowed_control_from_spec(field, bytes@, 0)
            )
        ),
        (field == InquiryField::Name || field == InquiryField::Message) ==> (
            res == Err(TextValidationError::TooLong) <==> (
                utf8_has_non_whitespace_from_spec(bytes@, 0)
                    && utf8_char_count_from_spec(bytes@, 0) > field_max_chars_spec(field) as nat
            )
        ),
        (field == InquiryField::Name || field == InquiryField::Message) ==> (
            res == Err(TextValidationError::InvalidFormat) <==> (
                utf8_has_non_whitespace_from_spec(bytes@, 0)
                    && utf8_char_count_from_spec(bytes@, 0) <= field_max_chars_spec(field) as nat
                    && utf8_has_disallowed_control_from_spec(field, bytes@, 0)
            )
        ),
))]
pub fn validate_field_utf8(field: InquiryField, bytes: &[u8]) -> Result<(), TextValidationError> {
    match field {
        InquiryField::Name | InquiryField::Message => {
            let max_chars: usize = field.max_chars();
            let scanned: (bool, usize, bool) = scan_utf8_for_field(field, bytes);
            validate_text_metrics(scanned.0, scanned.1, max_chars, scanned.2)
        }
        InquiryField::Email => match validate_email_utf8(bytes) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        },
    }
}

/// 文字列スライスに対して汎用テキストバリデーションを実行する
pub fn validate_str(value: &str, max_chars: usize) -> Result<(), TextValidationError> {
    let bytes: &[u8] = value.as_bytes();
    validate_utf8_bytes(bytes, max_chars)
}

/// 問い合わせ項目種別と文字列スライスに対してバリデーションを実行する
pub fn validate_field_str(field: InquiryField, value: &str) -> Result<(), TextValidationError> {
    let bytes: &[u8] = value.as_bytes();
    validate_field_utf8(field, bytes)
}

#[cfg(verus_keep_ghost)]
verus! {

/// 補題: 空白のみ（または空）の入力は文字数超過であっても `Empty` が優先される
pub proof fn lemma_empty_precedence_over_too_long(
    char_count: usize,
    max_chars: usize,
    has_disallowed_control: bool,
)
    requires
        char_count > max_chars,
    ensures
        !is_valid_text_metrics_spec(false, char_count, max_chars, has_disallowed_control),
{
}

/// 補題: バリデーション成功は文字数が `1..=max_chars` かつ禁止制御文字不在を含意する
pub proof fn lemma_valid_metrics_implies_bounded_positive_length(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
    has_disallowed_control: bool,
)
    requires
        is_valid_text_metrics_spec(
            has_non_whitespace,
            char_count,
            max_chars,
            has_disallowed_control,
        ),
    ensures
        has_non_whitespace,
        1 <= char_count,
        char_count <= max_chars,
        !has_disallowed_control,
{
}

/// 補題: 上限境界の厳密性（`max_chars` は妥当、`max_chars + 1` は不正）
pub proof fn lemma_boundary_exactness(max_chars: usize)
    requires
        1 <= max_chars < usize::MAX,
    ensures
        is_valid_text_metrics_spec(true, max_chars, max_chars, false),
        !is_valid_text_metrics_spec(true, (max_chars + 1) as usize, max_chars, false),
{
}

/// 補題: 最大文字数に対する単調性
pub proof fn lemma_max_chars_monotonicity(
    has_non_whitespace: bool,
    char_count: usize,
    max_a: usize,
    max_b: usize,
    has_disallowed_control: bool,
)
    requires
        is_valid_text_metrics_spec(has_non_whitespace, char_count, max_a, has_disallowed_control),
        max_a <= max_b,
    ensures
        is_valid_text_metrics_spec(has_non_whitespace, char_count, max_b, has_disallowed_control),
{
}

/// 補題: 問い合わせ各項目の上限値は厳密に順序付けられている（`0 < 100 < 254 < 5000`）
pub proof fn lemma_field_limits_strictly_ordered()
    ensures
        field_max_chars_spec(InquiryField::Name) == 100,
        field_max_chars_spec(InquiryField::Email) == 254,
        field_max_chars_spec(InquiryField::Message) == 5000,
        0 < field_max_chars_spec(InquiryField::Name),
        field_max_chars_spec(InquiryField::Name) < field_max_chars_spec(InquiryField::Email),
        field_max_chars_spec(InquiryField::Email) < field_max_chars_spec(InquiryField::Message),
{
}

/// 補題: NUL 文字（`U+0000`）は全フィールド（`Name` / `Email` / `Message`）で常に禁止され、
/// PostgreSQL の UTF-8 NUL バイトエラー（HTTP 500）を引き起こさない
pub proof fn lemma_nul_codepoint_always_disallowed(field: InquiryField)
    ensures
        is_disallowed_control_for_field_spec(field, 0u32),
{
}

/// 補題: 空の UTF-8 バイト列は常に非空白文字を持たず文字数 0 である
pub proof fn lemma_empty_utf8_has_no_non_whitespace(bytes: Seq<u8>)
    requires
        bytes.len() == 0,
    ensures
        !utf8_has_non_whitespace_from_spec(bytes, 0),
        utf8_char_count_from_spec(bytes, 0) == 0,
{
}

/// 補題: すべて Unicode 空白文字からなる `char` 列は長さによらず非空白文字を持たない
pub proof fn lemma_all_whitespace_chars_rejected(chars: Seq<char>)
    requires
        forall|i: int| 0 <= i < chars.len() ==> is_unicode_whitespace_codepoint_spec(#[trigger] chars[i] as u32),
    ensures
        !chars_has_non_whitespace_spec(chars),
{
}

/// 補題: 前後に空白があっても、少なくとも1つの非空白文字が存在すれば空白のみとは判定されない
pub proof fn lemma_non_whitespace_char_makes_slice_non_empty(chars: Seq<char>, witness: int)
    requires
        0 <= witness < chars.len(),
        !is_unicode_whitespace_codepoint_spec(chars[witness] as u32),
    ensures
        chars_has_non_whitespace_spec(chars),
{
    assert(!is_unicode_whitespace_codepoint_spec(chars[witness] as u32));
}

/// 補題: 不正・途切れたバイト列を含む任意のバイト列に対して UTF-8 1ステップ消費量は `1..=4` かつスライス範囲内に収まる
pub proof fn lemma_decode_utf8_step_bounds(bytes: Seq<u8>, index: int)
    requires
        0 <= index < bytes.len(),
    ensures
        1 <= decode_utf8_codepoint_spec(bytes, index).1,
        decode_utf8_codepoint_spec(bytes, index).1 <= 4,
        index + decode_utf8_codepoint_spec(bytes, index).1 <= bytes.len(),
{
}

}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// 証明: 全 Unicode スカラー値（`char`）において `is_unicode_whitespace` が
    /// Rust 標準の `char::is_whitespace` と完全に一致すること
    #[kani::proof]
    fn proof_unicode_whitespace_matches_core_char() {
        let ch: char = kani::any();
        assert_eq!(is_unicode_whitespace(ch), ch.is_whitespace());
    }

    /// 証明: 全 Unicode スカラー値（`char`）において `is_control_char` が
    /// Rust 標準の `char::is_control` と完全に一致し、`'\0'` を必ず検出すること
    #[kani::proof]
    fn proof_control_char_matches_core_char() {
        let ch: char = kani::any();
        assert_eq!(is_control_char(ch), ch.is_control());
        assert!(is_disallowed_control_for_field(InquiryField::Name, 0));
        assert!(is_disallowed_control_for_field(InquiryField::Email, 0));
        assert!(is_disallowed_control_for_field(InquiryField::Message, 0));
    }

    /// 証明: 全 Unicode スカラー値（1〜4バイト UTF-8 の全 `char`）において
    /// `decode_utf8_codepoint_at` が `char::encode_utf8` の完全な逆写像であること
    #[kani::proof]
    fn proof_decode_utf8_is_exact_inverse_of_encode_utf8() {
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let encoded: &str = ch.encode_utf8(&mut buf);
        let decoded: (u32, usize) = decode_utf8_codepoint_at(encoded.as_bytes(), 0);
        assert_eq!(decoded.0, ch as u32);
        assert_eq!(decoded.1, ch.len_utf8());
    }

    /// 証明: 不正な UTF-8 バイト列を含む完全に任意の生バイト列（最大 6 バイト）に対し、
    /// `scan_utf8_bytes`・`trim_utf8_range`・`validate_field_utf8` がパニックせず全域性を満たすこと
    #[kani::proof]
    #[kani::unwind(8)]
    fn proof_scan_and_validate_arbitrary_raw_bytes_never_panic() {
        let buf: [u8; 6] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= buf.len());

        let slice: &[u8] = &buf[..len];

        let metrics: (bool, usize) = scan_utf8_bytes(slice);
        assert!(metrics.1 <= len);

        let trim_range: (usize, usize) = trim_utf8_range(slice);
        assert!(trim_range.0 <= trim_range.1 && trim_range.1 <= len);
        assert_eq!(trim_range.0 == trim_range.1, !metrics.0);

        let _name_res: Result<(), TextValidationError> =
            validate_field_utf8(InquiryField::Name, slice);
        let _email_res: Result<(), TextValidationError> =
            validate_field_utf8(InquiryField::Email, slice);
        let _msg_res: Result<(), TextValidationError> =
            validate_field_utf8(InquiryField::Message, slice);
    }

    /// 証明: 任意の生バイト列（最大 8 バイト）に対して `is_valid_email_syntax_bytes` が
    /// パニックせず、成功時は `6..=254` バイトの印字可能 ASCII・単一 `@`・ドット規則を満たすこと
    #[kani::proof]
    #[kani::unwind(10)]
    fn proof_email_syntax_arbitrary_bytes_invariants() {
        let buf: [u8; 8] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= buf.len());

        let slice: &[u8] = &buf[..len];
        let valid: bool = is_valid_email_syntax_bytes(slice);

        if valid {
            assert!(len >= 6 && len <= INQUIRY_EMAIL_MAX_CHARS);
            let mut at_count: usize = 0;
            let mut i: usize = 0;
            while i < len {
                let b: u8 = slice[i];
                assert!(0x21 <= b && b <= 0x7E);
                if b == b'@' {
                    at_count += 1;
                }
                i += 1;
            }
            assert_eq!(at_count, 1);
        }
    }

    /// 証明: 任意の `(has_non_whitespace, char_count, max_chars, has_disallowed_control)` に対して
    /// `validate_text_metrics` が全分割・境界厳密性・優先順位を満たしパニックしないこと
    #[kani::proof]
    fn proof_validate_text_metrics_total_and_exact() {
        let has_non_whitespace: bool = kani::any();
        let char_count: usize = kani::any();
        let max_chars: usize = kani::any();
        let has_disallowed_control: bool = kani::any();

        let result: Result<(), TextValidationError> = validate_text_metrics(
            has_non_whitespace,
            char_count,
            max_chars,
            has_disallowed_control,
        );

        if !has_non_whitespace || char_count == 0 {
            assert_eq!(result, Err(TextValidationError::Empty));
        } else if char_count > max_chars {
            assert_eq!(result, Err(TextValidationError::TooLong));
        } else if has_disallowed_control {
            assert_eq!(result, Err(TextValidationError::InvalidFormat));
        } else {
            assert_eq!(result, Ok(()));
            assert!(char_count >= 1 && char_count <= max_chars && !has_disallowed_control);
        }
    }

    /// 証明: 全 Unicode スカラー値（`char`）において、1文字文字列の `str::chars()` が返す文字と
    /// `validate_str` および `is_unicode_whitespace(ch)` が完全に一致すること
    #[kani::proof]
    #[kani::unwind(4)]
    fn proof_single_char_str_matches_core_chars() {
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let s: &str = ch.encode_utf8(&mut buf);
        let max_chars: usize = kani::any();

        let mut iter: core::str::Chars<'_> = s.chars();
        let first: Option<char> = iter.next();
        let second: Option<char> = iter.next();
        assert_eq!(first, Some(ch));
        assert_eq!(second, None);

        let actual: Result<(), TextValidationError> = validate_str(s, max_chars);
        let expected: Result<(), TextValidationError> = if ch.is_whitespace() {
            Err(TextValidationError::Empty)
        } else if 1 > max_chars {
            Err(TextValidationError::TooLong)
        } else {
            Ok(())
        };
        assert_eq!(actual, expected);
    }

    /// 証明: 最大3文字の任意の Unicode 文字列（1〜12バイト、ASCII・日本語・絵文字・空白の任意の組合せ）において、
    /// `trim_utf8_range` が標準の `char::is_whitespace` による前後空白除去範囲と完全に一致し、
    /// `validate_utf8_bytes` が **`trim`前の文字数** で上限検証を行うこと
    #[kani::proof]
    #[kani::unwind(16)]
    fn proof_validate_utf8_bytes_pre_trim_count_and_trim_range() {
        let len: usize = kani::any();
        kani::assume(len <= 3);

        let c0: char = kani::any();
        let c1: char = kani::any();
        let c2: char = kani::any();
        let chars: [char; 3] = [c0, c1, c2];

        let mut utf8_buf: [u8; 12] = [0u8; 12];
        let mut byte_offsets: [usize; 4] = [0usize; 4];
        let mut byte_len: usize = 0;
        let mut i: usize = 0;
        while i < len {
            let mut tmp: [u8; 4] = [0u8; 4];
            let encoded: &str = chars[i].encode_utf8(&mut tmp);
            let encoded_bytes: &[u8] = encoded.as_bytes();
            let mut j: usize = 0;
            while j < encoded_bytes.len() {
                utf8_buf[byte_len + j] = encoded_bytes[j];
                j += 1;
            }
            byte_len += encoded_bytes.len();
            byte_offsets[i + 1] = byte_len;
            i += 1;
        }

        let max_chars: usize = kani::any();
        let actual: Result<(), TextValidationError> =
            validate_utf8_bytes(&utf8_buf[..byte_len], max_chars);

        let all_ws: bool = chars[..len].iter().all(|c: &char| c.is_whitespace());
        let expected: Result<(), TextValidationError> = if len == 0 || all_ws {
            Err(TextValidationError::Empty)
        } else if len > max_chars {
            // trim後の文字数ではなく trim前の文字数 `len` で判定されること
            Err(TextValidationError::TooLong)
        } else {
            Ok(())
        };

        assert_eq!(actual, expected);

        let actual_trim: (usize, usize) = trim_utf8_range(&utf8_buf[..byte_len]);
        if all_ws {
            assert_eq!(actual_trim, (0, 0));
        } else {
            let mut first_idx: usize = 0;
            while first_idx < len && chars[first_idx].is_whitespace() {
                first_idx += 1;
            }
            let mut last_idx: usize = len;
            while last_idx > first_idx && chars[last_idx - 1].is_whitespace() {
                last_idx -= 1;
            }
            assert_eq!(
                actual_trim,
                (byte_offsets[first_idx], byte_offsets[last_idx])
            );
        }
    }

    /// 証明: 各問い合わせ項目の上限値が仕様（氏名: 100, メール: 254, 本文: 5000）と厳密に一致すること
    #[kani::proof]
    fn proof_inquiry_field_max_chars_exact() {
        assert_eq!(InquiryField::Name.max_chars(), 100);
        assert_eq!(InquiryField::Email.max_chars(), 254);
        assert_eq!(InquiryField::Message.max_chars(), 5000);
    }
}
