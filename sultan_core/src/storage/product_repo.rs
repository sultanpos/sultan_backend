use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::product::{
        Product, ProductCreate, ProductUpdate, ProductVariant, ProductVariantCreate,
        ProductVariantUpdate,
    },
};

/// Repository trait for Product and ProductVariant operations.
///
/// This trait defines the contract for managing products and their variants in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteProductRepository`](crate::storage::sqlite::product::SqliteProductRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::product_repo::{ProductRepository, RepoCtx};
/// use sultan_core::storage::sqlite::product::SqliteProductRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteProductRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new product
///     let product = ProductCreate {
///         name: "Widget".to_string(),
///         description: Some("A useful widget".to_string()),
///         product_type: "product".to_string(),
///         main_image: None,
///         sellable: true,
///         buyable: true,
///         editable_price: false,
///         has_variant: true,
///         metadata: None,
///         category_ids: vec![1, 2],
///     };
///     repo.create_product(&ctx, 12345, &product).await?;
///     
///     // Create a variant for the product
///     let variant = ProductVariantCreate {
///         product_id: 12345,
///         barcode: Some("1234567890".to_string()),
///         name: Some("Red Widget".to_string()),
///         metadata: None,
///     };
///     repo.create_variant(&ctx, 67890, &variant).await?;
///     
///     // Get the product
///     let product = repo.get_by_id(&ctx, 12345).await?;
///     
///     // Get variants by product
///     let variants = repo.get_variant_by_product_id(&ctx, 12345).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Creates a new product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new product
    /// * `product` - Product data to create (includes category_ids for associations)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Product created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create_product(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        product: &ProductCreate,
    ) -> DomainResult<()>;

    /// Updates an existing product.
    ///
    /// Only provided fields in `ProductUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    /// If `category_ids` is provided, all existing category associations are replaced.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the product to update
    /// * `product` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Product updated successfully
    /// * `Err(Error::NotFound)` - Product not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update_product(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        product: &ProductUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a product.
    ///
    /// This method marks the product as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the product to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Product deleted successfully
    /// * `Err(Error::NotFound)` - Product not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete_product(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()>;

    /// Retrieves a product by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the product to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(product))` - Product found
    /// * `Ok(None)` - Product not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Product>>;

    /// Creates a new product variant.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new variant
    /// * `variant` - Variant data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Variant created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create_variant(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        variant: &ProductVariantCreate,
    ) -> DomainResult<()>;

    /// Updates an existing product variant.
    ///
    /// Only provided fields in `ProductVariantUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the variant to update
    /// * `variant` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Variant updated successfully
    /// * `Err(Error::NotFound)` - Variant not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update_variant(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        variant: &ProductVariantUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a product variant.
    ///
    /// This method marks the variant as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the variant to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Variant deleted successfully
    /// * `Err(Error::NotFound)` - Variant not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete_variant(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()>;

    /// Soft-deletes all variants for a product.
    ///
    /// This method marks all variants associated with a product as deleted.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_id` - ID of the product whose variants to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Variants deleted successfully
    /// * `Err(Error)` - Database error
    async fn delete_variants_by_product_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<()>;

    /// Retrieves a variant by barcode (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `barcode` - Barcode to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(variant))` - Variant found (includes associated product)
    /// * `Ok(None)` - Variant not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_variant_by_barcode(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        barcode: &str,
    ) -> DomainResult<Option<ProductVariant>>;

    /// Retrieves a variant by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the variant to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(variant))` - Variant found (includes associated product)
    /// * `Ok(None)` - Variant not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_variant_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<ProductVariant>>;

    /// Lists all non-deleted variants for a product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_id` - ID of the product
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<ProductVariant>)` - List of all active variants for the product
    /// * `Err(Error)` - Database error
    async fn get_variant_by_product_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<ProductVariant>>;

    /// Retrieves the IDs of all non-deleted variants for a product.
    ///
    /// This is a lightweight alternative to [`get_variant_by_product_id`](Self::get_variant_by_product_id)
    /// that only returns IDs instead of full variant objects (with their associated product).
    /// Useful for cascading operations such as deleting related sell prices or stock records.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_id` - ID of the product
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<i64>)` - List of variant IDs (empty if product has no active variants)
    /// * `Err(Error)` - Database error
    async fn get_variant_ids_by_product_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<i64>>;

    /// Lists all category IDs associated with a product.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_id` - ID of the product
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<i64>)` - List of category IDs
    /// * `Err(Error)` - Database error
    async fn get_product_category(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_id: i64,
    ) -> DomainResult<Vec<i64>>;

    /// Adds category associations to a product.
    ///
    /// This method creates associations between a product and one or more categories
    /// in the product_categories junction table. If an association already exists,
    /// it will be skipped (database constraint prevents duplicates).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_id` - ID of the product
    /// * `category_ids` - Array of category IDs to associate with the product
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Categories added successfully (or already exist)
    /// * `Err(Error)` - Database error
    ///
    /// # Notes
    ///
    /// - If the array is empty, this is a no-op and returns `Ok(())`
    /// - Duplicate associations are handled gracefully (no error)
    /// - This does not remove existing associations
    async fn add_product_category(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_id: i64,
        category_ids: &[i64],
    ) -> DomainResult<()>;
}
