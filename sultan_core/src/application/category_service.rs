use crate::{
    application::ServiceDbHelper,
    domain::{
        Context, DomainResult,
        model::{
            category::{Category, CategoryCreate, CategoryUpdate},
            permission::{action, resource},
        },
    },
    snowflake::IdGenerator,
    storage::CategoryRepository,
};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

#[async_trait]
pub trait CategoryServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, category: &CategoryCreate) -> DomainResult<i64>;
    async fn update(&self, ctx: &Context, id: i64, category: &CategoryUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Category>>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Category>>;
}

pub struct CategoryService<R, I> {
    repo: R,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R, I> CategoryService<R, I>
where
    R: CategoryRepository,
    I: IdGenerator,
{
    pub fn new(repo: R, id_generator: I, db: DatabaseConnection) -> Self {
        Self {
            repo,
            id_generator,
            db,
        }
    }
}

impl<R, I> ServiceDbHelper for CategoryService<R, I>
where
    R: CategoryRepository,
    I: IdGenerator,
{
    fn database(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl<R, I> CategoryServiceTrait for CategoryService<R, I>
where
    R: CategoryRepository,
    I: IdGenerator,
{
    async fn create(&self, ctx: &Context, category: &CategoryCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::CATEGORY, action::CREATE)?;
        let id = self.id_generator.generate()?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.create(&repo_ctx, id, category).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, category: &CategoryUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::CATEGORY, action::UPDATE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.update(&repo_ctx, id, category).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::CATEGORY, action::DELETE)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.delete(&repo_ctx, id).await
    }

    async fn get_all(&self, ctx: &Context) -> DomainResult<Vec<Category>> {
        ctx.require_access(None, resource::CATEGORY, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.get_all(&repo_ctx).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Category>> {
        ctx.require_access(None, resource::CATEGORY, action::READ)?;
        let repo_ctx = self.repo_ctx(ctx);
        self.repo.get_by_id(&repo_ctx, id).await
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
    use sea_orm::{ConnectionTrait, Database};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Manual mock implementation that works with impl Trait
    #[derive(Clone)]
    struct MockCategoryRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, CategoryCreate) -> DomainResult<()> + Send>>>>,
        update_fn: Arc<Mutex<Option<Box<dyn Fn(i64, CategoryUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_all_fn: Arc<Mutex<Option<Box<dyn Fn() -> DomainResult<Vec<Category>> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Category>> + Send>>>>,
    }

    impl MockCategoryRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, CategoryCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, CategoryUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn() -> DomainResult<Vec<Category>> + Send + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Category>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            category: &CategoryCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, category.clone())
            } else {
                panic!("create not mocked");
            }
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            category: &CategoryUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, category.clone())
            } else {
                panic!("update not mocked");
            }
        }

        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
            let func = self.delete_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("delete not mocked");
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
        ) -> DomainResult<Vec<Category>> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f()
            } else {
                panic!("get_all not mocked");
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Category>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked");
            }
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    /// Creates a test context with full permissions for CATEGORY resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::CATEGORY, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    /// Creates a test context with no permissions
    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_category_create() -> CategoryCreate {
        CategoryCreate {
            parent_id: None,
            name: "Test Category".to_string(),
            description: Some("Test Description".to_string()),
        }
    }

    fn create_category_update() -> CategoryUpdate {
        CategoryUpdate {
            parent_id: Update::Unchanged,
            name: Some("Updated Category".to_string()),
            description: Update::Unchanged,
        }
    }

    fn create_full_category() -> Category {
        Category {
            id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            is_deleted: false,
            name: "Test Category".to_string(),
            description: Some("Test Description".to_string()),
            children: None,
        }
    }

    // ==================== Create Tests ====================

    #[tokio::test]
    async fn test_create_category_success() {
        let mut mock_repo = MockCategoryRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|id, cat| {
            assert_eq!(id, 1);
            assert_eq!(cat.name, "Test Category");
            Ok(())
        });

        let service = CategoryService::new(mock_repo, mock_id_gen, db);
        let category = create_category_create();
        let result = service.create(&ctx, &category).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_create_category_repo_error() {
        let mut mock_repo = MockCategoryRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CategoryService::new(mock_repo, mock_id_gen, db);
        let category = create_category_create();
        let result = service.create(&ctx, &category).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    #[tokio::test]
    async fn test_create_category_with_parent() {
        let mut mock_repo = MockCategoryRepo::new();
        let mock_id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_create(|id, cat| {
            assert_eq!(id, 1);
            assert_eq!(cat.parent_id, Some(100));
            Ok(())
        });

        let service = CategoryService::new(mock_repo, mock_id_gen, db);
        let category = CategoryCreate {
            parent_id: Some(100),
            name: "Child Category".to_string(),
            description: None,
        };
        let result = service.create(&ctx, &category).await;

        assert!(result.is_ok());
    }

    // ==================== Update Tests ====================

    #[tokio::test]
    async fn test_update_category_success() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|id, _| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let category = create_category_update();
        let result = service.update(&ctx, 1, &category).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_category_not_found() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|_, _| Err(Error::NotFound("Category not found".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let category = create_category_update();
        let result = service.update(&ctx, 999, &category).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_category_repo_error() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_update(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let category = create_category_update();
        let result = service.update(&ctx, 1, &category).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==================== Delete Tests ====================

    #[tokio::test]
    async fn test_delete_category_success() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|id| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_category_not_found() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|_| Err(Error::NotFound("Category not found".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 999).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_category_repo_error() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_delete(|_| Err(Error::Database("DB Error".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==================== Get All Tests ====================

    #[tokio::test]
    async fn test_get_all_categories_success() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let categories = vec![create_full_category()];
        let categories_clone = categories.clone();

        mock_repo.expect_get_all(move || Ok(categories_clone.clone()));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_all(&ctx).await;

        assert!(result.is_ok());
        let cats = result.unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "Test Category");
    }

    #[tokio::test]
    async fn test_get_all_categories_empty() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|| Ok(vec![]));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_all(&ctx).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_all_categories_repo_error() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_all(|| Err(Error::Database("DB Error".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_all(&ctx).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==================== Get By ID Tests ====================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        let expected_category = create_full_category();
        let category_clone = expected_category.clone();

        mock_repo.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(category_clone.clone()))
        });

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let category = result.unwrap();
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, expected_category.name);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_repo_error() {
        let mut mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_test_context();

        mock_repo.expect_get_by_id(|_| Err(Error::Database("DB Error".to_string())));

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // ==================== No Permission Tests ====================

    #[tokio::test]
    async fn test_create_category_no_permission() {
        let mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let category = create_category_create();
        let result = service.create(&ctx, &category).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_category_no_permission() {
        let mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let category = create_category_update();
        let result = service.update(&ctx, 1, &category).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_category_no_permission() {
        let mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_categories_no_permission() {
        let mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_all(&ctx).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockCategoryRepo::new();
        let db = create_test_db().await;
        let ctx = create_no_permission_context();

        let service = CategoryService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }
}
