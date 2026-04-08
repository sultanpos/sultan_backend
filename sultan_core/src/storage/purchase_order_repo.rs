use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::purchase_order::{
        PurchaseOrder, PurchaseOrderCreate, PurchaseOrderItem, PurchaseOrderItemCreate,
        PurchaseOrderItemUpdate, PurchaseOrderPage, PurchaseOrderQuery, PurchaseOrderUpdate,
        PurchasePayment, PurchasePaymentCreate, PurchasePaymentUpdate,
    },
};

#[async_trait]
pub trait PurchaseOrderRepository: Send + Sync {
    /// Creates a new purchase order together with its line items.
    ///
    /// `id` is the Snowflake ID for the purchase_order row. `item_ids` must have
    /// the same length as `items` — each element is the Snowflake ID for the
    /// corresponding item.
    async fn create(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PurchaseOrderCreate,
        items: &[(i64, PurchaseOrderItemCreate)],
    ) -> DomainResult<()>;

    /// Appends a payment row to an existing purchase order and updates
    /// `paid_amount` and `payment_status` on the order header atomically.
    ///
    /// Returns `NotFound` if the order is soft-deleted or does not exist.
    async fn add_payment(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
        payment_id: i64,
        data: &PurchasePaymentCreate,
    ) -> DomainResult<()>;

    /// Fetches a purchase order by ID including its items and payments.
    /// Returns `None` if the order is soft-deleted or does not exist.
    async fn get_by_id(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<PurchaseOrder>>;

    /// Partially updates the purchase order header fields.
    /// Returns `NotFound` if the order is soft-deleted or does not exist.
    async fn update(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PurchaseOrderUpdate,
    ) -> DomainResult<()>;

    /// Soft-deletes a purchase order.
    /// Returns `NotFound` if the order is already deleted or does not exist.
    async fn delete(&self, ctx: &super::RepoCtx<impl ConnectionTrait>, id: i64)
    -> DomainResult<()>;

    /// Lists purchase orders with cursor-based pagination.
    async fn get_all(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        query: &PurchaseOrderQuery,
    ) -> DomainResult<PurchaseOrderPage>;

    // ── Item management ───────────────────────────────────────────────────────

    /// Appends a new line item to an existing purchase order and recalculates
    /// `subtotal` and `total_amount` on the order header atomically.
    ///
    /// Returns `NotFound` if the order is soft-deleted or does not exist.
    async fn add_item(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
        item_id: i64,
        data: &PurchaseOrderItemCreate,
    ) -> DomainResult<()>;

    /// Partially updates a purchase order line item and recalculates
    /// `subtotal` and `total_amount` on the order header if cost fields change.
    ///
    /// Returns `NotFound` if the item does not exist.
    async fn update_item(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        item_id: i64,
        data: &PurchaseOrderItemUpdate,
    ) -> DomainResult<()>;

    /// Hard-deletes a purchase order line item and recalculates
    /// `subtotal` and `total_amount` on the order header atomically.
    ///
    /// Returns `NotFound` if the item does not exist.
    async fn delete_item(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        item_id: i64,
    ) -> DomainResult<()>;

    /// Returns all line items for the given purchase order.
    async fn get_items(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
    ) -> DomainResult<Vec<PurchaseOrderItem>>;

    // ── Payment management ────────────────────────────────────────────────────

    /// Partially updates a payment row. If `amount` changes, also recalculates
    /// `paid_amount` and `payment_status` on the order header atomically.
    ///
    /// Returns `NotFound` if the payment does not exist.
    async fn update_payment(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        payment_id: i64,
        data: &PurchasePaymentUpdate,
    ) -> DomainResult<()>;

    /// Hard-deletes a payment row and recalculates `paid_amount` and
    /// `payment_status` on the order header atomically.
    ///
    /// Returns `NotFound` if the payment does not exist.
    async fn delete_payment(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        payment_id: i64,
    ) -> DomainResult<()>;

    /// Returns all payments for the given purchase order.
    async fn get_payments(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
    ) -> DomainResult<Vec<PurchasePayment>>;
}
