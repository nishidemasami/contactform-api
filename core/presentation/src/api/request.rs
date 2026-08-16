//! # APIリクエスト型
//!
//! HTTPリクエストのJSONデシリアライズ用構造体を定義する。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 問い合わせ作成リクエスト
///
/// ## フィールド
///
/// | フィールド | 型 | 説明 |
/// |---|---|---|
/// | `name` | `String` | 氏名 |
/// | `email` | `String` | 連絡先メールアドレス |
/// | `message` | `String` | 問い合わせ本文 |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateInquiryRequest {
    /// 氏名
    pub name: String,

    /// 連絡先メールアドレス
    pub email: String,

    /// 問い合わせ本文
    pub message: String,
}
