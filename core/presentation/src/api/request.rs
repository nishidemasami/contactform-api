//! # APIリクエスト型
//!
//! HTTPリクエストのJSONデシリアライズ用構造体を定義する。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 問い合わせ作成リクエスト
///
/// ## フィールド
///
/// | フィールド | 型 | 文字数制限 | 説明 |
/// |---|---|---|---|
/// | `name` | `String` | `trim`前 1〜100文字 | 氏名（空白のみ不可・制御文字不可） |
/// | `email` | `String` | `trim`後 6〜254文字 | 連絡先メールアドレス（RFC準拠形式） |
/// | `message` | `String` | `trim`前 1〜5000文字 | 問い合わせ本文（空白のみ不可・改行タブ以外の制御文字不可） |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateInquiryRequest {
    /// 氏名（`trim`前 1〜100文字）
    #[schema(min_length = 1, max_length = 100)]
    pub name: String,

    /// 連絡先メールアドレス（前後空白 `trim` 後 6〜254文字、メール形式）
    #[schema(min_length = 6, max_length = 254, format = Email)]
    pub email: String,

    /// 問い合わせ本文（`trim`前 1〜5000文字）
    #[schema(min_length = 1, max_length = 5000)]
    pub message: String,
}
