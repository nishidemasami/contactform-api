//! # 問い合わせLambdaハンドラー
//!
//! AWS Lambda上でHTTPリクエストを受け取り、問い合わせユースケースを実行する。

use std::sync::Arc;

use application::inquiry::{
    create::CreateInquiryUsecase, dto::CreateInquiryInput, error::CreateInquiryError,
};
use aws_lambda_events::{
    apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse},
    encodings::Body,
};
use domain::inquiry::{repository::InquiryRepository, repository_error::InquiryRepositoryError};
use http::{HeaderMap, HeaderValue};
use lambda_runtime::{Error, LambdaEvent};
use serde::Serialize;

use crate::{
    api::{
        request::CreateInquiryRequest,
        response::{CreateInquiryResponse, ErrorResponse},
    },
    lambda::router::{Route, route},
};

/// Lambda ハンドラー
///
/// ## フロー
///
/// ```text
/// `ApiGatewayV2httpRequest`
///   └─▶ ルーティング
///         ├─▶ `POST /api/v1/inquiry` → `create_inquiry`
///         ├─▶ `GET /api/v1/inquiries` → 501
///         ├─▶ 404 → 404
///         └─▶ 405 → 405
/// ```
pub async fn handler(
    repository: Arc<dyn InquiryRepository>,
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let (request, _): (ApiGatewayV2httpRequest, _) = event.into_parts();

    match route(&request) {
        Route::CreateInquiry => create_inquiry(repository, request).await,

        Route::FindInquiries => json_response(
            501,
            &ErrorResponse {
                message: "not implemented".into(),
            },
        ),

        Route::NotFound => json_response(
            404,
            &ErrorResponse {
                message: "not found".into(),
            },
        ),

        Route::MethodNotAllowed => json_response(
            405,
            &ErrorResponse {
                message: "method not allowed".into(),
            },
        ),
    }
}

/// 問い合わせを作成する
async fn create_inquiry(
    repository: Arc<dyn InquiryRepository>,
    request: ApiGatewayV2httpRequest,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let body: String = request.body.unwrap_or_default();

    let req: CreateInquiryRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("リクエストのデシリアライズに失敗しました: {:?}", e);
            return json_response(
                400,
                &ErrorResponse {
                    message: "invalid request body".into(),
                },
            );
        }
    };

    let input: CreateInquiryInput = CreateInquiryInput {
        name: req.name,
        email: req.email,
        message: req.message,
    };

    let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repository);

    match usecase.execute(input).await {
        Ok(output) => json_response(201, &CreateInquiryResponse { id: output.id }),

        Err(CreateInquiryError::Domain(e)) => {
            tracing::warn!("ドメインバリデーションエラー: {:?}", e);
            json_response(
                400,
                &ErrorResponse {
                    message: "invalid request".into(),
                },
            )
        }

        Err(CreateInquiryError::Repository(InquiryRepositoryError::Duplicate)) => json_response(
            409,
            &ErrorResponse {
                message: "inquiry already exists".into(),
            },
        ),

        Err(e) => {
            tracing::error!("問い合わせ作成エラー: {:?}", e);
            json_response(
                500,
                &ErrorResponse {
                    message: "internal server error".into(),
                },
            )
        }
    }
}

/// JSONレスポンスを生成する
fn json_response<T>(status_code: i64, payload: &T) -> Result<ApiGatewayV2httpResponse, Error>
where
    T: Serialize,
{
    let body: String = serde_json::to_string(payload)?;

    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/json"));

    let mut response: ApiGatewayV2httpResponse = ApiGatewayV2httpResponse::default();
    response.status_code = status_code;
    response.body = Some(Body::Text(body));
    response.headers = headers;
    Ok(response)
}
