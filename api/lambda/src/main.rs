//! # バックエンドAPIデータベース Lambda ハンドラー
//!
//! このモジュールは、AWS Lambda上で動作するコンタクトフォームAPIのエントリーポイントです。
//! Amazon API Gateway HTTP API からのリクエストを受け取り、
//! HTTPメソッドやパスに基づいて適切なハンドラーにルーティングします。
//!
//! ## アーキテクチャ概要
//!
//! クライアント
//!   └─▶ Amazon API Gateway HTTP API
//!         └─▶ AWS Lambda (このモジュール)
//!               └─▶ Amazon Aurora DSQL (SeaORM経由)
//!
//! ## 起動フロー
//!
//! `main()`
//!   └─▶ `executor()`
//!         ├─▶ [INITフェーズ] tracing初期化
//!         ├─▶ [INITフェーズ] 環境変数読み込み（DSQL_ENDPOINT, AWS_REGION）
//!         ├─▶ [INITフェーズ] DB接続確立（OnceCell）
//!         └─▶ [INVOKEフェーズ] lambda_executor(handler)
//!
//! ## 環境変数
//!
//! | 変数名 | 必須 | 説明 |
//! |--------|------|------|
//! | `DSQL_ENDPOINT` | ✓ | Aurora DSQLクラスターのエンドポイント |
//! | `AWS_REGION` | ✓ | Aurora DSQLクラスターのAWSリージョン |

use std::{env, sync::Arc};

use domain::inquiry::repository::InquiryRepository;
use infrastructure::{
    db::{config::DatabaseConfig, connection::create_connection},
    repository::inquiry::seaorm_inquiry_repository::SeaOrmInquiryRepository,
};
use presentation::lambda::{inquiry_handler::handler, lambda_executor::lambda_executor};
use sea_orm::DatabaseConnection;
use tokio::sync::OnceCell;

/// DSQLエンドポイントのキャッシュ（Cold Start時に初期化）
static DSQL_ENDPOINT: OnceCell<String> = OnceCell::const_new();

/// AWSリージョンのキャッシュ（Cold Start時に初期化）
static AWS_REGION: OnceCell<String> = OnceCell::const_new();

/// DB接続のキャッシュ（Cold Start時に確立し、Hot Start時に再利用）
static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

/// Lambda ランタイムへ型付きハンドラを登録するエントリーポイント
///
/// シングルスレッドで起動し、INITフェーズでDB接続を確立する。
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), lambda_runtime::Error> {
    executor().await
}

/// Lambda 処理の実態
///
/// DSQL 接続を初期化し、Lambda ランタイムへ HTTP ハンドラを登録する。
///
/// ## Fail-Fast原則
///
/// - 環境変数が未設定の場合は即座に `panic!` してプロセスを終了する
/// - DB接続の初期化に失敗した場合も即座に `panic!` する
async fn executor() -> Result<(), lambda_runtime::Error> {
    // INITフェーズここから
    // INITフェーズは最大10000ms。Cold Startの初回はCPUのブースト枠が与えられるので、この間に重い処理を済ませておく。
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint: &String = DSQL_ENDPOINT
        .get_or_init(|| async {
            match env::var("DSQL_ENDPOINT") {
                Ok(value) => value,
                Err(e) => {
                    tracing::error!("DSQL_ENDPOINT 環境変数の取得に失敗しました: {:?}", e);
                    panic!("Internal Server Error");
                }
            }
        })
        .await;

    let aws_region: &String = AWS_REGION
        .get_or_init(|| async {
            match env::var("AWS_REGION") {
                Ok(value) => value,
                Err(e) => {
                    tracing::error!("AWS_REGION 環境変数の取得に失敗しました: {:?}", e);
                    panic!("Internal Server Error");
                }
            }
        })
        .await;

    let db: &DatabaseConnection = DATABASE_CONNECTION
        .get_or_init(|| async {
            create_connection(&DatabaseConfig::AuroraDSQL {
                role: "insertonly".into(),
                endpoint: dsql_endpoint.as_str().to_string().into(),
                region: aws_region.as_str().to_string().into(),
            })
            .await
            .unwrap_or_else(|e| {
                tracing::error!("データベース接続の初期化に失敗しました: {:?}", e);
                panic!("Database Connection Error");
            })
        })
        .await;

    // INITフェーズここまで

    // INVOKEフェーズここから
    // Hot StartではCold Startで作成した共有済みコネクションをcloneしてHTTPハンドラを実行する。
    lambda_executor(|event| {
        let repository: Arc<dyn InquiryRepository> =
            Arc::new(SeaOrmInquiryRepository::new(db.clone()));
        handler(repository, event)
    })
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use domain::inquiry::{entity::Inquiry, repository_error::InquiryRepositoryError};

    use super::*;

    /// テスト用モックリポジトリ
    pub struct MockInquiryRepository {
        pub error_to_return: Mutex<Option<InquiryRepositoryError>>,
    }

    impl Default for MockInquiryRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockInquiryRepository {
        pub fn new() -> Self {
            Self {
                error_to_return: Mutex::new(None),
            }
        }

        pub fn set_error(&self, err: InquiryRepositoryError) {
            *self.error_to_return.lock().unwrap() = Some(err);
        }
    }

    #[async_trait]
    impl InquiryRepository for MockInquiryRepository {
        async fn save(&self, inquiry: Inquiry) -> Result<Inquiry, InquiryRepositoryError> {
            let err: Option<InquiryRepositoryError> = self.error_to_return.lock().unwrap().clone();
            if let Some(e) = err {
                return Err(e);
            }
            Ok(inquiry)
        }
    }

    #[tokio::test]
    async fn モックリポジトリを使用したハンドラーの動作確認() {
        use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
        use lambda_runtime::Context;

        let mock: Arc<MockInquiryRepository> = Arc::new(MockInquiryRepository::new());
        let repository: Arc<dyn InquiryRepository> = mock;

        let request: ApiGatewayV2httpRequest = serde_json::from_value(serde_json::json!({
            "version": "2.0",
            "rawPath": "/api/v1/inquiry",
            "requestContext": {
                "http": {
                    "method": "POST",
                    "path": "/api/v1/inquiry",
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
            "body": r#"{"name":"山田太郎","email":"yamada@example.com","message":"テストメッセージ"}"#,
            "isBase64Encoded": false
        }))
        .unwrap();

        let event: lambda_runtime::LambdaEvent<ApiGatewayV2httpRequest> =
            lambda_runtime::LambdaEvent::new(request, Context::default());

        let result: Result<
            aws_lambda_events::apigw::ApiGatewayV2httpResponse,
            lambda_runtime::Error,
        > = handler(repository, event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status_code, 201);
    }

    #[tokio::test]
    async fn リポジトリエラーの場合500を返す() {
        use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
        use lambda_runtime::Context;

        let mock: Arc<MockInquiryRepository> = Arc::new(MockInquiryRepository::new());
        mock.set_error(InquiryRepositoryError::Infrastructure);
        let repository: Arc<dyn InquiryRepository> = mock;

        let request: ApiGatewayV2httpRequest = serde_json::from_value(serde_json::json!({
            "version": "2.0",
            "rawPath": "/api/v1/inquiry",
            "requestContext": {
                "http": {
                    "method": "POST",
                    "path": "/api/v1/inquiry",
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
            "body": r#"{"name":"山田太郎","email":"yamada@example.com","message":"テストメッセージ"}"#,
            "isBase64Encoded": false
        }))
        .unwrap();

        let event: lambda_runtime::LambdaEvent<ApiGatewayV2httpRequest> =
            lambda_runtime::LambdaEvent::new(request, Context::default());

        let result: Result<
            aws_lambda_events::apigw::ApiGatewayV2httpResponse,
            lambda_runtime::Error,
        > = handler(repository, event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status_code, 500);
    }

    #[tokio::test]
    #[ignore = "ローカルPostgreSQLの起動が必要なためデフォルトでは実行しない"]
    async fn 実際にローカルの_postgre_sqlに接続するテスト() {
        use tokio::sync::OnceCell;

        static TEST_DB: OnceCell<DatabaseConnection> = OnceCell::const_new();
        let _db: &DatabaseConnection = TEST_DB
            .get_or_init(|| async {
                create_connection(&DatabaseConfig::PostgreSQL {
                    host: "localhost".into(),
                    port: 5432,
                    database: "postgres".into(),
                    username: "insertonly".into(),
                    password: "insertonly".into(),
                })
                .await
                .expect("DB接続に失敗しました")
            })
            .await;
    }
}
