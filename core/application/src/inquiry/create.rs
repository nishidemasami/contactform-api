//! # 問い合わせ作成ユースケース
//!
//! 問い合わせを作成してリポジトリに保存するユースケース。

use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

use domain::inquiry::{
    entity::Inquiry,
    repository::InquiryRepository,
    value_object::{InquiryEmail, InquiryId, InquiryMessage, InquiryName},
};

use super::{
    dto::{CreateInquiryInput, CreateInquiryOutput},
    error::CreateInquiryError,
};

/// 問い合わせ作成ユースケース
///
/// ## フロー
///
/// `CreateInquiryInput`
///   └─▶ ドメインオブジェクト生成（バリデーション）
///         └─▶ `InquiryRepository::save`
///               └─▶ `CreateInquiryOutput`
pub struct CreateInquiryUsecase {
    repository: Arc<dyn InquiryRepository>,
}

impl CreateInquiryUsecase {
    /// 問い合わせ作成ユースケースを生成する
    pub fn new(repository: Arc<dyn InquiryRepository>) -> Self {
        Self { repository }
    }

    /// 問い合わせを作成する
    ///
    /// ## エラー
    ///
    /// - `CreateInquiryError::Domain`: ドメインバリデーションエラー
    /// - `CreateInquiryError::Repository`: リポジトリエラー
    pub async fn execute(
        &self,
        input: CreateInquiryInput,
    ) -> Result<CreateInquiryOutput, CreateInquiryError> {
        let id: InquiryId = InquiryId::new(Uuid::now_v7());
        let name: InquiryName = InquiryName::new(input.name)?;
        let email: InquiryEmail = InquiryEmail::new(input.email)?;
        let message: InquiryMessage = InquiryMessage::new(input.message)?;
        let created_at: DateTime<FixedOffset> = DateTime::<FixedOffset>::from(Utc::now());

        let inquiry: Inquiry = Inquiry::new(id, name, email, message, created_at);

        let saved: Inquiry = self.repository.save(inquiry).await?;

        Ok(CreateInquiryOutput {
            id: *saved.id().value(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use domain::inquiry::{entity::Inquiry, repository_error::InquiryRepositoryError};

    use super::*;

    /// テスト用モックリポジトリ
    struct MockInquiryRepository {
        error_to_return: Mutex<Option<InquiryRepositoryError>>,
    }

    impl MockInquiryRepository {
        fn new() -> Self {
            Self {
                error_to_return: Mutex::new(None),
            }
        }

        fn set_error(&self, err: InquiryRepositoryError) {
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
    async fn 正常な入力で問い合わせを作成できる() {
        let repository: Arc<dyn InquiryRepository> = Arc::new(MockInquiryRepository::new());
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repository);

        let input: CreateInquiryInput = CreateInquiryInput {
            name: "山田太郎".to_string(),
            email: "yamada@example.com".to_string(),
            message: "お問い合わせです".to_string(),
        };

        let result: Result<CreateInquiryOutput, CreateInquiryError> = usecase.execute(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn 氏名が空の場合ドメインエラーになる() {
        let repository: Arc<dyn InquiryRepository> = Arc::new(MockInquiryRepository::new());
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repository);

        let input: CreateInquiryInput = CreateInquiryInput {
            name: "".to_string(),
            email: "yamada@example.com".to_string(),
            message: "お問い合わせです".to_string(),
        };

        let result: Result<CreateInquiryOutput, CreateInquiryError> = usecase.execute(input).await;
        assert!(matches!(result, Err(CreateInquiryError::Domain(_))));
    }

    #[tokio::test]
    async fn リポジトリエラーの場合エラーになる() {
        let mock: Arc<MockInquiryRepository> = Arc::new(MockInquiryRepository::new());
        mock.set_error(InquiryRepositoryError::Infrastructure);
        let repository: Arc<dyn InquiryRepository> = mock;
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repository);

        let input: CreateInquiryInput = CreateInquiryInput {
            name: "山田太郎".to_string(),
            email: "yamada@example.com".to_string(),
            message: "お問い合わせです".to_string(),
        };

        let result: Result<CreateInquiryOutput, CreateInquiryError> = usecase.execute(input).await;
        assert!(matches!(result, Err(CreateInquiryError::Repository(_))));
    }
}
