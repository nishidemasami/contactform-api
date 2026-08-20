//! # Lambdaエグゼキューター
//!
//! Lambda ランタイムへ型付きハンドラを登録するユーティリティ。

use std::future::Future;

use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use lambda_runtime::{Error, LambdaEvent, run, service_fn};

/// Lambda ランタイムへ型付きハンドラを登録してリクエスト待機状態にする
///
/// ## 引数
///
/// - `invoke`: リクエストごとに呼び出すクロージャ
///
/// ## フロー
///
/// ```text
/// Lambda ランタイム
///   └─▶ `service_fn(invoke)` で各リクエストをハンドリング
/// ```
pub async fn lambda_executor<F, Fut>(invoke: F) -> Result<(), Error>
where
    F: Fn(LambdaEvent<ApiGatewayV2httpRequest>) -> Fut,
    Fut: Future<Output = Result<ApiGatewayV2httpResponse, Error>>,
{
    run(service_fn(invoke)).await
}
