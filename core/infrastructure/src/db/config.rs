//! # DB接続設定
//!
//! Aurora DSQLおよびPostgreSQLの接続設定を定義する。

use std::borrow::Cow;

/// データベース接続設定
///
/// ## バリアント
///
/// - `AuroraDSQL`: Amazon Aurora DSQL（本番・開発環境）
/// - `PostgreSQL`: PostgreSQL（ローカル開発・テスト環境）
#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    /// Amazon Aurora DSQL接続設定
    ///
    /// | フィールド | 説明 |
    /// |---|---|
    /// | `endpoint` | DSQLクラスターのエンドポイント |
    /// | `region` | AWSリージョン |
    /// | `role` | 接続ロール名 |
    AuroraDSQL {
        /// DSQLクラスターのエンドポイント
        endpoint: Cow<'static, str>,
        /// AWSリージョン
        region: Cow<'static, str>,
        /// 接続ロール名
        role: Cow<'static, str>,
    },

    /// PostgreSQL接続設定
    ///
    /// | フィールド | 説明 |
    /// |---|---|
    /// | `host` | ホスト名 |
    /// | `port` | ポート番号 |
    /// | `database` | データベース名 |
    /// | `username` | ユーザー名 |
    /// | `password` | パスワード |
    PostgreSQL {
        /// ホスト名
        host: Cow<'static, str>,
        /// ポート番号
        port: u16,
        /// データベース名
        database: Cow<'static, str>,
        /// ユーザー名
        username: Cow<'static, str>,
        /// パスワード
        password: Cow<'static, str>,
    },
}
