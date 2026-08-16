//! # SeaORM問い合わせリポジトリ実装
//!
//! `InquiryRepository` トレイトをSeaORMを使用して実装する。
//! Aurora DSQLのINSERT専用ロール制約に対応するため、
//! `RETURNING`句を伴わないINSERT方式を採用する。

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde_json::json;
use uuid::Uuid;

use domain::inquiry::{
    entity::Inquiry, repository::InquiryRepository, repository_error::InquiryRepositoryError,
};

/// SeaORMを使用した問い合わせリポジトリ
///
/// ## セキュリティ注意事項
///
/// - Lambda実行ロールにはINSERT権限のみ付与されているため、
///   `ActiveModel::insert`（RETURNINGを伴う）は使用禁止。
/// - プレースホルダーを使用してSQLインジェクションを防止する。
pub struct SeaOrmInquiryRepository {
    db: DatabaseConnection,
}

impl SeaOrmInquiryRepository {
    /// SeaORM問い合わせリポジトリを生成する
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InquiryRepository for SeaOrmInquiryRepository {
    /// 問い合わせを保存する
    ///
    /// ## 実装方針
    ///
    /// - `execute_unprepared` ではなく `execute` でプレースホルダーを使用する
    /// - Aurora DSQLのINSERT専用ロール対応のため `RETURNING` 句を使用しない
    async fn save(&self, inquiry: Inquiry) -> Result<Inquiry, InquiryRepositoryError> {
        let id: Uuid = *inquiry.id().value();
        let name: &str = inquiry.name().value();
        let email: &str = inquiry.email().value();
        let message: &str = inquiry.message().value();
        let created_at: &DateTime<FixedOffset> = inquiry.created_at();

        // row_logにリクエストの内容をJSON形式で保存する
        let row_log: String = json!({
            "name": name,
            "email": email,
            "message": message,
        })
        .to_string();

        // プレースホルダーを使用してSQLインジェクションを防止する
        // RETURNINGを使用せずINSERTのみ実行する（INSERT専用ロール対応）
        let sql: &str = "INSERT INTO public.inquiries (id, name, email, created_at, body, row_log) \
                         VALUES ($1, $2, $3, $4, $5, $6)";

        let statement: Statement = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [
                id.into(),
                name.into(),
                email.into(),
                (*created_at).into(),
                message.into(),
                row_log.into(),
            ],
        );

        self.db.execute_raw(statement).await.map_err(|e| {
            tracing::error!("問い合わせの保存に失敗しました: {:?}", e);
            InquiryRepositoryError::Infrastructure
        })?;

        Ok(inquiry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "ローカルPostgreSQLの起動が必要なためデフォルトでは実行しない"]
    async fn ローカルのpostgresqlに問い合わせを保存できる() {
        use crate::db::{config::DatabaseConfig, connection::create_connection};
        use chrono::Utc;
        use domain::inquiry::value_object::{InquiryEmail, InquiryId, InquiryMessage, InquiryName};

        let db: DatabaseConnection = create_connection(&DatabaseConfig::PostgreSQL {
            host: "localhost".into(),
            port: 5432,
            database: "postgres".into(),
            username: "insertonly".into(),
            password: "insertonly".into(),
        })
        .await
        .expect("DB接続に失敗しました");

        let repository: SeaOrmInquiryRepository = SeaOrmInquiryRepository::new(db);

        let inquiry: Inquiry = Inquiry::new(
            InquiryId::new(Uuid::now_v7()),
            InquiryName::new("山田太郎").unwrap(),
            InquiryEmail::new("yamada@example.com").unwrap(),
            InquiryMessage::new("テストメッセージ").unwrap(),
            DateTime::<FixedOffset>::from(Utc::now()),
        );

        let result: Result<Inquiry, InquiryRepositoryError> = repository.save(inquiry).await;
        assert!(result.is_ok());
    }
}
