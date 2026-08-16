//! # APIレスポンス型
//!
//! HTTPレスポンスのJSONシリアライズ用構造体を定義する。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 問い合わせレスポンス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct InquiryResponse {
    /// 問い合わせID
    pub id: Uuid,
    /// 氏名
    pub name: String,
    /// 連絡先メールアドレス
    pub email: String,
    /// 問い合わせ本文
    pub message: String,
}

/// 問い合わせ作成レスポンス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateInquiryResponse {
    /// 生成された問い合わせID
    pub id: Uuid,
}

/// エラーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ErrorResponse {
    /// エラーメッセージ
    pub message: String,
}
