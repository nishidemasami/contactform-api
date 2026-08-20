//! # DB接続ファクトリ
//!
//! 設定に応じたDB接続を生成する。

use sea_orm::{DatabaseConnection, SqlxPostgresConnector, sqlx::PgPool};

use super::{aurora_dsql, config::DatabaseConfig, postgres};

/// DB接続を生成する
///
/// ## フロー
///
/// ```
/// `DatabaseConfig`
///   └─▶ `AuroraDSQL` または `PostgreSQL` へ振り分け
///         └─▶ `DatabaseConnection`
/// ```
///
/// ## エラー
///
/// 接続失敗時は `anyhow::Error` を返す
pub async fn create_connection(config: &DatabaseConfig) -> anyhow::Result<DatabaseConnection> {
    let pool: PgPool = match config {
        DatabaseConfig::AuroraDSQL {
            endpoint,
            region,
            role,
        } => aurora_dsql::connect(endpoint, region, role).await?,

        DatabaseConfig::PostgreSQL {
            host,
            port,
            database,
            username,
            password,
        } => postgres::connect(host, *port, database, username, password).await?,
    };
    Ok(SqlxPostgresConnector::from_sqlx_postgres_pool(pool))
}
