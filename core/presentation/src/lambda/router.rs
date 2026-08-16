//! # ルーター
//!
//! HTTPメソッドとパスに基づいてルートを判定する。

use aws_lambda_events::apigw::ApiGatewayV2httpRequest;

/// ルート定義
///
/// | バリアント | 説明 |
/// |---|---|
/// | `CreateInquiry` | `POST /api/v1/inquiry` |
/// | `FindInquiry` | `GET /api/v1/inquiries` |
/// | `NotFound` | 404 Not Found |
/// | `MethodNotAllowed` | 405 Method Not Allowed |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// POST /api/v1/inquiry - 問い合わせ作成
    CreateInquiry,
    /// GET /api/v1/inquiries - 問い合わせ一覧（未実装）
    FindInquiry,
    /// 404 Not Found
    NotFound,
    /// 405 Method Not Allowed
    MethodNotAllowed,
}

/// リクエストをルートに振り分ける
pub fn route(request: &ApiGatewayV2httpRequest) -> Route {
    let method: &str = request.request_context.http.method.as_str();
    let path: &str = request.raw_path.as_deref().unwrap_or_default();

    match (method, path) {
        ("POST", "/api/v1/inquiry") => Route::CreateInquiry,
        ("GET", "/api/v1/inquiries") => Route::FindInquiry,
        (_, "/api/v1/inquiry") | (_, "/api/v1/inquiries") => Route::MethodNotAllowed,
        _ => Route::NotFound,
    }
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
    fn get_api_v1_inquiries_はfind_inquiryになる() {
        let request: ApiGatewayV2httpRequest = make_request("GET", "/api/v1/inquiries");
        assert_eq!(route(&request), Route::FindInquiry);
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
