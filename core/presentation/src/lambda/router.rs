//! # ルーター
//!
//! HTTPメソッドとパスに基づいてルートを判定する。

use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

pub use super::http_decision::Route;
use super::http_decision::route_method_and_path;

/// `Option<String>` から `&str` 参照を取得するヘルパートレイト
pub trait OptionStrExt {
    /// 文字列スライス参照を返す（`None` の場合は空文字列）
    fn as_str(&self) -> &str;
}

impl OptionStrExt for Option<String> {
    fn as_str(&self) -> &str {
        self.as_deref().unwrap_or_default()
    }
}

/// リクエストをルートに振り分ける
///
/// `raw_path` が未設定または空文字の場合は `request_context.http.path` にフォールバックする。
pub fn route(request: &ApiGatewayV2httpRequest) -> Route {
    let method: &str = request.request_context.http.method.as_str();
    let raw_path: &str = request.raw_path.as_str();
    let path: &str = if !raw_path.is_empty() {
        raw_path
    } else {
        request.request_context.http.path.as_str()
    };

    route_method_and_path(method, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(method: &str, path: &str) -> ApiGatewayV2httpRequest {
        serde_json::from_value(serde_json::json!({
            "version": "2.0",
            "rawPath": path,
            "requestContext": {
                "http": {
                    "method": method,
                    "path": path,
                    "protocol": "HTTP/1.1",
                    "sourceIp": "127.0.0.1",
                    "userAgent": "test"
                },
                "accountId": "123456789012",
                "apiId": "test",
                "domainName": "test",
                "domainPrefix": "test",
                "requestId": "test",
                "routeKey": "$default",
                "stage": "$default",
                "time": "12/Mar/2020:19:03:58 +0000",
                "timeEpoch": 1583348638390u64
            },
            "isBase64Encoded": false
        }))
        .unwrap()
    }

    #[test]
    fn post_api_v1_inquiry_はcreate_inquiryになる() {
        let request: ApiGatewayV2httpRequest = make_request("POST", "/api/v1/inquiry");
        assert_eq!(route(&request), Route::CreateInquiry);
    }

    #[test]
    fn get_api_v1_inquiries_はfind_inquiriesになる() {
        let request: ApiGatewayV2httpRequest = make_request("GET", "/api/v1/inquiries");
        assert_eq!(route(&request), Route::FindInquiries);
    }

    #[test]
    fn get_api_v1_inquiry_はmethod_not_allowedになる() {
        let request: ApiGatewayV2httpRequest = make_request("GET", "/api/v1/inquiry");
        assert_eq!(route(&request), Route::MethodNotAllowed);
    }

    #[test]
    fn 未知のパスはnot_foundになる() {
        let request: ApiGatewayV2httpRequest = make_request("GET", "/unknown");
        assert_eq!(route(&request), Route::NotFound);
    }
}
