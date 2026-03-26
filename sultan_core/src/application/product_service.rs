//! Product Service
//!
//! This module provides the business logic layer for product management operations.
//! It serves as the intermediary between the web layer (handlers) and the storage
//! layer (repositories), ensuring proper authorization, validation, and domain
//! logic enforcement.
//!
//! # Architecture
//!
//! The service follows clean architecture principles:
//! - **Trait-based abstraction**: `ProductServiceTrait` defines the contract
//! - **Implementation**: `ProductService` provides the concrete implementation
//! - **Dependency injection**: Repository and ID generator are injected
//! - **RepoCtx pattern**: Uses `RepoCtx<DatabaseConnection>` for database operations
//!
//! # Authorization
//!
//! All operations require appropriate permissions:
//! - CREATE: `product:create` permission
//! - READ: `product:read` permission
//! - UPDATE: `product:update` permission
//! - DELETE: `product:delete` permission

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm::TransactionTrait;

use crate::domain::Context;
use crate::domain::DomainResult;
use crate::domain::model::permission::{action, resource};
use crate::domain::model::product::ProductFullCreate;
use crate::domain::model::product::{
    Product, ProductUpdate, ProductVariant, ProductVariantCreate, ProductVariantUpdate,
};
use crate::snowflake::IdGenerator;
use crate::storage::StockRepository;
use crate::storage::{ProductRepository, RepoCtx};

/// Trait defining the contract for product service operations.
///
/// This trait abstracts the product business logic, enabling:
/// - Easy testing through mock implementations
/// - Flexibility in service implementations
/// - Clear separation of concerns
#[async_trait]
pub trait ProductServiceTrait: Send + Sync {
    /// Creates a new product with optional variants.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `product` - The product data to create
    /// * `variants` - Optional list of variants to create with the product
    ///
    /// # Returns
    ///
    /// The ID of the newly created product, or an error if creation failed.
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if the user lacks `product:create` permission
    /// - `DatabaseError` if the database operation fails
    async fn create_product(&self, ctx: &Context, product: &ProductFullCreate)
    -> DomainResult<i64>;

    /// Updates an existing product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the product to update
    /// * `product` - The update data
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if the user lacks `product:update` permission
    /// - `NotFound` if the product doesn't exist
    async fn update_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()>;

    /// Soft deletes a product and all its variants.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the product to delete
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if the user lacks `product:delete` permission
    /// - `NotFound` if the product doesn't exist
    async fn delete_product(&self, ctx: &Context, id: i64) -> DomainResult<()>;

    /// Retrieves a product by its ID.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the product to retrieve
    ///
    /// # Returns
    ///
    /// The product if found, or None if not found.
    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>>;

    /// Creates a new product variant.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `variant` - The variant data to create
    ///
    /// # Returns
    ///
    /// The ID of the newly created variant.
    async fn create_variant(
        &self,
        ctx: &Context,
        variant: &ProductVariantCreate,
    ) -> DomainResult<i64>;

    /// Updates an existing product variant.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the variant to update
    /// * `variant` - The update data
    async fn update_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()>;

    /// Soft deletes a product variant.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the variant to delete
    async fn delete_variant(&self, ctx: &Context, id: i64) -> DomainResult<()>;

    /// Soft deletes all variants for a specific product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `product_id` - The ID of the product whose variants to delete
    async fn delete_variants_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<()>;

    /// Retrieves a product variant by its barcode.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `barcode` - The barcode to search for
    ///
    /// # Returns
    ///
    /// The variant if found, or None if not found.
    async fn get_variant_by_barcode(
        &self,
        ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>>;

    /// Retrieves a product variant by its ID.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `id` - The ID of the variant to retrieve
    ///
    /// # Returns
    ///
    /// The variant if found, or None if not found.
    async fn get_variant_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>>;

    /// Retrieves all variants for a specific product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The request context containing user info and permissions
    /// * `product_id` - The ID of the product
    ///
    /// # Returns
    ///
    /// A list of all variants belonging to the product.
    async fn get_variant_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>>;
}

/// Concrete implementation of the product service.
///
/// This implementation uses:
/// - A `ProductRepository` for data persistence (including sell prices and discounts)
/// - A `StockRepository` for managing stock data
/// - An `IdGenerator` for generating unique IDs
/// - A `DatabaseConnection` for SeaORM operations
pub struct ProductService<R, S, I> {
    repository: R,
    stock_repository: S,
    id_generator: I,
    db: DatabaseConnection,
}

impl<R: ProductRepository, S: StockRepository, I: IdGenerator> ProductService<R, S, I> {
    /// Creates a new ProductService instance.
    ///
    /// # Arguments
    ///
    /// * `repository` - The product repository implementation
    /// * `stock_repository` - The stock repository implementation
    /// * `id_generator` - The ID generator for creating unique IDs
    /// * `db` - The database connection for SeaORM operations
    pub fn new(
        repository: R,
        stock_repository: S,
        id_generator: I,
        db: DatabaseConnection,
    ) -> Self {
        Self {
            repository,
            stock_repository,
            id_generator,
            db,
        }
    }
}

#[async_trait]
impl<R: ProductRepository, S: StockRepository, I: IdGenerator> ProductServiceTrait
    for ProductService<R, S, I>
{
    async fn create_product(
        &self,
        ctx: &Context,
        product_create: &ProductFullCreate,
    ) -> DomainResult<i64> {
        ctx.require_access(None, resource::PRODUCT, action::CREATE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        let id = self.id_generator.generate()?;
        self.repository
            .create_product(&repo_ctx, id, &product_create.product)
            .await?;

        if !product_create.categories.is_empty() {
            self.repository
                .add_product_category(&repo_ctx, id, &product_create.categories)
                .await?;
        }

        // Insert all variants
        for variant in &product_create.variants {
            let variant_id = self.id_generator.generate()?;
            let mut variant_with_product = variant.variant.clone();
            variant_with_product.product_id = id;
            self.repository
                .create_variant(&repo_ctx, variant_id, &variant_with_product)
                .await?;

            for stock in &variant.stocks {
                let mut stock_with_variant = stock.clone();
                stock_with_variant.product_variant_id = variant_id;
                self.stock_repository
                    .create(
                        &repo_ctx,
                        self.id_generator.generate()?,
                        &stock_with_variant,
                    )
                    .await?;
            }

            for sell_price in &variant.sell_prices {
                let price_id = self.id_generator.generate()?;
                let mut sell_price_with_variant = sell_price.sell_price.clone();
                sell_price_with_variant.product_variant_id = variant_id;
                self.repository
                    .create_sell_price(&repo_ctx, price_id, &sell_price_with_variant)
                    .await?;
                for discount in &sell_price.discounts {
                    let mut discount_with_price_id = discount.clone();
                    discount_with_price_id.price_id = price_id;
                    self.repository
                        .create_sell_discount(
                            &repo_ctx,
                            self.id_generator.generate()?,
                            &discount_with_price_id,
                        )
                        .await?;
                }
            }
        }

        repo_ctx.db.commit().await?;

        Ok(id)
    }

    async fn update_product(
        &self,
        ctx: &Context,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::UPDATE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository.update_product(&repo_ctx, id, product).await
    }

    async fn delete_product(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        let variant_ids = self
            .repository
            .get_variant_ids_by_product_id(&repo_ctx, id)
            .await?;

        self.stock_repository
            .delete_by_product_variant_ids(&repo_ctx, &variant_ids)
            .await?;

        self.repository
            .delete_sell_prices_by_product_variant_ids(&repo_ctx, &variant_ids)
            .await?;

        self.repository
            .delete_variants_by_product_id(&repo_ctx, id)
            .await?;

        self.repository.delete_product(&repo_ctx, id).await?;

        repo_ctx.db.commit().await?;
        Ok(())
    }

    async fn get_by_id(&self, ctx: &Context, id: i64) -> DomainResult<Option<Product>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository.get_by_id(&repo_ctx, id).await
    }

    async fn create_variant(
        &self,
        ctx: &Context,
        variant: &ProductVariantCreate,
    ) -> DomainResult<i64> {
        ctx.require_access(None, resource::PRODUCT, action::CREATE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        let variant_id = self.id_generator.generate()?;
        self.repository
            .create_variant(&repo_ctx, variant_id, variant)
            .await?;
        Ok(variant_id)
    }

    async fn update_variant(
        &self,
        ctx: &Context,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::UPDATE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository.update_variant(&repo_ctx, id, variant).await
    }

    async fn delete_variant(&self, ctx: &Context, id: i64) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        self.stock_repository
            .delete_by_product_variant_ids(&repo_ctx, &[id])
            .await?;

        self.repository
            .delete_sell_prices_by_product_variant_ids(&repo_ctx, &[id])
            .await?;

        self.repository.delete_variant(&repo_ctx, id).await?;

        repo_ctx.db.commit().await?;

        Ok(())
    }

    async fn delete_variants_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<()> {
        ctx.require_access(None, resource::PRODUCT, action::DELETE)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.begin().await?,
        };

        let variant_ids = self
            .repository
            .get_variant_ids_by_product_id(&repo_ctx, product_id)
            .await?;

        self.stock_repository
            .delete_by_product_variant_ids(&repo_ctx, &variant_ids)
            .await?;

        self.repository
            .delete_sell_prices_by_product_variant_ids(&repo_ctx, &variant_ids)
            .await?;

        self.repository
            .delete_variants_by_product_id(&repo_ctx, product_id)
            .await?;

        repo_ctx.db.commit().await?;

        Ok(())
    }

    async fn get_variant_by_barcode(
        &self,
        ctx: &Context,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository
            .get_variant_by_barcode(&repo_ctx, barcode)
            .await
    }

    async fn get_variant_by_id(
        &self,
        ctx: &Context,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository.get_variant_by_id(&repo_ctx, id).await
    }

    async fn get_variant_by_product_id(
        &self,
        ctx: &Context,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>> {
        ctx.require_access(None, resource::PRODUCT, action::READ)?;

        let repo_ctx = RepoCtx {
            ctx: ctx.clone(),
            db: self.db.clone(),
        };

        self.repository
            .get_variant_by_product_id(&repo_ctx, product_id)
            .await
    }
}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::application::{MockIdGen, create_mock_id_gen};
    use crate::domain::Error;
    use crate::domain::model::Update;
    use crate::domain::model::product::ProductCreate;
    use crate::domain::model::sell_price::{
        SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
        SellPriceUpdate,
    };
    use crate::domain::model::stock::{Stock, StockCreate, StockUpdate};
    use crate::storage::RepoCtx;
    use async_trait::async_trait;
    use chrono::Utc;
    use sea_orm::{ConnectionTrait, DatabaseConnection};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Manual mock implementation that works with impl Trait
    #[derive(Clone)]
    struct MockProductRepo {
        create_product_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, ProductCreate) -> DomainResult<()> + Send>>>>,
        update_product_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, ProductUpdate) -> DomainResult<()> + Send>>>>,
        delete_product_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_by_id_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<Product>> + Send>>>>,
        create_variant_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, ProductVariantCreate) -> DomainResult<()> + Send>>>>,
        update_variant_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, ProductVariantUpdate) -> DomainResult<()> + Send>>>>,
        delete_variant_fn: Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        delete_variants_by_product_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<()> + Send>>>>,
        get_variant_by_barcode_fn:
            Arc<Mutex<Option<Box<dyn Fn(String) -> DomainResult<Option<ProductVariant>> + Send>>>>,
        get_variant_by_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Option<ProductVariant>> + Send>>>>,
        get_variant_by_product_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Vec<ProductVariant>> + Send>>>>,
        get_product_category_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Vec<i64>> + Send>>>>,
        get_variant_ids_by_product_id_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64) -> DomainResult<Vec<i64>> + Send>>>>,
        add_product_category_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, Vec<i64>) -> DomainResult<()> + Send>>>>,
        // SellPrice fields
        create_sell_price_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, SellPriceCreate) -> DomainResult<()> + Send>>>>,
        create_sell_discount_fn:
            Arc<Mutex<Option<Box<dyn Fn(i64, SellDiscountCreate) -> DomainResult<()> + Send>>>>,
        delete_sell_prices_by_product_variant_ids_fn:
            Arc<Mutex<Option<Box<dyn Fn(Vec<i64>) -> DomainResult<()> + Send>>>>,
    }

    impl MockProductRepo {
        fn new() -> Self {
            Self {
                create_product_fn: Arc::new(Mutex::new(None)),
                update_product_fn: Arc::new(Mutex::new(None)),
                delete_product_fn: Arc::new(Mutex::new(None)),
                get_by_id_fn: Arc::new(Mutex::new(None)),
                create_variant_fn: Arc::new(Mutex::new(None)),
                update_variant_fn: Arc::new(Mutex::new(None)),
                delete_variant_fn: Arc::new(Mutex::new(None)),
                delete_variants_by_product_id_fn: Arc::new(Mutex::new(None)),
                get_variant_by_barcode_fn: Arc::new(Mutex::new(None)),
                get_variant_by_id_fn: Arc::new(Mutex::new(None)),
                get_variant_by_product_id_fn: Arc::new(Mutex::new(None)),
                get_product_category_fn: Arc::new(Mutex::new(None)),
                get_variant_ids_by_product_id_fn: Arc::new(Mutex::new(None)),
                add_product_category_fn: Arc::new(Mutex::new(None)),
                create_sell_price_fn: Arc::new(Mutex::new(None)),
                create_sell_discount_fn: Arc::new(Mutex::new(None)),
                delete_sell_prices_by_product_variant_ids_fn: Arc::new(Mutex::new(None)),
            }
        }

        fn expect_create_product<F>(&mut self, f: F)
        where
            F: Fn(i64, ProductCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_product_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update_product<F>(&mut self, f: F)
        where
            F: Fn(i64, ProductUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_product_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete_product<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_product_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<Product>> + Send + 'static,
        {
            *self.get_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_create_variant<F>(&mut self, f: F)
        where
            F: Fn(i64, ProductVariantCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_variant_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_update_variant<F>(&mut self, f: F)
        where
            F: Fn(i64, ProductVariantUpdate) -> DomainResult<()> + Send + 'static,
        {
            *self.update_variant_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete_variant<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_variant_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete_variants_by_product_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_variants_by_product_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_get_variant_by_barcode<F>(&mut self, f: F)
        where
            F: Fn(String) -> DomainResult<Option<ProductVariant>> + Send + 'static,
        {
            *self.get_variant_by_barcode_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_variant_by_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Option<ProductVariant>> + Send + 'static,
        {
            *self.get_variant_by_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_get_variant_by_product_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Vec<ProductVariant>> + Send + 'static,
        {
            *self.get_variant_by_product_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_get_product_category<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Vec<i64>> + Send + 'static,
        {
            *self.get_product_category_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_get_variant_ids_by_product_id<F>(&mut self, f: F)
        where
            F: Fn(i64) -> DomainResult<Vec<i64>> + Send + 'static,
        {
            *self.get_variant_ids_by_product_id_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_add_product_category<F>(&mut self, f: F)
        where
            F: Fn(i64, Vec<i64>) -> DomainResult<()> + Send + 'static,
        {
            *self.add_product_category_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_create_sell_price<F>(&mut self, f: F)
        where
            F: Fn(i64, SellPriceCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_sell_price_fn.lock().unwrap() = Some(Box::new(f));
        }

        #[allow(dead_code)]
        fn expect_create_sell_discount<F>(&mut self, f: F)
        where
            F: Fn(i64, SellDiscountCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_sell_discount_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete_sell_prices_by_product_variant_ids<F>(&mut self, f: F)
        where
            F: Fn(Vec<i64>) -> DomainResult<()> + Send + 'static,
        {
            *self
                .delete_sell_prices_by_product_variant_ids_fn
                .lock()
                .unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl ProductRepository for MockProductRepo {
        async fn create_product(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            product: &ProductCreate,
        ) -> DomainResult<()> {
            let lock = self.create_product_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, product.clone())
            } else {
                panic!("create_product not mocked");
            }
        }

        async fn update_product(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            product: &ProductUpdate,
        ) -> DomainResult<()> {
            let lock = self.update_product_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, product.clone())
            } else {
                panic!("update_product not mocked");
            }
        }

        async fn delete_product(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<()> {
            let lock = self.delete_product_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("delete_product not mocked");
            }
        }

        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<Product>> {
            let lock = self.get_by_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("get_by_id not mocked");
            }
        }

        async fn create_variant(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            variant: &ProductVariantCreate,
        ) -> DomainResult<()> {
            let lock = self.create_variant_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, variant.clone())
            } else {
                panic!("create_variant not mocked");
            }
        }

        async fn update_variant(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            variant: &ProductVariantUpdate,
        ) -> DomainResult<()> {
            let lock = self.update_variant_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, variant.clone())
            } else {
                panic!("update_variant not mocked");
            }
        }

        async fn delete_variant(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<()> {
            let lock = self.delete_variant_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("delete_variant not mocked");
            }
        }

        async fn delete_variants_by_product_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_id: i64,
        ) -> DomainResult<()> {
            let lock = self.delete_variants_by_product_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_id)
            } else {
                panic!("delete_variants_by_product_id not mocked");
            }
        }

        async fn get_variant_by_barcode(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            barcode: &str,
        ) -> DomainResult<Option<ProductVariant>> {
            let lock = self.get_variant_by_barcode_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(barcode.to_string())
            } else {
                panic!("get_variant_by_barcode not mocked");
            }
        }

        async fn get_variant_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
        ) -> DomainResult<Option<ProductVariant>> {
            let lock = self.get_variant_by_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id)
            } else {
                panic!("get_variant_by_id not mocked");
            }
        }

        async fn get_variant_by_product_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_id: i64,
        ) -> DomainResult<Vec<ProductVariant>> {
            let lock = self.get_variant_by_product_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_id)
            } else {
                panic!("get_variant_by_product_id not mocked");
            }
        }

        async fn get_product_category(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_id: i64,
        ) -> DomainResult<Vec<i64>> {
            let lock = self.get_product_category_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_id)
            } else {
                panic!("get_product_category not mocked");
            }
        }

        async fn get_variant_ids_by_product_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_id: i64,
        ) -> DomainResult<Vec<i64>> {
            let lock = self.get_variant_ids_by_product_id_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_id)
            } else {
                panic!("get_variant_ids_by_product_id not mocked");
            }
        }

        async fn add_product_category(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_id: i64,
            category_ids: &[i64],
        ) -> DomainResult<()> {
            let lock = self.add_product_category_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_id, category_ids.to_vec())
            } else {
                panic!("add_product_category not mocked");
            }
        }

        async fn create_sell_price(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            price: &SellPriceCreate,
        ) -> DomainResult<()> {
            let lock = self.create_sell_price_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, price.clone())
            } else {
                Ok(())
            }
        }
        async fn update_sell_price(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
            _price: &SellPriceUpdate,
        ) -> DomainResult<()> {
            panic!("update_sell_price not mocked")
        }
        async fn delete_sell_price(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<()> {
            panic!("delete_sell_price not mocked")
        }
        async fn delete_sell_prices_by_product_variant_ids(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            product_variant_ids: &[i64],
        ) -> DomainResult<()> {
            let lock = self
                .delete_sell_prices_by_product_variant_ids_fn
                .lock()
                .unwrap();
            if let Some(f) = lock.as_ref() {
                f(product_variant_ids.to_vec())
            } else {
                Ok(())
            }
        }
        async fn get_all_sell_prices_by_product_variant_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _variant_id: i64,
        ) -> DomainResult<Vec<SellPrice>> {
            panic!("get_all_sell_prices_by_product_variant_id not mocked")
        }
        async fn get_sell_price_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<Option<SellPrice>> {
            panic!("get_sell_price_by_id not mocked")
        }
        async fn create_sell_discount(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            discount: &SellDiscountCreate,
        ) -> DomainResult<()> {
            let lock = self.create_sell_discount_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, discount.clone())
            } else {
                Ok(())
            }
        }
        async fn update_sell_discount(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
            _discount: &SellDiscountUpdate,
        ) -> DomainResult<()> {
            panic!("update_sell_discount not mocked")
        }
        async fn delete_sell_discount(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<()> {
            panic!("delete_sell_discount not mocked")
        }
        async fn delete_sell_discounts_by_sell_price_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _sell_price_id: i64,
        ) -> DomainResult<()> {
            panic!("delete_sell_discounts_by_sell_price_id not mocked")
        }
        async fn get_all_sell_discount_by_price_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _price_id: i64,
        ) -> DomainResult<Vec<SellDiscount>> {
            panic!("get_all_sell_discount_by_price_id not mocked")
        }
        async fn get_sell_discount_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<Option<SellDiscount>> {
            panic!("get_sell_discount_by_id not mocked")
        }
    }

    fn create_test_ctx() -> Context {
        let mut permissions = HashMap::new();
        // Grant all actions for PRODUCT resource globally (branch_id = None)
        // Using 0b1111 to cover all action values 1-4
        permissions.insert((resource::PRODUCT, None), 0b1111);
        Context::new_with_all(None, permissions, HashMap::new())
    }

    fn create_test_product() -> Product {
        Product {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            name: "Test Product".to_string(),
            description: Some("Test Description".to_string()),
            product_type: "product".to_string(),
            main_image: None,
            sellable: true,
            buyable: true,
            editable_price: false,
            variant_count: 0,
            metadata: None,
            categories: vec![],
            variants: vec![],
        }
    }

    fn create_test_variant() -> ProductVariant {
        ProductVariant {
            id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            is_deleted: false,
            barcode: Some("1234567890".to_string()),
            name: Some("Default Variant".to_string()),
            metadata: None,
            sell_prices: vec![],
        }
    }

    async fn create_test_db() -> DatabaseConnection {
        use sea_orm::Database;
        Database::connect("sqlite::memory:").await.unwrap()
    }

    // Mock Stock Repository
    #[derive(Clone)]
    struct MockStockRepo {
        create_fn: Arc<Mutex<Option<Box<dyn Fn(i64, StockCreate) -> DomainResult<()> + Send>>>>,
        delete_by_product_variant_ids_fn:
            Arc<Mutex<Option<Box<dyn Fn(Vec<i64>) -> DomainResult<()> + Send>>>>,
    }

    impl MockStockRepo {
        fn new() -> Self {
            Self {
                create_fn: Arc::new(Mutex::new(None)),
                delete_by_product_variant_ids_fn: Arc::new(Mutex::new(None)),
            }
        }

        #[allow(dead_code)]
        fn expect_create<F>(&mut self, f: F)
        where
            F: Fn(i64, StockCreate) -> DomainResult<()> + Send + 'static,
        {
            *self.create_fn.lock().unwrap() = Some(Box::new(f));
        }

        fn expect_delete_by_product_variant_ids<F>(&mut self, f: F)
        where
            F: Fn(Vec<i64>) -> DomainResult<()> + Send + 'static,
        {
            *self.delete_by_product_variant_ids_fn.lock().unwrap() = Some(Box::new(f));
        }
    }

    #[async_trait]
    impl StockRepository for MockStockRepo {
        async fn create(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            id: i64,
            stock: &StockCreate,
        ) -> DomainResult<()> {
            let lock = self.create_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(id, stock.clone())
            } else {
                Ok(())
            }
        }
        async fn get_by_id(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _id: i64,
        ) -> DomainResult<Option<Stock>> {
            panic!("get_by_id not mocked")
        }
        async fn get_by_branch_and_variant(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _branch_id: i64,
            _variant_id: i64,
        ) -> DomainResult<Option<Stock>> {
            panic!("get_by_branch_and_variant not mocked")
        }
        async fn update(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            _branch_id: i64,
            _variant_id: i64,
            _stock: &StockUpdate,
        ) -> DomainResult<()> {
            panic!("update not mocked")
        }
        async fn delete(&self, _ctx: &RepoCtx<impl ConnectionTrait>, _id: i64) -> DomainResult<()> {
            panic!("delete not mocked")
        }
        async fn delete_by_product_variant_ids(
            &self,
            _ctx: &RepoCtx<impl ConnectionTrait>,
            variant_ids: &[i64],
        ) -> DomainResult<()> {
            let lock = self.delete_by_product_variant_ids_fn.lock().unwrap();
            if let Some(f) = lock.as_ref() {
                f(variant_ids.to_vec())
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_create_product_success() {
        use crate::domain::model::product::{ProductVariantFullCreate, SellPriceFullCreate};

        let mut mock_repo = MockProductRepo::new();
        let mut mock_stock_repo = MockStockRepo::new();

        // Track calls to verify all methods are invoked
        let create_product_called = Arc::new(Mutex::new(false));
        let add_category_called = Arc::new(Mutex::new(false));
        let create_variant_called = Arc::new(Mutex::new(false));
        let create_stock_called = Arc::new(Mutex::new(false));
        let create_price_called = Arc::new(Mutex::new(false));
        let create_discount_called = Arc::new(Mutex::new(false));

        let create_product_flag = create_product_called.clone();
        mock_repo.expect_create_product(move |id, product| {
            *create_product_flag.lock().unwrap() = true;
            assert_eq!(id, 1);
            assert_eq!(product.name, "Test Product");
            assert_eq!(product.variant_count, 0);
            Ok(())
        });

        let add_category_flag = add_category_called.clone();
        mock_repo.expect_add_product_category(move |product_id, categories| {
            *add_category_flag.lock().unwrap() = true;
            assert_eq!(product_id, 1);
            assert_eq!(categories, vec![10, 20]);
            Ok(())
        });

        let create_variant_flag = create_variant_called.clone();
        mock_repo.expect_create_variant(move |id, variant| {
            *create_variant_flag.lock().unwrap() = true;
            assert_eq!(id, 2);
            assert_eq!(variant.product_id, 1);
            assert_eq!(variant.barcode, Some("123456".to_string()));
            Ok(())
        });

        let create_stock_flag = create_stock_called.clone();
        mock_stock_repo.expect_create(move |id, stock| {
            *create_stock_flag.lock().unwrap() = true;
            assert_eq!(id, 3);
            assert_eq!(stock.product_variant_id, 2);
            assert_eq!(stock.branch_id, 1);
            Ok(())
        });

        let create_price_flag = create_price_called.clone();
        mock_repo.expect_create_sell_price(move |id, price| {
            *create_price_flag.lock().unwrap() = true;
            assert_eq!(id, 4);
            assert_eq!(price.product_variant_id, 2);
            Ok(())
        });

        let create_discount_flag = create_discount_called.clone();
        mock_repo.expect_create_sell_discount(move |id, discount| {
            *create_discount_flag.lock().unwrap() = true;
            assert_eq!(id, 5);
            assert_eq!(discount.price_id, 4);
            Ok(())
        });

        // Create a mock ID generator that returns sequential IDs
        let mut mock_id_gen = MockIdGen::new();
        let id_counter = Arc::new(Mutex::new(1i64));
        mock_id_gen.expect_generate().returning(move || {
            let mut counter = id_counter.lock().unwrap();
            let id = *counter;
            *counter += 1;
            Ok(id)
        });

        let db = create_test_db().await;

        let service = ProductService::new(mock_repo, mock_stock_repo, mock_id_gen, db);
        let ctx = create_test_ctx();

        // Create a complete product with categories, variants, stocks, prices, and discounts
        let product = ProductFullCreate {
            product: ProductCreate {
                name: "Test Product".to_string(),
                description: Some("A test description".to_string()),
                product_type: "product".to_string(),
                main_image: None,
                sellable: true,
                buyable: true,
                editable_price: false,
                metadata: None,
                variant_count: 0,
                category_ids: vec![],
            },
            categories: vec![10, 20],
            variants: vec![ProductVariantFullCreate {
                variant: ProductVariantCreate {
                    product_id: 0, // Will be set by service
                    barcode: Some("123456".to_string()),
                    name: Some("Variant 1".to_string()),
                    metadata: None,
                },
                stocks: vec![StockCreate {
                    branch_id: 1,
                    product_variant_id: 0, // Will be set by service
                    quantity: 100,
                    min_stock: Some(10),
                    max_stock: Some(500),
                    last_buy_price: Some(800),
                    metadata: None,
                }],
                sell_prices: vec![SellPriceFullCreate {
                    sell_price: SellPriceCreate {
                        branch_id: None,
                        product_variant_id: 0, // Will be set by service
                        uom_id: 1,
                        quantity: 1,
                        price: 1000,
                        metadata: None,
                    },
                    discounts: vec![SellDiscountCreate {
                        price_id: 0, // Will be set by service
                        quantity: 10,
                        discount_formula: "price * 0.9".to_string(),
                        customer_level: Some(1),
                        metadata: None,
                    }],
                }],
            }],
        };

        let result = service.create_product(&ctx, &product).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Verify all repository methods were called
        assert!(
            *create_product_called.lock().unwrap(),
            "create_product was not called"
        );
        assert!(
            *add_category_called.lock().unwrap(),
            "add_product_category was not called"
        );
        assert!(
            *create_variant_called.lock().unwrap(),
            "create_variant was not called"
        );
        assert!(
            *create_stock_called.lock().unwrap(),
            "stock create was not called"
        );
        assert!(
            *create_price_called.lock().unwrap(),
            "sell_price create was not called"
        );
        assert!(
            *create_discount_called.lock().unwrap(),
            "create_discount was not called"
        );
    }

    #[tokio::test]
    async fn test_create_product_forbidden() {
        let mock_repo = MockProductRepo::new();
        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = Context::new(); // No permissions
        let product = ProductFullCreate {
            product: ProductCreate {
                name: "Test".to_string(),
                description: None,
                product_type: "product".to_string(),
                main_image: None,
                sellable: true,
                buyable: true,
                editable_price: false,
                metadata: None,
                variant_count: 0,
                category_ids: vec![],
            },
            variants: vec![],
            categories: vec![],
        };

        let result = service.create_product(&ctx, &product).await;
        assert!(matches!(result, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_update_product_success() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_update_product(|_, _| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();
        let product = ProductUpdate {
            name: Some("Updated".to_string()),
            description: Update::Unchanged,
            product_type: None,
            main_image: Update::Unchanged,
            sellable: None,
            buyable: None,
            editable_price: None,
            metadata: Update::Unchanged,
            category_ids: None,
        };

        let result = service.update_product(&ctx, 1, &product).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_product_not_found() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_update_product(|_, _| Err(Error::NotFound("Product not found".into())));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();
        let product = ProductUpdate {
            name: Some("Updated".to_string()),
            description: Update::Unchanged,
            product_type: None,
            main_image: Update::Unchanged,
            sellable: None,
            buyable: None,
            editable_price: None,
            metadata: Update::Unchanged,
            category_ids: None,
        };

        let result = service.update_product(&ctx, 999, &product).await;
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_product_success() {
        let mut mock_repo = MockProductRepo::new();
        // Mock get_variant_ids_by_product_id to return some variant IDs
        mock_repo.expect_get_variant_ids_by_product_id(|_| Ok(vec![100, 200]));
        mock_repo.expect_delete_product(|_| Ok(()));
        mock_repo.expect_delete_variants_by_product_id(|_| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mut mock_stock_repo = MockStockRepo::new();

        // Verify that delete_by_product_variant_ids is called with correct variant IDs
        mock_stock_repo.expect_delete_by_product_variant_ids(move |ids| {
            assert_eq!(ids, vec![100, 200]);
            Ok(())
        });
        mock_repo.expect_delete_sell_prices_by_product_variant_ids(move |ids| {
            assert_eq!(ids, vec![100, 200]);
            Ok(())
        });

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.delete_product(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_by_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let product = create_test_product();
        mock_repo.expect_get_by_id(move |_| Ok(Some(product.clone())));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.get_by_id(&ctx, 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_get_by_id(|_| Ok(None));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.get_by_id(&ctx, 999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_create_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_create_variant(|_, _| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();
        let variant = ProductVariantCreate {
            product_id: 1,
            barcode: Some("1234567890".to_string()),
            name: None,
            metadata: None,
        };

        let result = service.create_variant(&ctx, &variant).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_update_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_update_variant(|_, _| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();
        let variant = ProductVariantUpdate {
            barcode: Update::Set("9999999999".to_string()),
            name: Update::Unchanged,
            metadata: Update::Unchanged,
        };

        let result = service.update_variant(&ctx, 1, &variant).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_variant_success() {
        let mut mock_repo = MockProductRepo::new();
        mock_repo.expect_delete_variant(|_| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mut mock_stock_repo = MockStockRepo::new();

        // Verify that delete_by_product_variant_ids is called with correct variant ID
        mock_stock_repo.expect_delete_by_product_variant_ids(|ids| {
            assert_eq!(ids, vec![1]);
            Ok(())
        });
        mock_repo.expect_delete_sell_prices_by_product_variant_ids(|ids| {
            assert_eq!(ids, vec![1]);
            Ok(())
        });

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.delete_variant(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_variants_by_product_id_success() {
        let mut mock_repo = MockProductRepo::new();
        // Mock get_variant_ids_by_product_id to return some variant IDs
        mock_repo.expect_get_variant_ids_by_product_id(|_| Ok(vec![100, 200]));
        mock_repo.expect_delete_variants_by_product_id(|_| Ok(()));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mut mock_stock_repo = MockStockRepo::new();

        // Verify that delete_by_product_variant_ids is called with correct variant IDs
        mock_stock_repo.expect_delete_by_product_variant_ids(|ids| {
            assert_eq!(ids, vec![100, 200]);
            Ok(())
        });
        mock_repo.expect_delete_sell_prices_by_product_variant_ids(|ids| {
            assert_eq!(ids, vec![100, 200]);
            Ok(())
        });

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.delete_variants_by_product_id(&ctx, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_variant_by_id_success() {
        let mut mock_repo = MockProductRepo::new();
        let variant = create_test_variant();
        mock_repo.expect_get_variant_by_id(move |_| Ok(Some(variant.clone())));

        let id_gen = create_mock_id_gen(1);
        let db = create_test_db().await;
        let mock_stock_repo = MockStockRepo::new();

        let service = ProductService::new(mock_repo, mock_stock_repo, id_gen, db);
        let ctx = create_test_ctx();

        let result = service.get_variant_by_id(&ctx, 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
