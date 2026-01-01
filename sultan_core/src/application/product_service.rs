use crate::snowflake::IdGenerator;
use crate::{
    domain::{
        Context, DomainResult,
        model::{
            permission::{action, resource},
            product::{
                Product, ProductCreate, ProductUpdate, ProductVariant, ProductVariantCreate,
                ProductVariantUpdate,
            },
        },
    },
    storage::{ProductRepository, transaction::TransactionManager},
};
use async_trait::async_trait;

#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    async fn create_product(
        &self,
        ctx: &Context,
        product: &ProductCreate,
        variants: &[ProductVariantCreate],
    ) -> DomainResult<i64>;
    async fn update_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()>;
    async fn delete_product(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>>;
    async fn create_variant(
        &self,
        ctx: &Context,
        variant: &ProductVariantCreate,
    ) -> DomainResult<i64>;
    async fn update_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()>;
    async fn delete_variant(&self, ctx: &Context, id: i64) -> DomainResult<()>;
    async fn delete_variants_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<()>;
    async fn get_variant_by_barcode(
        &self,
        ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>>;
    async fn get_variant_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>>;
    async fn get_variant_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>>;
}

pub struct ProductService<R, T, I> {
    repository: R,
    tx_manager: T,
    id_generator: I,
}

impl<R, T, I> ProductService<R, T, I>
where
    T: TransactionManager,
    I: IdGenerator,
{
    pub fn new(repository: R, tx_manager: T, id_generator: I) -> Self {
        Self {
            repository,
            tx_manager,
            id_generator,
        }
    }
}

#[async_trait]
impl<R, T, I> ProductServiceTrait for ProductService<R, T, I>
where
    for<'a> R: ProductRepository<T::Transaction<'a>>,
    for<'a> T::Transaction<'a>: Send,
    T: TransactionManager,
    I: IdGenerator,
{
    async fn create_product(
        &self,
        ctx: &Context,
        product: &ProductCreate,
        variants: &[ProductVariantCreate],
    ) -> DomainResult<i64> {
        ctx.require_access(None, resource::PRODUCT, action::CREATE)?;
        let mut tx = self.tx_manager.begin().await?;

        let id = self.id_generator.generate()?;
        if let Err(e) = self
            .repository
            .create_product(ctx, id, product, &mut tx)
            .await
        {
            let _ = self.tx_manager.rollback(tx).await;
            return Err(e);
        }

        // Insert all variants
        for variant in variants {
            let variant_id = self.id_generator.generate()?;
            if let Err(e) = self
                .repository
                .create_variant(ctx, variant_id, variant, &mut tx)
                .await
            {
                let _ = self.tx_manager.rollback(tx).await;
                return Err(e);
            }
        }

        self.tx_manager.commit(tx).await?;
        Ok(id)
    }

    async fn update_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::UPDATE)?;
        let mut tx = self.tx_manager.begin().await?;
        match self
            .repository
            .update_product(ctx, id, product, &mut tx)
            .await
        {
            Ok(_) => {
                self.tx_manager.commit(tx).await?;
                Ok(())
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(e)
            }
        }
    }

    async fn delete_product(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;
        let mut tx = self.tx_manager.begin().await?;
        if let Err(e) = self.repository.delete_product(ctx, id, &mut tx).await {
            let _ = self.tx_manager.rollback(tx).await;
            return Err(e);
        }
        if let Err(e) = self
            .repository
            .delete_variants_by_product_id(ctx, id, &mut tx)
            .await
        {
            let _ = self.tx_manager.rollback(tx).await;
            return Err(e);
        }
        self.tx_manager.commit(tx).await?;
        Ok(())
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;
        self.repository.get_by_id(ctx, id).await
    }

    async fn create_variant(
        &self,
        ctx: &Context,
        variant: &ProductVariantCreate,
    ) -> DomainResult<i64> {
        ctx.require_access(None, resource::PRODUCT, action::CREATE)?;
        let mut tx = self.tx_manager.begin().await?;
        let variant_id = self.id_generator.generate()?;
        match self
            .repository
            .create_variant(ctx, variant_id, variant, &mut tx)
            .await
        {
            Ok(_) => {
                self.tx_manager.commit(tx).await?;
                Ok(variant_id)
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(e)
            }
        }
    }

    async fn update_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::UPDATE)?;
        self.repository.update_variant(ctx, id, variant).await
    }

    async fn delete_variant(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;
        let mut tx = self.tx_manager.begin().await?;
        match self.repository.delete_variant(ctx, id, &mut tx).await {
            Ok(_) => {
                self.tx_manager.commit(tx).await?;
                Ok(())
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(e)
            }
        }
    }

    async fn delete_variants_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;
        let mut tx = self.tx_manager.begin().await?;
        match self
            .repository
            .delete_variants_by_product_id(ctx, product_id, &mut tx)
            .await
        {
            Ok(_) => {
                self.tx_manager.commit(tx).await?;
                Ok(())
            }
            Err(e) => {
                let _ = self.tx_manager.rollback(tx).await;
                Err(e)
            }
        }
    }

    async fn get_variant_by_barcode(
        &self,
        ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;
        self.repository.get_variant_by_barcode(ctx, barcode).await
    }

    async fn get_variant_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;
        self.repository.get_variant_by_id(ctx, id).await
    }

    async fn get_variant_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;
        self.repository
            .get_variant_by_product_id(ctx, product_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{MockIdGen, create_mock_id_gen};
    use crate::domain::Error;
    use crate::domain::model::Update;
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use std::collections::HashMap;

    // Mock transaction type - simple unit struct for testing
    #[derive(Debug)]
    struct MockTx;

    mock! {
        pub ProductRepo {}
        #[async_trait]
        impl ProductRepository<MockTx> for ProductRepo {
            async fn create_product(&self, ctx: &Context, id: i64, product: &ProductCreate, tx: &mut MockTx) -> DomainResult<()>;
            async fn update_product(&self, ctx: &Context, id: i64, product: &ProductUpdate, tx: &mut MockTx) -> DomainResult<()>;
            async fn delete_product(&self, ctx: &Context, id: i64, tx: &mut MockTx) -> DomainResult<()>;
            async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>>;
            async fn create_variant(&self, ctx: &Context, id: i64, variant: &ProductVariantCreate, tx: &mut MockTx) -> DomainResult<()>;
            async fn update_variant(&self, ctx: &Context, id: i64, variant: &ProductVariantUpdate) -> DomainResult<()>;
            async fn delete_variant(&self, ctx: &Context, id: i64, tx: &mut MockTx) -> DomainResult<()>;
            async fn delete_variants_by_product_id(&self, ctx: &Context, product_id: i64, tx: &mut MockTx) -> DomainResult<()>;
            async fn get_variant_by_barcode(&self, ctx: &Context, barcode: &str) -> DomainResult<Option<ProductVariant>>;
            async fn get_variant_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<ProductVariant>>;
            async fn get_variant_by_product_id(&self, ctx: &Context, product_id: i64) -> DomainResult<Vec<ProductVariant>>;
            async fn get_product_category(&self, ctx: &Context, product_id: i64) -> DomainResult<Vec<i64>>;
        }
    }

    // Mock transaction manager that returns MockTx
    struct MockTxManager {
        begin_fn: Box<dyn Fn() -> DomainResult<MockTx> + Send + Sync>,
        commit_fn: Box<dyn Fn(MockTx) -> DomainResult<()> + Send + Sync>,
        rollback_fn: Box<dyn Fn(MockTx) -> DomainResult<()> + Send + Sync>,
    }

    impl MockTxManager {
        fn new() -> Self {
            Self {
                begin_fn: Box::new(|| Ok(MockTx)),
                commit_fn: Box::new(|_| Ok(())),
                rollback_fn: Box::new(|_| Ok(())),
            }
        }

        fn expect_rollback(mut self) -> Self {
            self.rollback_fn = Box::new(|_| Ok(()));
            self
        }
    }

    #[async_trait]
    impl TransactionManager for MockTxManager {
        type Transaction<'a> = MockTx;

        async fn begin(&self) -> DomainResult<MockTx> {
            (self.begin_fn)()
        }

        async fn commit<'a>(&self, tx: MockTx) -> DomainResult<()> {
            (self.commit_fn)(tx)
        }

        async fn rollback<'a>(&self, tx: MockTx) -> DomainResult<()> {
            (self.rollback_fn)(tx)
        }
    }

    /// Helper to create the service with correct types
    fn create_service(
        mock_repo: MockProductRepo,
        mock_tx: MockTxManager,
        mock_id_generator: MockIdGen,
    ) -> ProductService<MockProductRepo, MockTxManager, MockIdGen> {
        ProductService {
            repository: mock_repo,
            tx_manager: mock_tx,
            id_generator: mock_id_generator,
        }
    }

    /// Creates a test context with full permissions for PRODUCT resource
    fn create_test_context() -> Context {
        let mut permissions = HashMap::new();
        permissions.insert((resource::PRODUCT, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    /// Creates a test context with no permissions
    fn create_no_permission_context() -> Context {
        Context::new()
    }

    fn create_test_product_create() -> ProductCreate {
        ProductCreate {
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            product_type: "product".to_string(),
            main_image: Some("https://example.com/image.jpg".to_string()),
            sellable: true,
            buyable: true,
            editable_price: false,
            has_variant: true,
            metadata: None,
            category_ids: vec![],
        }
    }

    fn create_test_variant_create(product_id: i64) -> ProductVariantCreate {
        ProductVariantCreate {
            product_id,
            barcode: Some("1234567890".to_string()),
            name: Some("Default Variant".to_string()),
            metadata: None,
        }
    }

    fn create_test_product() -> Product {
        Product {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            product_type: "product".to_string(),
            main_image: Some("https://example.com/image.jpg".to_string()),
            sellable: true,
            buyable: true,
            editable_price: false,
            has_variant: true,
            metadata: None,
        }
    }

    fn create_test_variant() -> ProductVariant {
        ProductVariant {
            id: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            product: create_test_product(),
            barcode: Some("1234567890".to_string()),
            name: Some("Default Variant".to_string()),
            metadata: None,
        }
    }

    fn create_test_product_update() -> ProductUpdate {
        ProductUpdate {
            name: Some("Updated Product".to_string()),
            description: Update::Unchanged,
            product_type: None,
            main_image: Update::Unchanged,
            sellable: None,
            buyable: None,
            editable_price: None,
            has_variant: None,
            metadata: Update::Unchanged,
            category_ids: None,
        }
    }

    fn create_test_variant_update() -> ProductVariantUpdate {
        ProductVariantUpdate {
            barcode: Update::Set("9876543210".to_string()),
            name: Update::Unchanged,
            metadata: Update::Unchanged,
        }
    }

    // =============================================================================
    // Create Product Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_product_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_create_product()
            .withf(|_, _, product, _| product.name == "Test Product")
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let product = create_test_product_create();
        let result = service.create_product(&ctx, &product, &[]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_product_with_variants_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_create_product()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        mock_repo
            .expect_create_variant()
            .times(2)
            .returning(|_, _, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let product = create_test_product_create();
        let variants = vec![
            create_test_variant_create(1),
            ProductVariantCreate {
                product_id: 1,
                barcode: Some("0987654321".to_string()),
                name: Some("Second Variant".to_string()),
                metadata: None,
            },
        ];
        let result = service.create_product(&ctx, &product, &variants).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_product_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let product = create_test_product_create();
        let result = service.create_product(&ctx, &product, &[]).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_product_repo_error_rollback() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        let mock_tx = mock_tx.expect_rollback();

        mock_repo
            .expect_create_product()
            .times(1)
            .returning(|_, _, _, _| Err(Error::Database("DB Error".to_string())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let product = create_test_product_create();
        let result = service.create_product(&ctx, &product, &[]).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    #[tokio::test]
    async fn test_create_product_variant_error_rollback() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        let mock_tx = mock_tx.expect_rollback();

        mock_repo
            .expect_create_product()
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        mock_repo
            .expect_create_variant()
            .times(1)
            .returning(|_, _, _, _| Err(Error::Database("Variant Error".to_string())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let product = create_test_product_create();
        let variants = vec![create_test_variant_create(1)];
        let result = service.create_product(&ctx, &product, &variants).await;

        assert!(matches!(result, Err(Error::Database(_))));
    }

    // =============================================================================
    // Update Product Tests
    // =============================================================================

    #[tokio::test]
    async fn test_update_product_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_update_product()
            .withf(|_, id, product, _| {
                *id == 1 && product.name == Some("Updated Product".to_string())
            })
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let update = create_test_product_update();
        let result = service.update_product(&ctx, 1, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_product_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let update = create_test_product_update();
        let result = service.update_product(&ctx, 1, &update).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_product_not_found() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        let mock_tx = mock_tx.expect_rollback();

        mock_repo
            .expect_update_product()
            .times(1)
            .returning(|_, _, _, _| Err(Error::NotFound("Product not found".to_string())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let update = create_test_product_update();
        let result = service.update_product(&ctx, 999, &update).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Delete Product Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_product_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_delete_product()
            .withf(|_, id, _| *id == 1)
            .times(1)
            .returning(|_, _, _| Ok(()));

        mock_repo
            .expect_delete_variants_by_product_id()
            .withf(|_, id, _| *id == 1)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_product(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_product_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_product(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_delete_product_not_found() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        let mock_tx = mock_tx.expect_rollback();

        mock_repo
            .expect_delete_product()
            .times(1)
            .returning(|_, _, _| Err(Error::NotFound("Product not found".to_string())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_product(&ctx, 999).await;

        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    // =============================================================================
    // Get Product Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .withf(|_, id| *id == 1)
            .times(1)
            .returning(|_, _| Ok(Some(create_test_product())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 1).await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert!(product.is_some());
        assert_eq!(product.unwrap().name, "Test Product");
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_by_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Create Variant Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_create_variant()
            .withf(|_, _, variant, _| variant.barcode == Some("1234567890".to_string()))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let variant = create_test_variant_create(1);
        let result = service.create_variant(&ctx, &variant).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_variant_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let variant = create_test_variant_create(1);
        let result = service.create_variant(&ctx, &variant).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Update Variant Tests
    // =============================================================================

    #[tokio::test]
    async fn test_update_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_update_variant()
            .withf(|_, id, _| *id == 100)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let update = create_test_variant_update();
        let result = service.update_variant(&ctx, 100, &update).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_variant_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let update = create_test_variant_update();
        let result = service.update_variant(&ctx, 100, &update).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Delete Variant Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_delete_variant()
            .withf(|_, id, _| *id == 100)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_variant(&ctx, 100).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_variant_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_variant(&ctx, 100).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Delete Variants by Product ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_variants_by_product_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        // begin returns MockTx by default
        // commit returns Ok by default

        mock_repo
            .expect_delete_variants_by_product_id()
            .withf(|_, id, _| *id == 1)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_variants_by_product_id(&ctx, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_variants_by_product_id_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.delete_variants_by_product_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Get Variant by Barcode Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_variant_by_barcode_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_barcode()
            .withf(|_, barcode| barcode == "1234567890")
            .times(1)
            .returning(|_, _| Ok(Some(create_test_variant())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_barcode(&ctx, "1234567890").await;

        assert!(result.is_ok());
        let variant = result.unwrap();
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().barcode, Some("1234567890".to_string()));
    }

    #[tokio::test]
    async fn test_get_variant_by_barcode_not_found() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_barcode()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_barcode(&ctx, "nonexistent").await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_variant_by_barcode_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_barcode(&ctx, "1234567890").await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Get Variant by ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_variant_by_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_id()
            .withf(|_, id| *id == 100)
            .times(1)
            .returning(|_, _| Ok(Some(create_test_variant())));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_id(&ctx, 100).await;

        assert!(result.is_ok());
        let variant = result.unwrap();
        assert!(variant.is_some());
        assert_eq!(variant.unwrap().id, 100);
    }

    #[tokio::test]
    async fn test_get_variant_by_id_not_found() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_id()
            .times(1)
            .returning(|_, _| Ok(None));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_variant_by_id_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_id(&ctx, 100).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    // =============================================================================
    // Get Variants by Product ID Tests
    // =============================================================================

    #[tokio::test]
    async fn test_get_variant_by_product_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_product_id()
            .withf(|_, id| *id == 1)
            .times(1)
            .returning(|_, _| Ok(vec![create_test_variant()]));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_product_id(&ctx, 1).await;

        assert!(result.is_ok());
        let variants = result.unwrap();
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].id, 100);
    }

    #[tokio::test]
    async fn test_get_variant_by_product_id_empty() {
        let mut mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_test_context();

        mock_repo
            .expect_get_variant_by_product_id()
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_product_id(&ctx, 999).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_variant_by_product_id_no_permission() {
        let mock_repo = MockProductRepo::new();
        let mock_tx = MockTxManager::new();
        let ctx = create_no_permission_context();

        let service = create_service(mock_repo, mock_tx, create_mock_id_gen(1));
        let result = service.get_variant_by_product_id(&ctx, 1).await;

        assert!(matches!(result, Err(Error::Forbidden(_))));
    }
}
