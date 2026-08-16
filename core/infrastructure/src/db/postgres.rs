//! # PostgreSQL接続
//!
//! ローカル開発・テスト用のPostgreSQL接続。

use anyhow::Error;
use sea_orm::sqlx::PgPool;

/// PostgreSQLに接続する
///
/// ## 引数
///
/// - `host`: ホスト名
/// - `port`: ポート番号
/// - `database`: データベース名
/// - `username`: ユーザー名
/// - `password`: パスワード
///
/// ## エラー
///
/// 接続失敗時は `anyhow::Error` を返す
pub async fn connect(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: &str,
) -> Result<PgPool, Error> {
    let database_url: String = format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, host, port, database,
    );
    let pool: PgPool = PgPool::connect(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!("PostgreSQLへの接続に失敗しました: {}", e))?;

    Ok(pool)
}
