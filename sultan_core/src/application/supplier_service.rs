use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    domain::{
        Context, DomainResult,
        model::{
            pagination::PaginationOptions,
            permission::{action, resource},
            supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
        },
    },
    snowflake::IdGenerator,
    storage::{RepoCtx, SupplierRepository},
};

#[async_trait]
pub trait SupplierServiceTrait: Send + Sync {
    async fn create(&self, ctx: &Context, supplier: &SupplierCreate) -> DomainResult<i64>;
    async fn update(&self, ctx: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()>;
    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Supplier>>;
    async fn get_all(
        &self,
        ctx: &Context,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>>;
}

pub struct SupplierService<R, I> {
    repository: R,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R, I> SupplierService<R, I>
where
    R: SupplierRepository,
    I: IdGenerator,
{
    pub fn new(repository: R, id_generator: I, db: DatabaseConnection) -> Self {
        Self {
            repository,
            id_generator,
            db,
        }
    }
}

#[async_trait]
impl<R, I> SupplierServiceTrait for SupplierService<R, I>
where
    R: SupplierRepository,
    I: IdGenerator,
{
    async fn create(&self, ctx: &Context, supplier: &SupplierCreate) -> DomainResult<i64> {
        ctx.require_access(None, resource::SUPPLIER, action::CREATE)?;
        let id = self.id_generator.generate()?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.create(&repo_ctx, id, supplier).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::SUPPLIER, action::UPDATE)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.update(&repo_ctx, id, supplier).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::SUPPLIER, action::DELETE)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.delete(&repo_ctx, id).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Supplier>> {
        ctx.require_access(None, resource::SUPPLIER, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>> {
        ctx.require_access(None, resource::SUPPLIER, action::READ)?;
        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };
        self.repository.get_all(&repo_ctx, filter, pagination).await
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
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Manual mock implementation that works with impl Trait
    #[derive(Clone)]
    struct MockSupplierRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, SupplierCreate) -> DomainResult<()> + Send>>>>,
        update_fn: Arc<Mutex<Option<Box<dyn Fn(i64, SupplierUpdate) -> DomainResult<()> + Send>>>>,
        delete_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Supplier>> + Send>>>>,
        get_all_fn: Arc<
            Mutex<
                Option<
                    Box<
                        dyn Fn(SupplierFilter, PaginationOptions) -> DomainResult<Vec<Supplier>>
                            + Send,
                    >,
                >,
            >,
        >,
    }

    impl MockSupplierRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                update_fn: Arc::new(Mutex::new(None)),
                delete_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                get_all_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, SupplierCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update<F>(&mut self, f: F)
        where
            F: Fn(i64, SupplierUpdate) -> DomainResult<()> + Send + 'static,
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
            F: Fn(i64) -> DomainResult<Option<Supplier>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_all<F>(&mut self, f: F)
        where
            F: Fn(SupplierFilter, PaginationOptions) -> DomainResult<Vec<Supplier>>
                + Send
                + 'static,
        {
            *self.get_all_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl SupplierRepository for MockSupplierRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            supplier: &SupplierCreate,
        ) -> DomainResult<()> {
            let func = self.create_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, supplier.clone())
            } else {
                panic!("create not mocked")
            }
        }

        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            supplier: &SupplierUpdate,
        ) -> DomainResult<()> {
            let func = self.update_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id, supplier.clone())
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
        ) -> DomainResult<Option<Supplier>> {
            let func = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked")
            }
        }

        async fn get_all(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            filter: &SupplierFilter,
            pagination: &PaginationOptions,
        ) -> DomainResult<Vec<Supplier>> {
            let func = self.get_all_fn.lock().unwrap();
            if let Some(f) = func.as_ref() {
                f(filter.clone(), pagination.clone())
            } else {
                panic!("get_all not mocked")
            }
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    /// Creates a test context with full permissions for SUPPLIER resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::SUPPLIER, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    /// Creates a test context with no permissions
    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_test_supplier_create() -> SupplierCreate {
        SupplierCreate {
            name: "Test Supplier".to_string(),
            code: Some("TEST001".to_string()),
            email: Some("test@supplier.com".to_string()),
            address: Some("123 Test St".to_string()),
            phone: Some("555-1234".to_string()),
            npwp: Some("12345678901234".to_string()),
            npwp_name: Some("PT Test Supplier".to_string()),
            metadata: None,
        }
    }

    fn create_full_supplier() -> Supplier {
        Supplier {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            name: "Test Supplier".to_string(),
            code: Some("TEST001".to_string()),
            email: Some("test@supplier.com".to_string()),
            address: Some("123 Test St".to_string()),
            phone: Some("555-1234".to_string()),
            npwp: Some("12345678901234".to_string()),
            npwp_name: Some("PT Test Supplier".to_string()),
            metadata: None,
        }
    }

    fn create_supplier_update() -> SupplierUpdate {
        SupplierUpdate {
            name: Some("Updated Supplier".to_string()),
            code: Update::Set("UPD001".to_string()),
            email: Update::Unchanged,
            address: Update::Unchanged,
            phone: Update::Unchanged,
            npwp: Update::Unchanged,
            npwp_name: Update::Unchanged,
            metadata: Update::Unchanged,
        }
    }

    fn create_default_filter() -> SupplierFilter {
        SupplierFilter {
            name: None,
            code: None,
            phone: None,
            npwp: None,
            email: None,
        }
    }

    fn create_default_pagination() -> PaginationOptions {
        PaginationOptions::new(1, 10, None)
    }

    // =============================================================================
    // Create Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_supplier_success() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_create(|_, supplier| {
            assert_eq!(supplier.name, "Test Supplier");
            Ok(())
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let supplier = create_test_supplier_create();
        let result = service.create(&ctx, &supplier).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();
        let db = create_test_db().await;

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let supplier = create_test_supplier_create();
        let result = service.create(&ctx, &supplier).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_create(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let supplier = create_test_supplier_create();
        let result = service.create(&ctx, &supplier).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Update Tests
    // =============================================================================

    #[tokio::test]
    async fn test_update_supplier_success() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_update(|id, _| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();
        let db = create_test_db().await;

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_update(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_supplier_not_found() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_update(|_, _| Err(Error::NotFound("Supplier not found".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let update = create_supplier_update();
        let result = service.update(&ctx, 999, &update).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Delete Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_supplier_success() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_delete(|id| {
            assert_eq!(id, 1);
            Ok(())
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();
        let db = create_test_db().await;

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_delete(|_| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_supplier_not_found() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_delete(|_| Err(Error::NotFound("Supplier not found".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.delete(&ctx, 999).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Get By ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        let expected_supplier = create_full_supplier();
        let supplier_clone = expected_supplier.clone();

        mock_repo.expect_get_by_id(move |id| {
            assert_eq!(id, 1);
            Ok(Some(supplier_clone.clone()))
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let supplier = result.unwrap();
        assert!(supplier.is_some());
        let supplier = supplier.unwrap();
        assert_eq!(supplier.name, expected_supplier.name);
        assert_eq!(supplier.code, expected_supplier.code);
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_get_by_id(|_| Ok(None));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();
        let db = create_test_db().await;

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_get_by_id(|_| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    // =============================================================================
    // Get All Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_all_success() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo.expect_get_all(move |_, _| Ok(suppliers_clone.clone()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        let result_suppliers = result.unwrap();
        assert_eq!(result_suppliers.len(), 1);
        assert_eq!(result_suppliers[0].name, "Test Supplier");
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_get_all(|_, _| Ok(vec![]));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_all_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();
        let db = create_test_db().await;

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        mock_repo.expect_get_all(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_with_filter() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo.expect_get_all(move |filter, _| {
            assert_eq!(filter.name, Some("Test".to_string()));
            Ok(suppliers_clone.clone())
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = SupplierFilter {
            name: Some("Test".to_string()),
            code: None,
            phone: None,
            npwp: None,
            email: None,
        };
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
        let result_suppliers = result.unwrap();
        assert_eq!(result_suppliers.len(), 1);
    }

    #[tokio::test]
    async fn test_get_all_with_pagination() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();
        let db = create_test_db().await;

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo.expect_get_all(move |_, pagination| {
            assert_eq!(pagination.page, 2);
            assert_eq!(pagination.page_size, 20);
            Ok(suppliers_clone.clone())
        });

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1), db);
        let filter = create_default_filter();
        let pagination = PaginationOptions::new(2, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
    }
}
