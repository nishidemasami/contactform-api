//! # 問い合わせDTO
//!
//! アプリケーション層とプレゼンテーション層間のデータ転送オブジェクト。

use uuid::Uuid;

/// 問い合わせ作成入力DTO
#[derive(Debug, Clone)]
pub struct CreateInquiryInput {
    /// 氏名
    pub name: String,
    /// 連絡先メールアドレス
    pub email: String,
    /// 問い合わせ本文
    pub message: String,
}

/// 問い合わせ作成出力DTO
#[derive(Debug, Clone)]
pub struct CreateInquiryOutput {
    /// 生成された問い合わせID
    pub id: Uuid,
}
