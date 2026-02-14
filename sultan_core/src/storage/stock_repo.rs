use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::stock::{Stock, StockCreate, StockUpdate},
};

/// Repository trait for Stock operations.
///
/// This trait defines the contract for managing stock records in the system.
/// Stocks track inventory quantities per branch and product variant combination.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteStockRepository`](crate::storage::sqlite::stock::SqliteStockRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::stock_repo::{StockRepository, RepoCtx};
/// use sultan_core::storage::sqlite::stock::SqliteStockRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteStockRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new stock record
///     let stock = StockCreate {
///         branch_id: 1,
///         product_variant_id: 123,
///         quantity: 100,
///         min_stock: Some(10),
///         max_stock: Some(500),
///         last_buy_price: Some(15000),
///         metadata: None,
///     };
///     repo.create(&ctx, 12345, &stock).await?;
///     
///     // Get stock by ID
///     let stock = repo.get_by_id(&ctx, 12345).await?;
///     
///     // Get stock by branch and product variant
///     let stock = repo.get_by_branch_and_variant(&ctx, 1, 123).await?;
///     
///     // List all stocks for a branch
///     let stocks = repo.get_all_by_branch_id(&ctx, 1).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait StockRepository: Send + Sync {
    /// Creates a new stock record.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new stock record
    /// * `stock` - Stock data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stock created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        stock: &StockCreate,
    ) -> DomainResult<()>;

    /// Retrieves a stock record by its ID.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the stock record to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(stock))` - Stock found
    /// * `Ok(None)` - Stock not found
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Stock>>;

    /// Retrieves a stock record by branch ID and product variant ID.
    ///
    /// Since there is a unique constraint on (branch_id, product_variant_id),
    /// this returns at most one record.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `branch_id` - Branch ID to filter by
    /// * `product_variant_id` - Product variant ID to filter by
    ///
    /// # Returns
    ///
    /// * `Ok(Some(stock))` - Stock found
    /// * `Ok(None)` - Stock not found
    /// * `Err(Error)` - Database error
    async fn get_by_branch_and_variant(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        product_variant_id: i64,
    ) -> DomainResult<Option<Stock>>;

    /// Updates an existing stock record by branch ID and product variant ID.
    ///
    /// Only provided fields in `StockUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `branch_id` - Branch ID of the stock to update
    /// * `product_variant_id` - Product variant ID of the stock to update
    /// * `stock` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stock updated successfully
    /// * `Err(Error::NotFound)` - Stock not found
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        product_variant_id: i64,
        stock: &StockUpdate,
    ) -> DomainResult<()>;

    /// Deletes a stock record by its ID.
    ///
    /// This performs a soft delete by setting `is_deleted = true` and `deleted_at`.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the stock record to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stock deleted successfully
    /// * `Err(Error::NotFound)` - Stock not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Deletes all stock records for the given product variant IDs.
    ///
    /// This performs a soft delete on all stock records whose `product_variant_id`
    /// matches any ID in the provided array. This is typically used when product
    /// variants are being deleted and their associated stocks need to be cleaned up.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `variant_ids` - Array of product variant IDs whose stocks should be deleted
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All stock records deleted successfully (or no records found)
    /// * `Err(Error)` - Database error
    ///
    /// # Notes
    ///
    /// - If the array is empty, this is a no-op and returns `Ok(())`
    /// - This is a soft delete, setting `is_deleted = true` and `deleted_at`
    /// - No error is returned if no stock records exist for the given variant IDs
    async fn delete_by_product_variant_ids(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        variant_ids: &[i64],
    ) -> DomainResult<()>;
}
