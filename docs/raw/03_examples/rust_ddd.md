# Rust DDDレイヤー別 実装例

## ワークスペース親 `Cargo.toml` 例

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
panic = "abort"
strip = true

[workspace]
members = [
  "api/lambda",
  "core/application",
  "core/domain",
  "core/infrastructure",
  "core/presentation",
  "db/sea_orm_entities",
]
resolver = "3"

[workspace.package]
edition = "2024"

[workspace.dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["clock", "serde"] }
sea-orm = { version = "2", features = [
  "sqlx-postgres",
  "runtime-tokio-rustls",
  "macros",
  "with-chrono",
  "with-uuid",
] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt"] }
uuid = { version = "1", features = ["serde", "v7"] }
anyhow = "1"
aurora-dsql-sqlx-connector = { version = "0.2", features = ["pool"] }
tracing = "0.1"
lambda_runtime = "0.13"
aws_lambda_events = { version = "0.15", default-features = false, features = ["apigw"] }
utoipa = { version = "4", features = ["uuid"] }

# テスト用ライブラリ
rstest = "0.26.0"
```

---

## 1. ドメイン層 (Domain)

### `core/domain/src/inquiry/entity.rs`
```rust
pub struct Inquiry {
    id: InquiryId,
    name: InquiryName,
    email: InquiryEmail,
    message: InquiryMessage,
    created_at: DateTime<FixedOffset>,
}
```

### `core/domain/src/inquiry/repository.rs`
```rust
use async_trait::async_trait;

use super::{
    entity::Inquiry,
    repository_error::InquiryRepositoryError,
};

#[async_trait]
pub trait InquiryRepository: Send + Sync {
    async fn save(
        &self,
        inquiry: Inquiry,
    ) -> Result<Inquiry, InquiryRepositoryError>;
}
```

### `core/domain/src/inquiry/value_object.rs`
```rust
use super::domain_error::InquiryDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InquiryEmail(String);

impl InquiryEmail {
    pub fn new(
        value: impl Into<String>,
    ) -> Result<Self, InquiryDomainError> {
        let value: String = value.into();

        if value.trim().is_empty() {
            return Err(InquiryDomainError::EmptyEmail);
        }

        if value.chars().count() > 256 {
            return Err(InquiryDomainError::InvalidEmail);
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
```

---

## 2. アプリケーション層 (Application)

### `core/application/src/inquiry/dto.rs`
```rust
pub struct CreateInquiryInput {
    pub name: String,
    pub email: String,
    pub message: String,
}

pub struct CreateInquiryOutput {
    pub id: Uuid,
}
```

### `core/application/src/inquiry/create.rs`
```rust
pub struct CreateInquiryUsecase {
    repository: Arc<dyn InquiryRepository>,
}

impl CreateInquiryUsecase {
    pub fn new(
        repository: Arc<dyn InquiryRepository>,
    ) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: CreateInquiryInput,
    ) -> Result<CreateInquiryOutput, CreateInquiryError> {
        let inquiry: Inquiry = Inquiry::new(
            InquiryId::new(Uuid::now_v7()),
            InquiryName::new(input.name)?,
            InquiryEmail::new(input.email)?,
            InquiryMessage::new(input.message)?,
            DateTime::<FixedOffset>::from(Utc::now()),
        );

        let saved: Inquiry =
            self.repository.save(inquiry).await?;

        Ok(CreateInquiryOutput { id: *saved.id().value() })
    }
}
```

---

## 3. インフラストラクチャ層 (Infrastructure)

### `core/infrastructure/src/repository/inquiry/seaorm_inquiry_repository.rs`
```rust
pub struct SeaOrmInquiryRepository {
    db: DatabaseConnection,
}

#[async_trait]
impl InquiryRepository for SeaOrmInquiryRepository {
    async fn save(
        &self,
        inquiry: Inquiry,
    ) -> Result<Inquiry, InquiryRepositoryError> {
        let active_model = to_active_model(&inquiry);

        let result = Inquiry::insert(active_model)
            .exec_without_returning(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Database execution failed: {:?}", e);
                InquiryRepositoryError::Infrastructure
            })?;

        Ok(inquiry)
    }
}
```

---

## 4. プレゼンテーション層 (Presentation)

### `core/presentation/src/api/request.rs`
```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 問い合わせ作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CreateInquiryRequest {
    pub name: String,
    pub email: String,
    pub message: String,
}
```

### `core/presentation/src/api/openapi.rs`
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Inquiry API",
        version = "1.0.0",
        description = "問い合わせシステムAPI"
    ),
    paths(
        create_inquiry_doc,
        find_inquiry_doc,
    ),
    components(
        schemas(
            CreateInquiryRequest,
            CreateInquiryResponse,
            InquiryResponse,
            ErrorResponse,
        )
    )
)]
pub struct ApiDoc;
```
