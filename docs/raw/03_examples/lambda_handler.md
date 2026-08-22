# AWS Lambda エントリーポイント 実装例

```rust
use std::env;
use std::sync::Arc;
use tokio::sync::OnceCell;

use presentation::lambda::{lambda_executor, function_handler::handler};
use lambda_runtime::Error;
use sea_orm::DatabaseConnection;
use domain::inquiry::repository::InquiryRepository;
use infrastructure::repository::inquiry::seaorm_inquiry_repository::SeaOrmInquiryRepository;

static DSQL_ENDPOINT: OnceCell<String> = OnceCell::const_new();
static AWS_REGION: OnceCell<String> = OnceCell::const_new();
static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    executor().await
}

async fn executor() -> Result<(), Error> {
    // INITフェーズ: ログ・環境変数・DB接続の初期化
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = DSQL_ENDPOINT.get_or_init(|| async {
        match env::var("DSQL_ENDPOINT") {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("DSQL_ENDPOINT 環境変数の取得に失敗しました: {:?}", e);
                panic!("Internal Server Error");
            }
        }
    }).await;

    let aws_region = AWS_REGION.get_or_init(|| async {
        match env::var("AWS_REGION") {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("AWS_REGION 環境変数の取得に失敗しました: {:?}", e);
                panic!("Internal Server Error");
            }
        }
    }).await;

    let db = DATABASE_CONNECTION.get_or_init(|| async {
        use infrastructure::db::{config::DatabaseConfig::AuroraDSQL, connection::create_connection};
        create_connection(&AuroraDSQL {
            role: "insertonly".into(),
            endpoint: dsql_endpoint.as_str().into(),
            region: aws_region.as_str().into(),
        })
        .await
        .unwrap_or_else(|e| {
            tracing::error!("データベース接続の初期化に失敗しました: {:?}", e);
            panic!("Database Connection Error");
        })
    }).await;

    // INVOKEフェーズ: 共有接続を用いてハンドラーを実行
    lambda_executor(|event| {
        let repository: Arc<dyn InquiryRepository> =
            Arc::new(SeaOrmInquiryRepository::new(db.clone()));
        handler(Arc::clone(&repository), event)
    })
    .await
}
```
