use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::DomainResult;
use crate::domain::model::sell_price::{
    SellDiscount, SellDiscountCreate, SellDiscountUpdate, SellPrice, SellPriceCreate,
    SellPriceUpdate,
};

/// Repository trait for SellPrice and SellDiscount operations.
///
/// This trait defines the contract for managing sell prices and discounts in the system.
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// # Implementations
///
/// - SQLite: [`SqliteSellPriceRepository`](crate::storage::sqlite::sell_price::SqliteSellPriceRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::sell_price_repo::{SellPriceRepository, RepoCtx};
/// use sultan_core::storage::sqlite::sell_price::SqliteSellPriceRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteSellPriceRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Create a new sell price
///     let price = SellPriceCreate {
///         branch_id: Some(1),
///         product_variant_id: 123,
///         uom_id: 456,
///         quantity: 1,
///         price: 10000,
///         metadata: None,
///     };
///     repo.create(&ctx, 12345, &price).await?;
///     
///     // Create a discount for the price
///     let discount = SellDiscountCreate {
///         sell_price_id: 12345,
///         quantity: 10,
///         discount_formula: "-10%".to_string(),
///         calculated_price: 9000,
///         customer_level: Some(1),
///         metadata: None,
///     };
///     repo.create_discount(&ctx, 67890, &discount).await?;
///     
///     // Get all prices for a product variant
///     let prices = repo.get_all_by_product_variant_id(&ctx, 123).await?;
///     
///     // Get all discounts for a price
///     let discounts = repo.get_all_discount_by_price_id(&ctx, 12345).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SellPriceRepository: Send + Sync {
    /// Creates a new sell price.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new sell price
    /// * `price` - Sell price data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell price created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceCreate,
    ) -> DomainResult<()>;

    /// Updates an existing sell price.
    ///
    /// Only provided fields in `SellPriceUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell price to update
    /// * `price` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell price updated successfully
    /// * `Err(Error::NotFound)` - Sell price not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a sell price.
    ///
    /// This method marks the sell price as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell price to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell price deleted successfully
    /// * `Err(Error::NotFound)` - Sell price not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Soft-deletes all sell prices (and their associated discounts) for the given product variant IDs.
    ///
    /// This method marks all matching sell prices and their discounts as deleted instead of
    /// physically removing them. The `is_deleted` flag is set to true, and `deleted_at` is
    /// set to the current timestamp for each affected record.
    ///
    /// This is typically used when deleting a product and its variants, to cascade the
    /// soft-delete to all related pricing data.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_variant_ids` - Slice of product variant IDs whose sell prices (and discounts) to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All matching sell prices and discounts were soft-deleted (including when none matched)
    /// * `Err(Error)` - Database error
    async fn delete_by_product_variant_ids(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_variant_ids: &[i64],
    ) -> DomainResult<()>;

    /// Lists all non-deleted sell prices for a product variant.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `product_variant_id` - ID of the product variant
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SellPrice>)` - List of all active sell prices for the product variant
    /// * `Err(Error)` - Database error
    async fn get_all_by_product_variant_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_variant_id: i64,
    ) -> DomainResult<Vec<SellPrice>>;

    /// Retrieves a sell price by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell price to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(price))` - Sell price found
    /// * `Ok(None)` - Sell price not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellPrice>>;

    /// Creates a new sell discount.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - Snowflake ID for the new sell discount
    /// * `discount` - Sell discount data to create
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell discount created successfully
    /// * `Err(Error)` - Database error or validation error
    async fn create_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountCreate,
    ) -> DomainResult<()>;

    /// Updates an existing sell discount.
    ///
    /// Only provided fields in `SellDiscountUpdate` will be updated.
    /// The `updated_at` timestamp is automatically updated.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell discount to update
    /// * `discount` - Update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell discount updated successfully
    /// * `Err(Error::NotFound)` - Sell discount not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn update_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a sell discount.
    ///
    /// This method marks the sell discount as deleted instead of physically removing it.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell discount to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Sell discount deleted successfully
    /// * `Err(Error::NotFound)` - Sell discount not found or already deleted
    /// * `Err(Error)` - Database error
    async fn delete_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()>;

    /// Soft-deletes all discounts for a sell price.
    ///
    /// This method marks all discounts associated with a sell price as deleted.
    /// The `is_deleted` flag is set to true, and `deleted_at` is set to the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `sell_price_id` - ID of the sell price whose discounts to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Discounts deleted successfully
    /// * `Err(Error)` - Database error
    async fn delete_discounts_by_sell_price_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<()>;

    /// Lists all non-deleted discounts for a sell price.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `sell_price_id` - ID of the sell price
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SellDiscount>)` - List of all active discounts for the sell price
    /// * `Err(Error)` - Database error
    async fn get_all_discount_by_price_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<Vec<SellDiscount>>;

    /// Retrieves a discount by ID (excluding soft-deleted records).
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `id` - ID of the sell discount to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(discount))` - Sell discount found
    /// * `Ok(None)` - Sell discount not found or soft-deleted
    /// * `Err(Error)` - Database error
    async fn get_discount_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellDiscount>>;
}
