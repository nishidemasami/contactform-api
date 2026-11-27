//! # 問い合わせテキスト検証コア（`no_std` / 形式検証対象）
//!
//! 問い合わせドメインの値オブジェクト（氏名・メールアドレス・本文）に共通する
//! 純粋なバリデーションルールを、`std` および `alloc` に一切依存しない
//! `no_std` コアロジックとして提供する。
//!
//! ## 形式検証（Formal Verification）
//!
//! 本モジュールの実装そのものを以下の2つの形式手法ツールで直接検証している。
//!
//! | ツール | 手法 | 主な検証内容 |
//! |---|---|---|
//! | **Verus** | SMT（Z3）による演繹的検証 | 任意長（非有界）入力に対する仕様充足・停止性・オーバーフロー不在・境界厳密性・エラー優先順位（`--no-cheating`） |
//! | **Kani** | CBMC によるビット精密モデル検査 | 全 Unicode スカラー値（`char`）での `char::is_whitespace` 一致性、`core::str`（`trim`/`chars().count()`）との同値性、値オブジェクト・エンティティの不変条件 |
//!
//! ## 検証フロー
//!
//! ```text
//! `InquiryName::new` / `InquiryEmail::new` / `InquiryMessage::new`
//!   └─▶ `validate_field_str`
//!         └─▶ `validate_utf8_bytes`（Verus + Kani で検証）
//!               ├─▶ `scan_utf8_bytes`（単一パス UTF-8 走査・非空白検出・文字数計数）
//!               └─▶ `validate_text_metrics`（空判定優先 → 文字数上限判定）
//! ```

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

/// 氏名の最大文字数（Unicode スカラー値単位）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_NAME_MAX_CHARS: usize = 100;

/// メールアドレスの最大文字数（Unicode スカラー値単位）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_EMAIL_MAX_CHARS: usize = 256;

/// 問い合わせ本文の最大文字数（Unicode スカラー値単位）
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec)]
pub const INQUIRY_MESSAGE_MAX_CHARS: usize = 5000;

/// 問い合わせテキスト項目の種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InquiryField {
    /// 氏名（最大100文字）
    Name,
    /// メールアドレス（最大256文字）
    Email,
    /// 問い合わせ本文（最大5000文字）
    Message,
}

/// テキストバリデーションエラー（値オブジェクト共通コア）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextValidationError {
    /// 空文字または空白文字のみ
    Empty,
    /// 最大文字数超過
    TooLong,
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

/// メトリクス単位のテキスト妥当性仕様
pub open spec fn is_valid_text_metrics_spec(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
) -> bool {
    has_non_whitespace && char_count > 0 && char_count <= max_chars
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
///
/// Rust 標準の `char::is_whitespace`（`str::trim` が使用）と同一の判定集合を
/// 純粋な `const fn` として定義する。
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

/// 算出済みメトリクス（非空白文字の有無・文字数）に対してドメイン規則を検証する
///
/// ## 優先順位
///
/// 1. 空文字または空白文字のみ → `Err(TextValidationError::Empty)`
/// 2. `char_count > max_chars` → `Err(TextValidationError::TooLong)`
/// 3. 上記以外 → `Ok(())`
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.is_ok() <==> is_valid_text_metrics_spec(has_non_whitespace, char_count, max_chars),
        res == Err(TextValidationError::Empty) <==> (!has_non_whitespace || char_count == 0),
        res == Err(TextValidationError::TooLong)
            <==> (has_non_whitespace && char_count > 0 && char_count > max_chars),
))]
pub const fn validate_text_metrics(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
) -> Result<(), TextValidationError> {
    if !has_non_whitespace || char_count == 0 {
        return Err(TextValidationError::Empty);
    }

    if char_count > max_chars {
        return Err(TextValidationError::TooLong);
    }

    Ok(())
}

/// UTF-8 バイト列の指定位置から 1 文字分のコードポイントと消費バイト数をデコードする
///
/// ## 安全性・全域性
///
/// - `unsafe` を一切使用しない
/// - `index < bytes.len()` を満たす任意のバイト列でパニックせず、`1 <= step <= 4` かつ `index + step <= bytes.len()` を満たす
/// - 妥当な UTF-8（`&str` のバイト表現）に対しては RFC 3629 に従う正確な Unicode スカラー値を返す
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

/// UTF-8 バイト列に対してテキストバリデーションを実行する
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

    validate_text_metrics(has_non_whitespace, char_count, max_chars)
}

/// `char` スライスに対してテキストバリデーションを実行する
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

    validate_text_metrics(has_non_whitespace, chars.len(), max_chars)
}

/// 問い合わせ項目種別と UTF-8 バイト列に対してバリデーションを実行する
#[cfg_attr(verus_keep_ghost, verus_builtin_macros::verus_spec(res =>
    ensures
        res.is_ok() <==> (
            utf8_has_non_whitespace_from_spec(bytes@, 0)
                && utf8_char_count_from_spec(bytes@, 0) <= field_max_chars_spec(field) as nat
        ),
        res == Err(TextValidationError::Empty)
            <==> !utf8_has_non_whitespace_from_spec(bytes@, 0),
        res == Err(TextValidationError::TooLong) <==> (
            utf8_has_non_whitespace_from_spec(bytes@, 0)
                && utf8_char_count_from_spec(bytes@, 0) > field_max_chars_spec(field) as nat
        ),
))]
pub fn validate_field_utf8(field: InquiryField, bytes: &[u8]) -> Result<(), TextValidationError> {
    let max_chars: usize = field.max_chars();
    validate_utf8_bytes(bytes, max_chars)
}

/// 文字列スライスに対してテキストバリデーションを実行する
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
pub proof fn lemma_empty_precedence_over_too_long(char_count: usize, max_chars: usize)
    requires
        char_count > max_chars,
    ensures
        !is_valid_text_metrics_spec(false, char_count, max_chars),
{
}

/// 補題: バリデーション成功は文字数が `1..=max_chars` であることを含意する
pub proof fn lemma_valid_metrics_implies_bounded_positive_length(
    has_non_whitespace: bool,
    char_count: usize,
    max_chars: usize,
)
    requires
        is_valid_text_metrics_spec(has_non_whitespace, char_count, max_chars),
    ensures
        has_non_whitespace,
        1 <= char_count,
        char_count <= max_chars,
{
}

/// 補題: 上限境界の厳密性（`max_chars` は妥当、`max_chars + 1` は不正）
pub proof fn lemma_boundary_exactness(max_chars: usize)
    requires
        1 <= max_chars < usize::MAX,
    ensures
        is_valid_text_metrics_spec(true, max_chars, max_chars),
        !is_valid_text_metrics_spec(true, (max_chars + 1) as usize, max_chars),
{
}

/// 補題: 最大文字数に対する単調性（より緩い上限でも妥当性を保つ）
pub proof fn lemma_max_chars_monotonicity(
    has_non_whitespace: bool,
    char_count: usize,
    max_a: usize,
    max_b: usize,
)
    requires
        is_valid_text_metrics_spec(has_non_whitespace, char_count, max_a),
        max_a <= max_b,
    ensures
        is_valid_text_metrics_spec(has_non_whitespace, char_count, max_b),
{
}

/// 補題: 問い合わせ各項目の上限値は厳密に順序付けられている（`0 < 100 < 256 < 5000`）
pub proof fn lemma_field_limits_strictly_ordered()
    ensures
        0 < field_max_chars_spec(InquiryField::Name),
        field_max_chars_spec(InquiryField::Name) < field_max_chars_spec(InquiryField::Email),
        field_max_chars_spec(InquiryField::Email) < field_max_chars_spec(InquiryField::Message),
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

    /// 証明: 任意の `(has_non_whitespace, char_count, max_chars)` に対して
    /// `validate_text_metrics` が全分割・境界厳密性・空判定優先を満たしパニックしないこと
    #[kani::proof]
    fn proof_validate_text_metrics_total_and_exact() {
        let has_non_whitespace: bool = kani::any();
        let char_count: usize = kani::any();
        let max_chars: usize = kani::any();

        let result: Result<(), TextValidationError> =
            validate_text_metrics(has_non_whitespace, char_count, max_chars);

        if !has_non_whitespace || char_count == 0 {
            assert_eq!(result, Err(TextValidationError::Empty));
        } else if char_count > max_chars {
            assert_eq!(result, Err(TextValidationError::TooLong));
        } else {
            assert_eq!(result, Ok(()));
            assert!(char_count >= 1 && char_count <= max_chars);
        }
    }

    /// 証明: 全 Unicode スカラー値（`char`）において、1文字文字列の `str::chars()` が返す文字と
    /// `validate_str` および `is_unicode_whitespace(ch)` が完全に一致すること
    #[kani::proof]
    #[kani::unwind(4)]
    fn proof_single_char_str_matches_core_chars() {
        let ch: char = kani::any();
        let mut buf: [u8; 4] = [0u8; 4];
        let text: &str = ch.encode_utf8(&mut buf);

        let mut iter: core::str::Chars<'_> = text.chars();
        assert_eq!(iter.next(), Some(ch));
        assert_eq!(iter.next(), None);

        let core_is_ws: bool = ch.is_whitespace();
        assert_eq!(core_is_ws, is_unicode_whitespace(ch));

        let actual: Result<(), TextValidationError> = validate_str(text, 1);
        if core_is_ws {
            assert_eq!(actual, Err(TextValidationError::Empty));
        } else {
            assert_eq!(actual, Ok(()));
        }
    }

    /// 証明: 任意の Unicode スカラー値列（最大2文字・最大8バイト：1〜4バイト UTF-8・日本語・全角空白の組合せを網羅）に対し、
    /// `validate_utf8_bytes` が `char::is_whitespace` および文字数上限判定と完全同値であること
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_validate_utf8_bytes_matches_core_char_sequence() {
        let chars: [char; 2] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= chars.len());

        let max_chars: usize = kani::any();
        kani::assume(max_chars <= 3);

        let mut buf: [u8; 8] = [0u8; 8];
        let mut pos: usize = 0;
        if len >= 1 {
            let encoded0: &str = chars[0].encode_utf8(&mut buf[pos..]);
            pos += encoded0.len();
        }
        if len >= 2 {
            let encoded1: &str = chars[1].encode_utf8(&mut buf[pos..]);
            pos += encoded1.len();
        }

        let actual: Result<(), TextValidationError> = validate_utf8_bytes(&buf[..pos], max_chars);
        let all_whitespace: bool = chars[..len].iter().all(|ch: &char| ch.is_whitespace());
        let expected: Result<(), TextValidationError> = if all_whitespace {
            Err(TextValidationError::Empty)
        } else if len > max_chars {
            Err(TextValidationError::TooLong)
        } else {
            Ok(())
        };

        assert_eq!(actual, expected);
    }

    /// 証明: 任意の `char` スライス（最大 4 文字）に対し、
    /// `validate_chars` が `char::is_whitespace` と長さ判定に完全一致すること
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_validate_chars_matches_core_char() {
        let buf: [char; 4] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= buf.len());

        let max_chars: usize = kani::any();
        kani::assume(max_chars <= 6);

        let slice: &[char] = &buf[..len];
        let actual: Result<(), TextValidationError> = validate_chars(slice, max_chars);

        let all_whitespace: bool = slice.iter().all(|ch: &char| ch.is_whitespace());
        let expected: Result<(), TextValidationError> = if all_whitespace {
            Err(TextValidationError::Empty)
        } else if slice.len() > max_chars {
            Err(TextValidationError::TooLong)
        } else {
            Ok(())
        };

        assert_eq!(actual, expected);
    }

    /// 証明: 問い合わせ項目の最大文字数定義が厳密に仕様通りであること
    #[kani::proof]
    fn proof_inquiry_field_max_chars_exact() {
        assert_eq!(InquiryField::Name.max_chars(), 100);
        assert_eq!(InquiryField::Email.max_chars(), 256);
        assert_eq!(InquiryField::Message.max_chars(), 5000);
        assert!(InquiryField::Name.max_chars() < InquiryField::Email.max_chars());
        assert!(InquiryField::Email.max_chars() < InquiryField::Message.max_chars());
    }
}
