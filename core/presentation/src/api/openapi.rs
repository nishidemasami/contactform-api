//! # OpenAPIドキュメント定義
//!
//! utoipaによるOpenAPIドキュメントの集約。

use utoipa::OpenApi;

use super::{
    request::CreateInquiryRequest,
    response::{CreateInquiryResponse, ErrorResponse, InquiryResponse},
};

/// OpenAPIドキュメント
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::inquiry_paths::create_inquiry_doc,
        crate::api::inquiry_paths::find_inquiries_doc,
        crate::api::inquiry_paths::find_inquiry_doc,
    ),
    components(
        schemas(
            CreateInquiryRequest,
            CreateInquiryResponse,
            InquiryResponse,
            ErrorResponse,
        )
    ),
    tags(
        (name = "Inquiry", description = "問い合わせAPI")
    ),
    info(
        title = "contactform-api",
        description = "コンタクトフォームAPI",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;
