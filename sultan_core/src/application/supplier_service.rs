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
    storage::SupplierRepository,
};
use async_trait::async_trait;

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
}

impl<R, I> SupplierService<R, I>
where
    R: SupplierRepository,
    I: IdGenerator,
{
    pub fn new(repository: R, id_generator: I) -> Self {
        Self {
            repository,
            id_generator,
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
        self.repository.create(ctx, id, supplier).await?;
        Ok(id)
    }

    async fn update(&self, ctx: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()> {
        ctx.require_access(None, resource::SUPPLIER, action::UPDATE)?;
        self.repository.update(ctx, id, supplier).await
    }

    async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::SUPPLIER, action::DELETE)?;
        self.repository.delete(ctx, id).await
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Supplier>> {
        ctx.require_access(None, resource::SUPPLIER, action::READ)?;
        self.repository.get_by_id(ctx, id).await
    }

    async fn get_all(
        &self,
        ctx: &Context,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>> {
        ctx.require_access(None, resource::SUPPLIER, action::READ)?;
        self.repository.get_all(ctx, filter, pagination).await
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
        pub SupplierRepo {}
        #[async_trait]
        impl SupplierRepository for SupplierRepo {
            async fn create(&self, ctx: &Context, id: i64,supplier: &SupplierCreate) -> DomainResult<()>;
            async fn update(&self, ctx: &Context, id: i64, supplier: &SupplierUpdate) -> DomainResult<()>;
            async fn delete(&self, ctx: &Context, id: i64) -> DomainResult<()>;
            async fn get_all(&self, ctx: &Context, filter: &SupplierFilter, pagination: &PaginationOptions) -> DomainResult<Vec<Supplier>>;
            async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Supplier>>;
        }
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

        mock_repo
            .expect_create()
            .withf(|_, _, supplier| supplier.name == "Test Supplier")
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let supplier = create_test_supplier_create();
        let result = service.create(&ctx, &supplier).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let supplier = create_test_supplier_create();
        let result = service.create(&ctx, &supplier).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        mock_repo
            .expect_update()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(1),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let update = create_supplier_update();
        let result = service.update(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_update_supplier_not_found() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update()
            .times(1)
            .returning(|_, _, _| Err(Error::NotFound("Supplier not found".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        mock_repo
            .expect_delete()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(|_, _| Ok(()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_supplier_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_supplier_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let result = service.delete(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_delete_supplier_not_found() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(Error::NotFound("Supplier not found".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        let expected_supplier = create_full_supplier();
        let supplier_clone = expected_supplier.clone();

        mock_repo
            .expect_get_by_id()
            .with(mockall::predicate::always(), mockall::predicate::eq(1))
            .times(1)
            .returning(move |_, _| Ok(Some(supplier_clone.clone())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockSupplierRepo::new();
        let ctx = create_no_permission_context();

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo
            .expect_get_all()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(suppliers_clone.clone()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_get_all_repo_error() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_all()
            .times(1)
            .returning(|_, _, _| Err(Error::Database("DB Error".to_string())));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = create_default_pagination();
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(matches!(result, Err(Error::Database(msg)) if msg == "DB Error"));
    }

    #[tokio::test]
    async fn test_get_all_with_filter() {
        let mut mock_repo = MockSupplierRepo::new();
        let ctx = create_test_context();

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo
            .expect_get_all()
            .withf(|_, filter, _| filter.name == Some("Test".to_string()))
            .times(1)
            .returning(move |_, _, _| Ok(suppliers_clone.clone()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
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

        let suppliers = vec![create_full_supplier()];
        let suppliers_clone = suppliers.clone();

        mock_repo
            .expect_get_all()
            .withf(|_, _, pagination| pagination.page == 2 && pagination.page_size == 20)
            .times(1)
            .returning(move |_, _, _| Ok(suppliers_clone.clone()));

        let service = SupplierService::new(mock_repo, create_mock_id_gen(1));
        let filter = create_default_filter();
        let pagination = PaginationOptions::new(2, 20, None);
        let result = service.get_all(&ctx, &filter, &pagination).await;

        assert!(result.is_ok());
    }
}
