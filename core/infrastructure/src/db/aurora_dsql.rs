//! # Aurora DSQL接続
//!
//! `aurora-dsql-sqlx-connector` を使用してAurora DSQLに接続する。

use anyhow::Error;
use aurora_dsql_sqlx_connector::pool;
use sea_orm::sqlx::PgPool;

/// Aurora DSQL接続文字列を構築する
fn build_connection_string(role: &str, endpoint: &str, region: &str) -> String {
    format!("postgres://{role}@{endpoint}/postgres?region={region}")
}

/// Aurora DSQLに接続する
///
/// ## 引数
///
/// - `endpoint`: DSQLクラスターのエンドポイント
/// - `region`: AWSリージョン
/// - `role`: 接続ロール名
///
/// ## エラー
///
/// 接続失敗時は `anyhow::Error` を返す
pub async fn connect(endpoint: &str, region: &str, role: &str) -> Result<PgPool, Error> {
    tracing::info!("Aurora DSQLへの接続を開始します...");
    let connection_string: String = build_connection_string(role, endpoint, region);
    let pool: PgPool = pool::connect(&connection_string)
        .await
        .map_err(|e| anyhow::anyhow!("Aurora DSQLへの接続に失敗しました: {}", e))?;

    Ok(pool)
}
