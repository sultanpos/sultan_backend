use async_trait::async_trait;
use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::branch::{Branch, BranchCreate, BranchUpdate};
use crate::domain::model::permission::action;
use crate::domain::model::permission::resource;
use crate::snowflake::IdGenerator;
use crate::storage::{BranchRepository, RepoCtx};

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
    db: DatabaseConnection,
}

impl<R: BranchRepository, I: IdGenerator> BranchService<R, I> {
    pub fn new(repository: R, id_generator: I, db: DatabaseConnection) -> Self {
        Self {
            repository,
            id_generator,
            db,
        }
    }
}

#[async_trait]
impl<R: BranchRepository, I: IdGenerator> BranchServiceTrait for BranchService<R, I> {
    async fn create(&self, ctx: &Context, branch: &BranchCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::BRANCH, action::CREATE)?;
        let id = self.id_generator.generate()?;

        // Use transaction to ensure atomicity
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        // If this branch is being set as main, unset all other branches first
        if branch.is_main {
            self.repository
                .set_all_is_main_false(&repo_ctx, Some(id))
                .await?;
        }

        self.repository.create(&repo_ctx, id, branch).await?;

        // Commit the transaction
        repo_ctx.db.commit().await?;

        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, branch: &BranchUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::BRANCH, action::UPDATE)?;

        // Use transaction to ensure atomicity
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        // If this branch is being set as main, unset all other branches first
        if let Some(true) = branch.is_main {
            self.repository
                .set_all_is_main_false(&repo_ctx, Some(id))
                .await?;
        }

        self.repository.update(&repo_ctx, id, branch).await?;

        // Commit the transaction
        repo_ctx.db.commit().await?;

        Ok(())
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::BRANCH, action::DELETE)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.delete(&repo_ctx, id).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Branch>> {
        ctx.require_access(None, resource::BRANCH, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Branch>> {
        ctx.require_access(None, resource::BRANCH, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_all(&repo_ctx).await
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::create_mock_id_gen;
    use crate::domain::Error;
    use crate::domain::model::Update;
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Manual mock implementation that works with impl Trait
    #[derive(Clone)]
    struct MockBranchRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, BranchCreate) -> DomainResult<()> + Send>>>>,
        update_fn: Arc<Mutex<Option<Box<dyn Fn(i64, BranchUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Branch>> + Send>>>>,
        get_all_fn: Arc<Mutex<Option<Box<dyn Fn() -> DomainResult<Vec<Branch>> + Send>>>>,
        set_all_is_main_false_fn:
            Arc<Mutex<Option<Box<dyn Fn(Option<i64>) -> DomainResult<()> + Send>>>>,
    }

    impl MockBranchRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
                set_all_is_main_false_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, BranchCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, BranchUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Branch>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn() -> DomainResult<Vec<Branch>> + Send + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_set_all_is_main_false<F>(&mut self, f: F)
        where
            F: Fn(Option<i64>) -> DomainResult<()> + Send + 'static,
        {
            *self.set_all_is_main_false_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl BranchRepository for MockBranchRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            branch: &BranchCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, branch.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            branch: &BranchUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, branch.clone())
            } else {
                panic!("update not mocked")
            }
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
            let func = self.delete_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("delete not mocked")
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Branch>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn get_all(&self, _ctx: &RepoCtx<impl ConnectionTrait>) -> DomainResult<Vec<Branch>> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f()
            } else {
                panic!("get_all not mocked")
            }
        }

        async fn set_all_is_main_false(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            except_id: Option<i64>,
        ) -> DomainResult<()> {
            let func = self.set_all_is_main_false_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(except_id)
            } else {
                // By default, return Ok(()) for tests that don't care about this
                Ok(())
            }
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

    /// Creates an in-memory test database connection
    async fn create_test_db() -> DatabaseConnection {
        sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database")
    }

    #[tokio::test]
    async fn test_create_branch() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;

        let branch_create = BranchCreate {
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        mock_repo.expect_create(|_id, _branch| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
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
        let db = create_test_db().await;
        let branch = BranchCreate {
            is_main: true,
            name: "Test Branch".to_string(),
            code: "TEST".to_string(),
            address: None,
            phone: None,
            npwp: None,
            image: None,
        };

        mock_repo.expect_create(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.create(&ctx, &branch).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_branch_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;
        let branch = BranchUpdate {
            is_main: Some(true),
            name: Some("Updated Branch".to_string()),
            code: Some("UPDATED".to_string()),
            address: Update::Clear,
            phone: Update::Clear,
            npwp: Update::Clear,
            image: Update::Clear,
        };

        mock_repo.expect_update(|_, _| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.update(&ctx, 1, &branch).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_branch_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;
        let update_data = BranchUpdate {
            name: Some("Updated Branch".to_string()),
            ..Default::default()
        };

        mock_repo.expect_update(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.update(&ctx, 1, &update_data).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_branch_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_delete(|_| Ok(()));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_branch_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_delete(|_| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_branch_by_id_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;
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
        mock_repo.expect_get_by_id(move |_| Ok(Some(branch_clone.clone())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
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
        let db = create_test_db().await;

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_branch_by_id_error() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_get_by_id(|_| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_branches_success() {
        let mut mock_repo = MockBranchRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let ctx = create_test_context();
        let db = create_test_db().await;
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

        mock_repo.expect_get_all(move || Ok(vec![branch.clone()]));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
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
        let db = create_test_db().await;

        mock_repo.expect_get_all(|| Err(Error::Database("DB Error".to_string())));

        let service = BranchService::new(mock_repo, mock_id_gen, db);
        let result = service.get_all(&ctx).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }
}
