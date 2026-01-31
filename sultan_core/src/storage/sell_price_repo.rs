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
#[async_trait]
pub trait SellPriceRepository: Send + Sync {
    /// Creates a new sell price.
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceCreate,
    ) -> DomainResult<()>;

    /// Updates an existing sell price.
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        price: &SellPriceUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a sell price.
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Lists all non-deleted sell prices for a product variant.
    async fn get_all_by_product_variant_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        product_variant_id: i64,
    ) -> DomainResult<Vec<SellPrice>>;

    /// Retrieves a sell price by ID (excluding soft-deleted records).
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellPrice>>;

    /// Creates a new sell discount.
    async fn create_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountCreate,
    ) -> DomainResult<()>;

    /// Updates an existing sell discount.
    async fn update_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        discount: &SellDiscountUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a sell discount.
    async fn delete_discount(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<()>;

    /// Soft-deletes all discounts for a sell price.
    async fn delete_discounts_by_sell_price_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<()>;

    /// Lists all non-deleted discounts for a sell price.
    async fn get_all_discount_by_price_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        sell_price_id: i64,
    ) -> DomainResult<Vec<SellDiscount>>;

    /// Retrieves a discount by ID (excluding soft-deleted records).
    async fn get_discount_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<SellDiscount>>;
}
