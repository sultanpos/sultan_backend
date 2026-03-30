use async_trait::async_trait;
use chrono::Datelike;
use sea_orm::DatabaseConnection;

use crate::{
    application::ServiceDbHelper,
    domain::{Context, DomainResult, Error, model::number::NumberGenerateParams},
    storage::{BranchRepository, NumberRepository},
};

/// Service trait for number generation
///
/// This service generates formatted sequential identifiers for various entity types.
/// Format patterns:
/// - Base: PREFIX-YYNNNNN (e.g., CUS-2500001)
/// - With month: PREFIX-YYMMNNNN (e.g., CUS-25120001)
/// - With branch: BRANCH-PREFIX-YY[MM]NNNN (e.g., BRANCH-CUS-25120001)
#[async_trait]
pub trait NumberServiceTrait: Send + Sync {
    /// Generate a formatted number string
    ///
    /// # Arguments
    /// * `ctx` - Execution context
    /// * `prefix` - Entity prefix code (e.g., "CUS", "SUP", "SALE")
    /// * `branch_id` - Optional branch ID for branch-specific numbering
    /// * `month` - Optional month (1-12) for month-based numbering
    ///
    /// # Returns
    /// A formatted number string following the pattern rules
    async fn generate(
        &self,
        ctx: &Context,
        prefix: &str,
        branch_id: Option<i64>,
        month: Option<i32>,
    ) -> DomainResult<String>;
}

pub struct NumberService<R, B> {
    repository: R,
    branch_repository: B,
    db: DatabaseConnection,
}

impl<R, B> NumberService<R, B>
where
    R: NumberRepository,
    B: BranchRepository,
{
    pub fn new(repository: R, branch_repository: B, db: DatabaseConnection) -> Self {
        Self {
            repository,
            branch_repository,
            db,
        }
    }
    /// Format the number according to specification
    fn format_number(
        branch_code: Option<String>,
        prefix: &str,
        year: i32,
        month: Option<i32>,
        number: i32,
    ) -> String {
        let year_suffix = year % 100; // Get last 2 digits of year

        let number_part = if let Some(m) = month {
            format!("{:02}{:04}", m, number)
        } else {
            format!("{:05}", number)
        };

        if let Some(branch) = branch_code {
            format!("{}-{}-{}{}", branch, prefix, year_suffix, number_part)
        } else {
            format!("{}-{}{}", prefix, year_suffix, number_part)
        }
    }
}

impl<R, B> ServiceDbHelper for NumberService<R, B>
where
    R: NumberRepository,
    B: BranchRepository,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, B> NumberServiceTrait for NumberService<R, B>
where
    R: NumberRepository,
    B: BranchRepository,
{
    async fn generate(
        &self,
        ctx: &Context,
        prefix: &str,
        branch_id: Option<i64>,
        month: Option<i32>,
    ) -> DomainResult<String> {
        // Validate month if provided
        if let Some(m) = month
            && !(1..=12).contains(&m)
        {
            return Err(Error::ValidationError(format!(
                "Month must be between 1 and 12, got {}",
                m
            )));
        }

        // Get branch code if branch_id is provided
        let branch_code = if let Some(bid) = branch_id {
            let repo_ctx = self.repo_ctx(ctx);
            let branch = self
                .branch_repository
                .get_by_id(&repo_ctx, bid)
                .await?
                .ok_or_else(|| Error::NotFound(format!("Branch with id {} not found", bid)))?;
            Some(branch.code)
        } else {
            None
        };

        // Get current year
        let now = chrono::Utc::now();
        let year = now.year();

        // Generate parameters
        let params = NumberGenerateParams {
            prefix: prefix.to_string(),
            branch_id,
            year,
            month,
        };

        // Generate next number atomically
        let repo_ctx = self.repo_ctx(ctx);
        let next_number = self.repository.generate_next(&repo_ctx, &params).await?;

        // Format and return the number string
        Ok(Self::format_number(
            branch_code,
            prefix,
            year,
            month,
            next_number,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::domain::model::branch::Branch;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, Database};
    use std::sync::{Arc, Mutex};

    // Manual mock for NumberRepository (mockall doesn't support impl Trait)
    struct MockNumberRepo {
        generate_next_fn:
            Arc<Mutex<Option<Box<dyn Fn(&NumberGenerateParams) -> DomainResult<i32> + Send>>>>,
    }

    impl MockNumberRepo {
        fn new() -> Self {
            Self {
                generate_next_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_generate_next<F>(&self, f: F)
        where
            F: Fn(&NumberGenerateParams) -> DomainResult<i32> + Send + 'static,
        {
            *self.generate_next_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl NumberRepository for MockNumberRepo {
        async fn generate_next(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            params: &NumberGenerateParams,
        ) -> DomainResult<i32> {
            let lock = self.generate_next_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(params)
            } else {
                panic!("generate_next not mocked");
            }
        }

        async fn get_sequence(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _params: &NumberGenerateParams,
        ) -> DomainResult<Option<crate::domain::model::number::NumberSequence>> {
            panic!("get_sequence not mocked");
        }
    }

    // Manual mock for BranchRepository (mockall doesn't support impl Trait)
    struct MockBranchRepo {
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Branch>> + Send>>>>,
    }

    impl MockBranchRepo {
        fn new() -> Self {
            Self {
                get_by_id_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_get_by_id<F>(&self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Branch>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl BranchRepository for MockBranchRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
            _branch: &crate::domain::model::branch::BranchCreate,
        ) -> DomainResult<()> {
            panic!("create not mocked");
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
            _branch: &crate::domain::model::branch::BranchUpdate,
        ) -> DomainResult<()> {
            panic!("update not mocked");
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, _id: i64) -> DomainResult<()> {
            panic!("delete not mocked");
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Branch>> {
            let lock = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked");
            }
        }

        async fn get_all(&self, _ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<Branch>> {
            panic!("get_all not mocked");
        }

        async fn set_all_is_main_false(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _except_id: Option<i64>,
        ) -> DomainResult<()> {
            // Not needed for number service tests
            Ok(())
        }
    }

    fn create_test_context() -> Context {
        Context::new()
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    fn create_test_branch(id: i64, code: &str) -> Branch {
        Branch {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            is_main: false,
            name: "Test Branch".to_string(),
            code: code.to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        }
    }

    #[tokio::test]
    async fn test_generate_global_number() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        number_repo.expect_generate_next(|_| Ok(1));

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", None, None).await;

        assert!(result.is_ok());
        let number = result.unwrap();
        // Should match pattern: CUS-YYNNNNN
        assert!(number.starts_with("CUS-"));
        assert_eq!(number.len(), 11); // CUS-YYNNNNN = 11 chars
    }

    #[tokio::test]
    async fn test_generate_with_month() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        number_repo.expect_generate_next(|_| Ok(1));

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", None, Some(12)).await;

        assert!(result.is_ok());
        let number = result.unwrap();
        // Should match pattern: CUS-YYMMNNNN
        assert!(number.starts_with("CUS-"));
        assert!(number.contains("12")); // Month included
        // CUS-26120001 = 12 chars (not 13)
        assert_eq!(number.len(), 12); // CUS-YYMMNNNN = 12 chars
    }

    #[tokio::test]
    async fn test_generate_with_branch() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        number_repo.expect_generate_next(|_| Ok(1));

        branch_repo.expect_get_by_id(|_| Ok(Some(create_test_branch(1, "BR01"))));

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", Some(1), None).await;

        assert!(result.is_ok());
        let number = result.unwrap();
        // Should match pattern: BRANCH-CUS-YYNNNNN
        assert!(number.starts_with("BR01-CUS-"));
    }

    #[tokio::test]
    async fn test_generate_with_branch_and_month() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        number_repo.expect_generate_next(|_| Ok(1));

        branch_repo.expect_get_by_id(|_| Ok(Some(create_test_branch(1, "BR01"))));

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", Some(1), Some(6)).await;

        assert!(result.is_ok());
        let number = result.unwrap();
        // Should match pattern: BRANCH-CUS-YYMMNNNN
        assert!(number.starts_with("BR01-CUS-"));
        assert!(number.contains("06")); // Month included
    }

    #[tokio::test]
    async fn test_invalid_month() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", None, Some(13)).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ValidationError(msg) => {
                assert!(msg.contains("Month must be between 1 and 12"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_branch_not_found() {
        let number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();
        let db = create_test_db().await;

        branch_repo.expect_get_by_id(|_| Ok(None));

        let service = NumberService::new(number_repo, branch_repo, db);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", Some(999), None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NotFound(msg) => {
                assert!(msg.contains("Branch with id 999 not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_format_number_global() {
        let result = NumberService::<MockNumberRepo, MockBranchRepo>::format_number(
            None, "CUS", 2025, None, 1,
        );
        assert_eq!(result, "CUS-2500001");
    }

    #[test]
    fn test_format_number_with_month() {
        let result = NumberService::<MockNumberRepo, MockBranchRepo>::format_number(
            None,
            "CUS",
            2025,
            Some(12),
            1,
        );
        assert_eq!(result, "CUS-25120001");
    }

    #[test]
    fn test_format_number_with_branch() {
        let result = NumberService::<MockNumberRepo, MockBranchRepo>::format_number(
            Some("BR01".to_string()),
            "CUS",
            2025,
            None,
            1,
        );
        assert_eq!(result, "BR01-CUS-2500001");
    }

    #[test]
    fn test_format_number_with_branch_and_month() {
        let result = NumberService::<MockNumberRepo, MockBranchRepo>::format_number(
            Some("BR01".to_string()),
            "CUS",
            2025,
            Some(6),
            1,
        );
        assert_eq!(result, "BR01-CUS-25060001");
    }
}
