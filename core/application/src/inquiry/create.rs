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
/// ```text
/// `CreateInquiryInput`
///   └─▶ ドメインオブジェクト生成（バリデーション）
///         └─▶ `InquiryRepository::save`
///               └─▶ `CreateInquiryOutput`
/// ```
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
        let raw_id: Uuid = Uuid::now_v7();
        let created_at: DateTime<FixedOffset> = DateTime::<FixedOffset>::from(Utc::now());
        self.execute_with_metadata(input, raw_id, created_at).await
    }

    /// 指定されたID・作成日時を用いて決定論的に問い合わせを作成・保存する
    ///
    /// システム時刻や乱数生成器（I/O副作用）を外部から注入可能にすることで、
    /// ユースケースのオーケストレーション不変条件を形式検証可能にする。
    ///
    /// ## 保証される不変条件
    ///
    /// - ドメインバリデーション失敗時は `InquiryRepository::save` を一切呼び出さない（不正データの永続化防止）
    /// - バリデーション評価順序は厳密に `name` → `email` → `message` の順とする
    /// - リポジトリが返した保存済みエンティティのIDを欠損なく出力DTOへ転写する
    pub async fn execute_with_metadata(
        &self,
        input: CreateInquiryInput,
        raw_id: Uuid,
        created_at: DateTime<FixedOffset>,
    ) -> Result<CreateInquiryOutput, CreateInquiryError> {
        let id: InquiryId = InquiryId::new(raw_id);
        let name: InquiryName = InquiryName::new(input.name)?;
        let email: InquiryEmail = InquiryEmail::new(input.email)?;
        let message: InquiryMessage = InquiryMessage::new(input.message)?;

        let inquiry: Inquiry = Inquiry::new(id, name, email, message, created_at);

        let saved: Inquiry = self.repository.save(inquiry).await?;

        Ok(CreateInquiryOutput {
            id: *saved.id().value(),
        })
    }
}

#[cfg(kani)]
mod kani_proofs {
    use core::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicUsize, Ordering},
        task::{Context, Poll, Waker},
    };

    use async_trait::async_trait;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use domain::inquiry::{
        domain_error::InquiryDomainError, entity::Inquiry, repository_error::InquiryRepositoryError,
    };

    use super::*;

    /// 即時完了する `Future` を非同期ランタイムなしで1ステップ実行する決定論的ヘルパ
    fn poll_ready<F: Future>(future: F) -> F::Output {
        let mut pinned: Pin<&mut F> = core::pin::pin!(future);
        let waker: &Waker = Waker::noop();
        let mut cx: Context<'_> = Context::from_waker(waker);
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("同期モックリポジトリは常に1回のpollで完了する"),
        }
    }

    /// 形式検証用のロックフリー記録モックリポジトリ
    struct VerificationRepository {
        save_calls: AtomicUsize,
        forced_error: Option<InquiryRepositoryError>,
        expected_id: Option<Uuid>,
    }

    impl VerificationRepository {
        fn new(forced_error: Option<InquiryRepositoryError>, expected_id: Option<Uuid>) -> Self {
            Self {
                save_calls: AtomicUsize::new(0),
                forced_error,
                expected_id,
            }
        }

        fn call_count(&self) -> usize {
            self.save_calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl InquiryRepository for VerificationRepository {
        async fn save(&self, inquiry: Inquiry) -> Result<Inquiry, InquiryRepositoryError> {
            self.save_calls.fetch_add(1, Ordering::SeqCst);

            if let Some(expected_uuid) = self.expected_id {
                assert_eq!(inquiry.id().value().as_u128(), expected_uuid.as_u128());
            }

            if let Some(ref err) = self.forced_error {
                return Err(err.clone());
            }

            Ok(inquiry)
        }
    }

    fn sample_created_at(offset_secs: i32) -> DateTime<FixedOffset> {
        let date: NaiveDate = NaiveDate::from_ymd_opt(2026, 8, 16).unwrap();
        let time: NaiveTime = NaiveTime::from_num_seconds_from_midnight_opt(12_345, 0).unwrap();
        let naive: NaiveDateTime = NaiveDateTime::new(date, time);
        let offset: FixedOffset = FixedOffset::east_opt(offset_secs).unwrap();
        DateTime::<FixedOffset>::from_naive_utc_and_offset(naive, offset)
    }

    /// 証明: いずれかの入力項目がドメインルールに違反する場合、
    /// `repository.save` は一度も呼ばれず（不正データの永続化防止）、
    /// `name` → `email` → `message` の優先順位でドメインエラーが返ること
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_usecase_never_calls_repository_on_invalid_input() {
        let name_valid: bool = kani::any();
        let email_valid: bool = kani::any();
        let message_valid: bool = kani::any();
        kani::assume(!(name_valid && email_valid && message_valid));

        let input: CreateInquiryInput = CreateInquiryInput {
            name: if name_valid {
                String::from("a")
            } else {
                String::new()
            },
            email: if email_valid {
                String::from("b")
            } else {
                String::new()
            },
            message: if message_valid {
                String::from("c")
            } else {
                String::new()
            },
        };

        let raw_id: Uuid = Uuid::from_bytes(kani::any());
        let created_at: DateTime<FixedOffset> = sample_created_at(0);

        let repo: Arc<VerificationRepository> = Arc::new(VerificationRepository::new(None, None));
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repo.clone());

        let result: Result<CreateInquiryOutput, CreateInquiryError> =
            poll_ready(usecase.execute_with_metadata(input, raw_id, created_at));

        assert_eq!(repo.call_count(), 0);

        if !name_valid {
            assert_eq!(
                result,
                Err(CreateInquiryError::Domain(InquiryDomainError::EmptyName))
            );
        } else if !email_valid {
            assert_eq!(
                result,
                Err(CreateInquiryError::Domain(InquiryDomainError::EmptyEmail))
            );
        } else {
            assert_eq!(
                result,
                Err(CreateInquiryError::Domain(InquiryDomainError::EmptyMessage))
            );
        }
    }

    /// 証明: 入力が妥当な場合、`repository.save` がちょうど1回だけ呼ばれ、
    /// 渡されたIDが検証されたうえで保存結果エンティティのIDが `CreateInquiryOutput` に返ること
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_usecase_saves_once_and_returns_saved_id() {
        let raw_id: Uuid = Uuid::from_bytes(kani::any());
        let created_at: DateTime<FixedOffset> = sample_created_at(32_400);

        let input: CreateInquiryInput = CreateInquiryInput {
            name: String::from("a"),
            email: String::from("b"),
            message: String::from("c"),
        };

        let repo: Arc<VerificationRepository> =
            Arc::new(VerificationRepository::new(None, Some(raw_id)));
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repo.clone());

        let result: Result<CreateInquiryOutput, CreateInquiryError> =
            poll_ready(usecase.execute_with_metadata(input, raw_id, created_at));

        assert_eq!(repo.call_count(), 1);
        let output: CreateInquiryOutput =
            result.expect("妥当な入力と正常リポジトリでは必ず成功する");
        assert_eq!(output.id.as_u128(), raw_id.as_u128());
    }

    /// 証明: リポジトリ層の全4種類のエラーバリアントが欠損・変質なく
    /// `CreateInquiryError::Repository` として伝播すること
    #[kani::proof]
    #[kani::unwind(6)]
    fn proof_usecase_propagates_all_repository_errors() {
        let selector: u8 = kani::any();
        kani::assume(selector < 4);
        let repo_err: InquiryRepositoryError = match selector {
            0 => InquiryRepositoryError::Duplicate,
            1 => InquiryRepositoryError::NotFound,
            2 => InquiryRepositoryError::Infrastructure,
            _ => InquiryRepositoryError::CorruptedData,
        };

        let input: CreateInquiryInput = CreateInquiryInput {
            name: String::from("a"),
            email: String::from("b"),
            message: String::from("c"),
        };

        let raw_id: Uuid = Uuid::from_bytes(kani::any());
        let created_at: DateTime<FixedOffset> = sample_created_at(0);

        let repo: Arc<VerificationRepository> = Arc::new(VerificationRepository::new(
            Some(repo_err.clone()),
            Some(raw_id),
        ));
        let usecase: CreateInquiryUsecase = CreateInquiryUsecase::new(repo.clone());

        let result: Result<CreateInquiryOutput, CreateInquiryError> =
            poll_ready(usecase.execute_with_metadata(input, raw_id, created_at));

        assert_eq!(repo.call_count(), 1);
        assert_eq!(result, Err(CreateInquiryError::Repository(repo_err)));
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
