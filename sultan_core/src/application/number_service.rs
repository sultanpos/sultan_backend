use async_trait::async_trait;
use chrono::Datelike;

use crate::{
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
}

impl<R, B> NumberService<R, B>
where
    R: NumberRepository,
    B: BranchRepository,
{
    pub fn new(repository: R, branch_repository: B) -> Self {
        Self {
            repository,
            branch_repository,
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
            let branch = self
                .branch_repository
                .get_by_id(ctx, bid)
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
        let next_number = self.repository.generate_next(ctx, &params).await?;

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
mod tests {
    use super::*;
    use crate::domain::model::branch::Branch;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;

    mock! {
        pub NumberRepo {}
        #[async_trait]
        impl NumberRepository for NumberRepo {
            async fn generate_next(&self, ctx: &Context, params: &NumberGenerateParams) -> DomainResult<i32>;
            async fn get_sequence(&self, ctx: &Context, params: &NumberGenerateParams) -> DomainResult<Option<crate::domain::model::number::NumberSequence>>;
        }
    }

    mock! {
        pub BranchRepo {}
        #[async_trait]
        impl BranchRepository for BranchRepo {
            async fn create(&self, ctx: &Context, id: i64, branch: &crate::domain::model::branch::BranchCreate) -> DomainResult<()>;
            async fn update(&self, ctx: &Context, id: i64, branch: &crate::domain::model::branch::BranchUpdate) -> DomainResult<()>;
            async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
            async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>>;
            async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>>;
        }
    }

    fn create_test_context() -> Context {
        Context::new()
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
        let mut number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();

        number_repo.expect_generate_next().returning(|_, _| Ok(1));

        let service = NumberService::new(number_repo, branch_repo);
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
        let mut number_repo = MockNumberRepo::new();
        let branch_repo = MockBranchRepo::new();

        number_repo.expect_generate_next().returning(|_, _| Ok(1));

        let service = NumberService::new(number_repo, branch_repo);
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
        let mut number_repo = MockNumberRepo::new();
        let mut branch_repo = MockBranchRepo::new();

        number_repo.expect_generate_next().returning(|_, _| Ok(1));

        branch_repo
            .expect_get_by_id()
            .returning(|_, _| Ok(Some(create_test_branch(1, "BR01"))));

        let service = NumberService::new(number_repo, branch_repo);
        let ctx = create_test_context();

        let result = service.generate(&ctx, "CUS", Some(1), None).await;

        assert!(result.is_ok());
        let number = result.unwrap();
        // Should match pattern: BRANCH-CUS-YYNNNNN
        assert!(number.starts_with("BR01-CUS-"));
    }

    #[tokio::test]
    async fn test_generate_with_branch_and_month() {
        let mut number_repo = MockNumberRepo::new();
        let mut branch_repo = MockBranchRepo::new();

        number_repo.expect_generate_next().returning(|_, _| Ok(1));

        branch_repo
            .expect_get_by_id()
            .returning(|_, _| Ok(Some(create_test_branch(1, "BR01"))));

        let service = NumberService::new(number_repo, branch_repo);
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

        let service = NumberService::new(number_repo, branch_repo);
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
        let mut branch_repo = MockBranchRepo::new();

        branch_repo.expect_get_by_id().returning(|_, _| Ok(None));

        let service = NumberService::new(number_repo, branch_repo);
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
