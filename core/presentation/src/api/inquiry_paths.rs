//! # OpenAPIパス定義
//!
//! utoipaによるOpenAPIパス定義。

use super::{
    request::CreateInquiryRequest,
    response::{CreateInquiryResponse, ErrorResponse},
};

/// POST /api/v1/inquiry - 問い合わせ作成
#[utoipa::path(
    post,
    path = "/api/v1/inquiry",
    tag = "Inquiry",
    request_body = CreateInquiryRequest,
    responses(
        (
            status = 201,
            description = "問い合わせ作成成功",
            body = CreateInquiryResponse
        ),
        (
            status = 400,
            description = "入力エラー",
            body = ErrorResponse
        ),
        (
            status = 409,
            description = "重複エラー",
            body = ErrorResponse
        ),
        (
            status = 500,
            description = "内部エラー",
            body = ErrorResponse
        )
    )
)]
pub fn create_inquiry_doc() {}

/// GET /api/v1/inquiries - 問い合わせ一覧取得（未実装）
#[utoipa::path(
    get,
    path = "/api/v1/inquiries",
    tag = "Inquiry",
    responses(
        (
            status = 501,
            description = "未実装",
            body = ErrorResponse
        )
    )
)]
pub fn find_inquiries_doc() {}
