use async_trait::async_trait;

use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::branch::{Branch, BranchCreate, BranchUpdate};
use crate::domain::model::permission::action;
use crate::domain::model::permission::resource;
use crate::snowflake::IdGenerator;
use crate::storage::BranchRepository;

#[async_trait]
pub trait BranchServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, branch: &BranchCreate) -> DomainResult<i64>;
    async fn update(&self, ctx: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>>;
    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>>;
}

pub struct BranchService<R, I> {
    repository: R,
    id_generator: I,
}

impl<R: BranchRepository, I: IdGenerator> BranchService<R, I> {
    pub fn new(repository: R, id_generator: I) -> Self {
        Self {
            repository,
            id_generator,
        }
    }
}

#[async_trait]
impl<R: BranchRepository, I: IdGenerator> BranchServiceTrait for BranchService<R, I> {
    async fn create(&self, ctx: &Context, branch: &BranchCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::BRANCH, action::CREATE)?;
        let id = self.id_generator.generate()?;
        self.repository.create(ctx, id, branch).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::BRANCH, action::UPDATE)?;
        self.repository.update(ctx, id, branch).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::BRANCH, action::DELETE)?;
        self.repository.delete(ctx, id).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>> {
        ctx.require_access(None, resource::BRANCH, action::READ)?;
        self.repository.get_by_id(ctx, id).await
    }

    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>> {
        ctx.require_access(None, resource::BRANCH, action::READ)?;
        self.repository.get_all(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::domain::model::Update;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use std::collections::HashMap;

    mock! {
        pub BranchRepo {}
        #[async_trait]
        impl BranchRepository for BranchRepo {
            async fn create(&self, ctx: &Context, id: i64,branch: &BranchCreate) -> DomainResult<()>;
            async fn update(&self, ctx: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()>;
            async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
            async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>>;
            async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>>;
        }
    }
    /// Creates a test context with full permissions for BRANCH resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        // Grant all actions for BRANCH resource globally (branch_id = None)
        // Using 0b1111 to cover all action values 1-4
        permissions.insert((resource::BRANCH, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    #[tokio::test]
    async fn test_create_branch() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        let branch_create = BranchCreate {
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let created_id = service
            .create(&ctx, &branch_create)
            .await
            .expect("Failed to create branch");

        assert_eq!(created_id, 1);
    }

    #[tokio::test]
    async fn test_create_branch_repo_create_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let branch = BranchCreate {
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.create(&ctx, &branch).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_branch_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let branch = BranchUpdate {
            is_main: Some(true),
            name: Some("Updated Branch".to_string()),
            code: Some("UPDATED".to_string()),
            address: Update::Clear,
            phone: Update::Clear,
            npwp: Update::Clear,
            image: Update::Clear,
        };

        mock_repo
            .expect_update()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.update(&ctx, 1, &branch).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_branch_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let update_data = BranchUpdate {
            name: Some("Updated Branch".to_string()),
            ..Default::default()
        };

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.update(&ctx, 1, &update_data).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_branch_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_branch_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_branch_by_id_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let branch = Branch {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        let branch_clone = branch.clone();
        mock_repo
            .expect_get_by_id()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(move |_, _| Ok(Some(branch_clone.clone())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let fetched_branch = result.unwrap();
        assert!(fetched_branch.is_some());
        assert_eq!(fetched_branch.unwrap().name, "Test Branch");
    }

    #[tokio::test]
    async fn test_get_branch_by_id_not_found() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_branch_by_id_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_branches_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let branch = Branch {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(move |_| Ok(vec![branch.clone()]));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.get_all(&ctx).await;

        assert!(result.is_ok());
        let branches = result.unwrap();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "Test Branch");
    }

    #[tokio::test]
    async fn test_get_all_branches_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|_| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen);
        let result = service.get_all(&ctx).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }
}
