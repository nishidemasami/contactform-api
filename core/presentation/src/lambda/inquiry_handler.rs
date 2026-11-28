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
    lambda::{
        http_decision::{
            PublicErrorMessage, RepositoryFailureKind, UsecaseErrorCategory,
            classify_repository_failure, decide_public_error, is_request_body_acceptable,
            static_route_status_code,
        },
        router::{Route, route},
    },
};

/// リポジトリエラーを形式検証済みコアの分類型へ写像する
const fn map_repository_error_kind(error: &InquiryRepositoryError) -> RepositoryFailureKind {
    match error {
        InquiryRepositoryError::Duplicate => RepositoryFailureKind::Duplicate,
        InquiryRepositoryError::NotFound => RepositoryFailureKind::NotFound,
        InquiryRepositoryError::Infrastructure => RepositoryFailureKind::Infrastructure,
        InquiryRepositoryError::CorruptedData => RepositoryFailureKind::CorruptedData,
    }
}

/// ユースケースエラーを形式検証済みコアのエラー分類へ写像する
const fn classify_create_inquiry_error(error: &CreateInquiryError) -> UsecaseErrorCategory {
    match error {
        CreateInquiryError::Domain(_) => UsecaseErrorCategory::DomainValidation,
        CreateInquiryError::Repository(repo_err) => {
            classify_repository_failure(map_repository_error_kind(repo_err))
        }
    }
}

/// Lambda ハンドラー
///
/// ## フロー（Lambda到達リクエスト）
///
/// ```text
/// `ApiGatewayV2httpRequest`
///   └─▶ ルーティング（AWS環境でのCORSプリフライトOPTIONSはAPI Gatewayエッジで自動応答）
///         ├─▶ `OPTIONS /api/v1/inquiry` | `OPTIONS /api/v1/inquiries` → 204 (Empty Body)
///         ├─▶ `POST /api/v1/inquiry` → `create_inquiry` (201 / 400 / 409 / 500)
///         ├─▶ `GET /api/v1/inquiries` → 501
///         ├─▶ 未定義パス → 404
///         └─▶ 未許可メソッド → 405
/// ```
pub async fn handler(
    repository: Arc<dyn InquiryRepository>,
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let (request, _): (ApiGatewayV2httpRequest, _) = event.into_parts();
    let matched_route: Route = route(&request);

    match matched_route {
        Route::CreateInquiry => create_inquiry(repository, request).await,

        Route::Options => {
            let status: i64 = static_route_status_code(matched_route).unwrap_or(204);
            Ok(empty_response(status))
        }

        Route::FindInquiries => {
            let status: i64 = static_route_status_code(matched_route).unwrap_or(501);
            json_response(
                status,
                &ErrorResponse {
                    message: "not implemented".into(),
                },
            )
        }

        Route::NotFound => {
            let status: i64 = static_route_status_code(matched_route).unwrap_or(404);
            json_response(
                status,
                &ErrorResponse {
                    message: "not found".into(),
                },
            )
        }

        Route::MethodNotAllowed => {
            let status: i64 = static_route_status_code(matched_route).unwrap_or(405);
            json_response(
                status,
                &ErrorResponse {
                    message: "method not allowed".into(),
                },
            )
        }
    }
}

/// 問い合わせを作成する
async fn create_inquiry(
    repository: Arc<dyn InquiryRepository>,
    request: ApiGatewayV2httpRequest,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let is_base64_encoded: bool = request.is_base64_encoded;
    let body: String = request.body.unwrap_or_default();

    if !is_request_body_acceptable(body.len(), is_base64_encoded) {
        tracing::warn!(
            "リクエストボディが許容条件外です (len={}, base64={})",
            body.len(),
            is_base64_encoded
        );
        return json_response(
            400,
            &ErrorResponse {
                message: "invalid request body".into(),
            },
        );
    }

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

        Err(err) => {
            let category: UsecaseErrorCategory = classify_create_inquiry_error(&err);
            let decision: (i64, PublicErrorMessage) = decide_public_error(category);
            let status_code: i64 = decision.0;
            let public_message: PublicErrorMessage = decision.1;

            match category {
                UsecaseErrorCategory::DomainValidation => {
                    tracing::warn!("ドメインバリデーションエラー: {:?}", err);
                }
                UsecaseErrorCategory::DuplicateConflict => {
                    tracing::warn!("問い合わせ重複エラー: {:?}", err);
                }
                UsecaseErrorCategory::RepositoryInternal => {
                    tracing::error!("問い合わせ作成エラー: {:?}", err);
                }
            }

            json_response(
                status_code,
                &ErrorResponse {
                    message: public_message.as_str().into(),
                },
            )
        }
    }
}

/// ボディなしのレスポンス（204 No Content 等）を生成する
fn empty_response(status_code: i64) -> ApiGatewayV2httpResponse {
    let mut response: ApiGatewayV2httpResponse = ApiGatewayV2httpResponse::default();
    response.status_code = status_code;
    response.body = Some(Body::Empty);
    response
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
